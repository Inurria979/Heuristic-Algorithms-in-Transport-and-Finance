# NEH, RandNEH and ILS-ESP for the Permutation Flow-Shop Problem (Taillard)

Lab activity on the Permutation Flow-Shop Problem (PFSP) with makespan objective: `n` jobs
visit `m` machines in the same order, and the task is to choose the single job permutation
that minimises the completion time of the last job. Following Juan et al. (2014), we
implement the **NEH** heuristic, its biased-randomised version **RandNEH** and the complete
**ILS-ESP** metaheuristic, and add our own variant with an **adaptive β** (`Geo-adapt`).

## Files

| Path | Contents |
|---|---|
| [`src/PFSP_RandNEH_ILS_ESP_en.ipynb`](src/PFSP_RandNEH_ILS_ESP_en.ipynb) | **Main deliverable**: instance reader → makespan and Taillard's acceleration (Numba) → NEH / RandNEH → experiment A → ILS-ESP → experiment B → tests and plots |
| [`src/PFSP_RandNEH_ILS_ESP.ipynb`](src/PFSP_RandNEH_ILS_ESP.ipynb) | Same notebook in Spanish |
| `data/` | Taillard instances `tai*_inputs.txt` read by the notebook — **not in the repository yet** |
| [`docs/`](docs/) | 3-page article (`.tex` + PDF), its outline, class presentation (`.pptx` + PDF) |
| [`results/`](results/) | CSVs written by the notebook; `v1_una_construccion/` and `v2_antes_geoadapt/` keep the CSVs (and notebook) of two earlier versions |
| `ignore/` (git-ignored) | Article review reports and scratch files |

## Usage

Requires [uv](https://docs.astral.sh/uv/) only. This project pins **Python 3.13**, the
version the saved results were produced with.

```bash
uv sync    # numpy, scipy, pandas, matplotlib, numba (+ ipykernel)
```

The notebook reads the instances from `../data/` (so it does not run until the Taillard
files are added). By default it **reads** the experiment results saved in `results/`; set
`PFSP_RERUN=1` to re-run the experiments, which takes much longer. Headless:

```bash
uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/PFSP_RandNEH_ILS_ESP_en.ipynb
```

## Method in short

* **Instances.** Five Taillard instances: `tai001` (20×5), `tai051` (50×20), `tai081`
  (100×20), `tai101` (200×20) and `tai117` (500×20). Gap = `100 (makespan − BKS) / BKS`
  with BKS 1278, 3846, 6171, 11158 and 26389.
* **Experiment A — constructive phase only.** Deterministic NEH against RandNEH with the
  discretised triangular distribution, truncated geometric distributions with fixed β
  (0.05–0.7), a uniform control and `Geo-adapt`. As in Fig. 6 of Juan et al. (2014), each
  RandNEH execution repeats up to `n` biased constructions and returns the first one that
  improves on NEH, so it is never worse than NEH. 30 executions per instance and
  distribution.
* **Geo-adapt.** β starts at 0.05 and grows by 0.05 after every 3 failed attempts, up to 0.7.
* **Experiment B — complete ILS-ESP** (RandNEH + insertion local search + enhanced swap
  perturbation + Demon acceptance), changing only the initialisation. 10 independent runs
  per instance and initialisation, with a budget of 2 ms·n·m (the paper uses 10 ms·n·m),
  2 processes in parallel, base seed 2024. Each initialisation is compared with NEH by a
  two-sided Mann–Whitney test.

## Results (saved notebook run)

Mean gap (%) to the BKS ([`results/resumen_global.csv`](results/resumen_global.csv)):

| Instance | NEH | RandNEH Tri | RandNEH Geo-adapt | ILS-ESP from NEH | ILS-ESP from Tri | ILS-ESP from Geo-adapt | ILS-ESP from Tri, best |
|---|---|---|---|---|---|---|---|
| tai001_20_5 | 0.63 | 0.37 | 0.36 | 0.00 | 0.00 | 0.00 | 0.00 |
| tai051_50_20 | 6.14 | 5.61 | 5.58 | 1.30 | 1.22 | 1.29 | 0.65 |
| tai081_100_20 | 6.00 | 5.75 | 5.68 | 2.40 | 2.15 | 2.11 | 1.72 |
| tai101_200_20 | 3.91 | 3.77 | 3.73 | 1.76 | 1.72 | 1.75 | 1.42 |
| tai117_500_20 | 1.55 | 1.45 | 1.41 | 0.54 | 0.57 | 0.62 | 0.45 |

* On average RandNEH improves on NEH on every instance. `Geo-adapt` beats the triangular
  distribution on all five, but it is not the best one: on every instance some fixed β does
  better (0.7 on `tai001` and `tai051`, 0.1 on `tai081` and `tai101`, 0.5 on `tai117`),
  and no single fixed β wins everywhere
  ([`results/expA_randneh_resumen.csv`](results/expA_randneh_resumen.csv)).
* ILS-ESP from NEH divides the NEH gap by 2.2–4.7, and every initialisation reaches the
  BKS of `tai001` in all runs.
* Against ILS-ESP from NEH, a randomised start is only significantly better (p < 0.05) on
  `tai081`, from `Tri` and from `Geo-adapt`
  ([`results/expB_tests_vs_NEH.csv`](results/expB_tests_vs_NEH.csv)); with the reduced
  budget, the initialisation matters little once the metaheuristic runs.
