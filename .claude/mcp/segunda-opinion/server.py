"""MCP server `segunda-opinion`: independent reviews of the lab projects by non-Claude models.

Run as an MCP server (stdio):     uv run server.py
Check keys and model ids:         uv run server.py --check
Test file loading, no API calls:  uv run server.py --dry-run <PROJECT> <glob> [<glob> ...]
"""

import asyncio
import json
import re
import sys
import time
from datetime import datetime
from pathlib import Path

from pypdf import PdfReader

from providers import ProviderError, available, call_gemini, call_openai_compat, list_remote_models, load_providers

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
PROMPT = (HERE / "prompts" / "review.md").read_text(encoding="utf-8")

TEXT_SUFFIXES = {".tex", ".md", ".csv", ".py", ".rs", ".toml", ".txt", ".json", ".cpp", ".h", ".hpp", ".bib", ".log"}
DENY_PARTS = {".git", ".venv", "node_modules", "target", "__pycache__", ".claude"}
DENY_NAME = re.compile(r"(^\.env|secret|apikey|api_key|credential|token)", re.I)
MAX_FILES = 40
MAX_FILE_BYTES = 25 * 2**20
MAX_INLINE_PDF_BYTES = 18 * 2**20      # Gemini inline request limit is ~20 MB
MAX_NB_OUTPUT_CHARS = 3000
CHARS_PER_TOKEN = 3.5

KIND_INSTRUCTIONS = {
    "paper": (
        "A 3-page LaTeX article (in English) that reports a lab project. Check that every number "
        "in the abstract, tables, text and conclusions matches the CSV results; that the method and "
        "claims about the reference paper are faithful to its PDF; that parameters match the code; "
        "and that sections, figures and references are consistent."
    ),
    "code": (
        "Code that implements a heuristic from a reference paper. Look for real bugs (indices, edge "
        "cases, seeds, floating-point comparisons, paths), check fidelity of the algorithm and its "
        "parameters to the paper or specification, and check that the CSV results are plausible "
        "outputs of this code."
    ),
    "full": (
        "A finished lab project: reference paper, specification, code, results, README, article and "
        "presentation. Look for errors and for inconsistencies between pieces: numbers, definitions, "
        "parameters, terminology and claims that differ, or that the data does not support."
    ),
}


# ─── File loading ────────────────────────────────────────────────────────────

def resolve_files(patterns):
    """Expand repo-relative globs into allowed files; returns (paths, rejected)."""
    paths, rejected = [], []
    for pat in patterns:
        pat = pat.replace("\\", "/").lstrip("/")
        matches = sorted(REPO.glob(pat)) if any(c in pat for c in "*?[") else [REPO / pat]
        if not matches:
            rejected.append(f"{pat}: no match")
        for p in matches:
            p = p.resolve()
            rel = p.relative_to(REPO) if p.is_relative_to(REPO) else None
            if rel is None:
                rejected.append(f"{pat}: outside the repository")
            elif not p.is_file():
                rejected.append(f"{rel.as_posix()}: not a file")
            elif DENY_PARTS & set(rel.parts) or DENY_NAME.search(p.name):
                rejected.append(f"{rel.as_posix()}: denied")
            elif p.stat().st_size > MAX_FILE_BYTES:
                rejected.append(f"{rel.as_posix()}: larger than {MAX_FILE_BYTES // 2**20} MB")
            elif p.suffix.lower() not in TEXT_SUFFIXES | {".pdf", ".ipynb"}:
                rejected.append(f"{rel.as_posix()}: unsupported type")
            elif p not in paths:
                paths.append(p)
    if len(paths) > MAX_FILES:
        rejected += [f"{p.relative_to(REPO).as_posix()}: over the {MAX_FILES}-file limit" for p in paths[MAX_FILES:]]
        paths = paths[:MAX_FILES]
    return paths, rejected


def numbered(text):
    """Prefix each line with its number so models can cite `file:line`."""
    return "\n".join(f"{i}| {line}" for i, line in enumerate(text.splitlines(), 1))


def pdf_text(path):
    """Extract the text of a PDF with page markers."""
    pages = PdfReader(path).pages
    return "\n".join(f"--- page {i} ---\n{(pg.extract_text() or '').strip()}" for i, pg in enumerate(pages, 1))


