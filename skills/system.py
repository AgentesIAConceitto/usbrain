"""Skill: System info & diagnostics — 100% offline."""
from __future__ import annotations

import os
import platform
import shutil
import subprocess
from typing import Any

from tools.base import Tool


class SystemInfoTool(Tool):
    """Get detailed system information. No internet required."""

    @property
    def name(self) -> str:
        return "system_info"

    @property
    def description(self) -> str:
        return "Get detailed system info: OS, CPU, RAM, disk usage, processes, uptime."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "info_type": {
                    "type": "string",
                    "enum": ["full", "os", "cpu", "memory", "disk", "processes", "uptime"],
                    "description": "What info to retrieve",
                    "default": "full",
                },
            },
        }

    async def execute(self, info_type: str = "full") -> str:
        system = platform.system()
        results = []

        if info_type in ("full", "os"):
            results.append(f"## 💻 Sistema Operacional\n{system} {platform.release()} {platform.version()}")

        if info_type in ("full", "cpu"):
            cpu_info = self._get_cpu_info(system)
            results.append(f"## 🔧 CPU\n{cpu_info}")

        if info_type in ("full", "memory"):
            mem = self._get_memory(system)
            results.append(f"## 🧠 Memória\n{mem}")

        if info_type in ("full", "disk"):
            disk = self._get_disk(system)
            results.append(f"## 💾 Disco\n{disk}")

        if info_type in ("full", "uptime"):
            uptime = self._get_uptime(system)
            results.append(f"## ⏱ Uptime\n{uptime}")

        if info_type in ("full", "processes"):
            procs = self._get_top_processes(system)
            results.append(f"## 📊 Top Processos\n{procs}")

        return "\n\n".join(results)

    @staticmethod
    def _get_cpu_info(system: str) -> str:
        try:
            if system == "Windows":
                r = subprocess.run("wmic cpu get name", shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
            elif system == "Darwin":
                r = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
            else:
                r = subprocess.run("cat /proc/cpuinfo | grep 'model name' | head -1", shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
        except Exception as e:
            return f"Error: {e}"

    @staticmethod
    def _get_memory(system: str) -> str:
        try:
            if system == "Windows":
                r = subprocess.run('wmic OS get TotalVisibleMemorySize,FreePhysicalMemory /Value', shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
            elif system == "Darwin":
                r = subprocess.run(["vm_stat"], capture_output=True, text=True, timeout=5)
                r2 = subprocess.run(["sysctl", "-n", "hw.memsize"], capture_output=True, text=True, timeout=5)
                total_gb = round(int(r2.stdout.strip()) / (1024**3), 1)
                return f"Total: {total_gb}GB\n{r.stdout.strip()[:300]}"
            else:
                r = subprocess.run("free -h", shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
        except Exception as e:
            return f"Error: {e}"

    @staticmethod
    def _get_disk(system: str) -> str:
        try:
            if system == "Windows":
                r = subprocess.run("wmic logicaldisk get size,freespace,caption", shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
            else:
                r = subprocess.run("df -h /", shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
        except Exception as e:
            return f"Error: {e}"

    @staticmethod
    def _get_uptime(system: str) -> str:
        try:
            if system == "Windows":
                r = subprocess.run("wmic os get lastbootuptime", shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
            elif system == "Darwin":
                r = subprocess.run(["uptime"], capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
            else:
                r = subprocess.run("uptime -p", shell=True, capture_output=True, text=True, timeout=5)
                return r.stdout.strip() or "N/A"
        except Exception as e:
            return f"Error: {e}"

    @staticmethod
    def _get_top_processes(system: str) -> str:
        try:
            if system == "Windows":
                r = subprocess.run("powershell -Command \"Get-Process | Sort-Object CPU -Descending | Select-Object -First 5 Name, CPU, WorkingSet | Format-Table -AutoSize\"", shell=True, capture_output=True, text=True, timeout=5)
            elif system == "Darwin":
                r = subprocess.run("ps aux --sort=-%cpu 2>/dev/null || ps aux -r", shell=True, capture_output=True, text=True, timeout=5)
            else:
                r = subprocess.run("ps aux --sort=-%cpu | head -6", shell=True, capture_output=True, text=True, timeout=5)
            return r.stdout.strip()[:500] or "N/A"
        except Exception as e:
            return f"Error: {e}"


class ProcessListTool(Tool):
    """List running processes."""

    @property
    def name(self) -> str:
        return "process_list"

    @property
    def description(self) -> str:
        return "List running processes on the current system."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of processes to show",
                    "default": 15,
                },
                "sort_by": {
                    "type": "string",
                    "enum": ["cpu", "memory", "name"],
                    "description": "Sort order for the process list",
                    "default": "cpu",
                },
            },
        }

    async def execute(self, limit: int = 15, sort_by: str = "cpu") -> str:
        system = platform.system()
        limit = max(1, min(limit, 50))
        try:
            if system == "Windows":
                if sort_by == "memory":
                    cmd = (
                        "powershell -Command \"Get-Process | Sort-Object WorkingSet -Descending | "
                        f"Select-Object -First {limit} Name, Id, CPU, WorkingSet | Format-Table -AutoSize\""
                    )
                elif sort_by == "name":
                    cmd = (
                        "powershell -Command \"Get-Process | Sort-Object ProcessName | "
                        f"Select-Object -First {limit} Name, Id, CPU, WorkingSet | Format-Table -AutoSize\""
                    )
                else:
                    cmd = (
                        "powershell -Command \"Get-Process | Sort-Object CPU -Descending | "
                        f"Select-Object -First {limit} Name, Id, CPU, WorkingSet | Format-Table -AutoSize\""
                    )
            elif system == "Darwin":
                if sort_by == "memory":
                    cmd = f"ps aux -m | head -n {limit + 1}"
                elif sort_by == "name":
                    cmd = f"ps aux | sort -k11 | head -n {limit + 1}"
                else:
                    cmd = f"ps aux -r | head -n {limit + 1}"
            else:
                if sort_by == "memory":
                    cmd = f"ps aux --sort=-%mem | head -n {limit + 1}"
                elif sort_by == "name":
                    cmd = f"ps aux --sort=comm | head -n {limit + 1}"
                else:
                    cmd = f"ps aux --sort=-%cpu | head -n {limit + 1}"

            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=10)
            output = (r.stdout or r.stderr or "").strip()
            return output[:1500] or "Nenhum processo encontrado."
        except Exception as e:
            return f"Error listing processes: {e}"


class KillProcessTool(Tool):
    """Terminate a running process."""

    @property
    def name(self) -> str:
        return "kill_process"

    @property
    def description(self) -> str:
        return "Terminate a process by PID or name."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "pid": {
                    "type": "integer",
                    "description": "PID of the process to terminate",
                },
                "name": {
                    "type": "string",
                    "description": "Process name to terminate when PID is not known",
                },
                "force": {
                    "type": "boolean",
                    "description": "Force termination",
                    "default": False,
                },
            },
        }

    async def execute(self, pid: int | None = None, name: str | None = None, force: bool = False) -> str:
        if pid is None and not name:
            return "Informe `pid` ou `name` para encerrar um processo."

        system = platform.system()
        try:
            if system == "Windows":
                if pid is not None:
                    cmd = f"taskkill /PID {pid}" + (" /F" if force else "")
                else:
                    cmd = f'taskkill /IM "{name}"' + (" /F" if force else "")
            else:
                signal = "-9" if force else "-15"
                if pid is not None:
                    cmd = f"kill {signal} {pid}"
                else:
                    cmd = f'pkill {"-9 " if force else ""}-f "{name}"'

            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=10)
            output = (r.stdout or r.stderr or "").strip()
            if r.returncode != 0:
                return output or f"Falha ao encerrar processo ({r.returncode})."
            target = f"PID {pid}" if pid is not None else f"nome '{name}'"
            return output or f"✅ Processo encerrado: {target}"
        except Exception as e:
            return f"Error killing process: {e}"


class EnvironmentTool(Tool):
    """Inspect or set environment variables."""

    @property
    def name(self) -> str:
        return "environment"

    @property
    def description(self) -> str:
        return "Read, list or set environment variables for the current agent process."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "get", "set"],
                    "description": "Environment action to perform",
                    "default": "list",
                },
                "name": {
                    "type": "string",
                    "description": "Environment variable name",
                },
                "value": {
                    "type": "string",
                    "description": "Value to set when action is 'set'",
                },
            },
        }

    async def execute(self, action: str = "list", name: str | None = None, value: str | None = None) -> str:
        try:
            if action == "list":
                items = sorted(os.environ.items())[:80]
                return "\n".join(f"{key}={val}" for key, val in items) or "Nenhuma variavel encontrada."

            if action == "get":
                if not name:
                    return "Informe `name` para ler uma variavel."
                return f"{name}={os.environ.get(name, '')}"

            if action == "set":
                if not name:
                    return "Informe `name` para definir uma variavel."
                os.environ[name] = value or ""
                return f"✅ Variavel definida: {name}={os.environ[name]}"

            return f"Acao desconhecida: {action}"
        except Exception as e:
            return f"Error handling environment variables: {e}"


