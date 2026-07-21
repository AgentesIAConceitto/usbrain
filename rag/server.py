"""BGE Embeddings server + SQLite vector store for USBRAIN.

Provides:
  POST /embed   — returns embedding for text
  POST /store   — stores a chunk with embedding
  POST /search  — searches by embedding similarity
  GET  /stats   — returns count of stored documents
"""
from __future__ import annotations

import hashlib
import html
import json
import mimetypes
import os
import re
import sqlite3
import struct
import sys
import time
import traceback
import urllib.request
from pathlib import Path
from typing import Any
from urllib.parse import urljoin, urlparse, urlunparse

_BGE_BOOT_LOG_PATH = os.environ.get("USBRAIN_BGE_BOOT_LOG", "").strip()


def _boot_log(message: str) -> None:
    if not _BGE_BOOT_LOG_PATH:
        return
    try:
        with Path(_BGE_BOOT_LOG_PATH).open("a", encoding="utf-8") as fh:
            fh.write(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] {message}\n")
    except Exception:
        pass


# #region debug-point C:debug-server-reporting
def _debug_event(hypothesis_id: str, location: str, message: str, **data: Any) -> None:
    env_path = Path(__file__).resolve().parent.parent / ".dbg" / "usb-eject-upload-web.env"
    debug_server_url = "http://127.0.0.1:7777/event"
    session_id = "usb-eject-upload-web"
    try:
        if env_path.exists():
            for line in env_path.read_text(encoding="utf-8", errors="replace").splitlines():
                if line.startswith("DEBUG_SERVER_URL="):
                    debug_server_url = line.split("=", 1)[1].strip() or debug_server_url
                elif line.startswith("DEBUG_SESSION_ID="):
                    session_id = line.split("=", 1)[1].strip() or session_id
    except Exception:
        pass

    payload = json.dumps({
        "sessionId": session_id,
        "runId": "pre-fix",
        "hypothesisId": hypothesis_id,
        "location": location,
        "msg": f"[DEBUG] {message}",
        "data": data,
        "ts": int(time.time() * 1000),
    }).encode("utf-8")
    try:
        urllib.request.urlopen(
            urllib.request.Request(
                debug_server_url,
                data=payload,
                headers={"Content-Type": "application/json"},
            ),
            timeout=2,
        ).read()
    except Exception:
        pass
# #endregion


_boot_log("rag.server: importing numpy")
import numpy as np
_boot_log("rag.server: numpy import ok")
# Avoid sandbox-denied reads from /etc/apache2/mime.types during aiohttp import on macOS.
if sys.platform == "darwin":
    mimetypes.knownfiles = []
_boot_log("rag.server: importing aiohttp.web")
from aiohttp import web
_boot_log("rag.server: aiohttp.web import ok")

# Try to import BGE model
_boot_log("rag.server: importing sentence_transformers")
try:
    from sentence_transformers import SentenceTransformer
    _HAS_SENTENCE_TRANSFORMERS = True
except ImportError:
    _HAS_SENTENCE_TRANSFORMERS = False
_boot_log(f"rag.server: sentence_transformers available={_HAS_SENTENCE_TRANSFORMERS}")

BGE_M3_MODEL_DIRNAME = "bge-m3-model"
BGE_M3_REPO_ID = "BAAI/bge-m3"

# Fallback: embedding lexical deterministico para quando o modelo BGE
# nao estiver disponivel no runtime do pendrive.
_FALLBACK_EMBED_DIM = 384
MAX_CRAWL_DEPTH = 2
MAX_PAGES_PER_SITE = 4
DEFAULT_FETCH_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/126.0.0.0 Safari/537.36"
    ),
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7",
    "Cache-Control": "no-cache",
}


