"""Skill: File search & content grep — 100% offline."""
from __future__ import annotations

import os
import platform
import subprocess
from pathlib import Path
from typing import Any

from tools.base import Tool


class FileSearchTool(Tool):
    """Search files by name or content. No internet required."""

    @property
    def name(self) -> str:
        return "file_search"

    @property
    def description(self) -> str:
        return "Search files by name or content. Useful for finding configs, logs, documents."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "File name pattern or text to search (e.g. '*.log', 'error.txt')",
                },
                "search_type": {
                    "type": "string",
                    "enum": ["name", "content"],
                    "description": "Search by file name or file content",
                    "default": "name",
                },
                "directory": {
                    "type": "string",
                    "description": "Directory to search in. Default: C:\\ or /",
                    "default": "",
                },
                "max_results": {
                    "type": "integer",
                    "description": "Max results to return",
                    "default": 10,
                },
            },
            "required": ["pattern"],
        }

    async def execute(self, pattern: str, search_type: str = "name", directory: str = "", max_results: int = 10) -> str:
        system = platform.system()
        search_dir = directory or (os.environ.get("HOME") or os.path.expanduser("~"))

        try:
            if search_type == "name":
                return self._search_by_name(pattern, search_dir, system, max_results)
            else:
                return self._search_by_content(pattern, search_dir, system, max_results)
        except Exception as e:
            return f"Error searching: {e}"

    @staticmethod
    def _search_by_name(pattern: str, directory: str, system: str, max_results: int) -> str:
        if system == "Windows":
            cmd = f'dir /s /b "{directory}\\{pattern}" 2>nul'
        else:
            cmd = f'find {directory} -name "{pattern}" -type f 2>/dev/null | head -{max_results}'

        r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=15)
        results = r.stdout.strip()
        if not results:
            return f"Nenhum arquivo encontrado com padrão '{pattern}' em {directory}"
        lines = results.splitlines()
        return f"### 📁 {len(lines)} arquivo(s) encontrado(s):\n" + "\n".join(lines[:max_results])

    @staticmethod
    def _search_by_content(pattern: str, directory: str, system: str, max_results: int) -> str:
        if system == "Windows":
            cmd = f'findstr /s /m /i "{pattern}" "{directory}\\*" 2>nul'
        else:
            cmd = f'grep -r -l -i "{pattern}" {directory} 2>/dev/null | head -{max_results}'

        r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30)
        results = r.stdout.strip()
        if not results:
            return f"Nenhum arquivo contendo '{pattern}' encontrado em {directory}"
        lines = results.splitlines()
        return f"### 📁 {len(lines)} arquivo(s) com '{pattern}':\n" + "\n".join(lines[:max_results])


class FileInfoTool(Tool):
    """Get detailed file/folder info. 100% offline."""

    @property
    def name(self) -> str:
        return "file_info"

    @property
    def description(self) -> str:
        return "Get detailed info about a file or folder: size, permissions, last modified."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to file or folder",
                },
            },
            "required": ["path"],
        }

    async def execute(self, path: str) -> str:
        try:
            p = Path(path).expanduser().resolve()
            if not p.exists():
                return f"Path not found: {path}"

            if p.is_file():
                stat = p.stat()
                size = self._format_size(stat.st_size)
                modified = self._format_time(stat.st_mtime)
                return (
                    f"**📄 {p.name}**\n"
                    f"- Path: {p.parent}\n"
                    f"- Size: {size}\n"
                    f"- Modified: {modified}\n"
                    f"- Ext: {p.suffix}"
                )
            else:
                items = list(p.iterdir())
                total_size = sum(f.stat().st_size for f in items if f.is_file())
                dirs = sum(1 for f in items if f.is_dir())
                files = len(items) - dirs
                return (
                    f"**📁 {p.name}**\n"
                    f"- Path: {p}\n"
                    f"- {dirs} pastas, {files} arquivos\n"
                    f"- Tamanho total: {self._format_size(total_size)}"
                )
        except Exception as e:
            return f"Error: {e}"

    @staticmethod
    def _format_size(size: int) -> str:
        for unit in ["B", "KB", "MB", "GB"]:
            if size < 1024:
                return f"{size:.1f} {unit}"
            size /= 1024
        return f"{size:.1f} TB"

    @staticmethod
    def _format_time(ts: float) -> str:
        from datetime import datetime
        return datetime.fromtimestamp(ts).strftime("%d/%m/%Y %H:%M")
