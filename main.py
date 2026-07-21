#!/usr/bin/env python3
"""USBRAIN — main entry point. Starts BGE server + Web UI + LLM."""
from __future__ import annotations

import argparse
import atexit
import asyncio
import errno
import json
import os
import re
import socket
import signal
import subprocess
import sys
import threading
import time
import traceback
import urllib.request
from pathlib import Path


MODEL_PROFILES = [
    {
        "id": "phi-3.5-mini",
        "display": "Phi-3.5 Mini",
        "keywords": ["phi-3-5-mini", "phi-3-5-mini-instruct", "phi3-5-mini", "phi-3-mini", "phi3-mini"],
        "min_ram_gb": 8,
        "priority": 100,
    },
    {
        "id": "llama-3.2-3b",
        "display": "Llama 3.2 3B",
        "keywords": ["llama-3.2-3b", "llama3.2-3b"],
        "min_ram_gb": 8,
        "priority": 95,
    },
    {
        "id": "qwen2.5-3b",
        "display": "Qwen2.5 3B",
        "keywords": ["qwen2.5-3b", "qwen-2.5-3b", "qwen2-5-3b"],
        "min_ram_gb": 8,
        "priority": 92,
    },
    {
        "id": "gemma-2-2b",
        "display": "Gemma 2 2B",
        "keywords": ["gemma-2-2b", "gemma-2b"],
        "min_ram_gb": 8,
        "priority": 85,
    },
    {
        "id": "qwen2.5-7b",
        "display": "Qwen2.5 7B",
        "keywords": ["qwen2.5-7b", "qwen-2.5-7b", "qwen2-5-7b"],
        "min_ram_gb": 16,
        "priority": 80,
    },
    {
        "id": "phi-4-14b",
        "display": "Phi-4 14B",
        "keywords": ["phi-4-14b", "phi4-14b"],
        "min_ram_gb": 16,
        "priority": 75,
    },
    {
        "id": "qwen3-8b",
        "display": "Qwen3 8B",
        "keywords": ["qwen3-8b", "qwen_qwen3-8b"],
        "min_ram_gb": 8,
        "priority": 60,
    },
]

DEFAULT_LLM_PORT = 38421
DEFAULT_BGE_PORT = 38422
MIN_LLM_CTX_SIZE = 2048
DEFAULT_LLM_CTX_SIZE = 4096
MAX_LLM_CTX_SIZE = 8192
LLM_KV_CACHE_MB_PER_TOKEN_ESTIMATE = 0.375
LLM_RAM_HEADROOM_GB = 6.0

DEBUG_SESSION_ID = "windows-llamafile-start"
DEBUG_LOG_PATH = Path(__file__).parent / "llamafile.log"
DEBUG_ENV_PATH = Path(__file__).parent / ".dbg" / f"{DEBUG_SESSION_ID}.env"
CHILD_PROCESS_REGISTRY_PATH = Path(__file__).parent / ".usbrain_children.json"
LLM_SERVER_LOG_PATH = Path(__file__).parent / "llamafile_server.log"

ACTIVE_CHILD_PROCESSES: list[tuple[str, subprocess.Popen]] = []
_CLEANUP_DONE = False
_CLEANUP_HOOKS_INSTALLED = False
_WINDOWS_CTRL_HANDLER = None
_WINDOWS_JOB_HANDLE = None
_LLM_KEEPALIVE_STOP = threading.Event()

if (Path(__file__).parent / ".dbg" / "usb-python-chat-fail.env").exists():
    DEBUG_SESSION_ID = "usb-python-chat-fail"
    DEBUG_ENV_PATH = Path(__file__).parent / ".dbg" / "usb-python-chat-fail.env"


def _truncate_debug_text(value: str | bytes | None, limit: int = 4000) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="replace")
    if len(value) <= limit:
        return value
    return value[:limit] + "...<truncated>"


def _load_debug_server_config() -> tuple[str | None, str]:
    url = None
    session_id = DEBUG_SESSION_ID
    try:
        if DEBUG_ENV_PATH.exists():
            for line in DEBUG_ENV_PATH.read_text(encoding="utf-8", errors="replace").splitlines():
                if line.startswith("DEBUG_SERVER_URL="):
                    url = line.split("=", 1)[1].strip() or None
                elif line.startswith("DEBUG_SESSION_ID="):
                    session_id = line.split("=", 1)[1].strip() or session_id
    except Exception:
        pass
    return url, session_id