class PortInspectTool(Tool):
    """Inspect listening ports and connections."""

    @property
    def name(self) -> str:
        return "port_inspect"

    @property
    def description(self) -> str:
        return "Inspect open or listening network ports on the current machine."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "port": {
                    "type": "integer",
                    "description": "Specific port to inspect",
                },
                "listening_only": {
                    "type": "boolean",
                    "description": "Show only listening sockets",
                    "default": True,
                },
            },
        }

    async def execute(self, port: int | None = None, listening_only: bool = True) -> str:
        system = platform.system()
        try:
            if system == "Windows":
                cmd = "netstat -ano"
                if port is not None:
                    cmd += f' | findstr ":{port} "'
            else:
                if shutil.which("lsof"):
                    cmd = "lsof -nP -i"
                    if listening_only:
                        cmd += "TCP -sTCP:LISTEN"
                    if port is not None:
                        cmd += f" | grep ':{port} '"
                else:
                    cmd = "netstat -an"
                    if port is not None:
                        cmd += f" | grep '\\.{port} '"

            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=10)
            output = (r.stdout or r.stderr or "").strip()
            if not output:
                if port is not None:
                    return f"Nenhuma informacao encontrada para a porta {port}."
                return "Nenhuma porta correspondente encontrada."
            return output[:2000]
        except Exception as e:
            return f"Error inspecting ports: {e}"


