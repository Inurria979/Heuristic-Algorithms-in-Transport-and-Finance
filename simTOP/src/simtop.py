"""Simheuristic for the Team Orienteering Problem with lognormal travel times
(Juan et al. 2019): PJS savings + biased randomisation + local search, then fast and
intensive Monte Carlo simulation. Heavy loops are Numba kernels on numpy arrays.

A solution is a list of routes, each route a tuple of customer indices (depots left out).
Node 0 is the source and node n-1 the sink.
"""
from pathlib import Path
from time import perf_counter
from typing import NamedTuple

import numpy as np
from numba import njit

DATA = Path(__file__).resolve().parent.parent / "data" / "top-data"
EPS = 1e-9                      # tolerance of the tmax check, as in TOP (model.rs)
BETA = 0.3                      # geometric biased randomisation
N_BR = 50                       # biased-randomised constructions per run
N_FAST, N_FINAL = 7, 3          # candidates to the fast / intensive simulation
R_FAST, R_INT = 200, 1000       # replicas of each simulation
C_LEVELS = (0.05, 0.25, 0.75)
ALPHAS = (0.25, 0.5, 0.75)
BASE_SEED = 2019
# idx_inst is the position of the instance in the alphabetical list of the file stems of
# the 387 Chao instances (instance_names()). p1-p3 come first, so the index is the same in
# the list of p1-p3 alone.


class Instance(NamedTuple):
    name: str
    m: int
    tmax: float
    x: np.ndarray
    y: np.ndarray
    reward: np.ndarray
    t: np.ndarray               # expected travel times (Euclidean), n x n

    @property
    def n(self):
        return len(self.x)


# ---------------------------------------------------------------- F1 instances

def instance_names(directory=DATA):
    """Alphabetical list of the instance names (file stems) in the data folder."""
    return sorted(p.stem for p in Path(directory).glob("*.txt"))


def load_instance(path):
    """Read a Chao instance ("n;N", "m;M", "tmax;T", then N lines "x;y;reward")."""
    path = Path(path)
    if not path.exists():
        path = DATA / f"{path.name}.txt"
    lines = [ln.strip() for ln in path.read_text().splitlines() if ln.strip()]
    n, m, tmax = (float(ln.split(";")[1]) for ln in lines[:3])
    nodes = np.array([[float(v) for v in ln.split(";")] for ln in lines[3:]])
    assert len(nodes) == int(n), f"{path}: header says n={int(n)}, found {len(nodes)} nodes"
    x, y, reward = nodes[:, 0].copy(), nodes[:, 1].copy(), nodes[:, 2].copy()
    dx, dy = x[:, None] - x[None, :], y[:, None] - y[None, :]
    return Instance(path.stem, int(m), tmax, x, y, reward, np.sqrt(dx * dx + dy * dy))


# ------------------------------------------------------------ F2 lognormal law

def lognormal_params(t, c):
    """mu, sigma of ln T such that E[T] = t and Var[T] = c * t (0 where t = 0)."""
    pos = t > 0
    tt = np.where(pos, t, 1.0)
    s2 = np.log1p(c / tt)
    return np.where(pos, np.log(tt) - 0.5 * s2, 0.0), np.where(pos, np.sqrt(s2), 0.0)


# ------------------------------------------------------------- numba kernels

@njit(cache=True)
def _route_time(t, nodes, ln):
    """Time of source -> nodes[:ln] -> sink, summed leg by leg."""
    time, prev = 0.0, 0
    for k in range(ln):
        time += t[prev, nodes[k]]
        prev = nodes[k]
    return time + t[prev, t.shape[0] - 1]


