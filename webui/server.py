"""USBRAIN Web UI server."""
from __future__ import annotations

import json
import mimetypes
import os
import re
import sys
import threading
import time
import urllib.request
import webbrowser
from pathlib import Path
from urllib.parse import urljoin, urlparse

# Avoid sandbox-denied reads from /etc/apache2/mime.types during aiohttp import on macOS.
if sys.platform == "darwin":
    mimetypes.knownfiles = []
from aiohttp import web

from agent import USBAgent


def _extract_main_text_from_html(html: str) -> str:
    """Extract readable text from HTML without external dependencies."""
    if not html:
        return ""

    cleaned = re.sub(r"(?is)<script[^>]*>.*?</script>", " ", html)
    cleaned = re.sub(r"(?is)<style[^>]*>.*?</style>", " ", cleaned)
    cleaned = re.sub(r"(?is)<noscript[^>]*>.*?</noscript>", " ", cleaned)
    cleaned = re.sub(r"(?is)<svg[^>]*>.*?</svg>", " ", cleaned)
    cleaned = re.sub(r"(?is)<(nav|footer|header|aside|form)[^>]*>.*?</\1>", " ", cleaned)
    cleaned = re.sub(r"(?i)</(p|div|section|article|main|li|h1|h2|h3|h4|h5|h6|tr)>", "\n", cleaned)
    cleaned = re.sub(r"(?i)<br\s*/?>", "\n", cleaned)
    cleaned = re.sub(r"(?s)<[^>]+>", " ", cleaned)
    cleaned = re.sub(r"&nbsp;", " ", cleaned)
    cleaned = re.sub(r"&amp;", "&", cleaned)
    cleaned = re.sub(r"&lt;", "<", cleaned)
    cleaned = re.sub(r"&gt;", ">", cleaned)
    cleaned = re.sub(r"\r", "\n", cleaned)
    cleaned = re.sub(r"\n{3,}", "\n\n", cleaned)
    cleaned = re.sub(r"[ \t]{2,}", " ", cleaned)
    return cleaned.strip()


def _detect_text_encoding(content: bytes) -> str | None:
    """Best-effort text encoding detection for uploaded files."""
    for encoding in ("utf-8", "utf-8-sig", "cp1252", "latin-1"):
        try:
            content.decode(encoding)
            return encoding
        except UnicodeDecodeError:
            continue
    return None


def _looks_like_direct_web_target(query: str) -> bool:
    text = (query or "").strip()
    if not text:
        return False
    if text.startswith(("http://", "https://")):
        return True
    if " " in text:
        return False
    return "." in text and "/" not in text.strip("/")


# #region debug-point B:debug-server-reporting
def _debug_event(hypothesis_id: str, location: str, message: str, **data) -> None:
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


