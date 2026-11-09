# segunda-opinion — MCP server

Independent second opinions from non-Claude models (Gemini, DeepSeek, open to more) for the
reviewer agents (`paper-reviewer`, `code-reviewer`, `revisor-final`). Read-only: files go in,
a list of findings comes out. Claude verifies every finding; only confirmed ones count.

## Setup

1. Free API keys, as **user** environment variables (never in the repo):
   - `GEMINI_API_KEY` — <https://aistudio.google.com/apikey> (Gemini, free tier).
   - `NVIDIA_API_KEY` — <https://build.nvidia.com> (NVIDIA NIM, free: DeepSeek, GLM, Nemotron...).

   Or put both in `.env` next to `server.py` (git-ignored; loaded at start-up).

   PowerShell: `setx GEMINI_API_KEY "..."`, then restart the terminal and Claude Code.
2. Check keys and model ids: `cd .claude/mcp/segunda-opinion && uv run server.py --check`.
   Free model ids change often: if one is `NOT FOUND`, pick a listed one and edit `providers.toml`.
3. Claude Code picks the server up from `.mcp.json` at the repo root and asks once to approve it
   (`/mcp` shows its state).

Free tiers may use the prompts for training (`trains_on_data = true`); the authors accepted
sending course PDFs and our own work.

## Tools

| Tool | What it does |
|---|---|
| `list_models()` | configured models, key status, PDF support, context size |
| `second_opinion(project, files, task, kind="paper", models=None, max_findings=15)` | sends the files to every available model in parallel and returns their findings |

- `files` are repo-relative paths or globs; **the server reads them itself**, so Claude spends
  no tokens on them. PDFs go to Gemini as files (figures and formulas included) and as extracted
  text to the others; notebooks as code plus text outputs; text files with line numbers.
- `kind`: `paper`, `code` or `full` selects the instructions in `prompts/review.md`.
- Inputs longer than a model's `max_input_tokens` are truncated proportionally (marked
  `[TRUNCATED]`).
- Each finding gets `quote_found`: whether its quoted evidence appears verbatim in the files
  (`false` usually means a hallucination; `null` means it quoted nothing).
- Full answers are saved in `<project>/ignore/segunda_opinion/` (not committed).
- A failing model (quota, timeout) does not fail the call; it is reported in its own line.

Safety: only files inside the repository; `.git`, `.venv`, `.claude`, `target` and names that
look like secrets are refused; at most 40 files and 25 MB per file.

## Free tiers are flaky

Free models are often overloaded (Gemini 503/504, DeepSeek and GLM on NIM silent) and NIM still
lists many retired models (404 "Function ... Not found for account"). Hence `fallback_models`,
streaming with `first_token_s` (an overloaded model fails in ~90 s), and per-model failures that
never fail the whole call. Reasoning models need a large `max_output_tokens`, or they spend it all
thinking and return nothing.

## Adding a model

Copy a block in `providers.toml`. Any OpenAI-compatible `/chat/completions` API works
(OpenRouter, Groq, a local Ollama with `api_key_env = ""`); Gemini uses the native adapter.

## Testing without keys

`uv run server.py --dry-run <PROJECT> "<glob>" ...` shows what each model would receive and
whether it fits its budget.
