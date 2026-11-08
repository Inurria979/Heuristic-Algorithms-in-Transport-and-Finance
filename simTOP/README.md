# simTOP: simheuristic for the Team Orienteering Problem with stochastic travel times

Implementation of the **simheuristic** of Juan, Reyes-Rubiano, de la Torre, Panadero, Faulin and
Latorre (2019), *Simulation-based optimization in transportation and logistics: comparing sample
average approximation with simheuristics* (Winter Simulation Conference, pp. 1906–1917), on the
Team Orienteering Problem (TOP) with random travel times. The SAA with CPLEX of the paper is **not**
implemented (commercial solver).

## Problem and instances

* **TOP.** `m` identical vehicles leave a source depot and end at a sink depot; each customer pays
  its reward the first time it is visited; every route has a time limit `tmax`. Maximise the
  **expected** collected reward.
* **Stochastic times.** The travel time of every arc is `T_ij ~ LogNormal` with `E[T_ij] = t_ij`
  (Euclidean distance) and `Var[T_ij] = c · t_ij`, symmetric (`T_ij = T_ji`), independent between arcs
  and replicas. Three levels: `c = 0.05` (L), `0.25` (M), `0.75` (H). A route whose simulated time
  exceeds `tmax` **loses all its reward**.
* **Instances.** The 141 feasible instances of sets p1, p2 and p3 (fewer than 35 nodes, `tmax` from
  3.8 to 55) of Chao et al. (1996) in [`data/top-data/`](data/top-data/), unmodified (the six p1
  instances without any reachable customer are left out). Scenarios are drawn on the fly with a fixed
  seed, never stored. [`data/bks_chao.csv`](data/bks_chao.csv) is the deterministic best known solution
  (copied from `../TOP/data/`), used for the gap.

## Method

One run of the simheuristic for an (instance, `c`, `α`) case:

1. **Candidates.** The PJS savings heuristic (`s'_ij = α s_ij + (1 − α)(u_i + u_j)`, both directions of
   every arc, merging while the expected time fits `tmax`), taken greedily once and with
   **geometric biased randomisation** (β = 0.3) 50 times. Every solution is improved with a local
   search (2-opt with first improvement + insertion of the unvisited customer of highest reward,
   repeated). PJS, biased randomisation and local search are the ones of [`../TOP/`](../TOP/).
2. **Deduplication** of identical solutions (same routes, in any order).
3. **Deterministic filter:** the 7 distinct solutions with the highest deterministic reward.
4. **Fast simulation:** 200 replicas with **common random numbers** (one set of 200 days for the 7
   candidates), which ranks them fairly. The 3 best means go on.