class ServiceListTool(Tool):
    """List system services."""

    @property
    def name(self) -> str:
        return "service_list"

    @property
    def description(self) -> str:
        return "List system services or daemons on the current machine."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to return",
                    "default": 80,
                },
            },
        }

    async def execute(self, limit: int = 80) -> str:
        system = platform.system()
        limit = max(1, min(limit, 200))
        try:
            if system == "Windows":
                cmd = f"sc query type= service state= all | more +0"
            elif system == "Darwin":
                cmd = "launchctl list"
            else:
                cmd = "systemctl list-units --type=service --all --no-pager"

            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=15)
            output = (r.stdout or r.stderr or "").splitlines()
            if not output:
                return "Nenhum servico encontrado."
            visible = output[:limit]
            if len(output) > limit:
                visible.append(f"... {len(output) - limit} more lines omitted")
            return "\n".join(visible)
        except Exception as e:
            return f"Error listing services: {e}"


class SystemLogsTool(Tool):
    """Read recent system logs."""

    @property
    def name(self) -> str:
        return "system_logs"

    @property
    def description(self) -> str:
        return "Read recent system logs from Windows, macOS or Linux."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of log lines",
                    "default": 80,
                },
                "filter_text": {
                    "type": "string",
                    "description": "Optional plain-text filter applied by the shell command",
                },
            },
        }

    async def execute(self, limit: int = 80, filter_text: str | None = None) -> str:
        system = platform.system()
        limit = max(1, min(limit, 200))
        try:
            if system == "Windows":
                cmd = f'powershell -Command "Get-EventLog -LogName System -Newest {limit}"'
            elif system == "Darwin":
                cmd = f"log show --style compact --last 30m | tail -n {limit}"
            else:
                cmd = f"journalctl -n {limit} --no-pager"

            if filter_text:
                if system == "Windows":
                    cmd += f' | findstr /I "{filter_text}"'
                else:
                    cmd += f' | grep -i "{filter_text}"'

            r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=20)
            output = (r.stdout or r.stderr or "").strip()
            return output[:4000] or "Nenhum log encontrado."
        except Exception as e:
            return f"Error reading system logs: {e}"
