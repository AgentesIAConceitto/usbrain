"""USBRAIN Agent — core orchestration layer."""
from __future__ import annotations

import json
import os
import platform
import re
import subprocess
import time
import urllib.error
import urllib.request
import asyncio
import unicodedata
from pathlib import Path
from typing import Any

import httpx

from tools.registry import ToolRegistry

MODEL_HINTS = [
    ("phi-3.5-mini", 8),
    ("llama-3.2-3b", 8),
    ("qwen2.5-3b", 8),
    ("qwen2.5-7b", 16),
    ("phi-4-14b", 16),
]


class USBAgent:
    """Main agent that ties together LLM, tools, and RAG."""

    def __init__(
        self,
        llm_base_url: str = "http://127.0.0.1:8080",
        bge_url: str = "http://127.0.0.1:9090",
        model: str = "phi-3.5-mini",
        knowledge_dir: str | None = None,
        db_path: str | None = None,
        personality_path: str | None = None,
    ):
        self.llm_base_url = llm_base_url
        self.bge_url = bge_url
        self.model = model
        self.knowledge_dir = Path(knowledge_dir) if knowledge_dir else Path("knowledge")
        self.knowledge_dir.mkdir(parents=True, exist_ok=True)
        self.db_path = db_path or "vetor.db"
        self.personality_path = Path(personality_path) if personality_path else Path("personality.md")

        # Tool registry
        self.tools = ToolRegistry()

        # Conversation history (simple rolling window)
        self.history: list[dict[str, Any]] = []
        self.max_history = 8
        self.llm_timeout_seconds = 600
        self.llm_simple_timeout_seconds = 600
        self.llm_max_tokens = 1024
        self.llm_simple_max_tokens = 1024

        # Load personality
        self._load_personality()

    def _llm_headers(self) -> dict[str, str]:
        """Build headers for OpenAI-compatible local LLM backends."""
        return {
            "Content-Type": "application/json",
            "Authorization": "Bearer no-key",
        }

    # #region debug-point A:llm-debug-report
    def _debug_report(self, hypothesis_id: str, location: str, message: str, **data: Any) -> None:
        """Send runtime debug evidence to the debug server when enabled."""
        env_path = Path(__file__).parent / ".dbg" / "llm-400-error.env"
        if (Path(__file__).parent / ".dbg" / "usb-eject-rag-lag.env").exists():
            env_path = Path(__file__).parent / ".dbg" / "usb-eject-rag-lag.env"
        if (Path(__file__).parent / ".dbg" / "usb-eject-upload-web.env").exists():
            env_path = Path(__file__).parent / ".dbg" / "usb-eject-upload-web.env"
        if (Path(__file__).parent / ".dbg" / "windows-shell-access.env").exists():
            env_path = Path(__file__).parent / ".dbg" / "windows-shell-access.env"
        log_path = Path(__file__).parent / "llamafile.log"
        debug_url = "http://127.0.0.1:7777/event"
        session_id = {
            "usb-eject-rag-lag.env": "usb-eject-rag-lag",
            "usb-eject-upload-web.env": "usb-eject-upload-web",
            "windows-shell-access.env": "windows-shell-access",
        }.get(env_path.name, "llm-400-error")
        try:
            if env_path.exists():
                for line in env_path.read_text(encoding="utf-8", errors="replace").splitlines():
                    if line.startswith("DEBUG_SERVER_URL="):
                        debug_url = line.split("=", 1)[1].strip() or debug_url
                    elif line.startswith("DEBUG_SESSION_ID="):
                        session_id = line.split("=", 1)[1].strip() or session_id
        except Exception:
            pass

        payload = {
            "sessionId": session_id,
            "runId": "pre-fix",
            "hypothesisId": hypothesis_id,
            "location": location,
            "msg": f"[DEBUG] {message}",
            "data": data,
            "ts": int(time.time() * 1000),
        }
        try:
            with log_path.open("a", encoding="utf-8") as fh:
                fh.write(json.dumps(payload, ensure_ascii=True) + "\n")
        except Exception:
            pass
        try:
            req = urllib.request.Request(
                debug_url,
                data=json.dumps(payload, ensure_ascii=True).encode("utf-8"),
                headers={"Content-Type": "application/json"},
            )
            urllib.request.urlopen(req, timeout=1).read()
        except Exception:
            pass

    def _debug_llm_payload(self, stage: str, body: dict[str, Any]) -> None:
        """Report a summarized view of the outgoing LLM payload."""
        messages = body.get("messages", [])
        tool_defs = body.get("tools") or []
        self._debug_report(
            "A",
            f"agent.py:{stage}",
            "LLM request payload prepared",
            stage=stage,
            llm_base_url=self.llm_base_url,
            model=body.get("model"),
            message_count=len(messages),
            last_message_role=(messages[-1].get("role") if messages else None),
            last_message_preview=((messages[-1].get("content", "") or "")[:300] if messages else ""),
            has_tools="tools" in body,
            tool_count=len(tool_defs),
            tool_names=[tool.get("function", {}).get("name") for tool in tool_defs[:20] if isinstance(tool, dict)],
            temperature=body.get("temperature"),
            max_tokens=body.get("max_tokens"),
        )

    def _debug_llm_error(self, stage: str, body: dict[str, Any], error: Exception) -> None:
        """Report details about a failed LLM HTTP request."""
        status_code = None
        response_text = ""
        if isinstance(error, httpx.HTTPStatusError):
            status_code = error.response.status_code
            try:
                response_text = error.response.text[:2000]
            except Exception:
                response_text = "<unavailable>"
        elif isinstance(error, urllib.error.HTTPError):
            status_code = error.code
            try:
                response_text = error.read().decode("utf-8", "replace")[:2000]
            except Exception:
                response_text = "<unavailable>"
        self._debug_report(
            "D" if stage == "chat-first-call" else "E",
            f"agent.py:{stage}",
            "LLM HTTP request failed",
            stage=stage,
            model=body.get("model"),
            has_tools="tools" in body,
            tool_count=len(body.get("tools") or []),
            message_count=len(body.get("messages") or []),
            status_code=status_code,
            error_type=type(error).__name__,
            error_text=str(error),
            response_text=response_text,
        )

    def _body_without_tools(self, body: dict[str, Any]) -> dict[str, Any]:
        """Return a copy of the request body without tool-calling fields."""
        stripped = dict(body)
        stripped.pop("tools", None)
        stripped.pop("tool_choice", None)
        return stripped

    def _clean_llm_content(self, content: str) -> str:
        """Remove meta commentary that should not be shown in the chat UI."""
        if not content:
            return ""

        cleaned_lines: list[str] = []
        for line in content.splitlines():
            normalized = line.strip().lower()
            if normalized.startswith("note:") or normalized.startswith("nota:"):
                continue
            cleaned_lines.append(line)

        cleaned = "\n".join(cleaned_lines)
        cleaned = re.sub(r"\n{3,}", "\n\n", cleaned).strip()
        return cleaned

    def _should_enable_tools(self, message: str) -> bool:
        """Enable tool-calling only for requests that imply real actions or diagnostics."""
        normalized = self._normalize_text(message)
        if not normalized:
            return False

        tool_keywords = [
            "analise", "análise", "diagnost", "diagnose", "erro", "problema",
            "windows", "pc", "computador", "arquivo", "pasta", "diretorio",
            "diretório", "comando", "terminal", "shell", "rede", "internet",
            "pesquise", "pesquisar", "buscar", "web", "site", "url",
            "instalar", "corrigir", "consertar", "resolver", "log", "logs",
            "cpu", "ram", "disco", "driver", "registro", "registry",
            "criar arquivo", "escrever arquivo", "ler arquivo", "copiar",
            "mover", "apagar", "deletar", "excluir", "listar pasta",
            "listar diretorio", "diretorio", "processo", "processos",
            "encerrar processo", "kill", "copie", "movimente", "delete",
            "zip", "compactar", "descompactar", "porta", "porta em uso",
            "variavel de ambiente", "environment", "env",
            "download", "baixar", "checksum", "hash", "sha256",
            "comparar arquivo", "diff", "substituir texto", "replace",
            "manifesto", "inventario", "comparar pastas", "comparar diretorios",
            "servico", "servicos", "daemon", "logs do sistema", "system log",
            "apagar em lote", "delete em lote",
        ]
        return any(keyword in normalized for keyword in tool_keywords)

    def _should_query_rag(self, message: str) -> bool:
        """Use RAG only for knowledge-seeking questions, not operational/tooling requests."""
        normalized = self._normalize_text(message)
        if not normalized:
            return False
        if self._is_capabilities_query(message):
            return False

        short_non_knowledge = {
            "oi", "ola", "olá", "bom dia", "boa tarde", "boa noite",
            "obrigado", "valeu", "ok", "teste", "ping",
        }
        if normalized in short_non_knowledge:
            return False

        operational_markers = [
            "arquivo", "pasta", "diretorio", "diretorio", "shell", "terminal",
            "comando", "processo", "servico", "log", "porta", "variavel de ambiente",
            "env", "copiar", "mover", "deletar", "apagar", "criar arquivo",
            "substituir texto", "download", "baixar", "checksum", "hash",
        ]
        if any(marker in normalized for marker in operational_markers):
            return False

        knowledge_markers = [
            "quem e", "quem é", "o que e", "o que é", "explique", "explica",
            "resuma", "resumir", "pesquise", "pesquisar", "aprenda", "aprender",
            "sobre ", "qual a historia", "qual a história", "como funciona",
            "me fale", "me diga", "qual foi", "quais sao", "quais são",
            "onde fica", "quando foi", "por que", "porque", "?",
        ]
        return any(marker in normalized for marker in knowledge_markers)

    def _tool_system_guidance(self) -> str:
        """Provide concise guidance so the LLM prefers concrete actions over generic talk."""
        tool_names = ", ".join(tool.name for tool in self.tools.list_tools())
        return (
            "Operational tool guidance:\n"
            f"- Available tools: {tool_names}\n"
            "- If the user asks to inspect, modify, create, delete, compare, download, diagnose, run commands or inspect the system, prefer using tools instead of answering abstractly.\n"
            "- When the request is operational, act first and then summarize the result.\n"
            "- If the request asks about your skills or capabilities, describe the real loaded tools and what they do.\n"
            "- Do not answer operational requests with generic programming knowledge when a tool can perform the action."
        )

    def _merge_stream_choice_delta(self, aggregated: dict[int, dict[str, Any]], delta: dict[str, Any]) -> None:
        """Accumulate streaming tool-call fragments into a final tool-call structure."""
        for item in delta.get("tool_calls") or []:
            index = int(item.get("index", 0))
            entry = aggregated.setdefault(
                index,
                {
                    "id": "",
                    "type": "function",
                    "function": {"name": "", "arguments": ""},
                },
            )
            if item.get("id"):
                entry["id"] = item["id"]
            if item.get("type"):
                entry["type"] = item["type"]
            fn = item.get("function") or {}
            if fn.get("name"):
                entry["function"]["name"] += fn["name"]
            if fn.get("arguments"):
                entry["function"]["arguments"] += fn["arguments"]

    def _consume_streaming_chat_response(self, response: Any, request_started_at: float) -> dict[str, Any]:
        """Aggregate an OpenAI-style SSE stream into a standard chat completion payload."""
        content_parts: list[str] = []
        tool_calls: dict[int, dict[str, Any]] = {}
        finish_reason = None
        chunk_count = 0
        first_chunk_at = None

        for raw_line in response:
            line = raw_line.decode("utf-8", "replace").strip()
            if not line or not line.startswith("data:"):
                continue
            payload = line[5:].strip()
            if not payload:
                continue
            if payload == "[DONE]":
                break
            chunk = json.loads(payload)
            chunk_count += 1
            if first_chunk_at is None:
                first_chunk_at = time.time()
            for choice in chunk.get("choices", []):
                delta = choice.get("delta") or {}
                if delta.get("content"):
                    content_parts.append(delta["content"])
                if delta.get("tool_calls"):
                    self._merge_stream_choice_delta(tool_calls, delta)
                if choice.get("finish_reason") is not None:
                    finish_reason = choice.get("finish_reason")

        message: dict[str, Any] = {
            "role": "assistant",
            "content": "".join(content_parts),
        }
        if tool_calls:
            message["tool_calls"] = [tool_calls[index] for index in sorted(tool_calls)]

        self._debug_report(
            "A",
            "agent.py:_consume_streaming_chat_response",
            "Aggregated streaming LLM response",
            chunk_count=chunk_count,
            content_chars=len(message.get("content") or ""),
            tool_call_count=len(message.get("tool_calls") or []),
            finish_reason=finish_reason,
            first_chunk_delay_ms=(round((first_chunk_at - request_started_at) * 1000, 1) if first_chunk_at else None),
            total_stream_ms=round((time.time() - request_started_at) * 1000, 1),
        )
        return {"choices": [{"message": message, "finish_reason": finish_reason}]}

    def _llm_post_sync(self, body: dict[str, Any]) -> dict[str, Any]:
        """Send a JSON request to the local LLM using stdlib HTTP for compatibility."""
        request_started_at = time.time()
        request_body = dict(body)
        request_body["stream"] = True
        req = urllib.request.Request(
            f"{self.llm_base_url}/v1/chat/completions",
            data=json.dumps(request_body, ensure_ascii=False).encode("utf-8"),
            headers=self._llm_headers(),
            method="POST",
        )
        timeout_seconds = self.llm_timeout_seconds if "tools" in body else self.llm_simple_timeout_seconds
        with urllib.request.urlopen(req, timeout=timeout_seconds) as response:
            content_type = (response.headers.get("Content-Type") or "").lower()
            if "text/event-stream" in content_type:
                return self._consume_streaming_chat_response(response, request_started_at)
            return json.loads(response.read().decode("utf-8", "replace"))
    # #endregion

    def _load_personality(self):
        """Load personality from file or use defaults."""
        personality = self._parse_personality_file()
        tone = personality.get("tone", "friendly")
        empathy = personality.get("empathy", 3)
        verbosity = personality.get("verbosity", 2)
        humor = personality.get("humor", 1)
        tech_depth = personality.get("tech_depth", 3)
        custom = personality.get("custom", "")

        tone_map = {
            "professional": "Professional, formal, and precise. Use proper terminology.",
            "friendly": "Friendly, warm, and approachable. Use casual greetings like 'Beleza?' or 'E aí?'. Be supportive.",
            "sarcastic": "Witty and sarcastic. Smart jokes, playful teasing, but always helpful.",
            "technical": "Extremely precise and technical. No fluff, no emotion. Pure data and commands.",
        }

        # Get Brave key and agent name from personality
        brave_key = personality.get("brave_key", "BSAyaH63j2Mns6noio_RrQXjmiSUYD-")
        agent_name = personality.get("agent_name", "USBRAIN")

        # Sync Brave key to WebSearchTool
        try:
            from tools.web import WebSearchTool
            WebSearchTool.BRAVE_API_KEY = brave_key
        except ImportError:
            pass

        tone_instruction = tone_map.get(tone, tone_map["friendly"])

        empathy_level = int(empathy)
        verbosity_level = int(verbosity)
        humor_level = int(humor)
        tech_level = int(tech_depth)

        # Build personality prompt
        parts = [
            f"You are {agent_name}, a portable AI assistant running from a USB drive.",
            "You help with technical problems, answer questions using your knowledge base, and can execute commands.",
            "",
            "## PERSONALITY",
            f"Tone: {tone_instruction}",
        ]

        if empathy_level >= 4:
            parts.append("Show high empathy. Acknowledge feelings, be supportive, use encouraging language.")
        elif empathy_level <= 2:
            parts.append("Be direct and to the point. Minimal emotional expression.")

        if verbosity_level <= 2:
            parts.append("Be concise. Short answers, get straight to the point.")
        elif verbosity_level >= 4:
            parts.append("Be detailed and thorough. Explain reasoning, provide context.")

        if humor_level >= 3:
            parts.append("Use light humor and occasional jokes. Make the conversation enjoyable.")

        if tech_level >= 4:
            parts.append("Deep technical detail. Assume expert knowledge. Use precise terminology.")
        elif tech_level <= 2:
            parts.append("Keep it simple. Explain technical concepts in plain language. Assume beginner level.")

        parts.extend([
            "",
            "## CORE RULES",
            "1. Respond in the same language as the user's question.",
            "2. ALWAYS format responses using Markdown:",
            "   - Use **bold** for emphasis",
            "   - Use `code` for commands, file paths, technical terms",
            "   - Use ```code blocks``` for multi-line commands or output",
            "   - Use - for lists",
            "   - Use ## for section headers when needed",
            "   - Use > for quotes or tips",
            "3. Keep responses well-structured with proper line breaks.",
            "4. Never output raw JSON or internal state unless asked.",
            "5. You have access to tools. Use them when appropriate.",
            "6. When answering, cite your sources if you used the knowledge base.",
            "7. If the user asks about your skills, tools, abilities or capabilities, describe concrete operational actions you can perform with your registered tools. Do not answer with programming languages, frameworks or generic technical knowledge unless the user asked specifically about them.",
        ])

        if custom:
            parts.extend(["", "## CUSTOM INSTRUCTION", custom])

        self.system_prompt = "\n".join(parts)

    def _parse_personality_file(self) -> dict[str, Any]:
        """Parse the personality.md file and extract settings."""
        defaults = {
            "tone": "friendly",
            "empathy": 3,
            "verbosity": 2,
            "humor": 1,
            "tech_depth": 3,
            "custom": "",
            "brave_key": "BSAyaH63j2Mns6noio_RrQXjmiSUYD-",
            "agent_name": "USBRAIN",
        }

        try:
            if not self.personality_path.exists():
                return defaults

            text = self.personality_path.read_text(encoding="utf-8")

            # Extract tone
            tone_match = re.search(r'\[x\]\s*(\w+)', text)
            if tone_match:
                defaults["tone"] = tone_match.group(1).lower()

            # Extract slider values
            empathy_match = re.search(r'Low\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*High', text)
            if empathy_match:
                for i in range(1, 6):
                    if '[x]' in empathy_match.group(i):
                        defaults["empathy"] = i
                        break

            verbosity_match = re.search(r'Concise\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*Detailed', text)
            if verbosity_match:
                for i in range(1, 6):
                    if '[x]' in verbosity_match.group(i):
                        defaults["verbosity"] = i
                        break

            humor_match = re.search(r'None\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*Frequent', text)
            if humor_match:
                for i in range(1, 6):
                    if '[x]' in humor_match.group(i):
                        defaults["humor"] = i
                        break

            tech_match = re.search(r'Basic\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*(\[[ x]\])\s*Expert', text)
            if tech_match:
                for i in range(1, 6):
                    if '[x]' in tech_match.group(i):
                        defaults["tech_depth"] = i
                        break

            # Extract custom instruction
            custom_match = re.search(r'## Custom Instruction\s*\n(.+?)(?:\n##|\Z)', text, re.DOTALL)
            if custom_match:
                defaults["custom"] = custom_match.group(1).strip()

            # Extract Brave API key
            brave_match = re.search(r'## Brave API Key\s*\n(.+)', text)
            if brave_match:
                key = brave_match.group(1).strip()
                if key and key != 'none' and not key.startswith('#'):
                    defaults["brave_key"] = key

            # Extract agent name
            name_match = re.search(r'## Agent Name\s*\n(.+)', text)
            if name_match:
                name = name_match.group(1).strip()
                if name and not name.startswith('#'):
                    defaults["agent_name"] = name

        except Exception:
            pass

        # Sync Brave key to WebSearchTool
        try:
            from tools.web import WebSearchTool
            WebSearchTool.BRAVE_API_KEY = defaults["brave_key"]
        except ImportError:
            pass

        return defaults

    def reload_personality(self):
        """Reload personality from file and rebuild system prompt."""
        self._load_personality()

    def reload_personality(self):
        """Reload personality from file and rebuild system prompt."""
        self._load_personality()

    # --- LLM Interaction ---

    async def chat(self, message: str, use_tools: bool = True) -> str:
        """Send a message to the LLM and get a response."""
        chat_started_at = time.time()

        # --- Local commands (no LLM needed) ---
        cmd_result = await self._handle_local_commands(message)
        if cmd_result:
            self.history.append({"role": "user", "content": message})
            self.history.append({"role": "assistant", "content": cmd_result})
            return cmd_result

        self.history.append({"role": "user", "content": message})

        enable_tools = use_tools and self._should_enable_tools(message)
        should_use_rag = self._should_query_rag(message)
        self._debug_report(
            "A",
            "agent.py:chat:tool-routing",
            "Computed tool routing decision",
            message_preview=(message[:220] if message else ""),
            enable_tools=enable_tools,
            should_use_rag=should_use_rag,
            registered_tool_names=[tool.name for tool in self.tools.list_tools()],
        )

        # First, check RAG for relevant context for any normal user question.
        # #region debug-point C:chat-rag-start
        self._debug_report(
            "C",
            "agent.py:chat:before-rag",
            "Chat request starting RAG lookup",
            message_preview=(message[:220] if message else ""),
            enable_tools=enable_tools,
            should_use_rag=should_use_rag,
            history_size=len(self.history),
        )
        # #endregion
        rag_started_at = time.time()
        rag_context = await self._query_rag(message) if should_use_rag else ""
        rag_elapsed_ms = round((time.time() - rag_started_at) * 1000, 1)
        # #region debug-point C:chat-rag-finish
        self._debug_report(
            "C",
            "agent.py:chat:after-rag",
            "Chat request finished RAG lookup",
            rag_elapsed_ms=rag_elapsed_ms,
            rag_context_chars=len(rag_context),
            rag_context_sources=rag_context.count("[Fonte:"),
            rag_skipped=not should_use_rag,
        )
        # #endregion

        messages = [{"role": "system", "content": self.system_prompt}]
        if enable_tools and self.tools.list_tools():
            messages.append({"role": "system", "content": self._tool_system_guidance()})
        if rag_context:
            messages.append({
                "role": "system",
                "content": (
                    "Relevant knowledge base context:\n"
                    f"{rag_context}\n\n"
                    "Use this context as the primary source of truth for the answer. "
                    "If the context is insufficient, say clearly that the knowledge base does not contain enough information yet. "
                    "Do not invent facts that are not supported by the retrieved context."
                )
            })

        # Add conversation history
        for msg in self.history[-self.max_history:]:
            messages.append(msg)

        # Prepare request
        body = {
            "model": self.model,
            "messages": messages,
            "temperature": 0.2 if rag_context else 0.7,
            "max_tokens": self.llm_max_tokens if enable_tools else self.llm_simple_max_tokens,
        }

        if enable_tools and self.tools.list_tools():
            body["tools"] = self.tools.to_openai_schemas()
            body["tool_choice"] = "auto"

        # Call LLM
        try:
            # #region debug-point A:first-llm-call
            self._debug_llm_payload("chat-first-call", body)
            # #endregion
            llm_started_at = time.time()
            data = await asyncio.to_thread(self._llm_post_sync, body)
            # #region debug-point D:chat-first-llm-finish
            self._debug_report(
                "D",
                "agent.py:chat:first-llm-finish",
                "First LLM call completed",
                llm_elapsed_ms=round((time.time() - llm_started_at) * 1000, 1),
                total_chat_elapsed_ms=round((time.time() - chat_started_at) * 1000, 1),
                used_rag=bool(rag_context),
                used_tools=enable_tools,
            )
            # #endregion

            choice = data["choices"][0]
            msg_resp = choice["message"]

            # Check for tool calls
            if "tool_calls" in msg_resp and msg_resp["tool_calls"]:
                return await self._handle_tool_calls(msg_resp, messages)

            content = self._clean_llm_content(msg_resp.get("content", "") or "")
            self._debug_report(
                "D",
                "agent.py:chat:no-tool-call",
                "LLM answered without tool calls",
                message_preview=(message[:220] if message else ""),
                enable_tools=enable_tools,
                response_chars=len(content),
                response_preview=content[:400],
                finish_reason=choice.get("finish_reason"),
            )
            self.history.append({"role": "assistant", "content": content})
            # #region debug-point D:chat-finish
            self._debug_report(
                "D",
                "agent.py:chat:finish",
                "Chat request finished",
                total_chat_elapsed_ms=round((time.time() - chat_started_at) * 1000, 1),
                answer_chars=len(content),
                used_rag=bool(rag_context),
                used_tools=enable_tools,
            )
            # #endregion
            return content

        except Exception as e:
            if (
                (
                    isinstance(e, httpx.HTTPStatusError)
                    and e.response.status_code == 400
                )
                or (
                    isinstance(e, urllib.error.HTTPError)
                    and e.code == 400
                )
            ) and "tools" in body:
                retry_body = self._body_without_tools(body)
                # #region debug-point D:tools-fallback
                self._debug_report(
                    "D",
                    "agent.py:chat-first-call",
                    "Retrying LLM request without tools after 400",
                    original_tool_count=len(body.get("tools") or []),
                    retry_message_count=len(retry_body.get("messages") or []),
                )
                self._debug_llm_payload("chat-first-call-retry-no-tools", retry_body)
                # #endregion
                try:
                    retry_data = await asyncio.to_thread(self._llm_post_sync, retry_body)

                    content = self._clean_llm_content(
                        retry_data["choices"][0]["message"].get("content", "") or ""
                    )
                    self.history.append({"role": "assistant", "content": content})
                    return content
                except Exception as retry_error:
                    self._debug_llm_error("chat-first-call-retry-no-tools", retry_body, retry_error)
            # #region debug-point D:first-llm-error
            self._debug_llm_error("chat-first-call", body, e)
            # #endregion
            error_msg = f"Error communicating with LLM: {e}"
            return error_msg

    async def _handle_tool_calls(
        self, msg: dict[str, Any], messages: list[dict[str, Any]]
    ) -> str:
        """Handle tool calls from the LLM response."""
        messages.append({"role": "assistant", "content": msg.get("content", ""), "tool_calls": msg["tool_calls"]})
        self._debug_report(
            "B",
            "agent.py:_handle_tool_calls",
            "Handling tool calls from LLM",
            tool_call_count=len(msg["tool_calls"]),
            tool_names=[
                ((tc.get("function") or {}).get("name"))
                for tc in msg["tool_calls"]
            ],
        )

        for tc in msg["tool_calls"]:
            fn_name = tc["function"]["name"]
            try:
                fn_args = json.loads(tc["function"]["arguments"])
            except json.JSONDecodeError:
                fn_args = {}
            self._debug_report(
                "B",
                "agent.py:_handle_tool_calls:execute",
                "Executing tool call",
                tool_name=fn_name,
                tool_args=fn_args,
            )

            result = await self.tools.execute(fn_name, **fn_args)
            self._debug_report(
                "C",
                "agent.py:_handle_tool_calls:result",
                "Tool call finished",
                tool_name=fn_name,
                result_chars=len(str(result)),
                result_preview=str(result)[:600],
            )
            messages.append({
                "role": "tool",
                "tool_call_id": tc["id"],
                "content": str(result),
            })

        # Send tool results back to LLM for final response
        body = {
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": self.llm_max_tokens,
        }

        try:
            # #region debug-point E:second-llm-call
            self._debug_llm_payload("chat-after-tools", body)
            # #endregion
            data = await asyncio.to_thread(self._llm_post_sync, body)

            final_content = self._clean_llm_content(
                data["choices"][0]["message"].get("content", "") or ""
            )
            self.history.append({"role": "assistant", "content": final_content})
            return final_content
        except Exception as e:
            # #region debug-point E:second-llm-error
            self._debug_llm_error("chat-after-tools", body, e)
            # #endregion
            return f"Error after tool calls: {e}"

    # --- RAG ---

    async def _query_rag(self, query: str, top_k: int = 5, min_similarity: float = 0.28) -> str:
        """Query the vector database for relevant context."""
        started_at = time.time()
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                # Get embedding for query
                embed_started_at = time.time()
                embed_resp = await client.post(
                    f"{self.bge_url}/embed",
                    json={"text": query},
                )
                if embed_resp.status_code != 200:
                    return ""
                embedding = embed_resp.json()["embedding"]

                embed_elapsed_ms = round((time.time() - embed_started_at) * 1000, 1)

                # Search vector DB
                semantic_started_at = time.time()
                search_resp = await client.post(
                    f"{self.bge_url}/search",
                    json={"embedding": embedding, "top_k": top_k},
                )
                if search_resp.status_code != 200:
                    return ""
                semantic_results = search_resp.json().get("results", [])
                semantic_elapsed_ms = round((time.time() - semantic_started_at) * 1000, 1)

                lexical_started_at = time.time()
                lexical_resp = await client.post(
                    f"{self.bge_url}/search_text",
                    json={"query": query, "top_k": top_k},
                )
                lexical_results = lexical_resp.json().get("results", []) if lexical_resp.status_code == 200 else []
                lexical_elapsed_ms = round((time.time() - lexical_started_at) * 1000, 1)

            if not semantic_results and not lexical_results:
                self._debug_report(
                    "R",
                    "agent.py:_query_rag:no-results",
                    "Vector search returned no results",
                    query=query,
                )
                return ""

            filtered_semantic_results = [
                result for result in semantic_results
                if float(result.get("similarity", 0.0) or 0.0) >= min_similarity
            ]
            merged_results: list[dict[str, Any]] = []
            seen_keys: set[tuple[str, str]] = set()
            for result in filtered_semantic_results + lexical_results:
                key = (result.get("title", ""), result.get("source", ""))
                if key in seen_keys:
                    continue
                seen_keys.add(key)
                merged_results.append(result)
            self._debug_report(
                "R",
                "agent.py:_query_rag:results",
                "Vector search completed",
                query=query,
                semantic_returned=len(semantic_results),
                semantic_kept=len(filtered_semantic_results),
                lexical_returned=len(lexical_results),
                merged_kept=len(merged_results),
                embed_elapsed_ms=embed_elapsed_ms,
                semantic_elapsed_ms=semantic_elapsed_ms,
                lexical_elapsed_ms=lexical_elapsed_ms,
                total_rag_elapsed_ms=round((time.time() - started_at) * 1000, 1),
                top_results=[
                    {
                        "title": item.get("title", ""),
                        "source": item.get("source", ""),
                        "similarity": item.get("similarity", 0.0),
                    }
                    for item in (filtered_semantic_results + lexical_results)[:5]
                ],
            )
            if not merged_results:
                return ""

            # Format context
            chunks = []
            for r in merged_results[:top_k]:
                chunks.append(f"[Fonte: {r.get('title', 'desconhecida')}]\n{r.get('content', '')}")

            return "\n\n---\n\n".join(chunks)

        except Exception as error:
            self._debug_report(
                "R",
                "agent.py:_query_rag:error",
                "Vector search failed",
                query=query,
                error=repr(error),
            )
            return ""

    async def _handle_local_commands(self, message: str) -> str | None:
        """Handle commands without LLM. Returns response or None."""
        cmd = message.strip().lower()

        if cmd in ("/help", "/skills", "/tools", "!help", "!skills", "!tools", "comandos", "skills") or self._is_capabilities_query(message):
            self._debug_report(
                "A",
                "agent.py:_handle_local_commands",
                "Handled message via local capabilities command",
                message_preview=(message[:220] if message else ""),
                normalized_cmd=cmd,
            )
            return self._render_capabilities_overview()

        if cmd in ("/reset", "!reset", "limpar", "nova conversa"):
            self.reset_history()
            return "🔄 Conversa reiniciada. Como posso ajudar?"

        if cmd in ("/stats", "!stats", "status", "info"):
            hw = self.detect_hardware()
            return (
                f"## 📊 USBRAIN Status\n\n"
                f"- **Modelo:** {self.model}\n"
                f"- **RAM:** {hw['ram_gb']}GB\n"
                f"- **SO:** {hw['os']}\n"
                f"- **Skills:** {len(self.tools.list_tools())}\n"
                f"- **Histórico:** {len(self.history)} mensagens"
            )

        if cmd.startswith("!") or cmd.startswith("/"):
            return f"Comando desconhecido: `{cmd}`. Tente `/help` para ver os comandos disponíveis."

        return None

    @staticmethod
    def _normalize_text(value: str) -> str:
        normalized = unicodedata.normalize("NFKD", value or "")
        normalized = "".join(ch for ch in normalized if not unicodedata.combining(ch))
        return normalized.lower().strip()

    def _is_capabilities_query(self, message: str) -> bool:
        normalized = self._normalize_text(message)
        patterns = [
            "quais skills",
            "quais sao suas skills",
            "que skills voce tem",
            "quais ferramentas",
            "que ferramentas voce tem",
            "quais sao suas ferramentas",
            "suas ferramentas",
            "suas habilidades",
            "suas capacidades",
            "quais capacidades",
            "quais habilidades",
            "o que voce consegue fazer",
            "o que voce pode fazer",
            "o que sabe fazer",
            "quais recursos voce tem",
            "ferramentas do agente",
            "skills do agente",
        ]
        return any(pattern in normalized for pattern in patterns)

    def _render_capabilities_overview(self) -> str:
        tools = {tool.name: tool for tool in self.tools.list_tools()}
        groups: list[tuple[str, list[str], str]] = [
            (
                "Arquivos e disco",
                ["list_directory", "read_file", "write_file", "create_directory", "copy_path", "move_path", "create_zip", "extract_zip", "download_url", "file_checksum", "compare_files", "replace_in_file", "directory_manifest", "compare_directories", "batch_delete", "delete_path", "file_search", "file_info"],
                "Ler, criar, listar, localizar, copiar, mover, compactar, descompactar, baixar, comparar, inventariar, editar e apagar arquivos e pastas no disco local.",
            ),
            (
                "Execucao local",
                ["shell"],
                "Executar comandos no sistema operacional local, incluindo Windows e macOS quando o host permitir.",
            ),
            (
                "Sistema e diagnostico",
                ["system_info", "process_list", "kill_process", "environment", "port_inspect", "service_list", "system_logs", "network_diagnose", "windows_fix"],
                "Inspecionar hardware, memoria, disco, processos, ambiente, portas, servicos, logs de sistema e rodar rotinas de suporte do sistema.",
            ),
            (
                "Web e pesquisa",
                ["web_search"],
                "Pesquisar na web e trazer resultados atuais quando houver internet.",
            ),
        ]

        used_names: set[str] = set()
        lines = ["## Capacidades Operacionais do Agente", ""]
        lines.append("Estas sao as capacidades reais carregadas agora no USBRAIN, no estilo de um agente operacional baseado no core do Nanobot.")

        for title, tool_names, summary in groups:
            available = [tools[name] for name in tool_names if name in tools]
            if not available:
                continue
            used_names.update(tool.name for tool in available)
            lines.append("")
            lines.append(f"### {title}")
            lines.append(f"- {summary}")
            for tool in available:
                lines.append(f"- `{tool.name}`: {tool.description}")

        remaining = [tool for name, tool in tools.items() if name not in used_names]
        if remaining:
            lines.append("")
            lines.append("### Outras skills carregadas")
            for tool in remaining:
                lines.append(f"- `{tool.name}`: {tool.description}")

        lines.append("")
        lines.append("### Comandos de chat")
        lines.append("- `/help`, `/skills`, `/tools`: listar capacidades reais do agente")
        lines.append("- `/reset`: limpar o historico da conversa")
        lines.append("- `/stats`: mostrar status do agente, modelo e hardware")
        lines.append("")
        lines.append("### Exemplos do que este agente pode fazer")
        lines.append("- Ler um arquivo de configuracao ou log")
        lines.append("- Criar ou atualizar arquivos no disco")
        lines.append("- Criar pastas, copiar e mover arquivos")
        lines.append("- Compactar e extrair arquivos `.zip`")
        lines.append("- Baixar arquivo por URL e salvar no disco")
        lines.append("- Gerar hash/checksum e comparar arquivos")
        lines.append("- Substituir texto dentro de arquivos")
        lines.append("- Gerar manifesto de pasta e comparar diretorios")
        lines.append("- Apagar multiplos arquivos ou pastas em lote")
        lines.append("- Apagar arquivo ou pasta")
        lines.append("- Listar conteudo de diretorios")
        lines.append("- Executar comandos de shell no host")
        lines.append("- Listar processos e encerrar processo por PID ou nome")
        lines.append("- Ler, listar e definir variaveis de ambiente")
        lines.append("- Inspecionar portas abertas e listeners locais")
        lines.append("- Listar servicos e coletar logs recentes do sistema")
        lines.append("- Buscar arquivos por nome ou conteudo")
        lines.append("- Diagnosticar rede, sistema e Windows")
        lines.append("- Pesquisar na web e usar a base de conhecimento")
        return "\n".join(lines)

    async def add_document(self, title: str, content: str, source: str = "upload") -> str:
        """Add a document to the knowledge base."""
        try:
            # #region debug-point B:add-document-entry
            self._debug_report(
                "B",
                "agent.py:add_document:entry",
                "Starting add_document flow",
                title=title,
                source=source,
                content_chars=len(content),
            )
            # #endregion
            doc_path = self.knowledge_dir / f"{title}.md"
            doc_path.write_text(content, encoding="utf-8")

            chunks = self._chunk_text(content, title)
            # #region debug-point B:add-document-chunks
            self._debug_report(
                "B",
                "agent.py:add_document:chunks",
                "Document chunked for embedding",
                title=title,
                chunk_count=len(chunks),
                bge_url=self.bge_url,
            )
            # #endregion
            stored_chunks = 0
            async with httpx.AsyncClient(timeout=httpx.Timeout(180.0, connect=10.0)) as client:
                for chunk in chunks:
                    embed_resp = await client.post(
                        f"{self.bge_url}/embed",
                        json={"text": chunk["content"]},
                    )
                    # #region debug-point B:add-document-embed-response
                    self._debug_report(
                        "B",
                        "agent.py:add_document:embed-response",
                        "Embed endpoint responded during add_document",
                        title=title,
                        chunk_title=chunk["title"],
                        status_code=embed_resp.status_code,
                        body=embed_resp.text[:600],
                    )
                    # #endregion
                    if embed_resp.status_code != 200:
                        continue
                    embedding = embed_resp.json()["embedding"]

                    # Store in vector DB
                    store_resp = await client.post(
                        f"{self.bge_url}/store",
                        json={
                            "title": chunk["title"],
                            "content": chunk["content"],
                            "source": source,
                            "embedding": embedding,
                        },
                    )
                    if store_resp.status_code == 200:
                        stored_chunks += 1
                    # #region debug-point B:add-document-store-response
                    self._debug_report(
                        "B",
                        "agent.py:add_document:store-response",
                        "Store endpoint responded during add_document",
                        title=title,
                        chunk_title=chunk["title"],
                        status_code=store_resp.status_code,
                        body=store_resp.text[:600],
                    )
                    # #endregion

            return f"✅ Documento '{title}' adicionado à base de conhecimento ({stored_chunks}/{len(chunks)} chunks)."
        except Exception as e:
            # #region debug-point B:add-document-error
            self._debug_report(
                "B",
                "agent.py:add_document:error",
                "add_document raised an exception",
                title=title,
                error=repr(e),
            )
            # #endregion
            return f"Erro ao adicionar documento: {e}"

    @staticmethod
    def _chunk_text(text: str, title: str, chunk_size: int = 140, overlap_words: int = 24) -> list[dict[str, str]]:
        """Split text into overlapping chunks sized for CPU-only embedding."""
        words = text.split()
        chunks = []
        step = max(1, chunk_size - overlap_words)
        for i in range(0, len(words), step):
            chunk_text = " ".join(words[i : i + chunk_size])
            if not chunk_text.strip():
                continue
            chunk_title = f"{title} (parte {len(chunks) + 1})"
            chunks.append({"title": chunk_title, "content": chunk_text})
            if i + chunk_size >= len(words):
                break
        return chunks

    async def web_search_and_learn(self, query: str) -> str:
        """Search the web and add results to knowledge base."""
        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                # Use DuckDuckGo or similar
                search_url = f"https://api.duckduckgo.com/?q={query}&format=json"
                resp = await client.get(search_url)
                if resp.status_code != 200:
                    return f"Erro na pesquisa: {resp.status_code}"

                data = resp.json()
                results = []
                for topic in data.get("RelatedTopics", [])[:5]:
                    if "Text" in topic:
                        results.append(topic["Text"])

            if not results:
                return "Nenhum resultado encontrado."

            # Add to knowledge base
            combined = "\n".join(results)
            title = query.replace(" ", "-")[:50]
            return await self.add_document(title, combined, source="web_search")

        except Exception as e:
            return f"Erro na pesquisa web: {e}"

    # --- System info ---

    @staticmethod
    def detect_hardware() -> dict[str, Any]:
        """Detect system RAM and recommend model."""
        system = platform.system()
        try:
            if system == "Windows":
                import ctypes
                class MEMORYSTATUSEX(ctypes.Structure):
                    _fields_ = [
                        ("dwLength", ctypes.c_ulong),
                        ("dwMemoryLoad", ctypes.c_ulong),
                        ("ullTotalPhys", ctypes.c_ulonglong),
                        ("ullAvailPhys", ctypes.c_ulonglong),
                        ("ullTotalPageFile", ctypes.c_ulonglong),
                        ("ullAvailPageFile", ctypes.c_ulonglong),
                        ("ullTotalVirtual", ctypes.c_ulonglong),
                        ("ullAvailVirtual", ctypes.c_ulonglong),
                        ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
                    ]

                mem = MEMORYSTATUSEX()
                mem.dwLength = ctypes.sizeof(MEMORYSTATUSEX)
                if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(mem)):
                    raise OSError("GlobalMemoryStatusEx falhou")
                mem_bytes = mem.ullTotalPhys
            elif system == "Darwin":
                import subprocess
                result = subprocess.run(
                    ["sysctl", "-n", "hw.memsize"],
                    capture_output=True, text=True
                )
                mem_bytes = int(result.stdout.strip())
            else:
                result = subprocess.run(
                    ["grep", "MemTotal", "/proc/meminfo"],
                    capture_output=True, text=True
                )
                if result.stdout:
                    mem_kb = int(result.stdout.split()[1])
                    mem_bytes = mem_kb * 1024
                else:
                    mem_bytes = 8 * 1024**3  # default 8GB
        except Exception:
            mem_bytes = 8 * 1024**3  # fallback

        ram_gb = mem_bytes / (1024**3)

        model = "phi-3.5-mini"
        for candidate, min_ram_gb in reversed(MODEL_HINTS):
            if ram_gb >= min_ram_gb:
                model = candidate
                break

        return {
            "ram_gb": round(ram_gb, 1),
            "recommended_model": model,
            "os": system,
            "cpu": platform.machine(),
        }

    def reset_history(self):
        """Clear conversation history."""
        self.history = []
