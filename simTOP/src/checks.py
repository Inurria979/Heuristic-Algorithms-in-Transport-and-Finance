"""Sanity checks of simtop.py (section 10 of docs/explicacion_paper.md).
Run from simTOP/: `uv run python src/checks.py`. Prints OK/FAIL per check, exit code 1 if any fails."""
import itertools
import sys
from pathlib import Path

import numpy as np
import pandas as pd

import simtop as st

TOP_GRID = Path(__file__).resolve().parent.parent.parent / "TOP" / "results" / "alpha_grid.csv"
PROBE_NAMES = ("p1.2.b", "p1.3.h", "p2.2.d", "p2.4.k", "p3.2.f", "p3.4.m")


def make_instance(coords, rewards, m, tmax, name="hand"):
    """Instance built by hand from node coordinates (source first, sink last)."""
    x, y = np.array([p[0] for p in coords], float), np.array([p[1] for p in coords], float)
    return st.Instance(name, m, float(tmax), x, y, np.array(rewards, float), st.travel_times(x, y))


def valid(inst, sol):
    """At most m routes, customers unique and in range, every route within tmax."""
    seen = [v for r in sol for v in r]
    return (len(sol) <= inst.m and len(seen) == len(set(seen)) and all(1 <= v < inst.n - 1 for v in seen)
            and all(st.route_time(inst, r) <= inst.tmax + st.EPS for r in sol))


def greedy_ls(inst, alpha=0.5):
    """Greedy PJS solution improved by local search."""
    return st.local_search(inst, st.pjs_construct(inst, alpha))


def scenarios(inst, c, R, rng):
    """R scenarios of the travel times of `inst` at uncertainty level c."""
    mu, sg = st.lognormal_params(inst.t, c)
    return st.sample_scenarios(mu, sg, inst.t, R, rng)


# ------------------------------------------------------------------ F1, F2

def check_instance_count(names):
    """Exactly 141 feasible p1-p3 instances."""
    return len(names) == 141, f"{len(names)}"


def check_lognormal_moments():
    """Mean ~ t and variance ~ c t for t = 10 (1e6 samples per level)."""
    rng = np.random.default_rng(1)
    t10 = np.array([[0.0, 10.0], [10.0, 0.0]])
    bad = []
    for c in st.C_LEVELS:
        mu, sg = st.lognormal_params(t10, c)
        x = rng.lognormal(mu[0, 1], sg[0, 1], 10**6)
        if abs(x.mean() - 10) > 0.02 or abs(x.var() - 10 * c) > 0.03 * 10 * c + 0.01:
            bad.append((c, x.mean(), x.var()))
    return not bad, str(bad) if bad else ""


def check_lognormal_zero():
    """lognormal_params gives 0 (not ln 0) where t = 0."""
    mu0, sg0 = st.lognormal_params(np.zeros((2, 2)), 0.25)
    return bool(np.all(mu0 == 0) and np.all(sg0 == 0)), ""


# ------------------------------------------------------------------ F3-F6

def check_greedy_vs_top(names, insts):
    """Greedy PJS reward equals the reward column of TOP/results/alpha_grid.csv."""
    grid = pd.read_csv(TOP_GRID)
    mism = []
    for a in st.ALPHAS:
        g = grid[np.isclose(grid.alpha, a)].set_index("instance").reward
        for n in names:
            sol = st.pjs_construct(insts[n], a)
            if not valid(insts[n], sol) or st.det_reward(insts[n], sol) != g[n]:
                mism.append((n, a, st.det_reward(insts[n], sol), g[n]))
    return not mism, f"{3 * len(names)} cases, {len(mism)} mismatches {mism[:3]}"


def check_two_opt():
    """2-opt turns a hand-made zigzag into the optimal order."""
    cross = make_instance([(0, 0), (1, 1), (2, 1), (1, -1), (2, -1), (3, 0)], [0, 1, 1, 1, 1, 0], 1, 100)
    before = st.route_time(cross, (1, 4, 3, 2))
    after = st.local_search(cross, [(1, 4, 3, 2)])
    t_after = st.route_time(cross, after[0])
    best = min(st.route_time(cross, p) for p in itertools.permutations((1, 2, 3, 4)))
    ok = t_after < before - 1e-9 and abs(t_after - best) < 1e-9 and len(after[0]) == 4
    return ok, f"{before:.3f} -> {t_after:.3f}, best {best:.3f}"


