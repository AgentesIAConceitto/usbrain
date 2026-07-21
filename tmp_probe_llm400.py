from __future__ import annotations

import asyncio
import json
import urllib.request
from pathlib import Path

import httpx

from agent import USBAgent


def build_body(system_prompt: str) -> dict:
    return {
        "model": "Phi-3.5 Mini",
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": "boa noite"},
        ],
        "temperature": 0.7,
        "max_tokens": 4096,
    }


def run_urllib(body: dict) -> tuple[str, str]:
    req = urllib.request.Request(
        "http://127.0.0.1:8090/v1/chat/completions",
        data=json.dumps(body).encode(),
        headers={
            "Content-Type": "application/json",
            "Authorization": "Bearer no-key",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as response:
            return f"status={response.status}", response.read().decode("utf-8", "replace")[:1500]
    except Exception as exc:
        extra = exc.read().decode("utf-8", "replace")[:1500] if hasattr(exc, "read") else ""
        return f"error={type(exc).__name__}: {exc}", extra


async def run_httpx(body: dict) -> tuple[str, str]:
    async with httpx.AsyncClient(timeout=60.0) as client:
        try:
            response = await client.post(
                "http://127.0.0.1:8090/v1/chat/completions",
                json=body,
                headers={
                    "Content-Type": "application/json",
                    "Authorization": "Bearer no-key",
                },
            )
            return f"status={response.status_code}", response.text[:1500]
        except Exception as exc:
            extra = ""
            if hasattr(exc, "response") and exc.response is not None:
                extra = exc.response.text[:1500]
            return f"error={type(exc).__name__}: {exc}", extra


async def main() -> None:
    agent = USBAgent(
        llm_base_url="http://127.0.0.1:8090",
        bge_url="http://127.0.0.1:9090",
        model="Phi-3.5 Mini",
        knowledge_dir=str(Path("knowledge")),
        db_path="vetor.db",
        personality_path=str(Path("personality.md")),
    )

    cases = {
        "simple_prompt": "You are a helpful assistant.",
        "real_system_prompt": agent.system_prompt,
    }

    report_lines = []
    for name, prompt in cases.items():
        body = build_body(prompt)
        report_lines.append(f"== {name} ==")
        report_lines.append(f"prompt_chars={len(prompt)}")
        report_lines.append(f"body_bytes={len(json.dumps(body, ensure_ascii=True).encode())}")

        urllib_status, urllib_text = run_urllib(body)
        report_lines.append(f"urllib: {urllib_status}")
        report_lines.append(urllib_text)

        httpx_status, httpx_text = await run_httpx(body)
        report_lines.append(f"httpx: {httpx_status}")
        report_lines.append(httpx_text)
        report_lines.append("")

    output_path = Path("tmp_probe_llm400.out")
    output_path.write_text("\n".join(report_lines), encoding="utf-8")
    print(output_path)


if __name__ == "__main__":
    asyncio.run(main())