@njit(cache=True)
def _construct(t, reward, tmax, m, arc_i, arc_j, u, beta, use_br):
    """PJS merging over the sorted arc list; `u` are the uniforms of the BR selector."""
    n = t.shape[0]
    sink = n - 1
    nxt = -np.ones(n, np.int64)
    route_of = -np.ones(n, np.int64)
    head = -np.ones(n, np.int64)
    tail = -np.ones(n, np.int64)
    rtime = np.zeros(n)
    rrew = np.zeros(n)
    rlen = np.zeros(n, np.int64)
    alive = np.zeros(n, np.bool_)
    one = np.zeros(1, np.int64)
    for i in range(1, sink):                       # step 1: dummy routes
        one[0] = i
        tm = _route_time(t, one, 1)
        if tm <= tmax + EPS:
            route_of[i], head[i], tail[i] = i, i, i
            rtime[i], rrew[i], rlen[i], alive[i] = tm, reward[i], 1, True
    nar = arc_i.shape[0]
    left = np.arange(nar - 1, -1, -1)              # top of the list at the end
    cnt = nar
    for step in range(nar):                        # step 3: merging
        if use_br:
            k = int(np.log(1.0 - u[step]) / np.log(1.0 - beta)) % cnt
            pos = cnt - 1 - k
            idx = left[pos]
            for q in range(pos, cnt - 1):
                left[q] = left[q + 1]
            cnt -= 1
        else:
            idx = step
        i, j = arc_i[idx], arc_j[idx]
        a, b = route_of[i], route_of[j]
        if a < 0 or b < 0 or a == b or tail[a] != i or head[b] != j:
            continue
        if rtime[a] + rtime[b] - t[i, sink] - t[0, j] + t[i, j] > tmax + 1e-6:
            continue
        nxt[i] = j                                 # tentative link, exact time leg by leg
        tm, prev, node = 0.0, 0, head[a]
        while node != -1:
            tm += t[prev, node]
            prev, node = node, nxt[node]
        tm += t[prev, sink]
        if tm > tmax + EPS:
            nxt[i] = -1
            continue
        node = head[b]
        while node != -1:
            route_of[node] = a
            node = nxt[node]
        tail[a] = tail[b]
        rtime[a], rrew[a], rlen[a] = tm, rrew[a] + rrew[b], rlen[a] + rlen[b]
        alive[b] = False
    ids = np.empty(n, np.int64)                    # step 4: sort by reward desc, time asc
    ni = 0
    for a in range(n):
        if alive[a]:
            q = ni
            while q > 0 and (rrew[a] > rrew[ids[q - 1]] or
                             (rrew[a] == rrew[ids[q - 1]] and rtime[a] < rtime[ids[q - 1]])):
                ids[q] = ids[q - 1]
                q -= 1
            ids[q] = a
            ni += 1
    routes = -np.ones((m, n), np.int64)
    lens = -np.ones(m, np.int64)
    for s in range(min(m, ni)):
        node, ln = head[ids[s]], 0
        while node != -1:
            routes[s, ln] = node
            ln += 1
            node = nxt[node]
        lens[s] = ln
    return routes, lens


@njit(cache=True)
def _two_opt(t, nodes, ln):
    """2-opt, first improvement, in place; returns the number of reversals."""
    sink = t.shape[0] - 1
    moves = 0
    again = True
    while again:
        again = False
        for i in range(ln):
            for j in range(i + 1, ln):
                a = 0 if i == 0 else nodes[i - 1]
                d = sink if j + 1 == ln else nodes[j + 1]
                delta = t[a, nodes[j]] + t[nodes[i], d] - t[a, nodes[i]] - t[nodes[j], d]
                if delta < -EPS:
                    lo, hi = i, j
                    while lo < hi:
                        nodes[lo], nodes[hi] = nodes[hi], nodes[lo]
                        lo += 1
                        hi -= 1
                    moves += 1
                    again = True
                    break
            if again:
                break
    return moves


@njit(cache=True)
def _insert_one(t, reward, tmax, routes, lens, rtime, visited, reach):
    """Insert the highest-reward unvisited customer that fits at its cheapest position."""
    n = t.shape[0]
    sink = n - 1
    m = routes.shape[0]
    banned = np.zeros((n, m, n + 1), np.bool_)
    tmp = np.empty(n + 1, np.int64)
    while True:
        found = False
        bk, br, bp, bd = 0, 0, 0, 0.0
        for k in range(1, sink):
            if visited[k] or not reach[k]:
                continue
            for r in range(m):
                if lens[r] < 0:
                    continue
                for p in range(lens[r] + 1):
                    a = 0 if p == 0 else routes[r, p - 1]
                    b = sink if p == lens[r] else routes[r, p]
                    delta = t[a, k] + t[k, b] - t[a, b]
                    if rtime[r] + delta > tmax + EPS or banned[k, r, p]:
                        continue
                    if (not found) or reward[k] > reward[bk] or (reward[k] == reward[bk] and delta < bd):
                        found = True
                        bk, br, bp, bd = k, r, p, delta
        if not found:
            return False
        ln = lens[br]
        for q in range(bp):
            tmp[q] = routes[br, q]
        tmp[bp] = bk
        for q in range(bp, ln):
            tmp[q + 1] = routes[br, q]
        tm = _route_time(t, tmp, ln + 1)
        if tm > tmax + EPS:
            banned[bk, br, bp] = True
            continue
        for q in range(ln + 1):
            routes[br, q] = tmp[q]
        lens[br] = ln + 1
        rtime[br] = tm
        visited[bk] = True
        return True