def _debug_event(hypothesis_id: str, location: str, message: str, **data) -> None:
    payload = {
        "sessionId": DEBUG_SESSION_ID,
        "runId": os.environ.get("USBRAIN_DEBUG_RUN_ID", "pre-fix"),
        "hypothesisId": hypothesis_id,
        "location": location,
        "msg": f"[DEBUG] {message}",
        "data": data,
        "ts": int(time.time() * 1000),
    }
    try:
        with DEBUG_LOG_PATH.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(payload, ensure_ascii=True) + "\n")
    except Exception:
        pass

    url, session_id = _load_debug_server_config()
    payload["sessionId"] = session_id
    if not url:
        return
    try:
        req = urllib.request.Request(
            url,
            data=json.dumps(payload, ensure_ascii=True).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        urllib.request.urlopen(req, timeout=1).read()
    except Exception:
        pass


def _write_child_process_registry() -> None:
    try:
        payload = [
            {"name": name, "pid": proc.pid}
            for name, proc in ACTIVE_CHILD_PROCESSES
            if proc.pid
        ]
        CHILD_PROCESS_REGISTRY_PATH.write_text(
            json.dumps(payload, ensure_ascii=True, indent=2),
            encoding="utf-8",
        )
    except Exception:
        pass


def _clear_child_process_registry() -> None:
    try:
        if CHILD_PROCESS_REGISTRY_PATH.exists():
            CHILD_PROCESS_REGISTRY_PATH.unlink()
    except Exception:
        pass


def register_child_process(name: str, proc: subprocess.Popen | None) -> None:
    if proc is None:
        return
    ACTIVE_CHILD_PROCESSES.append((name, proc))
    _assign_process_to_windows_job(name, proc)
    _tune_windows_llm_process(name, proc)
    _write_child_process_registry()
    _debug_event(
        "E",
        "main.py:register_child_process",
        "Registered child process for cleanup",
        name=name,
        pid=proc.pid,
    )


def watch_child_process_exit(name: str, proc: subprocess.Popen | None, hypothesis_id: str, location: str) -> None:
    if proc is None:
        return

    def _watch() -> None:
        try:
            exit_code = proc.wait()
            _debug_event(
                hypothesis_id,
                location,
                f"{name} process exited after startup",
                pid=proc.pid,
                exit_code=exit_code,
            )
        except Exception as exc:
            _debug_event(
                hypothesis_id,
                location,
                f"{name} process watcher failed",
                pid=proc.pid,
                error=repr(exc),
                traceback=traceback.format_exc(),
            )

    threading.Thread(target=_watch, name=f"watch-{name.lower()}-{proc.pid}", daemon=True).start()


def _tune_windows_llm_process(name: str, proc: subprocess.Popen | None) -> None:
    if sys.platform != "win32" or proc is None:
        return
    if (name or "").strip().upper() != "LLM":
        return
    try:
        import ctypes
        from ctypes import wintypes

        kernel32 = ctypes.windll.kernel32
        handle = wintypes.HANDLE(proc._handle)

        class PROCESS_POWER_THROTTLING_STATE(ctypes.Structure):
            _fields_ = [
                ("Version", wintypes.DWORD),
                ("ControlMask", wintypes.DWORD),
                ("StateMask", wintypes.DWORD),
            ]

        ProcessPowerThrottling = 4
        PROCESS_POWER_THROTTLING_CURRENT_VERSION = 1
        PROCESS_POWER_THROTTLING_EXECUTION_SPEED = 0x1
        state = PROCESS_POWER_THROTTLING_STATE(
            Version=PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask=PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask=0,
        )
        set_info = getattr(kernel32, "SetProcessInformation", None)
        throttling_ok = None
        throttling_error = None
        if set_info is not None:
            ok = set_info(handle, ProcessPowerThrottling, ctypes.byref(state), ctypes.sizeof(state))
            throttling_ok = bool(ok)
            if not throttling_ok:
                throttling_error = int(kernel32.GetLastError())

        _debug_event(
            "B",
            "main.py:_tune_windows_llm_process",
            "Applied Windows LLM process tuning",
            pid=proc.pid,
            priority_class="unchanged",
            power_throttling_disabled=throttling_ok,
            power_throttling_error=throttling_error,
        )
    except Exception as exc:
        _debug_event(
            "B",
            "main.py:_tune_windows_llm_process",
            "Windows LLM process tuning failed",
            pid=getattr(proc, "pid", None),
            error=repr(exc),
            traceback=traceback.format_exc(),
        )


def _ensure_windows_kill_job():
    global _WINDOWS_JOB_HANDLE
    if sys.platform != "win32":
        return None
    if _WINDOWS_JOB_HANDLE is not None:
        return _WINDOWS_JOB_HANDLE

    import ctypes
    from ctypes import wintypes

    class JOBOBJECT_BASIC_LIMIT_INFORMATION(ctypes.Structure):
        _fields_ = [
            ("PerProcessUserTimeLimit", ctypes.c_longlong),
            ("PerJobUserTimeLimit", ctypes.c_longlong),
            ("LimitFlags", wintypes.DWORD),
            ("MinimumWorkingSetSize", ctypes.c_size_t),
            ("MaximumWorkingSetSize", ctypes.c_size_t),
            ("ActiveProcessLimit", wintypes.DWORD),
            ("Affinity", ctypes.c_size_t),
            ("PriorityClass", wintypes.DWORD),
            ("SchedulingClass", wintypes.DWORD),
        ]

    class IO_COUNTERS(ctypes.Structure):
        _fields_ = [
            ("ReadOperationCount", ctypes.c_ulonglong),
            ("WriteOperationCount", ctypes.c_ulonglong),
            ("OtherOperationCount", ctypes.c_ulonglong),
            ("ReadTransferCount", ctypes.c_ulonglong),
            ("WriteTransferCount", ctypes.c_ulonglong),
            ("OtherTransferCount", ctypes.c_ulonglong),
        ]

    class JOBOBJECT_EXTENDED_LIMIT_INFORMATION(ctypes.Structure):
        _fields_ = [
            ("BasicLimitInformation", JOBOBJECT_BASIC_LIMIT_INFORMATION),
            ("IoInfo", IO_COUNTERS),
            ("ProcessMemoryLimit", ctypes.c_size_t),
            ("JobMemoryLimit", ctypes.c_size_t),
            ("PeakProcessMemoryUsed", ctypes.c_size_t),
            ("PeakJobMemoryUsed", ctypes.c_size_t),
        ]

    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE = 0x00002000
    JobObjectExtendedLimitInformation = 9

    kernel32 = ctypes.windll.kernel32
    job_handle = kernel32.CreateJobObjectW(None, None)
    if not job_handle:
        raise ctypes.WinError()

    info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION()
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
    ok = kernel32.SetInformationJobObject(
        job_handle,
        JobObjectExtendedLimitInformation,
        ctypes.byref(info),
        ctypes.sizeof(info),
    )
    if not ok:
        kernel32.CloseHandle(job_handle)
        raise ctypes.WinError()

    _WINDOWS_JOB_HANDLE = job_handle
    return job_handle


def _assign_process_to_windows_job(name: str, proc: subprocess.Popen | None) -> None:
    if sys.platform != "win32" or proc is None:
        return
    try:
        import ctypes
        from ctypes import wintypes

        job_handle = _ensure_windows_kill_job()
        if not job_handle:
            return
        ok = ctypes.windll.kernel32.AssignProcessToJobObject(job_handle, wintypes.HANDLE(proc._handle))
        if not ok:
            raise ctypes.WinError()
        _debug_event(
            "E",
            "main.py:_assign_process_to_windows_job",
            "Assigned child process to Windows kill-on-close job",
            name=name,
            pid=proc.pid,
        )
    except Exception as exc:
        _debug_event(
            "E",
            "main.py:_assign_process_to_windows_job",
            "Failed to assign child process to Windows job",
            name=name,
            pid=(proc.pid if proc else None),
            error=repr(exc),
            traceback=traceback.format_exc(),
        )


def _close_windows_job_handle(reason: str) -> None:
    global _WINDOWS_JOB_HANDLE
    if sys.platform != "win32" or _WINDOWS_JOB_HANDLE is None:
        return
    try:
        import ctypes

        ctypes.windll.kernel32.CloseHandle(_WINDOWS_JOB_HANDLE)
        _debug_event(
            "E",
            "main.py:_close_windows_job_handle",
            "Closed Windows kill-on-close job handle",
            reason=reason,
        )
    except Exception:
        _debug_event(
            "E",
            "main.py:_close_windows_job_handle",
            "Failed to close Windows job handle",
            reason=reason,
            traceback=traceback.format_exc(),
        )
    finally:
        _WINDOWS_JOB_HANDLE = None


def _probe_http_ready_once(url: str, timeout_seconds: float = 3.0) -> tuple[bool, int | None, str | None]:
    """Probe a local HTTP endpoint once and return a lightweight status tuple."""
    try:
        req = urllib.request.Request(url, method="GET")
        with urllib.request.urlopen(req, timeout=timeout_seconds) as response:
            return True, response.status, None
    except urllib.error.HTTPError as exc:
        return False, exc.code, str(exc)
    except Exception as exc:
        return False, None, repr(exc)


def start_llm_keepalive_monitor(proc: subprocess.Popen, port: int, interval_seconds: float = 5.0) -> None:
    """Continuously probe the local LLM HTTP endpoint and log liveness transitions."""
    _LLM_KEEPALIVE_STOP.clear()
    url = f"http://127.0.0.1:{port}/v1/models"

    def _watch() -> None:
        last_state: tuple[bool | None, int | None, str | None] = (None, None, None)
        while not _LLM_KEEPALIVE_STOP.is_set():
            proc_returncode = proc.poll()
            alive, status_code, error = _probe_http_ready_once(url, timeout_seconds=3.0)
            current_state = (alive, status_code, error)
            if current_state != last_state:
                _debug_event(
                    "B",
                    "main.py:start_llm_keepalive_monitor",
                    "LLM keepalive state changed",
                    pid=proc.pid,
                    port=port,
                    url=url,
                    alive=alive,
                    status_code=status_code,
                    error=error,
                    process_returncode=proc_returncode,
                )
                last_state = current_state
            if proc_returncode is not None:
                break
            _LLM_KEEPALIVE_STOP.wait(interval_seconds)

    threading.Thread(target=_watch, name=f"llm-keepalive-{proc.pid}", daemon=True).start()


def _get_process_command_line(pid: int) -> str:
    try:
        if sys.platform == "win32":
            result = subprocess.run(
                ["wmic", "process", "where", f"processid={pid}", "get", "CommandLine", "/value"],
                capture_output=True,
                text=True,
                timeout=10,
            )
            for line in result.stdout.splitlines():
                if line.startswith("CommandLine="):
                    return line.split("=", 1)[1].strip()
        else:
            result = subprocess.run(
                ["ps", "-p", str(pid), "-o", "command="],
                capture_output=True,
                text=True,
                timeout=10,
            )
            return result.stdout.strip()
    except Exception:
        return ""
    return ""


def _is_expected_usbrain_child(pid: int, project_dir: Path) -> bool:
    cmdline = _get_process_command_line(pid)
    if not cmdline:
        return False
    normalized = cmdline.lower()
    project_marker = str(project_dir).lower()
    return project_marker in normalized and (
        "llamafile" in normalized or
        "rag\\server.py" in normalized or
        "rag/server.py" in normalized
    )


def _terminate_child_process(name: str, proc: subprocess.Popen | None, reason: str) -> None:
    if proc is None:
        return
    pid = proc.pid
    if proc.poll() is not None:
        _debug_event(
            "E",
            "main.py:_terminate_child_process",
            "Child process already exited before cleanup",
            name=name,
            pid=pid,
            returncode=proc.returncode,
            reason=reason,
        )
    else:
        try:
            if sys.platform == "win32":
                result = subprocess.run(
                    ["taskkill", "/PID", str(pid), "/T", "/F"],
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                _debug_event(
                    "E",
                    "main.py:_terminate_child_process",
                    "Issued Windows taskkill for child process",
                    name=name,
                    pid=pid,
                    reason=reason,
                    returncode=result.returncode,
                    stdout=_truncate_debug_text(result.stdout),
                    stderr=_truncate_debug_text(result.stderr),
                )
            else:
                proc.terminate()
                try:
                    proc.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)
                _debug_event(
                    "E",
                    "main.py:_terminate_child_process",
                    "Terminated child process",
                    name=name,
                    pid=pid,
                    reason=reason,
                    returncode=proc.returncode,
                )
        except Exception as exc:
            _debug_event(
                "E",
                "main.py:_terminate_child_process",
                "Failed while terminating child process",
                name=name,
                pid=pid,
                reason=reason,
                error=repr(exc),
                traceback=traceback.format_exc(),
            )
    for stream_name in ("stdout", "stderr"):
        try:
            stream = getattr(proc, stream_name)
            if stream:
                stream.close()
        except Exception:
            pass
    try:
        log_handle = getattr(proc, "_usbrain_log_handle", None)
        if log_handle:
            log_handle.close()
    except Exception:
        pass


def cleanup_child_processes(reason: str) -> None:
    global _CLEANUP_DONE
    if _CLEANUP_DONE:
        return
    _CLEANUP_DONE = True
    _LLM_KEEPALIVE_STOP.set()
    for name, proc in reversed(ACTIVE_CHILD_PROCESSES):
        _terminate_child_process(name, proc, reason)
    ACTIVE_CHILD_PROCESSES.clear()
    _close_windows_job_handle(reason)
    _clear_child_process_registry()
    _debug_event(
        "E",
        "main.py:cleanup_child_processes",
        "Cleanup routine finished",
        reason=reason,
    )


def cleanup_stale_child_processes(project_dir: Path) -> None:
    if not CHILD_PROCESS_REGISTRY_PATH.exists():
        return
    try:
        entries = json.loads(CHILD_PROCESS_REGISTRY_PATH.read_text(encoding="utf-8"))
    except Exception:
        _clear_child_process_registry()
        return

    for entry in entries:
        pid = int(entry.get("pid", 0) or 0)
        name = str(entry.get("name", "child"))
        if pid <= 0 or pid == os.getpid():
            continue
        if not _is_expected_usbrain_child(pid, project_dir):
            _debug_event(
                "E",
                "main.py:cleanup_stale_child_processes",
                "Skipping stale cleanup for unverified process",
                name=name,
                pid=pid,
                command_line=_truncate_debug_text(_get_process_command_line(pid)),
            )
            continue
        try:
            result = subprocess.run(
                ["taskkill", "/PID", str(pid), "/T", "/F"] if sys.platform == "win32" else ["kill", "-TERM", str(pid)],
                capture_output=True,
                text=True,
                timeout=15,
            )
            _debug_event(
                "E",
                "main.py:cleanup_stale_child_processes",
                "Attempted cleanup of stale child process",
                name=name,
                pid=pid,
                returncode=result.returncode,
                stdout=_truncate_debug_text(result.stdout),
                stderr=_truncate_debug_text(result.stderr),
            )
        except Exception as exc:
            _debug_event(
                "E",
                "main.py:cleanup_stale_child_processes",
                "Failed to cleanup stale child process",
                name=name,
                pid=pid,
                error=repr(exc),
                traceback=traceback.format_exc(),
            )
    _clear_child_process_registry()


def _handle_exit_signal(signum, _frame) -> None:
    _debug_event(
        "E",
        "main.py:_handle_exit_signal",
        "Exit signal received",
        signal_number=signum,
    )
    cleanup_child_processes(f"signal:{signum}")
    raise SystemExit(0)


def _install_windows_console_cleanup_handler() -> None:
    global _WINDOWS_CTRL_HANDLER
    if sys.platform != "win32" or _WINDOWS_CTRL_HANDLER is not None:
        return
    try:
        import ctypes

        handler_type = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_uint)

        @handler_type
        def _console_handler(ctrl_type):
            _debug_event(
                "E",
                "main.py:_install_windows_console_cleanup_handler",
                "Windows console event received",
                ctrl_type=int(ctrl_type),
            )
            cleanup_child_processes(f"console:{int(ctrl_type)}")
            return False

        if ctypes.windll.kernel32.SetConsoleCtrlHandler(_console_handler, True):
            _WINDOWS_CTRL_HANDLER = _console_handler
    except Exception:
        _debug_event(
            "E",
            "main.py:_install_windows_console_cleanup_handler",
            "Failed to install Windows console cleanup handler",
            traceback=traceback.format_exc(),
        )


