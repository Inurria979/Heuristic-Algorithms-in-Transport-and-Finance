"""F9: 141 feasible p1-p3 instances x c x alpha = 1269 simheuristic runs.
Writes results/runs.csv and results/best.csv. Run from simTOP/: `uv run python src/run_experiment.py`.

runs.csv columns: instance, set, m, tmax, c, alpha, seed (entropy list of the SeedSequence),
det_reward (deterministic reward of the winner), fast_mean, exp_reward (intensive mean),
exp_std, fail_rate (PERCENT of routes, 0-100, that overrun tmax in the intensive replicas),
n_distinct, winner_is_greedy, bks, gap_pct = 100 (bks - exp_reward) / bks, time_s, routes,
det_gap_pct = 100 (bks - det_reward) / bks, uncert_loss_pct = 100 (1 - exp_reward / det_reward)
(empty when det_reward = 0).
best.csv: per (instance, c) the alpha with the highest exp_reward; ties go to the smallest alpha.
"""
import sys
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path
from time import perf_counter

import pandas as pd

import simtop as st

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / "results"


def warm_up():
    """Load/compile the Numba kernels in this process so that time_s excludes the JIT."""
    st.simheuristic(st.load_instance("p2.2.a"), st.C_LEVELS[0], st.ALPHAS[0], 0)


def run_case(args):
    """One (instance, c, alpha) run; returns a CSV row."""
    name, idx_inst, ic, ia = args
    inst = st.load_instance(name)
    c, alpha = st.C_LEVELS[ic], st.ALPHAS[ia]
    r = st.simheuristic(inst, c, alpha, st.case_seed(idx_inst, ic, ia))
    return {"instance": name, "set": name.split(".")[0], "m": inst.m, "tmax": inst.tmax, "c": c,
            "alpha": alpha, "seed": r["seed"], "det_reward": r["det_reward"], "fast_mean": r["fast_mean"],
            "exp_reward": r["exp_reward"], "exp_std": r["exp_std"], "fail_rate": r["fail_rate"],
            "n_distinct": r["n_distinct"], "winner_is_greedy": r["winner_is_greedy"],
            "time_s": r["time_s"], "routes": st.format_routes(r["routes"])}


def main(limit=None, out=RESULTS):
    """Run the experiment (optionally only the first `limit` instances) and write the CSVs."""
    bks = st.load_bks()
    names = st.instance_names()                    # idx_inst indexes all 387 files, not the 141
    feasible = st.feasible_instances()
    if limit is not None:
        feasible = feasible[:limit]
    jobs = [(n, names.index(n), ic, ia) for n in feasible
            for ic in range(len(st.C_LEVELS)) for ia in range(len(st.ALPHAS))]
    start = perf_counter()
    with ProcessPoolExecutor(initializer=warm_up) as pool:
        rows = list(pool.map(run_case, jobs, chunksize=8))
    df = pd.DataFrame(rows)
    df["bks"] = df.instance.map(bks)
    df["gap_pct"] = 100 * (df.bks - df.exp_reward) / df.bks
    df["det_gap_pct"] = 100 * (df.bks - df.det_reward) / df.bks
    df["uncert_loss_pct"] = (100 * (1 - df.exp_reward / df.det_reward)).where(df.det_reward > 0)
    cols = ["instance", "set", "m", "tmax", "c", "alpha", "seed", "det_reward", "fast_mean", "exp_reward",
            "exp_std", "fail_rate", "n_distinct", "winner_is_greedy", "bks", "gap_pct", "det_gap_pct",
            "uncert_loss_pct", "time_s", "routes"]
    df = df[cols].sort_values(["instance", "c", "alpha"]).reset_index(drop=True)
    # best alpha per (instance, c): highest exp_reward, ties -> smallest alpha (rows are sorted by alpha)
    best = df.loc[df.groupby(["instance", "c"], sort=True)["exp_reward"].idxmax()].reset_index(drop=True)
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    df.to_csv(out / "runs.csv", index=False)
    best.to_csv(out / "best.csv", index=False)
    print(f"{len(df)} runs, {len(best)} best rows, total {perf_counter() - start:.1f} s")


if __name__ == "__main__":
    main(int(sys.argv[1]) if len(sys.argv) > 1 else None)