def _fake_embed(text: str) -> list[float]:
    """Deterministic token hashing fallback with useful lexical similarity."""
    emb = np.zeros(_FALLBACK_EMBED_DIM, dtype=np.float32)
    tokens = re.findall(r"\w+", text.lower())
    if not tokens:
        return emb.tolist()

    for idx, token in enumerate(tokens):
        for feature in (token, f"{token}:{idx % 7}"):
            digest = hashlib.blake2b(feature.encode("utf-8"), digest_size=8).digest()
            bucket = int.from_bytes(digest[:4], "little") % _FALLBACK_EMBED_DIM
            sign = 1.0 if digest[4] % 2 == 0 else -1.0
            emb[bucket] += sign

    for left, right in zip(tokens, tokens[1:]):
        digest = hashlib.blake2b(f"{left}_{right}".encode("utf-8"), digest_size=8).digest()
        bucket = int.from_bytes(digest[:4], "little") % _FALLBACK_EMBED_DIM
        emb[bucket] += 0.5

    emb = emb / (np.linalg.norm(emb) + 1e-10)
    return emb.tolist()


def _normalize_url(url: str) -> str:
    """Normalize URLs for deduplication and same-site crawling."""
    raw = (url or "").strip()
    if not raw:
        return ""
    if raw.startswith("//"):
        raw = f"https:{raw}"
    elif not re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*://", raw):
        raw = f"https://{raw}"
    parsed = urlparse(raw)
    if parsed.scheme not in {"http", "https"} or not parsed.netloc:
        return ""
    cleaned_path = parsed.path or "/"
    return urlunparse((parsed.scheme, parsed.netloc.lower(), cleaned_path, "", parsed.query, ""))


def _query_domain_hint(query: str) -> str | None:
    """Extract a domain hint from a search query when the user mentions a site directly."""
    query = (query or "").strip()
    if not query:
        return None
    match = re.search(r"((?:https?://)?(?:www\.)?[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})(?:/[^\s]*)?", query)
    if not match:
        return None
    normalized = _normalize_url(match.group(1))
    return urlparse(normalized).netloc if normalized else None


def _same_domain(url: str, domain_hint: str | None) -> bool:
    """Check whether a URL belongs to the requested or discovered domain."""
    if not domain_hint:
        return True
    netloc = urlparse(url).netloc.lower()
    hint = domain_hint.lower()
    return netloc == hint or netloc.endswith(f".{hint}") or hint.endswith(f".{netloc}")


def _extract_main_text_from_html(html_text: str) -> str:
    """Extract readable text from HTML without heavy dependencies."""
    if not html_text:
        return ""
    cleaned = re.sub(r"(?is)<script[^>]*>.*?</script>", " ", html_text)
    cleaned = re.sub(r"(?is)<style[^>]*>.*?</style>", " ", cleaned)
    cleaned = re.sub(r"(?is)<noscript[^>]*>.*?</noscript>", " ", cleaned)
    cleaned = re.sub(r"(?is)<svg[^>]*>.*?</svg>", " ", cleaned)
    cleaned = re.sub(r"(?is)<(nav|footer|header|aside|form)[^>]*>.*?</\1>", " ", cleaned)
    cleaned = re.sub(r"(?i)</(p|div|section|article|main|li|h1|h2|h3|h4|h5|h6|tr)>", "\n", cleaned)
    cleaned = re.sub(r"(?i)<br\s*/?>", "\n", cleaned)
    cleaned = re.sub(r"(?s)<[^>]+>", " ", cleaned)
    cleaned = html.unescape(cleaned)
    cleaned = re.sub(r"\r", "\n", cleaned)
    cleaned = re.sub(r"\n{3,}", "\n\n", cleaned)
    cleaned = re.sub(r"[ \t]{2,}", " ", cleaned)
    return cleaned.strip()


def _extract_title_from_html(html_text: str, fallback: str) -> str:
    """Extract page title from HTML."""
    match = re.search(r"(?is)<title[^>]*>(.*?)</title>", html_text)
    if not match:
        return fallback
    title = html.unescape(re.sub(r"\s+", " ", match.group(1))).strip()
    return title or fallback