5. **Intensive simulation:** 1000 **fresh** replicas (not the 200 of step 4, to avoid the selection
   bias of the winner's curse) shared by the 3 finalists. The winner has the highest intensive mean.

The heavy loops (PJS merging, local search, simulation over replicas) are Numba kernels; the random
numbers are drawn with numpy, so results do not depend on Numba's generator.

| Parameter | Value | Origin |
|---|---|---|
| `c` | 0.05 / 0.25 / 0.75 | paper (p. 1913) |
| Replicas, fast / intensive | 200 / 1000 | paper (p. 1912) |
| `α` | 0.25, 0.5, 0.75 (all three run) | our choice (the paper gives none) |
| Biased randomisation | geometric, β = 0.3 | our choice (as in `TOP/`; the paper cites Grasas et al. 2017) |
| Iterations | 50 biased constructions + 1 greedy per run | our choice (the paper stops by time) |
| Filter 7 → 3 | 7 by deterministic reward, 3 by fast mean | our choice |
| Local search | 2-opt + insertion of `TOP/` | our choice (the paper: destruction–reconstruction + 2-opt) |
| Random numbers | common in the fast simulation, fresh in the intensive one | our choice |
| Arcs | independent between arcs and replicas | our assumption (the paper does not say) |

**Seeds.** One seed per case: `SeedSequence([2019, idx_inst, idx_c, idx_alpha])`, split into three
independent generators (biased randomisation, fast simulation, intensive simulation). `idx_inst` is the
position of the instance in the alphabetical list of the 387 Chao file names (p1–p3 come first, so it
is the same in the list of p1–p3 alone). Results do not depend on the execution order. The `seed`
column of `runs.csv` holds that list.

## Files

| Path | Contents |
|---|---|
| [`src/simtop.py`](src/simtop.py) | Instance loading, lognormal law, PJS + biased randomisation, local search, scenarios, simulation and the simheuristic (Numba) |
| [`src/checks.py`](src/checks.py) | 18 sanity checks (see below) |
| [`src/run_experiment.py`](src/run_experiment.py) | The 1 269 runs; writes `results/runs.csv` and `results/best.csv` |
| [`src/analysis.ipynb`](src/analysis.ipynb) | Tables and figures of the results (reads `../results/`) |
| [`data/`](data/) | All 387 Chao instances (only p1–p3 are used) and `bks_chao.csv` |
| [`docs/`](docs/) | Paper, specification (`explicacion_paper.md`) and open questions |
| [`results/`](results/) | `runs.csv`, `best.csv`, summary CSVs, `figures/` |
| `ignore/` (git-ignored) | Review reports and scratch files |

## Usage

Requires [uv](https://docs.astral.sh/uv/) only (Python 3.14, numpy, numba, pandas, matplotlib).

```bash
uv sync
uv run python src/checks.py          # ~25 s the first time (Numba compilation), a few seconds afterwards; prints OK/FAIL per check
uv run python src/run_experiment.py  # 1 269 runs, ~25 s on 12 threads of a 6-core AMD Ryzen 5 5500U
uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/analysis.ipynb
```

`checks.py` verifies, among others: the lognormal moments, that the greedy PJS gives exactly the reward
of `TOP/results/alpha_grid.csv` for α = 0.25, 0.5 and 0.75 on the 141 instances, that 2-opt undoes a
crossing, that the local search never lowers the reward or exceeds `tmax`, that with `c → 0` the
simulated mean equals the deterministic reward, that a route with expected time `tmax` fails about 47 % of
the time, common random numbers, and reproducibility. The notebook is the source of the analysis and is
edited directly.

## Results

Gap = `100 (BKS − R̄) / BKS`, with `R̄` the intensive mean of the winner (`exp_reward`), over the 141
instances ([`results/runs.csv`](results/runs.csv)).

**α × c matrix** (mean / median gap %, 141 instances per cell,
[`results/gap_alpha_c_matrix.csv`](results/gap_alpha_c_matrix.csv)):

| α | c = 0.05 (L) | c = 0.25 (M) | c = 0.75 (H) |
|---|---|---|---|
| 0.25 | 16.69 / 16.90 | 26.92 / 27.52 | 31.78 / 33.02 |
| 0.50 | 16.86 / 16.94 | 26.83 / 28.56 | 31.97 / 32.42 |
| 0.75 | 16.29 / 17.07 | 26.90 / 28.77 | 31.94 / 33.45 |

α hardly matters: the best α of each (instance, `c`) ([`results/best.csv`](results/best.csv)) is 0.25,
0.5 and 0.75 in 142, 138 and 143 of the 423 cases. The gap grows with the level: with the best α, the
mean is 14.65 < 25.02 < 30.51 (L < M < H), as expected.

**Best α against the paper** (Fig. 3, SimHeu-L/M/H medians and boxes **read by eye**, about ±1–2
points; [`results/gap_best_by_level.csv`](results/gap_best_by_level.csv)):

| Level | Ours: median (Q1–Q3) | Paper: median (box) |
|---|---|---|
| L | 15.69 (9.90–20.23) | ≈ 7 (5–10) |
| M | 26.02 (20.04–31.97) | ≈ 17 (9–20) |
| H | 31.56 (26.35–36.44) | ≈ 24 (19–28) |

Our medians are 7–9 points above the paper's, with the same ordering L < M < H. The largest gap of the
best α is 42.7 % (p1.2.k, `c = 0.75`); none exceeds 50 %.

**Decomposition of the gap** (best α, mean %, [`results/gap_decomposition_by_level.csv`](results/gap_decomposition_by_level.csv)):

| Level | Total gap | Deterministic gap of the winner | Loss due to uncertainty, over the BKS | Same loss, over the deterministic reward | Routes that fail (%) |
|---|---|---|---|---|---|
| L | 14.65 | 3.15 (median 0) | 11.50 | 11.86 | 12.73 |
| M | 25.02 | 2.96 (median 0) | 22.06 | 22.72 | 23.57 |
| H | 30.51 | 2.41 (median 0) | 28.10 | 28.79 | 29.45 |

The split is additive over the BKS: gap = deterministic gap + loss over the BKS, with the loss over the BKS
`100 (det − R̄) / BKS` (per run, then averaged); `uncert_loss_pct` in `runs.csv` is the same loss over the
deterministic reward, `100 (1 − R̄ / det)`.

Most of the gap is the price of the uncertainty (about 80 % at L and 90 % at M and H). A likely
reason is that construction and local search build routes close to `tmax` (median expected time of the
winners' routes: 0.93 `tmax` at every level; 4 / 6 / 8 % of them within 1 % of `tmax`,
[`results/route_slack_by_level.csv`](results/route_slack_by_level.csv)) and the deterministic filter keeps
the highest rewards, which tend to be the tightest. With that little slack a sizeable share of routes fails
(13 / 24 / 29 % for L / M / H), whereas a route whose expected time equals `tmax` would fail a little under
half of the time. The winner is the greedy candidate in 13.2 % of the best-α rows.

**By set and `tmax`** (mean gap of the best α, L / M / H,
[`results/gap_by_set_level.csv`](results/gap_by_set_level.csv)): p1 18.38 / 29.37 / 33.87, p2 10.36 /
19.33 / 25.19, p3 14.02 / 24.68 / 30.75. With `Var = c · t` a short route is relatively more variable, yet
the gap tends to grow with `tmax` (Spearman correlation 0.49 / 0.56 / 0.67 for L / M / H) and p2, with the
smallest `tmax` (median 10.0 against 17.5 in p1 and 20.6 in p3), has the lowest gap. Part of the trend is
already in the deterministic gap, which also grows with `tmax` (Spearman 0.56 / 0.54 / 0.59), and so does
the loss due to uncertainty over the deterministic reward (0.33 / 0.44 / 0.52), so it is not only the uncertainty.
Set and `tmax` are confounded (p2 also has 21 nodes).

**Time per run** (mean, [`results/time_by_set_level.csv`](results/time_by_set_level.csv)): 0.16 s in p1,
0.07 s in p2 and 0.16–0.17 s in p3, at most 0.23 s per run (0.053–0.234 s over the 1 269 runs, after
loading the Numba kernels); the paper's Fig. 4 reports a few seconds up to 400 s on a Xeon in Java with a time
stop, so only a qualitative comparison makes sense.

## Limitations and future work

* The SAA with CPLEX is not implemented, so there is no comparison against it, only against the
  SimHeu boxes read by eye from Fig. 3; the paper does not say which instances each box covers nor how the
  gap is computed.
* The gap against the deterministic BKS mixes the loss of the heuristic and the price of the uncertainty, which
  no solution can avoid; the decomposition above separates them.
* Routes built close to `tmax` (see above). **Future work:** a safety margin, building and improving with
  `t'_max = t_max − δ` (fixed grid, `E_r + z √(c E_r) ≤ t_max` from the variance, or adaptive from the failure
  rate of the fast simulation) while the simulation keeps the true `tmax`.
* Choosing the best of three α with the same mean that is reported adds a small selection bias.