class WebUIServer:
    """Simple web interface for USBRAIN."""

    def __init__(self, agent: USBAgent, host: str = "127.0.0.1", port: int = 8000):
        self.agent = agent
        self.host = host
        self.port = port
        self.template_dir = Path(__file__).parent / "templates"

    def create_app(self) -> web.Application:
        app = web.Application()
        app["agent"] = self.agent
        static_dir = Path(__file__).parent / "static"

        app.router.add_get("/", self.handle_index)
        app.router.add_post("/api/chat", self.handle_chat)
        app.router.add_post("/api/upload", self.handle_upload)
        app.router.add_post("/api/add_url", self.handle_add_url)
        app.router.add_get("/api/search_web", self.handle_search_web)
        app.router.add_get("/api/stats", self.handle_stats)
        app.router.add_get("/api/reset", self.handle_reset)
        app.router.add_get("/api/personality", self.handle_get_personality)
        app.router.add_post("/api/personality", self.handle_set_personality)
        app.router.add_get("/api/skills", self.handle_list_skills)
        if static_dir.exists():
            app.router.add_static("/static", path=str(static_dir), name="static")

        return app

    async def handle_index(self, request: web.Request) -> web.Response:
        html_path = self.template_dir / "index.html"
        html = html_path.read_text(encoding="utf-8")

        # Inject hardware info
        hw = self.agent.detect_hardware()
        html = html.replace("{{HARDWARE_INFO}}", json.dumps(hw))
        html = html.replace("{{MODEL_NAME}}", json.dumps(self.agent.model))

        return web.Response(text=html, content_type="text/html")

    async def handle_chat(self, request: web.Request) -> web.Response:
        body = await request.json()
        message = body.get("message", "")

        if not message:
            return web.json_response({"error": "message is required"}, status=400)

        try:
            response = await self.agent.chat(message)
        except Exception as e:
            response = f"Erro: {e}"

        hw = self.agent.detect_hardware()
        return web.json_response({
            "response": response,
            "model": self.agent.model,
            "recommended_model": hw["recommended_model"],
        })

    async def handle_upload(self, request: web.Request) -> web.Response:
        reader = await request.multipart()

        field = await reader.next()
        if not field or field.name != "file":
            return web.json_response({"error": "no file field"}, status=400)

        filename = field.filename or "unknown"
        content = await field.read()
        lower_name = filename.lower()
        text = None
        # #region debug-point B:upload-entry
        _debug_event(
            "B",
            "webui/server.py:handle_upload:entry",
            "Upload request received",
            filename=filename,
            size_bytes=len(content),
        )
        # #endregion

        if lower_name.endswith((".txt", ".md", ".json", ".csv")):
            encoding = _detect_text_encoding(content)
            # #region debug-point B:upload-text-detected
            _debug_event(
                "B",
                "webui/server.py:handle_upload:text-detected",
                "Text upload encoding evaluated",
                filename=filename,
                encoding=encoding,
            )
            # #endregion
            if not encoding:
                return web.json_response(
                    {"error": f"Arquivo {filename} nao parece estar em um encoding de texto suportado."},
                    status=400,
                )
            try:
                text = content.decode(encoding)
            except Exception as exc:
                return web.json_response({"error": f"Falha ao ler {filename}: {exc}"}, status=400)
        elif lower_name.endswith(".pdf"):
            try:
                from pypdf import PdfReader
                import io

                pdf = PdfReader(io.BytesIO(content))
                text = "\n".join(page.extract_text() for page in pdf.pages if page.extract_text())
            except ImportError:
                return web.json_response({"error": "Leitura de PDF indisponivel no runtime atual."}, status=400)
            except Exception as exc:
                return web.json_response({"error": f"Falha ao extrair o PDF {filename}: {exc}"}, status=400)
        else:
            return web.json_response(
                {"error": "Formato nao suportado. Envie TXT, MD, JSON, CSV ou PDF."},
                status=400,
            )

        if not text or not text.strip():
            return web.json_response({"error": f"O arquivo {filename} nao possui texto legivel."}, status=400)

        title = Path(filename).stem
        result = await self.agent.add_document(title, text, source=f"upload:{filename}")
        # #region debug-point B:upload-agent-result
        _debug_event(
            "B",
            "webui/server.py:handle_upload:agent-result",
            "Agent upload flow completed",
            filename=filename,
            title=title,
            result=result,
        )
        # #endregion
        if result.lower().startswith("erro"):
            return web.json_response({"error": result}, status=500)
        return web.json_response({"result": result})

    async def handle_add_url(self, request: web.Request) -> web.Response:
        body = await request.json()
        url = body.get("url", "")
        if not url:
            return web.json_response({"error": "url is required"}, status=400)

        # Use the RAG server's add_url endpoint
        import httpx
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                resp = await client.post(
                    f"{self.agent.bge_url}/add_url",
                    json={"url": url},
                )
                if resp.status_code == 200:
                    data = resp.json()
                    return web.json_response({
                        "result": f"✅ {data.get('stored', 0)} chunks adicionados da URL."
                    })
                else:
                    return web.json_response({"result": f"Erro: {resp.text}"})
        except Exception as e:
            return web.json_response({"result": f"Erro ao processar URL: {e}"})

    async def handle_search_web(self, request: web.Request) -> web.Response:
        query = request.query.get("q", "")
        if not query:
            return web.json_response({"error": "query is required"}, status=400)

        domain = request.query.get("domain", "").strip()
        crawl_depth = 1
        try:
            crawl_depth = max(0, min(2, int(request.query.get("depth", "1"))))
        except ValueError:
            crawl_depth = 1

        try:
            # #region debug-point D:web-search-entry
            _debug_event(
                "D",
                "webui/server.py:handle_search_web:entry",
                "Web search request received",
                query=query,
                domain=domain,
                crawl_depth=crawl_depth,
            )
            # #endregion
            import httpx
            async with httpx.AsyncClient(timeout=httpx.Timeout(120.0, connect=10.0)) as client:
                resp = await client.post(
                    f"{self.agent.bge_url}/add_search",
                    json={
                        "query": query,
                        "domain": domain or None,
                        "crawl_depth": crawl_depth,
                    },
                )
            # #region debug-point D:web-search-rag-response
            _debug_event(
                "D",
                "webui/server.py:handle_search_web:rag-response",
                "RAG add_search completed",
                status_code=resp.status_code,
                response_text=resp.text[:1200],
            )
            # #endregion
            if resp.status_code != 200:
                return web.json_response({"result": f"Erro ao processar pesquisa web: {resp.text}"})

            payload = resp.json()
            mode = payload.get("mode", "search")
            sources = payload.get("sources") or []
            header = "🌐 Site indexado" if mode == "direct-url" or _looks_like_direct_web_target(query) else "🔎 Pesquisa absorvida"
            lines = [header, "", payload.get("message", "Pesquisa concluida.")]
            if sources:
                lines.append("")
                lines.append("Fontes efetivamente absorvidas:")
                lines.extend(f"- {source}" for source in sources[:6])
            return web.json_response({"result": "\n".join(lines)})
        except Exception as e:
            # #region debug-point D:web-search-error
            _debug_event(
                "D",
                "webui/server.py:handle_search_web:error",
                "Web search request failed",
                query=query,
                error=repr(e),
            )
            # #endregion
            return web.json_response({"result": f"Erro na pesquisa: {e}"})

    async def handle_stats(self, request: web.Request) -> web.Response:
        hw = self.agent.detect_hardware()
        stats = {"hardware": hw, "active_model": self.agent.model}

        try:
            import httpx
            async with httpx.AsyncClient(timeout=5.0) as client:
                resp = await client.get(f"{self.agent.bge_url}/stats")
                if resp.status_code == 200:
                    stats["knowledge"] = resp.json()
        except Exception:
            stats["knowledge"] = {"total_chunks": 0}

        return web.json_response(stats)

    async def handle_reset(self, request: web.Request) -> web.Response:
        self.agent.reset_history()
        return web.json_response({"status": "ok"})

    async def handle_list_skills(self, request: web.Request) -> web.Response:
        """List all available tools/skills."""
        tools = self.agent.tools.list_tools()
        skills = []
        for t in tools:
            skills.append({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
            })
        return web.json_response({"skills": skills})

    async def handle_get_personality(self, request: web.Request) -> web.Response:
        """Return current personality configuration."""
        try:
            text = self.agent.personality_path.read_text(encoding="utf-8")
        except FileNotFoundError:
            text = "# Personality not configured"
        return web.Response(text=text, content_type="text/plain", charset="utf-8")

    async def handle_set_personality(self, request: web.Request) -> web.Response:
        """Update personality configuration."""
        body = await request.json()

        # Read existing
        try:
            lines = self.agent.personality_path.read_text(encoding="utf-8").splitlines()
        except FileNotFoundError:
            lines = []

        # Track changes for each setting
        if "tone" in body:
            tone = body["tone"]
            new_lines = []
            for line in lines:
                for t in ["Professional", "Friendly", "Sarcastic", "Technical"]:
                    if t.lower() == tone.lower():
                        line = line.replace(f"[x] {t}", f"[ ] {t}") if "[x]" in line else line
                        if f"[ ] {t}" in line:
                            line = line.replace(f"[ ] {t}", f"[x] {t}")
                        break
                new_lines.append(line)
            lines = new_lines

        for slider in ["empathy", "verbosity", "humor", "tech_depth"]:
            if slider in body:
                val = int(body[slider])
                # Patterns for each slider
                patterns = {
                    "empathy": ("Low", "High"),
                    "verbosity": ("Concise", "Detailed"),
                    "humor": ("None", "Frequent"),
                    "tech_depth": ("Basic", "Expert"),
                }
                start_label, end_label = patterns[slider]
                new_lines = []
                for line in lines:
                    if start_label in line and end_label in line:
                        # Rebuild slider with mark at position val
                        slots = []
                        for i in range(5):
                            slots.append("[x]" if i + 1 == val else "[ ]")
                        line = f"{start_label} {' '.join(slots)} {end_label}"
                    new_lines.append(line)
                lines = new_lines

        if "custom" in body:
            # Find or create Custom Instruction section
            custom_text = body["custom"]
            found = False
            new_lines = []
            in_custom = False
            for line in lines:
                if line.startswith("## Custom Instruction"):
                    in_custom = True
                    new_lines.append(line)
                    new_lines.append(custom_text)
                    found = True
                    continue
                if in_custom:
                    if line.startswith("## "):
                        in_custom = False
                        new_lines.append(line)
                    else:
                        continue  # Skip old custom text
                else:
                    new_lines.append(line)
            if not found:
                new_lines.append("")
                new_lines.append("## Custom Instruction")
                new_lines.append(custom_text)
            lines = new_lines

        if "brave_key" in body:
            brave_key = body["brave_key"]
            found = False
            new_lines = []
            in_brave = False
            for line in lines:
                if line.startswith("## Brave API Key"):
                    in_brave = True
                    new_lines.append(line)
                    new_lines.append(brave_key if brave_key else "none")
                    found = True
                    continue
                if in_brave:
                    if line.startswith("## "):
                        in_brave = False
                        new_lines.append(line)
                    else:
                        continue
                else:
                    new_lines.append(line)
            if not found:
                new_lines.append("")
                new_lines.append("## Brave API Key")
                new_lines.append(brave_key if brave_key else "none")
            lines = new_lines

            # Also update the WebSearchTool API key in memory
            try:
                from tools.web import WebSearchTool
                WebSearchTool.BRAVE_API_KEY = brave_key
            except Exception:
                pass

        if "agent_name" in body:
            agent_name = body["agent_name"]
            found = False
            new_lines = []
            in_name = False
            for line in lines:
                if line.startswith("## Agent Name"):
                    in_name = True
                    new_lines.append(line)
                    new_lines.append(agent_name)
                    found = True
                    continue
                if in_name:
                    if line.startswith("## "):
                        in_name = False
                        new_lines.append(line)
                    else:
                        continue
                else:
                    new_lines.append(line)
            if not found:
                new_lines.append("")
                new_lines.append("## Agent Name")
                new_lines.append(agent_name)
            lines = new_lines

        # Write back
        text = "\n".join(lines)
        self.agent.personality_path.write_text(text, encoding="utf-8")

        # Reload personality in agent
        self.agent.reload_personality()

        return web.json_response({"status": "ok", "message": "Personalidade atualizada!"})

    def run(self):
        app = self.create_app()
        url = f"http://{self.host}:{self.port}"
        print(f"\nUSBRAIN Web interface: {url}")
        disable_browser_open = str(
            os.environ.get("USBRAIN_DISABLE_BROWSER_OPEN", "")
        ).strip().lower() in {"1", "true", "yes", "on"}
        if not disable_browser_open:
            threading.Timer(1.0, lambda: webbrowser.open(url)).start()
        web.run_app(app, host=self.host, port=self.port)
