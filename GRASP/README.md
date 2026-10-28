# GRASP with savings and biased randomisation for the TSP (TSPLIB)

Lab activity on GRASP for the Travelling Salesman Problem: build tours with the
**Clarke & Wright savings** heuristic, randomise the choice of the next saving, improve
every tour with **2-opt** and keep the best one. The question is whether **biased
randomisation** (a skewed distribution over the whole sorted savings list) beats the
classical **restricted candidate list (RCL)**.

Five selection rules are compared, each tuned on instances different from the test ones:

Over the `m` feasible savings, sorted from largest to smallest (position `k = 0, 1, …`):

| Rule | How the next saving is picked | Tuned parameter |
|---|---|---|
| Standard GRASP (RCL) | uniformly among the savings with `s ≥ s_max − α (s_max − s_min)` | α = 0.05 |
| Biased geometric | `P(k) ∝ β (1 − β)^k`, truncated to the list | β = 0.4 |
| Biased triangular | linearly decreasing density over the first `L = max(3, ⌈f·m⌉)` positions | f = 0.002 |
| Biased Zipf | `P(k) ∝ 1 / (k + 1)^e` | e = 3 |
| Biased softmax | `P(k) ∝ exp(z_k / T)` with `z_k = (s_k − s_max) / (s_max − s_min)`: it looks at the value, not the position | T = 0.02 |

## Files

| Path | Contents |
|---|---|
| [`src/GRASP_TSP_savings_biased_EN.ipynb`](src/GRASP_TSP_savings_biased_EN.ipynb) | **Main deliverable**: TSPLIB parser → savings list → GRASP with the five rules + 2-opt (Numba) → tuning → experiment → statistical tests and plots |
| [`src/GRASP_TSP_savings_biased.ipynb`](src/GRASP_TSP_savings_biased.ipynb) | Same notebook in Spanish |
| [`data/`](data/) | The 10 TSPLIB instances used (7 test, 3 tuning) and their `.opt.tour` files, copied from [`../TSP/data/`](../TSP/data/) |
| [`docs/`](docs/) | 3-page article (`.tex` + PDF), its outline, class presentation (`.pptx` + PDF) and script |
| [`results/`](results/) | CSVs and figures extracted from the saved notebook outputs (the notebook does not write them itself) |
| `ignore/` (git-ignored) | Article review reports and scratch files |

## Usage

Requires [uv](https://docs.astral.sh/uv/) only. This project pins **Python 3.13**, the
version the saved results were produced with.

```bash
uv sync    # numpy, scipy, pandas, matplotlib, numba, jinja2 (+ ipykernel)
```

Open the notebook with this folder's `.venv` as the kernel (it reads `../data/`). The
tuning takes about 5 min and the experiment about 17 min. Headless:

```bash
uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/GRASP_TSP_savings_biased_EN.ipynb
```

## Method in short

* **Instances.** Test: `eil51`, `berlin52`, `st70`, `eil76`, `kroA100`, `ch130`, `a280`.
  Tuning: `kroB100`, `eil101`, `ch150`. Gap = `100 (L − L_opt) / L_opt`.
* **Construction.** Savings `s(i,j) = d(h,i) + d(h,j) − d(i,j)` with respect to a hub `h`,
  the node closest to the centre of gravity; the list is sorted once per instance, and each
  step filters the feasible savings and draws one with the selected rule.
* **Protocol.** 150 GRASP iterations per run and 20 runs per rule and instance (seeds
  0–19); tuning with 80 iterations and 6 runs per grid value (seeds from 10 000).
* **Statistics.** Each biased rule is compared with the RCL per instance (Holm-corrected
  tests at 5 %) and globally with a Friedman test.

## Results (saved notebook run)

Mean gap (%) over 20 runs × 150 iterations ([`results/mean_gap.csv`](results/mean_gap.csv)):

| Instance | CW det. + 2-opt | RCL | Geometric | Triangular | Zipf | Softmax |
|---|---|---|---|---|---|---|
| eil51 | 4.46 | 1.41 | 1.10 | 0.95 | 1.20 | 1.41 |
| berlin52 | 6.58 | 3.69 | 0.93 | 2.04 | 3.35 | 3.44 |
| st70 | 3.56 | 0.82 | 1.01 | 0.99 | 1.26 | 0.93 |
| eil76 | 3.53 | 1.78 | 1.65 | 1.66 | 2.02 | 1.89 |
| kroA100 | 3.18 | 0.94 | 1.14 | 1.12 | 1.19 | 0.91 |
| ch130 | 4.70 | 1.71 | 1.73 | 1.80 | 1.54 | 1.46 |
| a280 | 9.46 | 5.16 | 4.31 | 6.14 | 4.74 | 5.70 |
| **Mean** | **5.07** | **2.22** | **1.70** | **2.10** | **2.18** | **2.25** |

* GRASP with the RCL more than halves the gap of the deterministic savings tour + 2-opt.
* The geometric rule has the lowest mean gap; against the RCL it is significantly better
  on 2 instances (`berlin52`, `a280`) and never worse
  ([`results/verdict_summary.csv`](results/verdict_summary.csv)). The triangular rule wins
  2 and loses 1; Zipf and softmax are never better. The Friedman test does not reject
  equality of the five rules, so the evidence is moderate.
* Time per run goes from 0.14 s to 10.18 s ([`results/mean_time.csv`](results/mean_time.csv)):
  geometric and triangular cost the same as the RCL or less, while Zipf and softmax, which
  weigh the whole list, cost up to 1.75 times more.