def check_insertion():
    """Insertion takes the best reward that fits, then stops."""
    ins = make_instance([(0, 0), (5, 0), (5, 0.1), (5, 3), (10, 0)], [0, 1, 5, 9, 0], 1, 10.5)
    r = st.local_search(ins, [(1,)])
    return sorted(r[0]) == [1, 2], str(r)


def check_local_search(names, insts):
    """LS never lowers the reward nor exceeds tmax (greedy on all instances, BR on a sample)."""
    worse = []
    for n in names:
        inst = insts[n]
        sol = st.pjs_construct(inst, 0.5)
        ls = st.local_search(inst, sol)
        if not valid(inst, ls) or st.det_reward(inst, ls) < st.det_reward(inst, sol):
            worse.append(n)
    rng_t = np.random.default_rng(7)
    for n in names[::10]:
        inst = insts[n]
        for _ in range(5):
            sol = st.pjs_construct(inst, 0.5, "br", rng_t)
            ls = st.local_search(inst, sol)
            if not valid(inst, sol) or not valid(inst, ls) or st.det_reward(inst, ls) < st.det_reward(inst, sol):
                worse.append(n)
            again = st.local_search(inst, ls)
            if st.det_reward(inst, again) != st.det_reward(inst, ls):
                worse.append(n)
    return not worse, str(worse[:3])


# ------------------------------------------------------------------ F7

def check_sim_matches_det(probe, insts):
    """With c = 1e-9 the simulated mean is the deterministic reward."""
    dev = []
    for n in probe:
        inst = insts[n]
        sol = greedy_ls(inst)
        det = st.det_reward(inst, sol)
        mean0 = st.simulate(inst, sol, scenarios(inst, 1e-9, 200, np.random.default_rng(3)))[0]
        if abs(mean0 - det) > 1e-6:
            dev.append((n, det, mean0))
    return not dev, str(dev)


def check_sim_below_det(probe, insts):
    """With c > 0 the simulated mean never exceeds the deterministic reward."""
    over = []
    for n in probe:
        inst = insts[n]
        sol = greedy_ls(inst)
        det = st.det_reward(inst, sol)
        for c in st.C_LEVELS:
            if st.simulate(inst, sol, scenarios(inst, c, 200, np.random.default_rng(4)))[0] > det + 1e-9:
                over.append((n, c))
    return not over, str(over)


def check_route_at_tmax():
    """A route whose expected time equals tmax fails in about half of the replicas."""
    line = make_instance([(0, 0), (2.5, 0), (5, 0), (7.5, 0), (10, 0)], [0, 1, 1, 1, 0], 1, 10.0)
    Tl = scenarios(line, 0.25, 20000, np.random.default_rng(5))
    fail = st.simulate(line, [(1, 2, 3)], Tl)[2]
    return 45 <= fail <= 50, f"{fail:.2f}%"


def check_crn_identical(inst):
    """Identical solutions on the same T give identical results."""
    sol = greedy_ls(inst)
    Tf = scenarios(inst, 0.25, 200, np.random.default_rng(6))
    return st.simulate(inst, sol, Tf) == st.simulate(inst, list(sol), Tf), ""


def check_crn_subset(inst):
    """With the same T, a subset of routes never has a higher mean."""
    sol = greedy_ls(inst)
    Tf = scenarios(inst, 0.25, 200, np.random.default_rng(6))
    ok = all(st.simulate(inst, sol[:k], Tf)[0] <= st.simulate(inst, sol, Tf)[0] for k in range(len(sol) + 1))
    return ok and len(sol) >= 1, f"{len(sol)} routes"


def intensive_and_fast(inst):
    """T_fast and T_int of the case (0, 1, 1), 1000 replicas each, as the generators of a run would give."""
    _, rng_fast, rng_int = st.spawn_rngs(st.case_seed(0, 1, 1))
    return scenarios(inst, 0.25, 1000, rng_fast), scenarios(inst, 0.25, 1000, rng_int)