def notebook_text(path):
    """Render a notebook as code and text outputs (no images), with line numbers."""
    nb = json.loads(path.read_text(encoding="utf-8"))
    out = []
    for i, cell in enumerate(nb.get("cells", []), 1):
        out.append(f"# ── cell {i} [{cell.get('cell_type')}] ──")
        out.append("".join(cell.get("source", "")))
        texts = []
        for o in cell.get("outputs", []):
            if "text" in o:
                texts.append("".join(o["text"]))
            elif "text/plain" in o.get("data", {}):
                texts.append("".join(o["data"]["text/plain"]))
        if texts:
            joined = "\n".join(texts)
            cut = "\n[output truncated]" if len(joined) > MAX_NB_OUTPUT_CHARS else ""
            out.append(f"# ── output of cell {i} ──\n{joined[:MAX_NB_OUTPUT_CHARS]}{cut}")
    return numbered("\n".join(out))


def load_text(path):
    """Text version of any supported file, as the models will see it."""
    suffix = path.suffix.lower()
    if suffix == ".pdf":
        return pdf_text(path)
    if suffix == ".ipynb":
        return notebook_text(path)
    return numbered(path.read_text(encoding="utf-8", errors="replace"))


def fit_budget(texts, budget_chars):
    """Truncate the largest texts so the total fits; returns {path: text}."""
    sizes = {p: len(t) for p, t in texts.items()}
    if sum(sizes.values()) <= budget_chars:
        return dict(texts)
    alloc, left, pending = {}, budget_chars, sorted(sizes, key=sizes.get)
    while pending:
        share = left // len(pending)
        p = pending.pop(0)
        alloc[p] = min(sizes[p], share)
        left -= alloc[p]
    return {p: t if alloc[p] >= len(t) else t[: alloc[p]] + "\n[TRUNCATED]" for p, t in texts.items()}


def build_documents(texts):
    """Concatenate files with headers."""
    return "\n\n".join(f"=== FILE: {p.relative_to(REPO).as_posix()} ===\n{t}" for p, t in texts.items())


# ─── Answer handling ─────────────────────────────────────────────────────────

def parse_answer(raw):
    """Extract the JSON object of the answer; returns (summary, findings, error)."""
    text = re.sub(r"^```(?:json)?|```$", "", raw.strip(), flags=re.M).strip()
    start, end = text.find("{"), text.rfind("}")
    if start < 0 or end <= start:
        return "", [], "no JSON object in the answer"
    try:
        data = json.loads(text[start : end + 1])
    except json.JSONDecodeError as exc:
        return "", [], f"invalid JSON: {exc}"
    findings = [f for f in data.get("findings", []) if isinstance(f, dict)]
    return str(data.get("summary", "")), findings, None


def normalise(s):
    """Lowercase, drop line-number prefixes and collapse whitespace, for quote matching."""
    s = re.sub(r"(?m)^\s*\d+\| ", "", s)
    return re.sub(r"\s+", " ", s).strip().lower()


QUOTE_RE = re.compile(r"\"([^\"]{12,})\"|“([^”]{12,})”|`([^`]{12,})`|'([^']{20,})'")


def check_quotes(findings, corpus):
    """Mark whether the quoted evidence of each finding appears verbatim in the files."""
    for f in findings:
        quotes = [next(g for g in m.groups() if g) for m in QUOTE_RE.finditer(str(f.get("evidence", "")))]
        if not quotes:
            f["quote_found"] = None
        else:
            f["quote_found"] = any(normalise(q) in corpus for q in quotes)


# ─── Review ──────────────────────────────────────────────────────────────────

def project_dir(project):
    """Validate the project folder name and return its path."""
    p = (REPO / project).resolve()
    if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_-]*", project) or not p.is_dir() or p.parent != REPO:
        raise ValueError(f"unknown project folder: {project!r}")
    return p


