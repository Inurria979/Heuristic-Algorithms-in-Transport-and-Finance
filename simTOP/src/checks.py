"""Sanity checks of simtop.py (section 10 of docs/explicacion_paper.md).
Run from simTOP/: `uv run python src/checks.py`. Prints OK/FAIL per check, exit code 1 if any fails."""
import sys
from pathlib import Path

import numpy as np
import pandas as pd

import simtop as st

ROOT = Path(__file__).resolve().parent.parent
TOP_GRID = ROOT.parent / "TOP" / "results" / "alpha_grid.csv"
results = []


def report(name, ok, detail=""):
    """Print and record the outcome of one check."""
    results.append(ok)
    print(f"{'OK   ' if ok else 'FAIL '} {name}" + (f"  [{detail}]" if detail else ""))


def make_instance(coords, rewards, m, tmax, name="hand"):
    """Instance built by hand from node coordinates (source first, sink last)."""
    x, y = np.array([p[0] for p in coords], float), np.array([p[1] for p in coords], float)
    dx, dy = x[:, None] - x[None, :], y[:, None] - y[None, :]
    return st.Instance(name, m, float(tmax), x, y, np.array(rewards, float), np.sqrt(dx * dx + dy * dy))


def valid(inst, sol):
    """At most m routes, customers unique and in range, every route within tmax."""
    seen = [v for r in sol for v in r]
    return (len(sol) <= inst.m and len(seen) == len(set(seen)) and all(1 <= v < inst.n - 1 for v in seen)
            and all(st.route_time(inst, r) <= inst.tmax + st.EPS for r in sol))


bks = pd.read_csv(ROOT / "data" / "bks_chao.csv")
names = [n for n in st.instance_names() if n in set(bks.instance) and n.split(".")[0] in ("p1", "p2", "p3")]
insts = {n: st.load_instance(n) for n in names}
report("141 feasible instances of p1-p3 found", len(names) == 141, f"{len(names)}")

# --- F2: lognormal moments (t = 10, 1e6 samples)
rng = np.random.default_rng(1)
t10 = np.array([[0.0, 10.0], [10.0, 0.0]])
bad = []
for c in st.C_LEVELS:
    mu, sg = st.lognormal_params(t10, c)
    x = rng.lognormal(mu[0, 1], sg[0, 1], 10**6)
    if abs(x.mean() - 10) > 0.02 or abs(x.var() - 10 * c) > 0.03 * 10 * c + 0.01:
        bad.append((c, x.mean(), x.var()))
report("lognormal mean ~ 10 and var ~ 10c (c = 0.05, 0.25, 0.75)", not bad, str(bad) if bad else "")
mu0, sg0 = st.lognormal_params(np.zeros((2, 2)), 0.25)
report("lognormal_params with t = 0 gives 0, no ln 0", np.all(mu0 == 0) and np.all(sg0 == 0))

# --- F3/F4: greedy PJS equals TOP alpha_grid.csv (reward column)
grid = pd.read_csv(TOP_GRID)
mism = []
for a in st.ALPHAS:
    g = grid[np.isclose(grid.alpha, a)].set_index("instance").reward
    for n in names:
        sol = st.pjs_construct(insts[n], a)
        if not valid(insts[n], sol) or st.det_reward(insts[n], sol) != g[n]:
            mism.append((n, a, st.det_reward(insts[n], sol), g[n]))
report("greedy PJS reward == TOP alpha_grid.csv, alpha 0.25/0.50/0.75, 141 instances",
       not mism, f"{3 * len(names)} cases, {len(mism)} mismatches {mism[:3]}")

# --- F6: 2-opt undoes a crossing
cross = make_instance([(0, 0), (1, 1), (2, 1), (1, -1), (2, -1), (3, 0)], [0, 1, 1, 1, 1, 0], 1, 100)
before = st.route_time(cross, (1, 4, 3, 2))
after = st.local_search(cross, [(1, 4, 3, 2)])
t_after = st.route_time(cross, after[0])
best = min(st.route_time(cross, p) for p in __import__("itertools").permutations((1, 2, 3, 4)))
report("2-opt shortens a hand-made zigzag to the optimal order",
       t_after < before - 1e-9 and abs(t_after - best) < 1e-9 and len(after[0]) == 4,
       f"{before:.3f} -> {t_after:.3f}, best {best:.3f}")
ins = make_instance([(0, 0), (5, 0), (5, 0.1), (5, 3), (10, 0)], [0, 1, 5, 9, 0], 1, 10.5)
r = st.local_search(ins, [(1,)])
report("insertion takes the best reward that fits (and stops)", sorted(r[0]) == [1, 2], str(r))

# --- F6: LS never lowers reward nor exceeds tmax (all instances, alpha 0.5; BR too on a few)
worse = []
for n in names:
    inst = insts[n]
    sol = st.pjs_construct(inst, 0.5)
    ls = st.local_search(inst, sol)
    if not valid(inst, ls) or st.det_reward(inst, ls) < st.det_reward(inst, sol):
        worse.append(n)
