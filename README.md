# Heuristic Algorithms in Transport and Finance

Lab projects for the master's course *Heuristic Algorithms in Transport and Finance*.
There is **one folder per project**, and each folder is a self-contained
[uv](https://docs.astral.sh/uv/) project: its own `pyproject.toml`, `uv.lock`,
pinned Python version and its own `.venv`.

## Projects

| Folder | Problem | State | Main deliverable |
|---|---|---|---|
| [`POP/`](POP/) | **Portfolio Optimization Problem.** Basic Markowitz mean–variance model solved as a quadratic program with [OSQP](https://osqp.org); unconstrained efficient frontier of the OR-Library instances `port1`–`port5` compared point by point with the reference frontiers `portef1`–`portef5`. | Finished | [`src/markowitz_osqp.ipynb`](POP/src/markowitz_osqp.ipynb) + 3-page tutorial article in `docs/` (`.tex` and PDF) |
| [`TSP/`](TSP/) | **Travelling Salesman Problem.** Two constructive heuristics without local search — nearest neighbour and greedy edge (*multi-fragment*) — run over the 111 TSPLIB instances and measured against the 32 known optimal tours. | Code finished | [`src/tsp_greedy_en.ipynb`](TSP/src/tsp_greedy_en.ipynb) |
| [`VRP/`](VRP/) | **Capacitated Vehicle Routing Problem** with the Clarke & Wright savings heuristic on 33 classic benchmark instances (Augerat A/B/P, Christofides E/M, Fisher F), compared with Table 1 of Juan et al. (2011). | In progress — sequential and parallel CWS done, article pending | [`src/vrp.ipynb`](VRP/src/vrp.ipynb) + 3-page article in `docs/` (to do) |
| [`DTOP/`](DTOP/) | **Dynamic Team Orienteering Problem** approached with Transformers + reinforcement learning. No code: the activity was a presentation in class. | Finished | Class presentation, reference papers |

Each folder has its own `README.md` with the model, the method and the results.

## Layout

Every code project (`POP/`, `TSP/`, `VRP/`) follows the same structure:

| Folder | Contents | In git |
|---|---|---|
| `src/` | code: notebooks and `.py` modules | yes |
| `data/` | problem instances (benchmarks, reference solutions) | yes |
| `docs/` | PDFs from the lecturer (slides, handouts, reference papers) and our own documents (articles, outlines, notes) | yes, PDFs included |
| `results/` | generated outputs: CSVs and figures | yes |
| `ignore/` | anything that must not be uploaded: original downloads, zips, archived scripts | no |

`DTOP/` is the exception: it contains no code — the activity was only a presentation in class —
so it just holds the papers and slides, with no subfolders.

```
Heuristic-Algorithms-in-Transport-and-Finance/
├── .gitignore          one ignore file for the whole repository
├── .gitattributes      LF normalisation inside the repo, binaries marked as such
├── CLAUDE.md           instructions for the coding agent
├── README.md           this file
│
├── POP/
│   ├── .python-version     3.14
│   ├── pyproject.toml      dependencies (numpy, scipy, osqp, matplotlib, pandas)
│   ├── uv.lock             exact resolution — committed
│   ├── .venv/              git-ignored, rebuilt with `uv sync`
│   ├── CLAUDE.md           project-specific notes for the coding agent
│   ├── src/                markowitz_osqp.ipynb
│   ├── data/               port1..5.txt, portef1..5.txt
│   ├── docs/               slides, reference papers, the article (.tex + PDF), outlines
│   ├── results/            frontier CSVs, summary, figures
│   └── ignore/             original data download, archived scripts
│
├── TSP/
│   ├── .python-version, pyproject.toml, uv.lock, .venv/
│   ├── src/                tsp_greedy_en.ipynb
│   ├── data/               111 TSPLIB instances + 32 .opt.tour files
│   ├── docs/               course slides
│   ├── results/            resultados.csv, figures/
│   └── ignore/
│
├── VRP/
│   ├── .python-version, pyproject.toml, uv.lock, .venv/
│   ├── src/                vrp.py, vrp.ipynb
│   ├── data/               33 instances, one file per instance
│   ├── docs/               course slides, Juan et al. (2011) paper
│   ├── results/            sequential_cws.csv, cws_comparison.csv, figures/
│   └── ignore/
│
└── DTOP/
    ├── .python-version, pyproject.toml, uv.lock, .venv/
    └── *.pdf                reference papers and the class presentation (no code)
```

Notebooks run with `src/` as their working directory, so they read `../data/` and write
to `../results/`.

## Environments

All projects pin **Python 3.14** and are managed with **uv**. They are deliberately
*independent*: there is no `pyproject.toml` at the repository root and **no uv workspace**,
so every folder resolves and locks its own dependencies and their `.venv` can never
interfere with one another.

The only prerequisite is [uv](https://docs.astral.sh/uv/getting-started/installation/) —
it downloads the interpreter itself, so there is no need for a system Python, conda or
`pip install`.

```bash
cd TSP        # always work from inside the project folder
uv sync       # creates ./.venv exactly as uv.lock says
```

Everyday commands, all of them run from inside a project folder:

| Command | Effect |
|---|---|
| `uv sync` | create or update `.venv` to match `uv.lock` |
| `uv run python main.py` | run inside that project's env, no activation needed |
| `uv run pytest` | same, for the test suite |
| `uv add scipy` | add a dependency (updates `pyproject.toml` and `uv.lock`) |
| `uv add --dev pytest` | add a development-only dependency |
| `uv lock --upgrade` | re-resolve to the newest compatible versions |
| `uv tree` | show the resolved dependency tree |

**Notebooks.** Open the project folder in VS Code or Jupyter and select the `.venv` of
*that* folder as the kernel; `ipykernel` is in the `dev` group of every project, which
`uv sync` installs by default. To run a notebook without a UI:

```bash
cd TSP
uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/tsp_greedy_en.ipynb
```

**Windows.** If uv fails while hard-linking packages into the venv (OneDrive-backed
folders, antivirus), set `UV_LINK_MODE=copy` for the command or the session.

### Adding a new project

```bash
mkdir DPP && cd DPP
mkdir src data docs results ignore
uv init --bare --python 3.14      # writes pyproject.toml, nothing else
echo 3.14 > .python-version       # --bare does not pin the interpreter, so pin it by hand
uv add numpy matplotlib           # creates .venv and uv.lock on the first add
uv add --dev ipykernel            # so the notebook kernel shows up
```

Then add a row to the table above and a short `README.md` in the new folder. The root
`.gitignore` already covers its `.venv/` and caches, so nothing else needs changing.

## What goes into git

**Committed:** everything in `src/`, `data/`, `docs/` (PDFs included) and `results/`,
plus `pyproject.toml`, `uv.lock`, `.python-version`, `README.md` and `CLAUDE.md`.

**Ignored** (see [`.gitignore`](.gitignore)): every `ignore/` folder, `.venv/`,
`__pycache__/`, tool caches and LaTeX auxiliary files.

Locks are committed on purpose: `uv sync` then rebuilds a byte-identical environment on
any machine, which is what makes the reported timings comparable.
