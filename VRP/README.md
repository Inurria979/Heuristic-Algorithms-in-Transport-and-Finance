# Capacitated Vehicle Routing Problem

Lab activity on the CVRP: a fleet of identical vehicles of capacity `Q` leaves a single
depot, every customer `i` must be served exactly once with its demand `q_i`, and the total
distance travelled is minimised.

```
min  Σ_routes Σ_{(i,j)∈route} d_ij
s.t. every customer on exactly one route,
     Σ_{i∈route} q_i ≤ Q  for every route,
     every route starts and ends at the depot
```

> **State: in progress.** The benchmark instances are in place and the environment is set
> up; [`src/vrp.py`](src/vrp.py) and [`src/vrp.ipynb`](src/vrp.ipynb) are still only the
> scaffolding (data path and imports), no heuristic is implemented yet.

## Files

| Path | Contents |
|---|---|
| [`data/`](data/) | 33 benchmark instances, one text file per instance |
| [`src/vrp.ipynb`](src/vrp.ipynb) | Notebook — intended main deliverable |
| [`src/vrp.py`](src/vrp.py) | Reusable functions imported by the notebook |
| [`docs/`](docs/) | Course slides and our own write-up (empty for now) |
| [`results/`](results/) | Generated CSVs and figures (empty for now) |
| `ignore/` (git-ignored) | Local material that is not uploaded |

## Instances

33 classic instances from four families, named `<family>-n<nodes>-k<vehicles>`:

| Family | Instances | Source |
|---|---|---|
| `A` | `A-n32-k5` … `A-n80-k10` (8) | Augerat et al. — random coordinates and demands |
| `B` | `B-n50-k7` … `B-n78-k10` (4) | Augerat et al. — clustered coordinates |
| `E` | `E-n22-k4` … `E-n76-k14` (7) | Christofides & Eilon |
| `F` | `F-n45-k4`, `F-n72-k4`, `F-n135-k7` (3) | Fisher — real-world instances |
| `M` | `M-n101-k10`, `M-n121-k7` (2) | Christofides et al. — large instances |
| `P` | `P-n22-k8` … `P-n101-k4` (9) | Augerat et al. — modified from A/B/E |

Sizes run from 22 to 135 nodes. `n` counts the depot; `k` is the number of vehicles used by
the best-known solution.

### File format

`<name>_input_nodes.txt` holds `n` tab-separated rows, one per node:

```
x   y   demand
```

* The **first row is the depot**, always `0  0  0`: the coordinates of each instance have
  been **translated so that the depot sits at the origin**, which is why many coordinates
  are negative. Distances are Euclidean, so the translation does not change any solution.
* The remaining rows are the customers with their demand.
* **The vehicle capacity `Q` is not in these files.** Take it from the original benchmark
  (CVRPLIB) together with the best-known cost used as the reference for the gap.

## Usage

Requires [uv](https://docs.astral.sh/uv/) only; it fetches Python 3.14 itself.

```bash
uv sync                  # numpy, pandas, matplotlib (+ ipykernel)
uv run python src/vrp.py
```

Open `src/vrp.ipynb` and select this folder's `.venv` as the kernel.

## References

* Augerat, P. et al. (1995). *Computational results with a branch and cut code for the
  capacitated vehicle routing problem*. Research report RR 949-M, Université Joseph
  Fourier, Grenoble.
* Christofides, N. & Eilon, S. (1969). An algorithm for the vehicle dispatching problem.
  *Operational Research Quarterly* 20(3), 309–318.
* Clarke, G. & Wright, J. W. (1964). Scheduling of vehicles from a central depot to a
  number of delivery points. *Operations Research* 12(4), 568–581.
* CVRPLIB (instances and best-known solutions):
  <http://vrp.galgos.inf.puc-rio.br/index.php/en/>
