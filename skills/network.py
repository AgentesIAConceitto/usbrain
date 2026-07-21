"""Skill: Network diagnostics — 100% offline."""
from __future__ import annotations

import platform
import subprocess
from typing import Any

from tools.base import Tool


class NetworkDiagnoseTool(Tool):
    """Diagnose network issues on Windows/Linux/Mac. No internet required."""

    @property
    def name(self) -> str:
        return "network_diagnose"

    @property
    def description(self) -> str:
        return "Run network diagnostics: ping, DNS, gateway, interfaces. Use when user reports 'no internet' or 'network down'."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Host to test (IP or domain). Default: 8.8.8.8",
                    "default": "8.8.8.8",
                },
                "diagnose": {
                    "type": "string",
                    "enum": ["full", "ping", "dns", "gateway", "interfaces"],
                    "description": "What to diagnose",
                    "default": "full",
                },
            },
        }

    async def execute(self, target: str = "8.8.8.8", diagnose: str = "full") -> str:
        system = platform.system()
        results = []

        if diagnose in ("full", "interfaces"):
            results.append("## 🌐 Interfaces de Rede")
            if system == "Windows":
                r = self._run("ipconfig /all")
            elif system == "Darwin":
                r = self._run("ifconfig")
            else:
                r = self._run("ip addr show")
            results.append(self._truncate(r))

        if diagnose in ("full", "gateway"):
            results.append("## 🚪 Gateway Padrão")
            if system == "Windows":
                r = self._run("ipconfig | findstr Gateway")
            elif system == "Darwin":
                r = self._run("netstat -rn | grep default")
            else:
                r = self._run("ip route | grep default")
            results.append(r or "Nenhum gateway encontrado")

        if diagnose in ("full", "ping"):
            results.append(f"## 📡 Ping {target}")
            count_flag = "-n" if system == "Windows" else "-c"
            r = self._run(f"ping {count_flag} 4 {target}")
            results.append(r or "Ping falhou (firewall ou sem rede)")

        if diagnose in ("full", "dns"):
            results.append("## 🔍 DNS Resolution")
            r = self._run(f"nslookup google.com" if system == "Windows" else "nslookup google.com 2>/dev/null || host google.com")
            results.append(r or "DNS resolution failed")

        return "\n\n".join(results)

    @staticmethod
    def _run(cmd: str) -> str:
        try:
            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=10)
            output = r.stdout or r.stderr or ""
            return output.strip()[:800]
        except Exception as e:
            return f"Error: {e}"

    @staticmethod
    def _truncate(text: str, max_len: int = 600) -> str:
        if len(text) > max_len:
            return text[:max_len] + "\n... [truncado]"
        return text