def install_cleanup_hooks() -> None:
    global _CLEANUP_HOOKS_INSTALLED
    if _CLEANUP_HOOKS_INSTALLED:
        return
    _CLEANUP_HOOKS_INSTALLED = True
    atexit.register(lambda: cleanup_child_processes("atexit"))
    for signame in ("SIGINT", "SIGTERM", "SIGBREAK"):
        sig = getattr(signal, signame, None)
        if sig is None:
            continue
        try:
            signal.signal(sig, _handle_exit_signal)
        except Exception:
            pass
    _install_windows_console_cleanup_handler()


def _log_process_state(name: str, proc: subprocess.Popen | None, hypothesis_id: str, location: str) -> None:
    if proc is None:
        _debug_event(hypothesis_id, location, f"{name} process was not created")
        return

    exit_code = proc.poll()
    if exit_code is None:
        _debug_event(
            hypothesis_id,
            location,
            f"{name} process is still running after startup wait",
            pid=proc.pid,
        )
        return

    stdout_text = ""
    stderr_text = ""
    try:
        stdout_data, stderr_data = proc.communicate(timeout=1)
        stdout_text = _truncate_debug_text(stdout_data)
        stderr_text = _truncate_debug_text(stderr_data)
    except Exception as exc:
        stderr_text = f"<unable to read process output: {exc}>"

    _debug_event(
        hypothesis_id,
        location,
        f"{name} process exited during startup",
        pid=proc.pid,
        exit_code=exit_code,
        stdout=stdout_text,
        stderr=stderr_text,
    )