@njit(cache=True)
def _local_search(t, reward, tmax, routes0, lens0):
    """Repeat { 2-opt in every route; insert one customer } until nothing can be inserted."""
    n = t.shape[0]
    sink = n - 1
    routes, lens = routes0.copy(), lens0.copy()
    m = routes.shape[0]
    visited = np.zeros(n, np.bool_)
    rtime = np.zeros(m)
    for r in range(m):
        if lens[r] >= 0:
            rtime[r] = _route_time(t, routes[r], lens[r])
            for q in range(lens[r]):
                visited[routes[r, q]] = True
    reach = np.zeros(n, np.bool_)
    one = np.zeros(1, np.int64)
    for k in range(1, sink):
        one[0] = k
        reach[k] = _route_time(t, one, 1) <= tmax + EPS
    while True:
        for r in range(m):
            if lens[r] > 0 and _two_opt(t, routes[r], lens[r]) > 0:
                rtime[r] = _route_time(t, routes[r], lens[r])
        if not _insert_one(t, reward, tmax, routes, lens, rtime, visited, reach):
            break
    return routes, lens


@njit(cache=True)
def _simulate(reward, tmax, routes, lens, T):
    """Reward per replica (a route that overruns tmax pays nothing) and failed routes."""
    R = T.shape[0]
    m = routes.shape[0]
    sink = T.shape[1] - 1
    out = np.zeros(R)
    fails = 0
    for r in range(R):
        for s in range(m):
            if lens[s] < 0:
                continue
            time, prev, rew = 0.0, 0, 0.0
            for q in range(lens[s]):
                time += T[r, prev, routes[s, q]]
                prev = routes[s, q]
                rew += reward[prev]
            time += T[r, prev, sink]
            if time <= tmax:
                out[r] += rew
            else:
                fails += 1
    return out, fails


# ------------------------------------------------------------ packing helpers

def _pack(inst, sol):
    """Solution (list of tuples) -> padded arrays (routes, lens) for the kernels."""
    routes = -np.ones((max(inst.m, len(sol)), inst.n), np.int64)
    lens = -np.ones(routes.shape[0], np.int64)
    for s, route in enumerate(sol):
        routes[s, :len(route)] = route
        lens[s] = len(route)
    return routes, lens


def _unpack(routes, lens):
    """Padded arrays -> list of tuples of Python ints."""
    return [tuple(int(v) for v in routes[s, :lens[s]]) for s in range(len(lens)) if lens[s] >= 0]


def route_time(inst, route):
    """Expected time of source -> route -> sink."""
    return float(_route_time(inst.t, np.array(route, np.int64), len(route)))


def det_reward(inst, sol):
    """Deterministic reward: sum of the rewards of the visited customers."""
    return float(sum(inst.reward[v] for route in sol for v in route))


def det_time(inst, sol):
    """Total expected time of the routes."""
    return float(sum(route_time(inst, r) for r in sol))


def canonical(sol):
    """Canonical form: the set of routes, whatever their order (direction matters)."""
    return frozenset(tuple(r) for r in sol)


# ------------------------------------------------------------- F3-F5 PJS + BR

def savings_arcs(inst, alpha):
    """Arcs (i, j) between reachable customers sorted by decreasing enriched savings."""
    t, sink = inst.t, inst.n - 1
    cust = np.array([i for i in range(1, sink) if t[0, i] + t[i, sink] <= inst.tmax + EPS], np.int64)
    a, b = np.triu_indices(len(cust), 1)
    ii = np.empty(2 * len(a), np.int64)
    jj = np.empty(2 * len(a), np.int64)
    ii[0::2], jj[0::2] = cust[a], cust[b]          # generation order of TOP: (i, j), (j, i)
    ii[1::2], jj[1::2] = cust[b], cust[a]
    ts = t[ii, sink] + t[0, jj] - t[ii, jj]
    sav = alpha * ts + (1.0 - alpha) * (inst.reward[ii] + inst.reward[jj])
    order = np.argsort(-sav, kind="stable")
    return ii[order], jj[order]


def pjs_construct(inst, alpha, selector="greedy", rng=None, beta=BETA, arcs=None):
    """PJS savings heuristic; selector "greedy" (F4) or "br" (F5, geometric, needs rng)."""
    arc_i, arc_j = savings_arcs(inst, alpha) if arcs is None else arcs
    assert selector in ("greedy", "br"), f"unknown selector {selector!r}"
    use_br = selector == "br"
    u = rng.random(len(arc_i)) if use_br else np.zeros(1)
    routes, lens = _construct(inst.t, inst.reward, inst.tmax, inst.m, arc_i, arc_j, u, beta, use_br)
    return _unpack(routes, lens)


