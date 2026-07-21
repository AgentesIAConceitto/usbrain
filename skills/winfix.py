"""Skill: Windows troubleshooting — 100% offline. Comandos úteis pra diagnosticar Windows."""
from __future__ import annotations

import platform
import subprocess
from typing import Any

from tools.base import Tool


class WindowsFixTool(Tool):
    """Run common Windows repair commands. Only works on Windows."""

    @property
    def name(self) -> str:
        return "windows_fix"

    @property
    def description(self) -> str:
        return "Run common Windows repair commands: SFC, DISM, chkdsk, cleanmgr. Windows only."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["sfc", "dism", "chkdsk", "cleanmgr", "flushdns", "ipconfig_renew", "temp_files"],
                    "description": "What repair action to run",
                },
            },
            "required": ["action"],
        }

    async def execute(self, action: str = "sfc") -> str:
        if platform.system() != "Windows":
            return "❌ Este comando só funciona no Windows."

        commands = {
            "sfc": ("SFC / Scannow", "sfc /scannow", "Verificando integridade do sistema..."),
            "dism": ("DISM Restore Health", "dism /online /cleanup-image /restorehealth", "Restaurando imagem do Windows..."),
            "chkdsk": ("Check Disk (somente leitura)", "chkdsk C:", "Verificando disco C:..."),
            "flushdns": ("Limpar Cache DNS", "ipconfig /flushdns", "Limpando cache DNS..."),
            "ipconfig_renew": ("Renovar IP", "ipconfig /release && ipconfig /renew", "Renovando configuração IP..."),
            "temp_files": ("Limpar Arquivos Temporários", "del /q /s %temp%\\* 2>nul", "Limpando arquivos temporários..."),
            "cleanmgr": ("Limpeza de Disco", "cleanmgr /sagerun:1", "Abrindo limpeza de disco..."),
        }

        if action not in commands:
            return f"Ação desconhecida: {action}. Use: {', '.join(commands.keys())}"

        name, cmd, msg = commands[action]

        try:
            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=120)
            output = r.stdout or r.stderr or ""
            return f"### ⚙️ {name}\n{msg}\n\n```\n{output.strip()[:500]}```"
        except subprocess.TimeoutExpired:
            return f"⚠️ Comando {name} excedeu o tempo limite (120s). Pode estar rodando em segundo plano."
        except Exception as e:
            return f"Erro ao executar {name}: {e}"