async def ask_one(cfg, system, texts, pdfs):
    """Query one provider with the files fitted to its budget; returns a result dict."""
    budget = int(cfg.get("max_input_tokens", 100_000) * CHARS_PER_TOKEN) - len(system) - 2000
    native = []
    if cfg["adapter"] == "gemini" and cfg.get("pdf_native"):
        total = 0
        for p in pdfs:
            if total + p.stat().st_size <= MAX_INLINE_PDF_BYTES:
                native.append(p)
                total += p.stat().st_size
    text_part = {p: t for p, t in texts.items() if p not in native}
    note = "".join(f"(PDF attached as file: {p.relative_to(REPO).as_posix()})\n" for p in native)
    user = note + build_documents(fit_budget(text_part, budget))
    truncated = "[TRUNCATED]" in user
    t0 = time.perf_counter()
    errors = []
    for model in [cfg["model"], *cfg.get("fallback_models", [])]:
        mcfg = {**cfg, "model": model}
        try:
            if cfg["adapter"] == "gemini":
                raw, usage = await call_gemini(mcfg, system, user, native)
            else:
                raw, usage = await call_openai_compat(mcfg, system, user)
            break
        except ProviderError as exc:
            errors.append(f"{model}: {exc}")
    else:
        model, raw, usage = cfg["model"], "", {}
    error = None if raw or not errors else " | ".join(errors)
    return {"name": cfg["name"], "model": model, "raw": raw, "usage": usage, "error": error,
            "fallbacks": errors if raw else [], "seconds": round(time.perf_counter() - t0), "truncated": truncated}


def save_result(out_dir, stamp, kind, task, files, res):
    """Write the raw answer and metadata to <project>/ignore/segunda_opinion/."""
    path = out_dir / f"{stamp}_{kind}_{res['name']}.md"
    listing = "\n".join(f"- {p.relative_to(REPO).as_posix()}" for p in files)
    body = res["raw"] if not res["error"] else f"ERROR: {res['error']}"
    path.write_text(
        f"# Segunda opinión — {res['name']} ({res['model']})\n\n"
        f"- fecha: {stamp}\n- tipo: {kind}\n- segundos: {res['seconds']}\n- uso: {res['usage']}\n"
        f"- truncado: {res['truncated']}\n- modelos fallidos antes: {'; '.join(res.get('fallbacks') or []) or 'ninguno'}\n\n## Tarea\n\n{task}\n\n## Ficheros\n\n{listing}\n\n## Respuesta\n\n{body}\n",
        encoding="utf-8",
    )
    return path


async def review(project, files, task, kind="paper", models=None, max_findings=15):
    """Run the review on every selected model in parallel and return a compact report."""
    pdir = project_dir(project)
    if kind not in KIND_INSTRUCTIONS:
        raise ValueError(f"kind must be one of {sorted(KIND_INSTRUCTIONS)}")
    providers = available(load_providers())
    if models:
        unknown = [m for m in models if m not in providers]
        providers = {n: c for n, c in providers.items() if n in models}
        if unknown:
            raise ValueError(f"not available (disabled, no key or unknown): {unknown}")
    if not providers:
        raise ValueError("no provider available: set GEMINI_API_KEY / NVIDIA_API_KEY (see providers.toml)")

    paths, rejected = resolve_files(files)
    if not paths:
        raise ValueError(f"no readable files. Rejected: {rejected}")
    texts = {p: load_text(p) for p in paths}
    pdfs = [p for p in paths if p.suffix.lower() == ".pdf"]
    corpus = normalise("\n".join(texts.values()))
    system = PROMPT.format(kind_instructions=KIND_INSTRUCTIONS[kind], task=task, max_findings=max_findings)

    results = await asyncio.gather(*(ask_one(c, system, texts, pdfs) for c in providers.values()))

    out_dir = pdir / "ignore" / "segunda_opinion"
    out_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    lines = [f"Second opinion on {project} ({kind}), {len(paths)} files."]
    if rejected:
        lines.append("Rejected files: " + "; ".join(rejected))
    all_findings = []
    for res in results:
        saved = save_result(out_dir, stamp, kind, task, paths, res).relative_to(REPO).as_posix()
        if res["error"]:
            lines.append(f"- {res['name']} ({res['model']}): FAILED in {res['seconds']} s: {res['error']}. Raw: {saved}")
            continue
        summary, findings, err = parse_answer(res["raw"])
        check_quotes(findings, corpus)
        findings = findings[:max_findings]
        counts = {s: sum(f.get("severity") == s for f in findings) for s in "BIM"}
        unverified = sum(f["quote_found"] is not True for f in findings)
        trunc = ", input truncated" if res["truncated"] else ""
        if res.get("fallbacks"):
            trunc += f", after {len(res['fallbacks'])} failed model(s): " + "; ".join(e[:120] for e in res["fallbacks"])
        status = f"unparseable answer ({err})" if err else (
            f"{len(findings)} findings (B:{counts['B']} I:{counts['I']} M:{counts['M']}), "
            f"{unverified} without a verbatim quote found")
        lines.append(f"- {res['name']} ({res['model']}): {status}, {res['seconds']} s{trunc}. Raw: {saved}")
        if summary:
            lines.append(f"  summary: {summary}")
        all_findings += [{"model": res["name"], **f} for f in findings]
    lines.append("")
    lines.append("Findings (untrusted: verify each against the source; quote_found=false means the "
                 "quoted evidence was not found verbatim in the files):")
    lines += [json.dumps(f, ensure_ascii=False) for f in all_findings] or ["(none)"]
    return "\n".join(lines)