sample = names[::10]
rng_t = np.random.default_rng(7)
for n in sample:
    inst = insts[n]
    for _ in range(5):
        sol = st.pjs_construct(inst, 0.5, "br", rng_t)
        ls = st.local_search(inst, sol)
        if not valid(inst, sol) or not valid(inst, ls) or st.det_reward(inst, ls) < st.det_reward(inst, sol):
            worse.append(n)
        again = st.local_search(inst, ls)
        if st.det_reward(inst, again) != st.det_reward(inst, ls):
            worse.append(n)
report("local search: valid, never lower reward, within tmax, 141 greedy + BR samples", not worse, str(worse[:3]))

# --- F7: simulation
probe = [n for n in names if n in ("p1.2.b", "p1.3.h", "p2.2.d", "p2.4.k", "p3.2.f", "p3.4.m")] or names[::25]
dev = []
for n in probe:
    inst = insts[n]
    sol = st.local_search(inst, st.pjs_construct(inst, 0.5))
    det = st.det_reward(inst, sol)
    rg = np.random.default_rng(3)
    mu, sg = st.lognormal_params(inst.t, 1e-9)
    mean0 = st.simulate(inst, sol, st.sample_scenarios(mu, sg, inst.t, 200, rg))[0]
    if abs(mean0 - det) > 1e-6:
        dev.append((n, det, mean0))
report("c = 1e-9: simulated mean == deterministic reward", not dev, str(dev))
over = []
for n in probe:
    inst = insts[n]
    sol = st.local_search(inst, st.pjs_construct(inst, 0.5))
    det = st.det_reward(inst, sol)
    for c in st.C_LEVELS:
        mu, sg = st.lognormal_params(inst.t, c)
        if st.simulate(inst, sol, st.sample_scenarios(mu, sg, inst.t, 200, np.random.default_rng(4)))[0] > det + 1e-9:
            over.append((n, c))
report("c > 0: simulated mean never above deterministic reward", not over, str(over))

line = make_instance([(0, 0), (2.5, 0), (5, 0), (7.5, 0), (10, 0), (10, 0)][:5], [0, 1, 1, 1, 0], 1, 10.0)
mu, sg = st.lognormal_params(line.t, 0.25)
Tl = st.sample_scenarios(mu, sg, line.t, 20000, np.random.default_rng(5))
_, _, fail = st.simulate(line, [(1, 2, 3)], Tl)
report("route with E = tmax fails in 45-50% of replicas", 45 <= fail <= 50, f"{fail:.2f}%")

# --- CRN
inst = insts["p2.2.d"]
mu, sg = st.lognormal_params(inst.t, 0.25)
Tf = st.sample_scenarios(mu, sg, inst.t, 200, np.random.default_rng(6))
sol = st.local_search(inst, st.pjs_construct(inst, 0.5))
a, b = st.simulate(inst, sol, Tf), st.simulate(inst, list(sol), Tf)
report("CRN: identical solutions, same T -> identical mean", a == b)
sub_ok = all(st.simulate(inst, sol[:k], Tf)[0] <= st.simulate(inst, sol, Tf)[0] for k in range(len(sol) + 1))
report("CRN: sub-set of routes never has a higher mean", sub_ok and len(sol) >= 1, f"{len(sol)} routes")

# --- T_int vs T_fast, symmetry
ss = st.case_seed(0, 1, 1)
_, s_fast, s_int = ss.spawn(3)
Tf = st.sample_scenarios(mu, sg, inst.t, 1000, np.random.default_rng(s_fast))
Ti = st.sample_scenarios(mu, sg, inst.t, 1000, np.random.default_rng(s_int))
report("T_int differs from T_fast", not np.array_equal(Tf, Ti))
sym = all(np.array_equal(T, T.transpose(0, 2, 1)) and np.all(np.diagonal(T, axis1=1, axis2=2) == 0) for T in (Tf, Ti))
report("T is symmetric with zero diagonal", sym)

# --- F8: reproducibility, greedy extra, dedup
rep_ok, extra_ok, dedup_ok, valid_ok = True, True, True, True
for k, n in enumerate(probe):
    inst = insts[n]
    for ia, alpha in enumerate(st.ALPHAS):
        seed = st.case_seed(st.instance_names().index(n), 1, ia)
        r1, r2 = st.simheuristic(inst, 0.25, alpha, seed), st.simheuristic(inst, 0.25, alpha, seed)
        keys = [q for q in r1 if q != "time_s"]
        rep_ok &= all(r1[q] == r2[q] for q in keys)
        extra_ok &= r1["max_det_reward"] >= r1["greedy_ls_reward"]
        valid_ok &= valid(inst, r1["routes"]) and r1["det_reward"] == st.det_reward(inst, r1["routes"])
        _, distinct = st.generate_candidates(inst, alpha, np.random.default_rng(st.case_seed(st.instance_names().index(n), 1, ia).spawn(3)[0]))
        forms = [st.canonical(s) for _, s in distinct]
        dedup_ok &= len(forms) == len(set(forms)) and len(forms) == r1["n_distinct"]
report("reproducibility: same seed, same result", rep_ok)
report("best deterministic candidate >= greedy + local search", extra_ok)
report("no repeated canonical forms after dedup", dedup_ok)
report("winner is a valid solution", valid_ok)

print(f"\n{sum(results)}/{len(results)} checks passed")
sys.exit(0 if all(results) else 1)