def _extract_links_from_html(html_text: str, base_url: str, domain_hint: str | None) -> list[str]:
    """Extract crawl candidates from HTML, limited to the same domain."""
    links: list[str] = []
    for href in re.findall(r'(?is)href=["\']([^"\']+)["\']', html_text):
        href = href.strip()
        if not href or href.startswith(("#", "mailto:", "javascript:", "tel:")):
            continue
        candidate = _normalize_url(urljoin(base_url, href))
        if not candidate or not _same_domain(candidate, domain_hint):
            continue
        links.append(candidate)
    deduped: list[str] = []
    seen: set[str] = set()
    for link in links:
        if link in seen:
            continue
        seen.add(link)
        deduped.append(link)
    return deduped


async def _fetch_page_snapshot(client: Any, url: str, domain_hint: str | None) -> tuple[str, str, list[str]]:
    """Fetch a page and return extracted title, readable text and internal links."""
    resp = await client.get(url, timeout=20.0, follow_redirects=True, headers=DEFAULT_FETCH_HEADERS)
    resp.raise_for_status()
    content_type = (resp.headers.get("content-type") or "").lower()
    if "text/html" not in content_type and "application/xhtml+xml" not in content_type:
        raise ValueError(f"Conteudo nao-HTML ignorado: {content_type or 'desconhecido'}")
    html_text = resp.text
    title = _extract_title_from_html(html_text, url)
    text = _extract_main_text_from_html(html_text)
    links = _extract_links_from_html(html_text, str(resp.url), domain_hint)
    return title, text, links


async def _crawl_site_documents(
    client: Any,
    start_url: str,
    *,
    domain_hint: str | None = None,
    max_depth: int = 1,
    max_pages: int = MAX_PAGES_PER_SITE,
) -> list[dict[str, str]]:
    """Crawl a site with bounded depth and collect readable page texts."""
    normalized_start = _normalize_url(start_url)
    if not normalized_start:
        return []

    if domain_hint is None:
        domain_hint = urlparse(normalized_start).netloc.lower()

    queue: list[tuple[str, int]] = [(normalized_start, 0)]
    visited: set[str] = set()
    documents: list[dict[str, str]] = []

    while queue and len(documents) < max_pages:
        current_url, depth = queue.pop(0)
        if current_url in visited:
            continue
        visited.add(current_url)
        try:
            title, text, links = await _fetch_page_snapshot(client, current_url, domain_hint)
        except Exception:
            continue

        if len(text) >= 200:
            documents.append({
                "title": title,
                "url": current_url,
                "content": text,
            })

        if depth >= max_depth:
            continue
        for link in links:
            if link not in visited and all(existing_url != link for existing_url, _ in queue):
                queue.append((link, depth + 1))

    return documents


def _store_chunked_documents(documents: list[dict[str, str]], source_label: str) -> tuple[int, int]:
    """Chunk and store a list of crawled documents."""
    stored = 0
    pages_indexed = 0
    for document in documents:
        words = document["content"].split()
        if not words:
            continue
        chunk_size = 500
        chunks_made = 0
        for i in range(0, len(words), chunk_size):
            chunk_text = " ".join(words[i:i + chunk_size])
            if len(chunk_text) <= 100:
                continue
            emb = store.embed(chunk_text)
            title = document["title"][:80]
            chunk_title = title if chunks_made == 0 else f"{title} (pt {chunks_made + 1})"
            store.store(chunk_title, chunk_text, document["url"], emb)
            chunks_made += 1
            stored += 1
        if chunks_made:
            pages_indexed += 1
    return stored, pages_indexed


def _unique_urls(documents: list[dict[str, str]]) -> list[str]:
    seen: set[str] = set()
    urls: list[str] = []
    for document in documents:
        url = document.get("url", "").strip()
        if url and url not in seen:
            seen.add(url)
            urls.append(url)
    return urls


