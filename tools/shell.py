"""Shell execution tool."""
from __future__ import annotations

import json
import os
import subprocess
import time
import urllib.request
from pathlib import Path
from typing import Any

from tools.base import Tool


def _debug_shell_event(message: str, **data: Any) -> None:
    env_path = Path(__file__).resolve().parent.parent / ".dbg" / "windows-shell-access.env"
    log_path = Path(__file__).resolve().parent.parent / "llamafile.log"
    payload = {
        "sessionId": "windows-shell-access",
        "runId": os.environ.get("USBRAIN_DEBUG_RUN_ID", "pre-fix"),
        "hypothesisId": "C",
        "location": "tools/shell.py:execute",
        "msg": f"[DEBUG] {message}",
        "data": data,
        "ts": int(time.time() * 1000),
    }
    try:
        with log_path.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(payload, ensure_ascii=True) + "\n")
    except Exception:
        pass
    if not env_path.exists():
        return

    debug_url = "http://127.0.0.1:7777/event"
    session_id = "windows-shell-access"
    try:
        for line in env_path.read_text(encoding="utf-8", errors="replace").splitlines():
            if line.startswith("DEBUG_SERVER_URL="):
                debug_url = line.split("=", 1)[1].strip() or debug_url
            elif line.startswith("DEBUG_SESSION_ID="):
                session_id = line.split("=", 1)[1].strip() or session_id
    except Exception:
        pass

    payload["sessionId"] = session_id
    try:
        req = urllib.request.Request(
            debug_url,
            data=json.dumps(payload, ensure_ascii=True).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        urllib.request.urlopen(req, timeout=1).read()
    except Exception:
        pass


class ShellTool(Tool):
    """Execute shell commands on the local system."""

    @property
    def name(self) -> str:
        return "shell"

    @property
    def description(self) -> str:
        return "Execute a shell command on the local system. Returns stdout and stderr."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute",
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 30,
                },
            },
            "required": ["command"],
        }

    async def execute(self, command: str, timeout: int = 30) -> str:
        _debug_shell_event(
            "Shell tool starting",
            command=command,
            timeout=timeout,
            cwd=os.getcwd(),
        )
        try:
            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=timeout,
            )
            _debug_shell_event(
                "Shell tool finished",
                command=command,
                returncode=result.returncode,
                stdout_preview=(result.stdout or "")[:600],
                stderr_preview=(result.stderr or "")[:600],
            )
            output = result.stdout
            if result.stderr:
                output += f"\n[stderr]\n{result.stderr}"
            if result.returncode != 0:
                output += f"\n[exit code: {result.returncode}]"
            return output.strip() or "(no output)"
        except subprocess.TimeoutExpired:
            _debug_shell_event(
                "Shell tool timed out",
                command=command,
                timeout=timeout,
            )
            return f"Command timed out after {timeout}s"
        except Exception as e:
            _debug_shell_event(
                "Shell tool raised exception",
                command=command,
                error_type=type(e).__name__,
                error_text=str(e),
            )
            return f"Error: {e}"