def check_int_differs(inst):
    """T_int is not T_fast."""
    Tf, Ti = intensive_and_fast(inst)
    return not np.array_equal(Tf, Ti), ""


def check_symmetric(inst):
    """Scenarios are symmetric with a zero diagonal."""
    sym = all(np.array_equal(T, T.transpose(0, 2, 1)) and np.all(np.diagonal(T, axis1=1, axis2=2) == 0)
              for T in intensive_and_fast(inst))
    return bool(sym), ""


# ------------------------------------------------------------------ F8

def check_f8(probe, insts):
    """Four properties of the full run, over the probe instances x 3 alphas (c = 0.25)."""
    names = st.instance_names()
    ok = dict(rep=True, extra=True, dedup=True, valid=True)
    for n in probe:
        inst = insts[n]
        for ia, alpha in enumerate(st.ALPHAS):
            seed = st.case_seed(names.index(n), 1, ia)
            r1, r2 = st.simheuristic(inst, 0.25, alpha, seed), st.simheuristic(inst, 0.25, alpha, seed)
            ok["rep"] &= all(r1[q] == r2[q] for q in r1 if q != "time_s")
            ok["extra"] &= r1["max_det_reward"] >= r1["greedy_ls_reward"]
            ok["valid"] &= valid(inst, r1["routes"]) and r1["det_reward"] == st.det_reward(inst, r1["routes"])
            _, distinct = st.generate_candidates(inst, alpha, st.spawn_rngs(seed)[0])
            forms = [st.canonical(s) for _, s in distinct]
            ok["dedup"] &= len(forms) == len(set(forms)) and len(forms) == r1["n_distinct"]
    return ok


def main():
    """Run every check, print OK/FAIL and return the exit code."""
    names = st.feasible_instances()
    insts = {n: st.load_instance(n) for n in names}
    probe = [n for n in names if n in PROBE_NAMES] or names[::25]
    crn = insts["p2.2.d"]
    f8 = check_f8(probe, insts)
    checks = [
        ("141 feasible instances of p1-p3 found", check_instance_count(names)),
        ("lognormal mean ~ 10 and var ~ 10c (c = 0.05, 0.25, 0.75)", check_lognormal_moments()),
        ("lognormal_params with t = 0 gives 0, no ln 0", check_lognormal_zero()),
        ("greedy PJS reward == TOP alpha_grid.csv, alpha 0.25/0.50/0.75, 141 instances",
         check_greedy_vs_top(names, insts)),
        ("2-opt shortens a hand-made zigzag to the optimal order", check_two_opt()),
        ("insertion takes the best reward that fits (and stops)", check_insertion()),
        ("local search: valid, never lower reward, within tmax, 141 greedy + BR samples",
         check_local_search(names, insts)),
        ("c = 1e-9: simulated mean == deterministic reward", check_sim_matches_det(probe, insts)),
        ("c > 0: simulated mean never above deterministic reward", check_sim_below_det(probe, insts)),
        ("route with E = tmax fails in 45-50% of replicas", check_route_at_tmax()),
        ("CRN: identical solutions, same T -> identical mean", check_crn_identical(crn)),
        ("CRN: sub-set of routes never has a higher mean", check_crn_subset(crn)),
        ("T_int differs from T_fast", check_int_differs(crn)),
        ("T is symmetric with zero diagonal", check_symmetric(crn)),
        ("reproducibility: same seed, same result", (f8["rep"], "")),
        ("best deterministic candidate >= greedy + local search", (f8["extra"], "")),
        ("no repeated canonical forms after dedup", (f8["dedup"], "")),
        ("winner is a valid solution", (f8["valid"], "")),
    ]
    for name, (ok, detail) in checks:
        print(f"{'OK   ' if ok else 'FAIL '} {name}" + (f"  [{detail}]" if detail else ""))
    passed = [bool(ok) for _, (ok, _) in checks]
    print(f"\n{sum(passed)}/{len(passed)} checks passed")
    return 0 if all(passed) else 1


if __name__ == "__main__":
    sys.exit(main())