class BGEStore:
    """SQLite-backed vector store with cosine similarity search."""

    def __init__(self, db_path: str = "vetor.db"):
        _boot_log(f"BGEStore.__init__: db_path={db_path}")
        self.db_path = Path(db_path)
        self.conn = sqlite3.connect(str(self.db_path))
        self.conn.execute("PRAGMA journal_mode=WAL")
        self._init_db()
        _boot_log("BGEStore.__init__: sqlite initialized")

        # Load model or use fallback
        self.model = None
        self.model_name = "fallback-lexical"
        self.use_fallback = True
        if _HAS_SENTENCE_TRANSFORMERS:
            try:
                for model_path in self._candidate_model_paths():
                    _boot_log(f"BGEStore.__init__: checking model path {model_path}")
                    if model_path.exists():
                        _boot_log(f"BGEStore.__init__: loading SentenceTransformer from {model_path}")
                        self.model = SentenceTransformer(str(model_path))
                        self.model_name = BGE_M3_REPO_ID
                        self.use_fallback = False
                        _boot_log("BGEStore.__init__: SentenceTransformer loaded")
                        break
            except Exception as exc:
                _boot_log(
                    "BGEStore.__init__: SentenceTransformer load failed: "
                    f"{exc!r}\n{traceback.format_exc()}"
                )

        self.embed_dim = _FALLBACK_EMBED_DIM
        if not self.use_fallback and self.model is not None:
            self.embed_dim = self.model.get_sentence_embedding_dimension() or _FALLBACK_EMBED_DIM
        _boot_log(
            "BGEStore.__init__: ready "
            f"use_fallback={self.use_fallback} model_name={self.model_name} embed_dim={self.embed_dim}"
        )

    @staticmethod
    def _candidate_model_paths() -> list[Path]:
        base_dir = Path(__file__).parent
        return [
            base_dir / BGE_M3_MODEL_DIRNAME,
            base_dir.parent / "cache" / BGE_M3_MODEL_DIRNAME,
        ]

    def _init_db(self):
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                source TEXT DEFAULT 'upload',
                embedding BLOB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """)
        self.conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_chunks_source ON chunks(source)
        """)
        self.conn.commit()

    def embed(self, text: str) -> list[float]:
        """Get embedding vector for text."""
        if self.use_fallback:
            return _fake_embed(text)
        emb = self.model.encode(text, normalize_embeddings=True)
        return emb.tolist()

    def store(self, title: str, content: str, source: str, embedding: list[float]) -> int:
        """Store a chunk with its embedding."""
        blob = struct.pack(f"{len(embedding)}f", *embedding)
        cursor = self.conn.execute(
            "INSERT INTO chunks (title, content, source, embedding) VALUES (?, ?, ?, ?)",
            (title, content, source, blob),
        )
        self.conn.commit()
        return cursor.lastrowid

    def search(self, query_embedding: list[float], top_k: int = 5) -> list[dict[str, Any]]:
        """Search for most similar chunks by cosine similarity."""
        query_vec = np.array(query_embedding, dtype=np.float32)

        rows = self.conn.execute(
            "SELECT id, title, content, source, embedding FROM chunks ORDER BY id"
        ).fetchall()

        if not rows:
            return []

        # Brute force cosine similarity (SQLite doesn't have vector search)
        scores = []
        for row_id, title, content, source, emb_blob in rows:
            if emb_blob is None:
                continue
            vec = np.frombuffer(emb_blob, dtype=np.float32)
            if len(vec) != len(query_vec):
                continue
            # Cosine similarity
            sim = float(np.dot(query_vec, vec) / (np.linalg.norm(query_vec) * np.linalg.norm(vec) + 1e-10))
            scores.append((sim, title, content, source))

        scores.sort(key=lambda x: x[0], reverse=True)

        results = []
        for sim, title, content, source in scores[:top_k]:
            results.append({
                "title": title,
                "content": content,
                "source": source,
                "similarity": round(sim, 4),
            })

        return results

    def search_text(self, query: str, top_k: int = 5) -> list[dict[str, Any]]:
        """Search chunks by lexical overlap to improve recall on explicit entities/domains."""
        tokens = [token for token in re.findall(r"\w+", (query or "").lower()) if len(token) >= 2]
        if not tokens:
            return []

        rows = self.conn.execute(
            "SELECT id, title, content, source FROM chunks ORDER BY id DESC"
        ).fetchall()

        scored: list[tuple[float, str, str, str]] = []
        for _row_id, title, content, source in rows:
            haystack = f"{title}\n{source}\n{content}".lower()
            unique_hits = sum(1 for token in set(tokens) if token in haystack)
            if unique_hits == 0:
                continue

            title_hits = sum(1 for token in set(tokens) if token in (title or "").lower())
            source_hits = sum(1 for token in set(tokens) if token in (source or "").lower())
            score = unique_hits + (title_hits * 1.5) + (source_hits * 1.2)
            scored.append((score, title, content, source))

        scored.sort(key=lambda item: item[0], reverse=True)
        return [
            {
                "title": title,
                "content": content,
                "source": source,
                "similarity": round(float(score), 4),
            }
            for score, title, content, source in scored[:top_k]
        ]

    def stats(self) -> dict:
        count = self.conn.execute("SELECT COUNT(*) FROM chunks").fetchone()[0]
        return {
            "total_chunks": count,
            "embedding_backend": self.model_name,
            "use_fallback": self.use_fallback,
            "embedding_dim": self.embed_dim,
        }

    def close(self):
        self.conn.close()