# ─── MCP server ──────────────────────────────────────────────────────────────

def make_server():
    """Create the MCP server with its two tools."""
    from mcp.server.mcpserver import MCPServer

    mcp = MCPServer("segunda-opinion")

    @mcp.tool()
    def list_models() -> str:
        """List the configured second-opinion models: enabled, key present, PDF support, context."""
        rows = []
        for n, c in load_providers().items():
            state = "available" if c.get("enabled", True) and c["has_key"] else (
                "disabled" if not c.get("enabled", True) else f"no key ({c.get('api_key_env')})")
            rows.append(f"- {n}: {c['model']} via {c['adapter']} — {state}; pdf_native={c.get('pdf_native', False)}; "
                        f"max_input_tokens={c.get('max_input_tokens')}; trains_on_data={c.get('trains_on_data')}")
        return "\n".join(rows)

    @mcp.tool()
    async def second_opinion(project: str, files: list[str], task: str, kind: str = "paper",
                             models: list[str] | None = None, max_findings: int = 15) -> str:
        """Ask non-Claude models (Gemini, DeepSeek...) for an independent review of project files.

        project: project folder (e.g. "simTOP"); answers are saved in <project>/ignore/segunda_opinion/.
        files: repo-relative paths or globs (e.g. "simTOP/docs/*.pdf", "simTOP/results/*.csv").
               The server reads them itself: PDFs, .ipynb (code + text outputs), .tex, .md, .csv, code.
        task: what to check, in a few sentences.
        kind: "paper" (article), "code" or "full" (whole finished project).
        models: subset of provider names (default: all available).
        Returns per-model status and findings as JSON lines. Findings are untrusted: verify each one.
        """
        try:
            return await review(project, files, task, kind, models, max_findings)
        except ValueError as exc:
            return f"ERROR: {exc}"

    return mcp


# ─── CLI ─────────────────────────────────────────────────────────────────────

async def check():
    """Print each provider's key status and whether its model id exists."""
    for n, c in load_providers().items():
        if not c.get("enabled", True):
            print(f"{n}: disabled")
            continue
        if not c["has_key"]:
            print(f"{n}: missing key {c['api_key_env']}")
            continue
        try:
            ids = await list_remote_models(c)
            ok = c["model"] in ids
            print(f"{n}: key OK, model {c['model']} {'found' if ok else 'NOT FOUND'} ({len(ids)} models listed)")
            if not ok:
                hint = [i for i in ids if c["model"].split("/")[-1].split("-")[0] in i][:15]
                print("   similar:", ", ".join(hint))
        except Exception as exc:  # noqa: BLE001
            print(f"{n}: error listing models: {exc}")


def dry_run(project, patterns):
    """Show what each provider would receive, without calling any API."""
    project_dir(project)
    paths, rejected = resolve_files(patterns)
    texts = {p: load_text(p) for p in paths}
    for p, t in texts.items():
        print(f"{p.relative_to(REPO).as_posix()}: {len(t):,} chars (~{int(len(t) / CHARS_PER_TOKEN):,} tokens)")
    for r in rejected:
        print("rejected:", r)
    for n, c in load_providers().items():
        budget = int(c.get("max_input_tokens", 100_000) * CHARS_PER_TOKEN)
        fitted = fit_budget({p: t for p, t in texts.items()
                             if not (c["adapter"] == "gemini" and c.get("pdf_native") and p.suffix == ".pdf")}, budget)
        print(f"{n}: {'truncated' if any('[TRUNCATED]' in t for t in fitted.values()) else 'fits'} "
              f"(budget ~{c.get('max_input_tokens'):,} tokens)")


if __name__ == "__main__":
    if sys.argv[1:2] == ["--check"]:
        asyncio.run(check())
    elif sys.argv[1:2] == ["--dry-run"] and len(sys.argv) >= 4:
        dry_run(sys.argv[2], sys.argv[3:])
    else:
        make_server().run()