# ------------------------------------------------------------ F6 local search

def local_search(inst, sol):
    """2-opt (first improvement) + best-reward insertion, repeated; never worsens `sol`."""
    routes, lens = _pack(inst, sol)
    return _unpack(*_local_search(inst.t, inst.reward, inst.tmax, routes, lens))


# ------------------------------------------------------------- F7 simulation

def sample_scenarios(mu, sigma, t, R, rng):
    """Travel times T[r, i, j] = T[r, j, i] ~ LogNormal(mu_ij, sigma_ij), independent per
    arc and replica; 0 where t_ij = 0. Shape (R, N, N)."""
    N = t.shape[0]
    z = rng.standard_normal((R, N, N))
    full = np.exp(mu[None] + sigma[None] * z) * (t > 0)[None]
    upper = np.triu(np.ones((N, N), bool), 1)[None]
    return np.where(upper, full, full.transpose(0, 2, 1))


def simulate(inst, sol, T):
    """Mean and std of the reward over the replicas of T, and % of routes that overrun tmax."""
    routes, lens = _pack(inst, sol)
    out, fails = _simulate(inst.reward, inst.tmax, routes, lens, T)
    n_routes = max(1, int((lens >= 0).sum()))
    return float(out.mean()), float(out.std(ddof=1)), 100.0 * fails / (len(T) * n_routes)


# ------------------------------------------------------------------ F8 run

def case_seed(idx_inst, idx_c, idx_alpha, base=BASE_SEED):
    """SeedSequence of one (instance, c, alpha) case."""
    return np.random.SeedSequence([base, idx_inst, idx_c, idx_alpha])


def generate_candidates(inst, alpha, rng_br, n_br=N_BR, beta=BETA):
    """Greedy + n_br biased-randomised constructions, each improved by local search.
    Returns (all, distinct): lists in generation order; `distinct` keeps the first of each
    canonical form."""
    arcs = savings_arcs(inst, alpha)
    cands = [local_search(inst, pjs_construct(inst, alpha, "greedy", arcs=arcs))]
    for _ in range(n_br):
        cands.append(local_search(inst, pjs_construct(inst, alpha, "br", rng_br, beta, arcs)))
    seen, distinct = set(), []
    for k, sol in enumerate(cands):
        form = canonical(sol)
        if form not in seen:
            seen.add(form)
            distinct.append((k, sol))
    return cands, distinct


def simheuristic(inst, c, alpha, seed):
    """One run: 51 candidates -> 7 by deterministic reward -> fast simulation (common
    random numbers) -> 3 -> intensive simulation (fresh days) -> winner."""
    start = perf_counter()
    # fresh copy: spawn() mutates a SeedSequence, and the same seed must give the same run
    if isinstance(seed, np.random.SeedSequence):
        ss = np.random.SeedSequence(seed.entropy, spawn_key=seed.spawn_key)
    else:
        ss = np.random.SeedSequence(seed)
    s_br, s_fast, s_int = ss.spawn(3)
    rng_br, rng_fast, rng_int = (np.random.default_rng(s) for s in (s_br, s_fast, s_int))
    cands, distinct = generate_candidates(inst, alpha, rng_br)
    # deterministic filter: reward desc, total time asc, generation order
    rows = [(-det_reward(inst, sol), det_time(inst, sol), k, sol) for k, sol in distinct]
    rows.sort(key=lambda r: r[:3])
    top = rows[:N_FAST]
    mu, sigma = lognormal_params(inst.t, c)
    t_fast = sample_scenarios(mu, sigma, inst.t, R_FAST, rng_fast)
    fast = [simulate(inst, r[3], t_fast)[0] for r in top]
    order = sorted(range(len(top)), key=lambda q: (-fast[q], top[q][0], q))[:N_FINAL]
    t_int = sample_scenarios(mu, sigma, inst.t, R_INT, rng_int)
    final = [simulate(inst, top[q][3], t_int) for q in order]
    w = max(range(len(order)), key=lambda q: (final[q][0], -q))
    q = order[w]
    sol = top[q][3]
    return {
        "det_reward": -top[q][0], "fast_mean": fast[q], "exp_reward": final[w][0],
        "exp_std": final[w][1], "fail_rate": final[w][2], "n_distinct": len(distinct),
        "winner_is_greedy": top[q][2] == 0, "routes": sol, "seed": str(ss.entropy),
        "max_det_reward": -rows[0][0], "greedy_ls_reward": det_reward(inst, cands[0]),
        "time_s": perf_counter() - start,
    }
