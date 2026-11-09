"""Adapters that send one review request to one model and return its raw text answer."""

import asyncio
import json
import os
import tomllib
from pathlib import Path

import httpx

CONFIG = Path(__file__).resolve().parent / "providers.toml"
DOTENV = Path(__file__).resolve().parent / ".env"
RETRY_STATUS = {429}   # 5xx (overloaded) goes straight to the next fallback model
BACKOFF_S = (20, 40)


class ProviderError(Exception):
    """Request failed after retries; the message is shown to the reviewer."""


def load_dotenv(path=DOTENV):
    """Load KEY=value lines from .env without overriding variables already set."""
    if not path.is_file():
        return
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#") and "=" in line:
            key, value = line.split("=", 1)
            os.environ.setdefault(key.strip(), value.strip().strip("\"'"))


load_dotenv()


def load_providers(path=CONFIG):
    """Read the catalogue and add `name` and `has_key` to each entry."""
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    for name, cfg in data.items():
        cfg["name"] = name
        cfg["has_key"] = bool(os.environ.get(cfg.get("api_key_env", ""), "")) or not cfg.get("api_key_env")
    return data


def available(providers):
    """Enabled providers whose key is set."""
    return {n: c for n, c in providers.items() if c.get("enabled", True) and c["has_key"]}


async def _with_retries(fn, name):
    """Run an async request, retrying on rate limits and server errors."""
    for attempt in range(len(BACKOFF_S) + 1):
        try:
            return await fn()
        except Exception as exc:  # noqa: BLE001 - each SDK raises its own types
            status = getattr(exc, "status_code", None) or getattr(exc, "code", None)
            if isinstance(exc, httpx.HTTPStatusError):
                status = exc.response.status_code
            transient = (httpx.ConnectError, httpx.RemoteProtocolError)   # timeouts are not retried
            retryable = status in RETRY_STATUS or isinstance(exc, transient)
            if not retryable or attempt == len(BACKOFF_S):
                raise ProviderError(f"{name}: {type(exc).__name__}: {str(exc)[:300]}") from exc
            await asyncio.sleep(BACKOFF_S[attempt])


def auth_headers(cfg):
    """Bearer header, or none for keyless local servers."""
    key = os.environ.get(cfg.get("api_key_env", ""), "") if cfg.get("api_key_env") else ""
    return {"Authorization": f"Bearer {key}"} if key else {}


async def call_openai_compat(cfg, system, user_text):
    """POST to an OpenAI-compatible /chat/completions endpoint; returns (text, usage)."""
    headers = auth_headers(cfg)
    body = {
        "model": cfg["model"],
        "messages": [{"role": "system", "content": system}, {"role": "user", "content": user_text}],
        "temperature": 0.2,
        "max_tokens": cfg.get("max_output_tokens", 8000),
        "stream": True,
        "stream_options": {"include_usage": True},
        **cfg.get("extra_body", {}),
    }
    url = cfg["base_url"].rstrip("/") + "/chat/completions"
    # Streaming: an overloaded model sends nothing and fails after `first_token_s`;
    # a working one keeps sending chunks (reasoning included), bounded overall by `timeout_s`.
    timeout = httpx.Timeout(cfg.get("first_token_s", 90), connect=20)

    async def request():
        parts, usage = [], {}
        async with httpx.AsyncClient(timeout=timeout) as client:
            async with asyncio.timeout(cfg.get("timeout_s", 600)):
                async with client.stream("POST", url, headers=headers, json=body) as r:
                    if r.status_code >= 400:
                        await r.aread()
                        r.raise_for_status()
                    async for line in r.aiter_lines():
                        if not line.startswith("data:") or line.strip() == "data: [DONE]":
                            continue
                        chunk = json.loads(line[5:])
                        usage = chunk.get("usage") or usage
                        for choice in chunk.get("choices", []):
                            parts.append((choice.get("delta") or {}).get("content") or "")
        text = "".join(parts)
        if not text.strip():
            details = usage.get("completion_tokens_details") or {}
            raise ProviderError(f"{cfg['name']}: empty answer (completion tokens {usage.get('completion_tokens')}, "
                                f"reasoning {details.get('reasoning_tokens')}): raise max_output_tokens")
        return text, usage

    return await _with_retries(request, cfg["name"])


async def call_gemini(cfg, system, user_text, pdfs):
    """Call Gemini with the native SDK, attaching PDFs as files; returns (text, usage)."""
    from google import genai
    from google.genai import types

    client = genai.Client(api_key=os.environ[cfg["api_key_env"]])
    parts = [types.Part.from_bytes(data=p.read_bytes(), mime_type="application/pdf") for p in pdfs]
    parts.append(types.Part.from_text(text=user_text))
    config = types.GenerateContentConfig(
        system_instruction=system,
        temperature=0.2,
        max_output_tokens=cfg.get("max_output_tokens", 16000),
        response_mime_type="application/json",
        http_options=types.HttpOptions(timeout=cfg.get("timeout_s", 300) * 1000),
    )

    async def request():
        resp = await client.aio.models.generate_content(model=cfg["model"], contents=parts, config=config)
        um = resp.usage_metadata
        usage = {"prompt_tokens": getattr(um, "prompt_token_count", None),
                 "completion_tokens": getattr(um, "candidates_token_count", None)}
        return resp.text or "", usage

    return await _with_retries(request, cfg["name"])


async def list_remote_models(cfg):
    """Model ids the provider exposes for this key (used by --check)."""
    if cfg["adapter"] == "gemini":
        from google import genai

        client = genai.Client(api_key=os.environ[cfg["api_key_env"]])
        return [m.name.removeprefix("models/") async for m in await client.aio.models.list()]
    headers = auth_headers(cfg)
    async with httpx.AsyncClient(timeout=30) as client:
        r = await client.get(cfg["base_url"].rstrip("/") + "/models", headers=headers)
        r.raise_for_status()
        return [m["id"] for m in r.json()["data"]]