# --- HTTP Server ---

store: BGEStore | None = None


async def handle_embed(request: web.Request) -> web.Response:
    body = await request.json()
    text = body.get("text", "")
    if not text:
        return web.json_response({"error": "text is required"}, status=400)
    embedding = store.embed(text)
    return web.json_response({"embedding": embedding})


async def handle_store(request: web.Request) -> web.Response:
    body = await request.json()
    # #region debug-point B:rag-store-entry
    _debug_event(
        "B",
        "rag/server.py:handle_store:entry",
        "Store endpoint received chunk",
        title=body.get("title", "untitled"),
        source=body.get("source", "upload"),
        embedding_len=len(body.get("embedding", []) or []),
        content_chars=len(body.get("content", "") or ""),
    )
    # #endregion
    chunk_id = store.store(
        title=body.get("title", "untitled"),
        content=body.get("content", ""),
        source=body.get("source", "upload"),
        embedding=body.get("embedding", []),
    )
    return web.json_response({"id": chunk_id})


async def handle_search(request: web.Request) -> web.Response:
    body = await request.json()
    embedding = body.get("embedding", [])
    top_k = body.get("top_k", 5)
    if not embedding:
        return web.json_response({"error": "embedding is required"}, status=400)
    results = store.search(embedding, top_k)
    return web.json_response({"results": results})


async def handle_search_text(request: web.Request) -> web.Response:
    body = await request.json()
    query = body.get("query", "")
    top_k = body.get("top_k", 5)
    if not query:
        return web.json_response({"error": "query is required"}, status=400)
    results = store.search_text(query, top_k)
    return web.json_response({"results": results})


async def handle_stats(request: web.Request) -> web.Response:
    return web.json_response(store.stats())


