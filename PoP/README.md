# Markowitz efficient frontier with OSQP (OR-Library port1–port5)

Code for the lab activity (see `image.png`): write a 3-page tutorial article on the basic
Markowitz mean-variance model (Beasley, 2013), solve it as a quadratic program with
[OSQP](https://osqp.org), trace the unconstrained efficient frontier (UEF) of the five
OR-Library benchmark instances and compare it with the reference frontiers `portef1`–`portef5`.

```
min  x' Σ x   s.t.  μ' x ≥ R_min,  Σ_i x_i = 1,  x ≥ 0
```

## Files

| Path | Contents |
|---|---|
| `markowitz_osqp.ipynb` | **Main deliverable**: load data → solve with OSQP → times table and frontier plots |
| `figures/frontiers.pdf/.png` | Figure written by the notebook for the article (created on execution) |
| `data/` | OR-Library instances `port1..5.txt` and reference frontiers `portef1..5.txt` |
| `docs/esquema_articulo.md` | Outline of the tutorial article and bibliography (Spanish) |
| `results/` | Outputs of the earlier extended script: per-instance frontier CSVs, `summary.csv/.md`, `environment.json`, `figures/` |
| `Beasley 2014 - ….pdf` | Reference paper |
| `image.png` | Assignment statement |
| `OR-Library port and portf-…/` | Original download of the data (same files as `data/` plus the OR-Library README) |
| `ignore/` (git-ignored) | Archived extended script `markowitz_osqp.py` and the data zip |

## Usage

Requires Python 3.14 and [uv](https://docs.astral.sh/uv/).

```bash
uv sync      # numpy, scipy, osqp, matplotlib, pandas (+ ipykernel, pytest)
```

Open `markowitz_osqp.ipynb` in VS Code/Jupyter with the `.venv` kernel and run all cells
(about 6 minutes, most of it port5). Headless:

```bash
UV_LINK_MODE=copy uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace markowitz_osqp.ipynb
```

## Method in short

* Σ_ij = ρ_ij σ_i σ_j is built from the standard deviations and correlations in `portK.txt`.
  All five covariance matrices are positive definite, so each QP has a unique optimum.
* OSQP form: P = 2Σ (upper triangle), q = 0, A = [μᵀ; 1ᵀ; I], l = [R_min; 1; 0], u = [∞; 1; ∞].
* The problem is set up once per instance. For each return level only `l[0]` changes
  (`update(l=...)`): the KKT factorisation is reused and each solve is warm-started.
* Settings: `eps_abs = eps_rel = 1e-6`, solution polishing, `max_iter = 20000`.
  The OSQP defaults (`eps = 1e-3`, no polishing, 4000 iterations) give variance errors of
  several percent and lose many points near the top of the frontier.
* The frontier is solved at the 2000 return levels of `portefK.txt`, so our variances are
  compared with the benchmark point by point.

## Results (last notebook run, AMD Ryzen 5 5500U, one thread)

| Instance | Index | N | Time for 2000 QPs (s) | Points not solved |
|---|---|---|---|---|
| port1 | Hang Seng | 31 | 0.9 | 0 |
| port2 | DAX 100 | 85 | 25.9 | 0 |
| port3 | FTSE 100 | 89 | 10.1 | 0 |
| port4 | S&P 100 | 98 | 33.9 | 3 |
| port5 | Nikkei 225 | 225 | 275.0 | 60 |

Times vary between runs (port5 has taken 183–275 s). On the points reported as `solved`
the variance matches `portef` to about 10⁻⁵ %, i.e. the rounding of the benchmark file.

**Limitation.** Near the maximum return the feasible set is a thin corner of the simplex and
ADMM converges very slowly, so 63 of the 10 000 points (port4/port5) end as
`solved inaccurate` / `maximum iterations reached`. They are counted in the table and left
out of the plots. Alternatives that solve them: much tighter tolerances with more
iterations (the archived extended script, ≈ 9 min), the weighted formulation
min x'Σx − t μ'x, or an active-set / interior-point solver.

## References

* Beasley, J. E. (2013). Portfolio optimisation: models and solution approaches. In
  *Theory Driven by Influential Applications* (pp. 201–221). INFORMS.
* OR-Library, portfolio data: <https://people.brunel.ac.uk/~mastjjb/jeb/orlib/portinfo.html>
* Chang, Meade, Beasley & Sharaiha (2000). Heuristics for cardinality constrained
  portfolio optimisation. *Computers & Operations Research* 27, 1271–1302.
* Stellato et al. (2020). OSQP: an operator splitting solver for quadratic programs.
  *Mathematical Programming Computation* 12, 637–672.
