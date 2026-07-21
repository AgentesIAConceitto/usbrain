"""Web search tool — uses Clawdbot-style web_search if available, falls back to direct web_fetch."""

from __future__ import annotations

import json
import os
import re
import subprocess
from typing import Any

import httpx

from tools.base import Tool


class WebSearchTool(Tool):
    """Search the web for information. Uses Brave Search API."""

    BRAVE_API_KEY = "BSAyaH63j2Mns6noio_RrQXjmiSUYD-"

    @property
    def name(self) -> str:
        return "web_search"

    @property
    def description(self) -> str:
        return "Search the web for current information. Returns results with titles, descriptions and URLs."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query",
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum results to return",
                    "default": 5,
                },
            },
            "required": ["query"],
        }

    async def execute(self, query: str, max_results: int = 5) -> str:
        # Try Brave Search (primary)
        result = await self._search_brave(query, max_results)
        if result:
            return result

        return f"Não foi possível buscar por '{query}'. Verifique a conexão com internet."

    async def _search_brave(self, query: str, max_results: int) -> str | None:
        """Search using Brave Search API."""
        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                resp = await client.get(
                    "https://api.search.brave.com/res/v1/web/search",
                    params={"q": query, "count": max_results, "country": "BR"},
                    headers={
                        "Accept": "application/json",
                        "Accept-Encoding": "gzip",
                        "X-Subscription-Token": self.BRAVE_API_KEY,
                    },
                )
                if resp.status_code != 200:
                    return None

                data = resp.json()
                results = data.get("web", {}).get("results", [])

                if not results:
                    return None

                output = []
                for item in results[:max_results]:
                    title = item.get("title", "")
                    desc = item.get("description", "")
                    url = item.get("url", "")
                    output.append(f"• **{title}**")
                    if desc:
                        output.append(f"  {desc[:300]}")
                    output.append(f"  _{url}_")
                    output.append("")

                return "\n".join(output).strip()

        except Exception as e:
            return None

    async def _search_google(self, query: str, max_results: int) -> str | None:
        """Search using Google HTML (scrape)."""
        try:
            headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
                "Accept": "text/html,application/xhtml+xml",
                "Accept-Language": "en-US,en;q=0.9,pt-BR;q=0.8",
            }
            async with httpx.AsyncClient(timeout=15.0, follow_redirects=True) as client:
                resp = await client.get(
                    "https://www.google.com/search",
                    params={"q": query, "hl": "en", "num": max_results},
                    headers=headers,
                )
                if resp.status_code != 200:
                    return None

                text = resp.text

                # Extract result blocks
                results = []

                # Pattern for modern Google SERP
                # Look for h3 tags (titles) followed by divs with data-sncf (snippets)
                titles = re.findall(r'<h3[^>]*>(.*?)</h3>', text)
                snippets = re.findall(r'<div[^>]*data-sncf[^>]*>(.*?)</div>', text)
                urls_raw = re.findall(r'<a[^>]*href="/url\?q=([^"&]+)', text)

                if titles:
                    for i, title in enumerate(titles[:max_results]):
                        title_clean = re.sub(r'<[^>]+>', '', title).strip()
                        snippet = ""
                        if i < len(snippets):
                            snippet = re.sub(r'<[^>]+>', '', snippets[i]).strip()
                        url = urls_raw[i] if i < len(urls_raw) else ""
                        results.append(f"• **{title_clean}**")
                        if snippet:
                            results.append(f"  {snippet[:200]}")
                        if url:
                            results.append(f"  _{url}_")
                        results.append("")

                if results:
                    return "\n".join(results).strip()

                # Alternative: simpler extraction
                alt_results = re.findall(
                    r'<div[^>]*class="[^"]*BNeawe[^"]*"[^>]*>(.*?)</div>',
                    text
                )
                if alt_results:
                    for r in alt_results[:max_results * 2]:
                        clean = re.sub(r'<[^>]+>', '', r).strip()
                        if len(clean) > 30:
                            results.append(f"• {clean[:200]}")
                    if results:
                        return "\n".join(results[:max_results])

                return None

        except Exception:
            return None