def wait_for_http_ready(
    name: str,
    url: str,
    timeout: int,
    hypothesis_id: str,
    location: str,
    proc: subprocess.Popen | None = None,
) -> bool:
    start = time.time()
    last_report = -10.0
    while time.time() - start < timeout:
        if proc and proc.poll() is not None:
            _debug_event(
                hypothesis_id,
                location,
                f"{name} process exited before HTTP readiness",
                returncode=proc.returncode,
                url=url,
            )
            return False
        try:
            with urllib.request.urlopen(url, timeout=2) as response:
                if response.status < 500:
                    _debug_event(
                        hypothesis_id,
                        location,
                        f"{name} HTTP endpoint is ready",
                        url=url,
                        status=response.status,
                        wait_seconds=round(time.time() - start, 1),
                    )
                    return True
        except Exception:
            pass
        elapsed = time.time() - start
        if elapsed - last_report >= 10:
            print(f"Aguardando {name} ficar pronto... {int(elapsed)}s")
            last_report = elapsed
        time.sleep(2)
    _debug_event(
        hypothesis_id,
        location,
        f"Timeout waiting for {name} HTTP readiness",
        url=url,
        timeout=timeout,
    )
    return False


def wait_for_tcp_ready(
    name: str,
    host: str,
    port: int,
    timeout: int,
    hypothesis_id: str,
    location: str,
    proc: subprocess.Popen | None = None,
) -> bool:
    start = time.time()
    last_report = -10.0
    while time.time() - start < timeout:
        if proc and proc.poll() is not None:
            _debug_event(
                hypothesis_id,
                location,
                f"{name} process exited before TCP readiness",
                returncode=proc.returncode,
                host=host,
                port=port,
            )
            return False
        try:
            with socket.create_connection((host, port), timeout=2):
                _debug_event(
                    hypothesis_id,
                    location,
                    f"{name} TCP port is ready",
                    host=host,
                    port=port,
                    wait_seconds=round(time.time() - start, 1),
                )
                return True
        except OSError:
            pass
        elapsed = time.time() - start
        if elapsed - last_report >= 10:
            print(f"Aguardando {name} ficar pronto... {int(elapsed)}s")
            last_report = elapsed
        time.sleep(2)
    _debug_event(
        hypothesis_id,
        location,
        f"Timeout waiting for {name} TCP readiness",
        host=host,
        port=port,
        timeout=timeout,
    )
    return False