async def handle_add_search(request: web.Request) -> web.Response:
    """Search the web, fetch results, chunk and store automatically."""
    body = await request.json()
    query = body.get("query", "")
    if not query:
        return web.json_response({"error": "query is required"}, status=400)

    domain_hint = body.get("domain") or _query_domain_hint(query)
    crawl_depth = body.get("crawl_depth", 1)
    try:
        crawl_depth = max(0, min(MAX_CRAWL_DEPTH, int(crawl_depth)))
    except (TypeError, ValueError):
        crawl_depth = 1

    import httpx
    brave_key = "BSAyaH63j2Mns6noio_RrQXjmiSUYD-"
    try:
        # #region debug-point D:rag-add-search-entry
        _debug_event(
            "D",
            "rag/server.py:handle_add_search:entry",
            "RAG add_search started",
            query=query,
            domain_hint=domain_hint,
            crawl_depth=crawl_depth,
        )
        # #endregion
        async with httpx.AsyncClient(timeout=15.0) as client:
            results = []
            direct_url = _normalize_url(query)
            if direct_url:
                try:
                    title, text, links = await _fetch_page_snapshot(client, direct_url, domain_hint)
                    documents = [{"title": title, "url": direct_url, "content": text}]
                    if crawl_depth > 0:
                        documents = await _crawl_site_documents(
                            client,
                            direct_url,
                            domain_hint=domain_hint,
                            max_depth=crawl_depth,
                        )
                    stored, pages_indexed = _store_chunked_documents(documents, direct_url)
                    # #region debug-point D:rag-add-search-direct-url
                    _debug_event(
                        "D",
                        "rag/server.py:handle_add_search:direct-url",
                        "RAG add_search handled query as direct URL",
                        query=query,
                        direct_url=direct_url,
                        pages_indexed=pages_indexed,
                        stored=stored,
                    )
                    # #endregion
                except Exception as exc:
                    # #region debug-point D:rag-add-search-direct-url-blocked
                    _debug_event(
                        "D",
                        "rag/server.py:handle_add_search:direct-url-blocked",
                        "Direct URL fetch was blocked during add_search",
                        query=query,
                        direct_url=direct_url,
                        error=repr(exc),
                    )
                    # #endregion
                    return web.json_response({
                        "stored": 0,
                        "query": query,
                        "results_count": 1,
                        "pages_indexed": 0,
                        "message": (
                            f"⚠️ O site {urlparse(direct_url).netloc} bloqueou a raspagem direta nesta tentativa. "
                            "Nenhum chunk foi adicionado. Tente usar uma pesquisa descritiva em vez da URL exata."
                        ),
                    })
                return web.json_response({
                    "stored": stored,
                    "query": query,
                    "results_count": 1,
                    "pages_indexed": pages_indexed,
                    "mode": "direct-url",
                    "sources": _unique_urls(documents)[:8],
                    "message": f"✅ {stored} chunks adicionados de {pages_indexed} pagina(s) do site {urlparse(direct_url).netloc}.",
                })

            resp = await client.get(
                "https://api.search.brave.com/res/v1/web/search",
                params={"q": query, "count": 5},
                headers={"X-Subscription-Token": brave_key, "Accept": "application/json"},
            )
            if resp.status_code != 200:
                return web.json_response({"error": f"Search API error: {resp.status_code}"}, status=500)
            data = resp.json()
            results = data.get("web", {}).get("results", [])
            # #region debug-point D:rag-add-search-brave-results
            _debug_event(
                "D",
                "rag/server.py:handle_add_search:brave-results",
                "Brave search returned results",
                query=query,
                result_count=len(results),
                domain_hint=domain_hint,
            )
            # #endregion
            if domain_hint:
                filtered_results = [
                    item for item in results
                    if _same_domain(_normalize_url(item.get("url", "")), domain_hint)
                ]
                if filtered_results:
                    results = filtered_results
            if not results:
                return web.json_response({"error": "No results found"}, status=404)
            stored = 0
            pages_indexed = 0
            indexed_sources: list[str] = []
            for item in results[:5]:
                title = item.get("title", "Untitled")
                desc = item.get("description", "")
                url = _normalize_url(item.get("url", ""))
                if not url:
                    continue
                try:
                    documents = await _crawl_site_documents(
                        client,
                        url,
                        domain_hint=urlparse(url).netloc.lower(),
                        max_depth=crawl_depth,
                    )
                    if not documents:
                        fallback_content = f"{title}\n\n{desc}".strip()
                        if fallback_content:
                            documents = [{"title": title, "url": url, "content": fallback_content}]
                except Exception:
                    fallback_content = f"{title}\n\n{desc}".strip()
                    documents = [{"title": title, "url": url, "content": fallback_content}] if fallback_content else []
                stored_now, pages_now = _store_chunked_documents(documents, url)
                stored += stored_now
                pages_indexed += pages_now
                indexed_sources.extend(_unique_urls(documents))
            # #region debug-point D:rag-add-search-finish
            _debug_event(
                "D",
                "rag/server.py:handle_add_search:finish",
                "RAG add_search finished",
                query=query,
                stored=stored,
                pages_indexed=pages_indexed,
                results_count=len(results),
            )
            # #endregion
            return web.json_response({
                "stored": stored, "query": query,
                "results_count": len(results),
                "pages_indexed": pages_indexed,
                "mode": "search",
                "sources": list(dict.fromkeys(indexed_sources))[:8],
                "message": f"✅ {stored} chunks adicionados de {pages_indexed} pagina(s) a partir da pesquisa sobre '{query}'."
            })
    except Exception as e:
        # #region debug-point D:rag-add-search-error
        _debug_event(
            "D",
            "rag/server.py:handle_add_search:error",
            "RAG add_search failed",
            query=query,
            error=repr(e),
            traceback=traceback.format_exc()[:4000],
        )
        # #endregion
        return web.json_response({"error": str(e)}, status=500)

