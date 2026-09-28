# Greedy constructive heuristics for the TSP (TSPLIB)

Lab activity on the Travelling Salesman Problem: implement two **purely constructive
heuristics**, with no local-improvement phase, run them over all the TSPLIB instances in
[`data/`](data/) and measure their quality against the known optimal tours.

```
min  Σ_i Σ_{j≠i} d_ij x_ij
s.t. Σ_{j≠i} x_ij = 1,  Σ_{i≠j} x_ij = 1,
     Σ_{i∈S} Σ_{j∈S} x_ij ≤ |S| − 1   for every S ⊂ V, 2 ≤ |S| ≤ n−1
```

* **Nearest neighbour** — start at a city and always jump to the closest unvisited one.
* **Greedy edge** (*multi-fragment*) — sort the edges by length and add each one whenever
  both endpoints still have degree < 2 and it does not close a premature cycle; then join
  the remaining path fragments and close the cycle.

## Files

| Path | Contents |
|---|---|
| [`src/tsp_greedy_en.ipynb`](src/tsp_greedy_en.ipynb) | **Main deliverable**: TSPLIB parser → candidate neighbours → both heuristics → run over the 111 instances → gaps, times and plots |
| [`data/`](data/) | 111 TSPLIB instances (`*.tsp`) and the 32 known optimal tours (`*.opt.tour`) |
| [`docs/`](docs/) | Course slides, our write-up `TSP_heuristic.pdf` and presentation `tsp-greedy-heuristics.pptx` |
| [`results/resultados.csv`](results/resultados.csv) | One row per instance: `n`, `tipo`, `optimo`, and length / gap / time for each heuristic |
| [`results/figures/`](results/figures/) | `results.{png,pdf}` (gaps and times over the 111 instances) and `eil51_tours.{png,pdf}` (the two tours of `eil51`) |
| `ignore/` (git-ignored) | Local material that is not uploaded |

## Usage

Requires [uv](https://docs.astral.sh/uv/) only; it fetches Python 3.14 itself.

```bash
uv sync    # numpy, scipy, pandas, matplotlib (+ ipykernel)
```

Open `src/tsp_greedy_en.ipynb` and select this folder's `.venv` as the kernel, then run all
cells (the 111 instances take about 75 s in total). Headless:

```bash
uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/tsp_greedy_en.ipynb
```

## Method in short

* **Parsing.** Distances are always **integers** computed with the exact TSPLIB formulas —
  `EUC_2D` (78 instances), `EXPLICIT` (17), `GEO` (10), `CEIL_2D` (4) and `ATT` (2) — using
  the value `PI = 3.141592` that TSPLIB fixes for `GEO`. Other formulas, or plain
  `math.pi`, do not reproduce the published optima. `linhp318` carries a
  `FIXED_EDGES_SECTION` that the tour must contain.
* **Candidate neighbours.** A `scipy.spatial.cKDTree` gives the `K_NEIGHBORS = 10` nearest
  neighbours of every node; both heuristics work on that sparse candidate graph. Greedy
  edge ignores the candidates and uses **all** the edges when `n ≤ EXACT_MAX = 2500`.
* **Validation.** Every tour is checked with `is_valid_tour` (a complete permutation that
  also respects the fixed edges) before its length is recorded.
* **Timing.** The reported time of each method excludes the candidate computation
  (column `t_candidatos`), except for exact greedy edge, which builds its own neighbours.
* **Gap.** `100 (L − L_opt) / L_opt`, computable on the 32 instances that ship an
  `.opt.tour`.

## Results (last notebook run)

Over the 32 instances with a known optimum:

| Heuristic | Mean gap | Mean gap without `brg180` | Median gap | Best | Total time, 111 instances |
|---|---|---|---|---|---|
| Nearest neighbour | 42.4 % | 26.5 % | 25.0 % | 11.8 % | 2.6 s |
| Greedy edge | 75.6 % | 18.7 % | 19.0 % | 5.4 % | 28.3 s |

Greedy edge is the better heuristic on the median and on most instances — it loses to
nearest neighbour on 18 of the 111 — but it costs roughly ten times more time, and its mean
gap is dragged up by a single pathological case. Instance sizes span `burma14` (14 nodes)
to `pla85900` (85 900 nodes).

**`brg180` is pathological.** It has only six distinct weights (0, 20, 30, 3500, 9000,
10000) and thousands of ties at weight 30. Greedy consumes the cheapest edges first (the
90 of weight 0 and the 75 of weight 20), corners itself and ends up paying for 4 edges of
weight 9000 where the optimum uses 15 of weight 30: 37 830 against 1950, a gap of 1840 %.
This is a property of the instance, not a bug, which is why the averages are reported both
with and without it.

**Limitations.**

* *No local improvement.* Gaps around 18–19 % are what a purely constructive heuristic
  gives. The weak spots are the final joining of fragments and instances with many ties;
  2-opt / Or-opt over neighbour lists, or joining the fragments with Savings, are the
  natural next steps.
* *Candidates versus exact greedy.* With 10 neighbours the tour changed in 42 of the 81
  small instances compared during development, up to ±10 % in individual cases but −0.1 %
  on average: no systematic loss of quality, just a noisier result. The effect on large
  instances with very dense regions has not been measured.
* *Scope.* Only 32 of the 111 instances ship an optimal tour; on the rest the two methods
  can only be compared against each other. Using the best-known values published by
  TSPLIB would extend the gap to all of them.

## References

* Reinelt, G. (1991). TSPLIB — a traveling salesman problem library.
  *ORSA Journal on Computing* 3(4), 376–384.
* Jünger, M., Reinelt, G. & Rinaldi, G. (1995). The traveling salesman problem. In
  *Handbooks in Operations Research and Management Science* 7, 225–330.
* TSPLIB instances and optimal tours:
  <http://comopt.ifi.uni-heidelberg.de/software/TSPLIB95/>