def is_port_available(host: str, port: int) -> bool:
    """Return True when a TCP port can be bound locally."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            sock.bind((host, port))
        except OSError:
            return False
    return True


def choose_available_port(
    name: str,
    host: str,
    preferred_port: int,
    hypothesis_id: str,
    location: str,
    reserved_ports: set[int] | None = None,
    max_attempts: int = 100,
) -> int:
    """Choose a free local port, preferring the provided one."""
    reserved_ports = reserved_ports or set()
    if preferred_port not in reserved_ports and is_port_available(host, preferred_port):
        _debug_event(
            hypothesis_id,
            location,
            f"{name} port available",
            host=host,
            requested_port=preferred_port,
            selected_port=preferred_port,
        )
        return preferred_port

    for offset in range(1, max_attempts):
        alt_port = preferred_port + offset
        if alt_port in reserved_ports:
            continue
        if is_port_available(host, alt_port):
            print(f"Porta {preferred_port} ocupada para {name}, usando {alt_port}.")
            _debug_event(
                hypothesis_id,
                location,
                f"{name} port fallback selected",
                host=host,
                requested_port=preferred_port,
                selected_port=alt_port,
            )
            return alt_port

    raise RuntimeError(f"Nenhuma porta livre encontrada para {name} entre {preferred_port} e {preferred_port + max_attempts - 1}")


def normalize_model_name(name: str) -> str:
    """Normalize model filenames for fuzzy matching."""
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def identify_model_profile(name: str) -> dict | None:
    """Match a model filename or label to a known profile."""
    normalized = normalize_model_name(name)
    for profile in MODEL_PROFILES:
        if profile["id"] == normalized:
            return profile
        if any(keyword in normalized for keyword in profile["keywords"]):
            return profile
    return None


def choose_model_file(models_dir: Path, ram_gb: float, preferred_model: str | None = None) -> Path | None:
    """Choose the best local GGUF model that fits the available RAM."""
    ggufs = sorted(
        model_file
        for model_file in models_dir.glob("*.gguf")
        if model_file.is_file() and not model_file.name.startswith("._")
    )
    if not ggufs:
        return None

    preferred_normalized = normalize_model_name(preferred_model) if preferred_model else None
    best_choice: tuple[int, int, str, Path] | None = None

    for model_file in ggufs:
        profile = identify_model_profile(model_file.stem)
        if profile and ram_gb < profile["min_ram_gb"]:
            continue

        size_bytes = model_file.stat().st_size
        priority = profile["priority"] if profile else 10
        if preferred_normalized:
            if profile and profile["id"] == preferred_normalized:
                priority += 1000
            elif preferred_normalized in normalize_model_name(model_file.stem):
                priority += 900

        candidate = (priority, -size_bytes, normalize_model_name(model_file.name), model_file)
        if best_choice is None or candidate > best_choice:
            best_choice = candidate

    return best_choice[3] if best_choice else None


def list_compatible_model_options(
    models_dir: Path,
    ram_gb: float,
    preferred_model: str | None = None,
) -> list[dict[str, str | int | bool]]:
    """List local GGUF models that fit the available RAM, ordered by boot preference."""
    ggufs = sorted(
        model_file
        for model_file in models_dir.glob("*.gguf")
        if model_file.is_file() and not model_file.name.startswith("._")
    )
    if not ggufs:
        return []

    preferred_normalized = normalize_model_name(preferred_model) if preferred_model else None
    ranked_candidates: list[tuple[int, int, str, Path, dict | None]] = []
    for model_file in ggufs:
        profile = identify_model_profile(model_file.stem)
        min_ram_gb = int(profile["min_ram_gb"]) if profile else 0
        if profile and ram_gb < min_ram_gb:
            continue

        size_bytes = model_file.stat().st_size
        priority = int(profile["priority"]) if profile else 10
        normalized_name = normalize_model_name(model_file.stem)
        if preferred_normalized:
            if profile and profile["id"] == preferred_normalized:
                priority += 1000
            elif preferred_normalized in normalized_name:
                priority += 900

        ranked_candidates.append((priority, -size_bytes, normalized_name, model_file, profile))

    ranked_candidates.sort(reverse=True)
    best_path = ranked_candidates[0][3] if ranked_candidates else None

    options: list[dict[str, str | int | bool]] = []
    for index, (_, _, _, model_file, profile) in enumerate(ranked_candidates, start=1):
        min_ram_gb = int(profile["min_ram_gb"]) if profile else 0
        options.append({
            "index": index,
            "label": get_model_label(model_file.stem),
            "file_name": model_file.name,
            "path": str(model_file),
            "recommended": bool(best_path and model_file == best_path),
            "min_ram_gb": min_ram_gb,
        })
    return options


def resolve_model_from_installed_manifest(project_dir: Path) -> tuple[Path | None, str | None]:
    """Resolve the default model declared by the mounted USBRAIN manifest."""
    manifest_path = project_dir / "manifests" / "installed-models.json"
    if not manifest_path.exists():
        return None, None

    try:
        payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception:
        return None, None

    models = payload.get("models")
    default_model_id = payload.get("default_model_id")
    if not isinstance(models, list) or not default_model_id:
        return None, None

    for model in models:
        if not isinstance(model, dict):
            continue
        installed_model_id = model.get("installed_model_id")
        catalog_model_id = model.get("catalog_model_id")
        relative_path = model.get("relative_path")
        if default_model_id not in {installed_model_id, catalog_model_id}:
            continue
        if not isinstance(relative_path, str) or not relative_path.strip():
            continue

        model_path = (project_dir / relative_path).resolve()
        try:
            model_path.relative_to(project_dir.resolve())
        except ValueError:
            continue
        if model_path.exists() and model_path.is_file():
            return model_path, str(installed_model_id or catalog_model_id or default_model_id)

    return None, str(default_model_id)


def get_model_label(model_name: str) -> str:
    """Return a friendly label for a model identifier or filename."""
    profile = identify_model_profile(model_name)
    return profile["display"] if profile else Path(model_name).stem


def get_preferred_model_for_ram(ram_gb: float) -> str:
    """Return the preferred validation model for the available RAM."""
    if ram_gb >= 16:
        return "phi-4-14b"
    return "phi-3.5-mini"


def _round_down_to_step(value: int, step: int) -> int:
    return max(step, (value // step) * step)


def choose_llm_ctx_size(hw: dict) -> int:
    """Estimate a safe context size from currently available RAM."""
    free_ram_gb = hw.get("free_ram_gb")
    if not isinstance(free_ram_gb, (int, float)) or free_ram_gb <= 0:
        return DEFAULT_LLM_CTX_SIZE

    kv_budget_gb = max(0.75, (free_ram_gb - LLM_RAM_HEADROOM_GB) * 0.25)
    estimated_ctx = int((kv_budget_gb * 1024) / LLM_KV_CACHE_MB_PER_TOKEN_ESTIMATE)
    estimated_ctx = _round_down_to_step(estimated_ctx, 512)
    return max(MIN_LLM_CTX_SIZE, min(MAX_LLM_CTX_SIZE, estimated_ctx))


def detect_hardware() -> dict:
    """Detect system RAM and recommend model."""
    import platform
    mem_bytes = 8 * 1024**3  # default 8GB
    free_bytes = None
    logical_cores = os.cpu_count() or 4
    physical_cores = max(1, logical_cores // 2)
    try:
        if platform.system() == "Darwin":
            result = subprocess.run(
                ["sysctl", "-n", "hw.memsize"],
                capture_output=True, text=True
            )
            if result.stdout:
                mem_bytes = int(result.stdout.strip())
            core_result = subprocess.run(
                ["sysctl", "-n", "hw.physicalcpu", "hw.logicalcpu"],
                capture_output=True, text=True
            )
            if core_result.stdout:
                values = [line.strip() for line in core_result.stdout.splitlines() if line.strip().isdigit()]
                if len(values) >= 2:
                    physical_cores = int(values[0])
                    logical_cores = int(values[1])
        elif platform.system() == "Linux":
            result = subprocess.run(
                ["grep", "-E", "MemTotal|MemAvailable", "/proc/meminfo"],
                capture_output=True, text=True
            )
            if result.stdout:
                for line in result.stdout.splitlines():
                    if line.startswith("MemTotal:"):
                        mem_kb = int(line.split()[1])
                        mem_bytes = mem_kb * 1024
                    elif line.startswith("MemAvailable:"):
                        free_kb = int(line.split()[1])
                        free_bytes = free_kb * 1024
            logical_cores = os.cpu_count() or logical_cores
        elif platform.system() == "Windows":
            ram_mb_env = os.environ.get("USBRAIN_RAM_MB")
            if ram_mb_env and ram_mb_env.isdigit():
                mem_bytes = int(ram_mb_env) * 1024 * 1024
            else:
                result = subprocess.run(
                    ["wmic", "os", "get", "TotalVisibleMemorySize,FreePhysicalMemory", "/value"],
                    capture_output=True, text=True
                )
                if result.stdout:
                    for line in result.stdout.splitlines():
                        if line.startswith("TotalVisibleMemorySize="):
                            mem_kb = int(line.split("=", 1)[1].strip())
                            mem_bytes = mem_kb * 1024
                        elif line.startswith("FreePhysicalMemory="):
                            free_kb = int(line.split("=", 1)[1].strip())
                            free_bytes = free_kb * 1024
            cpu_result = subprocess.run(
                ["wmic", "cpu", "get", "NumberOfCores,NumberOfLogicalProcessors", "/value"],
                capture_output=True,
                text=True,
            )
            if cpu_result.stdout:
                physical_total = 0
                logical_total = 0
                for line in cpu_result.stdout.splitlines():
                    if line.startswith("NumberOfCores="):
                        value = line.split("=", 1)[1].strip()
                        if value.isdigit():
                            physical_total += int(value)
                    elif line.startswith("NumberOfLogicalProcessors="):
                        value = line.split("=", 1)[1].strip()
                        if value.isdigit():
                            logical_total += int(value)
                if physical_total > 0:
                    physical_cores = physical_total
                if logical_total > 0:
                    logical_cores = logical_total
    except Exception:
        pass

    logical_cores = max(1, logical_cores)
    physical_cores = max(1, min(physical_cores, logical_cores))

    ram_gb = mem_bytes / (1024**3)
    model = get_preferred_model_for_ram(ram_gb)
    free_ram_gb = round(free_bytes / (1024**3), 1) if free_bytes else None

    return {
        "ram_gb": round(ram_gb, 1),
        "free_ram_gb": free_ram_gb,
        "recommended_model": model,
        "physical_cores": physical_cores,
        "logical_cores": logical_cores,
    }


def choose_llm_runtime_tuning(hw: dict, model_path: str | Path) -> dict[str, int | bool | float | None]:
    """Choose CPU-oriented llamafile tuning parameters from hardware and model size."""
    model_size_gb = round(Path(model_path).stat().st_size / (1024**3), 2)
    ram_gb = float(hw.get("ram_gb") or 8.0)
    free_ram_gb = hw.get("free_ram_gb")
    physical_cores = int(hw.get("physical_cores") or max(1, (os.cpu_count() or 4) // 2))
    logical_cores = int(hw.get("logical_cores") or os.cpu_count() or physical_cores)
    has_smt_or_hybrid = logical_cores > physical_cores

    if sys.platform == "win32":
        # Saturating all logical threads on Windows increased total latency and froze the OS.
        gen_threads = max(6, min(physical_cores, 12 if has_smt_or_hybrid else physical_cores))
        batch_threads = max(gen_threads, min(logical_cores, gen_threads + (4 if has_smt_or_hybrid else 2)))
        batch_size = 256
    else:
        gen_threads = max(4, min(physical_cores, logical_cores))
        batch_threads = max(gen_threads, min(logical_cores, physical_cores * 2))
        batch_size = 512

    available_for_model_gb = float(free_ram_gb) if isinstance(free_ram_gb, (int, float)) and free_ram_gb > 0 else ram_gb - 2.0
    use_no_mmap = available_for_model_gb >= model_size_gb + 3.0

    return {
        "model_size_gb": model_size_gb,
        "generation_threads": gen_threads,
        "batch_threads": batch_threads,
        "batch_size": batch_size,
        "use_no_mmap": use_no_mmap,
    }


def start_bge_server(db_path: str, port: int = DEFAULT_BGE_PORT):
    """Start the BGE embedding server."""
    project_dir = Path(__file__).parent
    bge_script = project_dir / "rag" / "server.py"
    bge_boot_log = project_dir / "bge_boot.log"
    try:
        if bge_boot_log.exists():
            bge_boot_log.unlink()
    except Exception:
        pass
    env = os.environ.copy()
    env["PYTHONFAULTHANDLER"] = "1"
    env["PYTHONUNBUFFERED"] = "1"
    env["USBRAIN_BGE_BOOT_LOG"] = str(bge_boot_log)

    # #region debug-point C:bge-spawn
    _debug_event(
        "C",
        "main.py:start_bge_server",
        "Spawning BGE server",
        script=str(bge_script),
        script_exists=bge_script.exists(),
        db_path=db_path,
        port=port,
        bge_boot_log=str(bge_boot_log),
    )
    # #endregion
    proc = subprocess.Popen(
        [sys.executable, "-X", "faulthandler", str(bge_script), db_path, str(port)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )
    return proc


def start_llama_server(
    model_path: str,
    port: int = DEFAULT_LLM_PORT,
    ctx_size: int = DEFAULT_LLM_CTX_SIZE,
    runtime_tuning: dict[str, int | bool | float | None] | None = None,
):
    """Start llamafile LLM server."""
    project_dir = Path(__file__).parent
    if sys.platform == "win32":
        llamafile = project_dir / "llamafile.exe"
    elif sys.platform.startswith("linux"):
        llamafile = project_dir / "llamafile-linux"
        if not llamafile.exists():
            llamafile = project_dir / "llamafile"
    else:
        llamafile = project_dir / "llamafile"

    if not llamafile.exists():
        print("⚠️ llamafile not found. Place it in the usbrain directory.")
        # #region debug-point B:llm-missing-binary
        _debug_event(
            "B",
            "main.py:start_llama_server",
            "llamafile binary missing",
            binary=str(llamafile),
        )
        # #endregion
        return None
    
    model = Path(model_path)
    if not model.exists():
        print(f"⚠️ Model not found: {model_path}")
        # #region debug-point D:model-missing
        _debug_event(
            "D",
            "main.py:start_llama_server",
            "Model file missing before llamafile spawn",
            model_path=str(model),
        )
        # #endregion
        return None

    runtime_tuning = runtime_tuning or {}
    generation_threads = int(runtime_tuning.get("generation_threads") or max(1, (os.cpu_count() or 4) // 2))
    batch_threads = int(runtime_tuning.get("batch_threads") or generation_threads)
    batch_size = int(runtime_tuning.get("batch_size") or 512)
    use_no_mmap = bool(runtime_tuning.get("use_no_mmap"))

    # #region debug-point B:llm-spawn
    _debug_event(
        "B",
        "main.py:start_llama_server",
        "Spawning llamafile process",
        binary=str(llamafile),
        model_path=str(model),
        port=port,
        platform=sys.platform,
        ctx_size=ctx_size,
        generation_threads=generation_threads,
        batch_threads=batch_threads,
        batch_size=batch_size,
        use_no_mmap=use_no_mmap,
        server_log=str(LLM_SERVER_LOG_PATH),
    )
    # #endregion
    try:
        if LLM_SERVER_LOG_PATH.exists():
            LLM_SERVER_LOG_PATH.unlink()
    except Exception:
        pass
    log_handle = LLM_SERVER_LOG_PATH.open("a", encoding="utf-8", errors="replace")
    command = [
        str(llamafile),
        "--server",
        "-m",
        str(model),
        "--host",
        "127.0.0.1",
        "--port",
        str(port),
        "-c",
        str(ctx_size),
        "-t",
        str(generation_threads),
        "-tb",
        str(batch_threads),
        "-b",
        str(batch_size),
        "-ngl",
        "0",
    ]
    if use_no_mmap:
        command.append("--no-mmap")
    try:
        proc = subprocess.Popen(
            command,
            stdout=log_handle,
            stderr=log_handle,
        )
    except OSError as exc:
        # Some llamafile distributions on Unix-like hosts rely on shell fallback
        # when direct exec returns ENOEXEC.
        if exc.errno != errno.ENOEXEC or sys.platform == "win32":
            raise
        shell_command = ["/bin/sh", str(llamafile), *command[1:]]
        _debug_event(
            "B",
            "main.py:start_llama_server",
            "Retrying llamafile via shell after ENOEXEC",
            binary=str(llamafile),
            shell="/bin/sh",
            port=port,
            model_path=str(model),
        )
        proc = subprocess.Popen(
            shell_command,
            stdout=log_handle,
            stderr=log_handle,
        )
    proc._usbrain_log_handle = log_handle
    return proc


def start_web_ui(host: str = "127.0.0.1", port: int = 8000, bge_port: int = DEFAULT_BGE_PORT, llm_port: int = DEFAULT_LLM_PORT, model: str = "phi-3.5-mini"):
    """Start the web UI server (blocking)."""
    # Late import to avoid circular dependency
    sys.path.insert(0, str(Path(__file__).parent))
    # #region debug-point E:webui-preflight
    _debug_event(
        "E",
        "main.py:start_web_ui",
        "Preparing Web UI",
        host=host,
        port=port,
        bge_port=bge_port,
        llm_port=llm_port,
        model=model,
        static_dir_exists=(Path(__file__).parent / "webui" / "static").exists(),
        template_exists=(Path(__file__).parent / "webui" / "templates" / "index.html").exists(),
    )
    # #endregion
    # #region debug-point B:webui-critical-imports
    try:
        from webui.server import WebUIServer
        from agent import USBAgent
        _debug_event(
            "B",
            "main.py:start_web_ui:imports",
            "Imported Web UI and agent modules",
            webui_server_imported=True,
            agent_imported=True,
        )
    except Exception as error:
        _debug_event(
            "B",
            "main.py:start_web_ui:imports",
            "Failed importing Web UI or agent modules",
            error_type=type(error).__name__,
            error_text=str(error),
            traceback=traceback.format_exc(),
        )
        raise
    # #endregion

    # #region debug-point B:agent-bootstrap
    try:
        agent = USBAgent(
            llm_base_url=f"http://127.0.0.1:{llm_port}",
            bge_url=f"http://127.0.0.1:{bge_port}",
            model=model,
            knowledge_dir=str(Path(__file__).parent / "knowledge"),
            db_path=str(Path(__file__).parent / "vetor.db"),
            personality_path=str(Path(__file__).parent / "personality.md"),
        )
        _debug_event(
            "B",
            "main.py:start_web_ui:agent-bootstrap",
            "USBAgent initialized",
            model=model,
            knowledge_dir=str(Path(__file__).parent / "knowledge"),
            db_path=str(Path(__file__).parent / "vetor.db"),
        )
    except Exception as error:
        _debug_event(
            "B",
            "main.py:start_web_ui:agent-bootstrap",
            "USBAgent initialization failed",
            error_type=type(error).__name__,
            error_text=str(error),
            traceback=traceback.format_exc(),
        )
        raise
    # #endregion

    # Register core tools
    # #region debug-point A:core-tools-bootstrap
    try:
        from tools.shell import ShellTool
        from tools.filesystem import (
            ReadFileTool,
            WriteFileTool,
            CreateDirectoryTool,
            ListDirectoryTool,
            CopyPathTool,
            MovePathTool,
            CreateZipTool,
            ExtractZipTool,
            DownloadUrlTool,
            FileChecksumTool,
            CompareFilesTool,
            ReplaceInFileTool,
            DirectoryManifestTool,
            CompareDirectoriesTool,
            BatchDeleteTool,
            DeletePathTool,
        )
        from tools.web import WebSearchTool
        agent.tools.register(ShellTool())
        agent.tools.register(ReadFileTool())
        agent.tools.register(WriteFileTool())
        agent.tools.register(CreateDirectoryTool())
        agent.tools.register(ListDirectoryTool())
        agent.tools.register(CopyPathTool())
        agent.tools.register(MovePathTool())
        agent.tools.register(CreateZipTool())
        agent.tools.register(ExtractZipTool())
        agent.tools.register(DownloadUrlTool())
        agent.tools.register(FileChecksumTool())
        agent.tools.register(CompareFilesTool())
        agent.tools.register(ReplaceInFileTool())
        agent.tools.register(DirectoryManifestTool())
        agent.tools.register(CompareDirectoriesTool())
        agent.tools.register(BatchDeleteTool())
        agent.tools.register(DeletePathTool())
        agent.tools.register(WebSearchTool())
        _debug_event(
            "A",
            "main.py:start_web_ui:core-tools-bootstrap",
            "Core tools registered",
            tool_count=len(agent.tools.list_tools()),
            tool_names=[tool.name for tool in agent.tools.list_tools()],
        )
    except Exception as error:
        _debug_event(
            "A",
            "main.py:start_web_ui:core-tools-bootstrap",
            "Core tools import or registration failed",
            error_type=type(error).__name__,
            error_text=str(error),
            traceback=traceback.format_exc(),
        )
        raise
    # #endregion

    # Register skills
    # #region debug-point A:skills-bootstrap
    try:
        from skills.network import NetworkDiagnoseTool
        from skills.system import (
            SystemInfoTool,
            ProcessListTool,
            KillProcessTool,
            EnvironmentTool,
            PortInspectTool,
            ServiceListTool,
            SystemLogsTool,
        )
        from skills.files import FileSearchTool, FileInfoTool
        from skills.winfix import WindowsFixTool
        agent.tools.register(NetworkDiagnoseTool())
        agent.tools.register(SystemInfoTool())
        agent.tools.register(ProcessListTool())
        agent.tools.register(KillProcessTool())
        agent.tools.register(EnvironmentTool())
        agent.tools.register(PortInspectTool())
        agent.tools.register(ServiceListTool())
        agent.tools.register(SystemLogsTool())
        agent.tools.register(FileSearchTool())
        agent.tools.register(FileInfoTool())
        agent.tools.register(WindowsFixTool())
        _debug_event(
            "A",
            "main.py:start_web_ui:skills-bootstrap",
            "Skills registered",
            tool_count=len(agent.tools.list_tools()),
            tool_names=[tool.name for tool in agent.tools.list_tools()],
        )
    except Exception as error:
        _debug_event(
            "A",
            "main.py:start_web_ui:skills-bootstrap",
            "Skills import or registration failed",
            error_type=type(error).__name__,
            error_text=str(error),
            traceback=traceback.format_exc(),
        )
        raise
    # #endregion

    # #region debug-point C:webui-server-run
    try:
        server = WebUIServer(agent, host=host, port=port)
        _debug_event(
            "C",
            "main.py:start_web_ui:web-server",
            "WebUIServer created successfully",
            host=host,
            port=port,
        )
        server.run()
    except Exception as error:
        _debug_event(
            "C",
            "main.py:start_web_ui:web-server",
            "Web UI server failed before or during run",
            error_type=type(error).__name__,
            error_text=str(error),
            traceback=traceback.format_exc(),
        )
        raise
    # #endregion


def main():
    parser = argparse.ArgumentParser(description="USBRAIN - Portable AI Agent")
    parser.add_argument("--host", default="127.0.0.1", help="Web UI host")
    parser.add_argument("--port", type=int, default=8000, help="Web UI port")
    parser.add_argument("--bge-port", type=int, default=DEFAULT_BGE_PORT, help="BGE server port")
    parser.add_argument("--llm-port", type=int, default=DEFAULT_LLM_PORT, help="LLM server port")
    parser.add_argument("--model", default=None, help="Model name (auto-detect if not set)")
    parser.add_argument("--model-path", default=None, help="Path to GGUF model file")
    parser.add_argument(
        "--list-model-options",
        action="store_true",
        help="List compatible local GGUF models for the current hardware and exit",
    )
    parser.add_argument("--no-llm", action="store_true", help="Don't start LLM (BGE only + UI)")
    parser.add_argument("--db", default="vetor.db", help="Vector database path")

    args = parser.parse_args()
    project_dir = Path(__file__).parent
    install_cleanup_hooks()

    if args.list_model_options:
        hw = detect_hardware()
        manifest_model_path, manifest_model_id = resolve_model_from_installed_manifest(project_dir)
        preferred_model = args.model or manifest_model_id or hw["recommended_model"]
        options = list_compatible_model_options(project_dir / "models", hw["ram_gb"], preferred_model)
        for option in options:
            print(
                f"{option['index']}|{option['label']}|{option['file_name']}|"
                f"{1 if option['recommended'] else 0}|{option['path']}"
            )
        return

    try:
        if DEBUG_LOG_PATH.exists():
            DEBUG_LOG_PATH.unlink()
    except Exception:
        pass

    try:
        cleanup_stale_child_processes(project_dir)

        hw = detect_hardware()
        free_ram_label = f", {hw['free_ram_gb']}GB livres" if hw.get("free_ram_gb") is not None else ""
        print(f"Hardware detectado: {hw['ram_gb']}GB RAM{free_ram_label}")
        # #region debug-point E:boot-args
        _debug_event(
            "E",
            "main.py:main",
            "Boot sequence started",
            argv=sys.argv,
            cwd=os.getcwd(),
            project_dir=str(project_dir),
            python_executable=sys.executable,
            platform=sys.platform,
            web_port=args.port,
            bge_port=args.bge_port,
            llm_port=args.llm_port,
            no_llm=args.no_llm,
            ram_gb=hw["ram_gb"],
            free_ram_gb=hw.get("free_ram_gb"),
            recommended_model=hw["recommended_model"],
        )
        # #endregion
        
        manifest_model_path, manifest_model_id = resolve_model_from_installed_manifest(project_dir)
        model_name = args.model or manifest_model_id or hw["recommended_model"]
        model_path = args.model_path
        if model_path and not args.model:
            model_name = get_model_label(Path(model_path).stem)
        print(f"Modelo preferencial: {get_model_label(model_name)}")
        if not args.no_llm and not model_path:
            model_path_obj = manifest_model_path or choose_model_file(
                project_dir / "models",
                hw["ram_gb"],
                args.model or manifest_model_id or hw["recommended_model"],
            )
            if model_path_obj:
                model_path = str(model_path_obj)
                model_name = get_model_label(model_path_obj.stem)
                print(f"Modelo selecionado: {model_path_obj.name}")
                # #region debug-point D:model-selected
                _debug_event(
                    "D",
                    "main.py:main",
                    "Model selected for boot",
                    selected_model=str(model_path_obj),
                    model_label=model_name,
                )
                # #endregion
            else:
                print("Nenhum modelo GGUF compativel encontrado em models/. Iniciando sem LLM.")
                args.no_llm = True
                # #region debug-point D:no-model
                _debug_event(
                    "D",
                    "main.py:main",
                    "No compatible GGUF model found; switching to no-llm mode",
                    models_dir=str(project_dir / "models"),
                )
                # #endregion

        args.bge_port = choose_available_port("BGE", "127.0.0.1", args.bge_port, "C", "main.py:main")
        if not args.no_llm and model_path:
            args.llm_port = choose_available_port(
                "LLM",
                "127.0.0.1",
                args.llm_port,
                "B",
                "main.py:main",
                reserved_ports={args.bge_port},
            )
        llm_ctx_size = choose_llm_ctx_size(hw)
        llm_runtime_tuning = choose_llm_runtime_tuning(hw, model_path) if model_path else None
        _debug_event(
            "B",
            "main.py:main",
            "Calculated dynamic LLM context size",
            free_ram_gb=hw.get("free_ram_gb"),
            reserve_gb=LLM_RAM_HEADROOM_GB,
            min_ctx=MIN_LLM_CTX_SIZE,
            max_ctx=MAX_LLM_CTX_SIZE,
            selected_ctx=llm_ctx_size,
        )
        if llm_runtime_tuning:
            _debug_event(
                "B",
                "main.py:main",
                "Calculated LLM runtime tuning",
                physical_cores=hw.get("physical_cores"),
                logical_cores=hw.get("logical_cores"),
                model_size_gb=llm_runtime_tuning.get("model_size_gb"),
                generation_threads=llm_runtime_tuning.get("generation_threads"),
                batch_threads=llm_runtime_tuning.get("batch_threads"),
                batch_size=llm_runtime_tuning.get("batch_size"),
                use_no_mmap=llm_runtime_tuning.get("use_no_mmap"),
            )

        # Start BGE server (opcional)
        bge_proc = None
        try:
            print(f"Iniciando BGE server na porta {args.bge_port}...")
            bge_proc = start_bge_server(args.db, args.bge_port)
            if bge_proc:
                register_child_process("BGE", bge_proc)
                watch_child_process_exit("BGE", bge_proc, "C", "main.py:main")
                bge_ready = wait_for_http_ready(
                    "BGE",
                    f"http://127.0.0.1:{args.bge_port}/stats",
                    timeout=600,
                    hypothesis_id="C",
                    location="main.py:main",
                    proc=bge_proc,
                )
                # #region debug-point C:bge-state
                _log_process_state("BGE", bge_proc, "C", "main.py:main")
                # #endregion
                if not bge_ready:
                    print("  BGE nao ficou pronto a tempo.")
                    sys.exit(1)
        except Exception as e:
            print(f"  BGE nao iniciou: {e}")
            # #region debug-point C:bge-exception
            _debug_event(
                "C",
                "main.py:main",
                "Exception raised while starting BGE server",
                error=repr(e),
                traceback=traceback.format_exc(),
            )
            # #endregion

        # Start LLM if requested
        llm_proc = None
        try:
            if not args.no_llm and model_path:
                print(f"Iniciando LLM ({model_name}) na porta {args.llm_port} com contexto {llm_ctx_size}...")
                llm_proc = start_llama_server(model_path, args.llm_port, llm_ctx_size, llm_runtime_tuning)
                if llm_proc:
                    register_child_process("LLM", llm_proc)
                    watch_child_process_exit("LLM", llm_proc, "B", "main.py:main")
                    llm_ready = wait_for_http_ready(
                        "LLM",
                        f"http://127.0.0.1:{args.llm_port}/v1/models",
                        timeout=900,
                        hypothesis_id="B",
                        location="main.py:main",
                        proc=llm_proc,
                    )
                    # #region debug-point B:llm-state
                    _log_process_state("LLM", llm_proc, "B", "main.py:main")
                    # #endregion
                    if not llm_ready:
                        print("  LLM nao ficou pronta a tempo.")
                        sys.exit(1)
                    start_llm_keepalive_monitor(llm_proc, args.llm_port)
        except Exception as e:
            print(f"  LLM nao iniciou: {e}")
            # #region debug-point B:llm-exception
            _debug_event(
                "B",
                "main.py:main",
                "Exception raised while starting llamafile",
                error=repr(e),
                traceback=traceback.format_exc(),
            )
            # #endregion

        # Start Web UI (blocking) - com fallback de porta
        print(f"Iniciando interface web em http://{args.host}:{args.port}")
        try:
            start_web_ui(args.host, args.port, args.bge_port, args.llm_port, model_name)
        except OSError as e:
            err = str(e).lower()
            if "address already in use" in err or "cannot assign" in err:
                for alt_port in range(args.port + 1, args.port + 100):
                    try:
                        print(f"Porta {args.port} ocupada, tentando {alt_port}...")
                        # #region debug-point E:webui-port-fallback
                        _debug_event(
                            "E",
                            "main.py:main",
                            "Primary Web UI port unavailable; trying fallback",
                            requested_port=args.port,
                            fallback_port=alt_port,
                        )
                        # #endregion
                        start_web_ui(args.host, alt_port, args.bge_port, args.llm_port, model_name)
                        break
                    except OSError:
                        continue
                else:
                    print(f"Nenhuma porta disponivel de {args.port} a {args.port + 99}")
                    # #region debug-point E:webui-port-exhausted
                    _debug_event(
                        "E",
                        "main.py:main",
                        "No available Web UI ports in fallback range",
                        requested_port=args.port,
                    )
                    # #endregion
                    sys.exit(1)
            else:
                # #region debug-point E:webui-oserror
                _debug_event(
                    "E",
                    "main.py:main",
                    "Unhandled OSError while starting Web UI",
                    error=repr(e),
                    traceback=traceback.format_exc(),
                )
                # #endregion
                raise
        except Exception as e:
            # #region debug-point E:webui-exception
            _debug_event(
                "E",
                "main.py:main",
                "Unhandled exception while starting Web UI",
                error=repr(e),
                traceback=traceback.format_exc(),
            )
            # #endregion
            raise
    finally:
        cleanup_child_processes("main-exit")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        # #region debug-point E:main-crash
        _debug_event(
            "E",
            "main.py:__main__",
            "Process terminated by unhandled exception",
            error=repr(exc),
            traceback=traceback.format_exc(),
        )
        # #endregion
        raise