async def handle_add_url(request: web.Request) -> web.Response:
    """Receive a URL, fetch content, chunk and store."""
    body = await request.json()
    url = body.get("url", "")
    if not url:
        return web.json_response({"error": "url is required"}, status=400)

    crawl_depth = body.get("crawl_depth", 1)
    try:
        crawl_depth = max(0, min(MAX_CRAWL_DEPTH, int(crawl_depth)))
    except (TypeError, ValueError):
        crawl_depth = 1

    import httpx
    try:
        # #region debug-point D:rag-add-url-entry
        _debug_event(
            "D",
            "rag/server.py:handle_add_url:entry",
            "RAG add_url started",
            url=url,
            crawl_depth=crawl_depth,
        )
        # #endregion
        async with httpx.AsyncClient(timeout=15.0, follow_redirects=True, headers=DEFAULT_FETCH_HEADERS) as client:
            documents = await _crawl_site_documents(
                client,
                url,
                domain_hint=urlparse(_normalize_url(url)).netloc.lower() if _normalize_url(url) else None,
                max_depth=crawl_depth,
                max_pages=max(2, MAX_PAGES_PER_SITE + crawl_depth),
            )

        stored, pages_indexed = _store_chunked_documents(documents, url)
        # #region debug-point D:rag-add-url-finish
        _debug_event(
            "D",
            "rag/server.py:handle_add_url:finish",
            "RAG add_url finished",
            url=url,
            stored=stored,
            pages_indexed=pages_indexed,
            documents=len(documents),
        )
        # #endregion
        return web.json_response({
            "stored": stored,
            "url": url,
            "pages_indexed": pages_indexed,
        })

    except Exception as e:
        # #region debug-point D:rag-add-url-error
        _debug_event(
            "D",
            "rag/server.py:handle_add_url:error",
            "RAG add_url failed",
            url=url,
            error=repr(e),
            traceback=traceback.format_exc()[:4000],
        )
        # #endregion
        return web.json_response({"error": str(e)}, status=500)


def create_app(db_path: str = "vetor.db") -> web.Application:
    global store
    _boot_log(f"create_app: start db_path={db_path}")
    store = BGEStore(db_path)

    app = web.Application()
    app["store"] = store

    app.router.add_post("/embed", handle_embed)
    app.router.add_post("/store", handle_store)
    app.router.add_post("/search", handle_search)
    app.router.add_post("/search_text", handle_search_text)
    app.router.add_get("/stats", handle_stats)
    app.router.add_post("/add_url", handle_add_url)
    app.router.add_post("/add_search", handle_add_search)

    _boot_log("create_app: routes ready")
    return app


if __name__ == "__main__":
    import sys
    db_path = sys.argv[1] if len(sys.argv) > 1 else "vetor.db"
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 9090
    _boot_log(f"__main__: start db_path={db_path} port={port}")
    app = create_app(db_path)
    print(f"BGE Server running on http://127.0.0.1:{port}")
    _boot_log(f"__main__: web.run_app host=127.0.0.1 port={port}")
    web.run_app(app, host="127.0.0.1", port=port)
