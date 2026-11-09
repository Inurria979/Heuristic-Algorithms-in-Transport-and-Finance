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
import pandas as pd
from numba import njit

DATA = Path(__file__).resolve().parent.parent / "data" / "top-data"
BKS_FILE = DATA.parent / "bks_chao.csv"
SETS = ("p1", "p2", "p3")       # Chao sets used (those with a known BKS and feasible sizes)
EPS = 1e-9                      # tolerance of the tmax check, as in TOP (model.rs)
MERGE_PRETOL = 1e-6             # cheap prefilter of a merge; the exact check uses EPS
BETA = 0.3                      # geometric biased randomisation
N_BR = 50                       # biased-randomised constructions per run
N_FAST, N_FINAL = 7, 3          # candidates to the fast / intensive simulation
R_FAST, R_INT = 200, 1000       # replicas of each simulation
C_LEVELS = (0.05, 0.25, 0.75)
ALPHAS = (0.25, 0.5, 0.75)
BASE_SEED = 2019


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


def travel_times(x, y):
    """Euclidean expected travel-time matrix between the nodes (x, y)."""
    dx, dy = x[:, None] - x[None, :], y[:, None] - y[None, :]
    return np.sqrt(dx * dx + dy * dy)


def read_instance(path):
    """Read a Chao instance file ("n;N", "m;M", "tmax;T", then N lines "x;y;reward")."""
    path = Path(path)
    lines = [ln.strip() for ln in path.read_text().splitlines() if ln.strip()]
    n, m, tmax = (float(ln.split(";")[1]) for ln in lines[:3])
    nodes = np.array([[float(v) for v in ln.split(";")] for ln in lines[3:]])
    assert len(nodes) == int(n), f"{path}: header says n={int(n)}, found {len(nodes)} nodes"
    x, y, reward = nodes[:, 0].copy(), nodes[:, 1].copy(), nodes[:, 2].copy()
    return Instance(path.stem, int(m), tmax, x, y, reward, travel_times(x, y))


def load_instance(name):
    """Instance by name (e.g. "p2.3.g") from the data folder; use read_instance for a path."""
    return read_instance(DATA / f"{name}.txt")


def load_bks():
    """Best-known rewards of the Chao instances (PSOiA), a Series indexed by instance name."""
    return pd.read_csv(BKS_FILE).set_index("instance").bks


def feasible_instances():
    """Names of the instances of p1-p3 that have a BKS (141), in alphabetical order."""
    bks = load_bks()
    return [n for n in instance_names() if n in bks.index and n.split(".")[0] in SETS]


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
def _reachable(t, tmax):
    """reach[k]: customer k fits alone (source -> k -> sink within tmax); False for source and sink."""
    n = t.shape[0]
    reach = np.zeros(n, np.bool_)
    for k in range(1, n - 1):
        reach[k] = t[0, k] + t[k, n - 1] <= tmax + EPS
    return reach


@njit(cache=True)
def _dummy_routes(t, reward, reach, route_of, head, tail, rtime, rrew, rlen, alive):
    """PJS step 1: one route source -> i -> sink for every reachable customer."""
    one = np.zeros(1, np.int64)
    for i in range(1, t.shape[0] - 1):
        if reach[i]:
            one[0] = i
            route_of[i], head[i], tail[i] = i, i, i
            rtime[i], rrew[i], rlen[i], alive[i] = _route_time(t, one, 1), reward[i], 1, True


@njit(cache=True)
def _br_pick(left, cnt, u, beta):
    """Biased pick: remove from left[:cnt] the arc at geometric position k from the top and return it."""
    k = int(np.log(1.0 - u) / np.log(1.0 - beta)) % cnt
    pos = cnt - 1 - k
    idx = left[pos]
    for q in range(pos, cnt - 1):
        left[q] = left[q + 1]
    return idx


@njit(cache=True)
def _try_merge(t, tmax, i, j, nxt, route_of, head, tail, rtime, rrew, rlen, alive):
    """PJS step 3 for arc (i, j): join the route ending at i with the one starting at j if it fits."""
    sink = t.shape[0] - 1
    a, b = route_of[i], route_of[j]
    if a < 0 or b < 0 or a == b or tail[a] != i or head[b] != j:
        return
    if rtime[a] + rtime[b] - t[i, sink] - t[0, j] + t[i, j] > tmax + MERGE_PRETOL:
        return
    nxt[i] = j                                     # tentative link, exact time leg by leg
    tm, prev, node = 0.0, 0, head[a]
    while node != -1:
        tm += t[prev, node]
        prev, node = node, nxt[node]
    tm += t[prev, sink]
    if tm > tmax + EPS:
        nxt[i] = -1
        return
    node = head[b]
    while node != -1:
        route_of[node] = a
        node = nxt[node]
    tail[a] = tail[b]
    rtime[a], rrew[a], rlen[a] = tm, rrew[a] + rrew[b], rlen[a] + rlen[b]
    alive[b] = False


@njit(cache=True)
def _select_routes(m, rtime, rrew, alive, head, nxt):
    """PJS step 4: the m best routes (reward desc, time asc) as padded arrays (routes, lens)."""
    n = alive.shape[0]
    ids = np.empty(n, np.int64)
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
def _construct(t, reward, tmax, m, arc_i, arc_j, u, beta, use_br, reach):
    """PJS merging over the sorted arc list; `u` are the uniforms of the BR selector."""
    n = t.shape[0]
    nxt = -np.ones(n, np.int64)
    route_of = -np.ones(n, np.int64)
    head = -np.ones(n, np.int64)
    tail = -np.ones(n, np.int64)
    rtime = np.zeros(n)
    rrew = np.zeros(n)
    rlen = np.zeros(n, np.int64)
    alive = np.zeros(n, np.bool_)
    _dummy_routes(t, reward, reach, route_of, head, tail, rtime, rrew, rlen, alive)
    nar = arc_i.shape[0]
    left = np.arange(nar - 1, -1, -1)              # top of the list at the end
    cnt = nar
    for step in range(nar):
        if use_br:
            idx = _br_pick(left, cnt, u[step], beta)
            cnt -= 1
        else:
            idx = step
        _try_merge(t, tmax, arc_i[idx], arc_j[idx], nxt, route_of, head, tail, rtime, rrew, rlen, alive)
    return _select_routes(m, rtime, rrew, alive, head, nxt)


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
    """Insert the best-reward unvisited customer that fits; True if one was inserted."""
    n = t.shape[0]
    sink = n - 1
    m = routes.shape[0]
    # banned: (customer, route, position) slots that passed the delta test but failed the exact time check
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
def _local_search(t, reward, tmax, routes0, lens0, reach):
    """Repeat { 2-opt in every route; insert one customer } until nothing can be inserted."""
    n = t.shape[0]
    routes, lens = routes0.copy(), lens0.copy()
    m = routes.shape[0]
    visited = np.zeros(n, np.bool_)
    rtime = np.zeros(m)
    for r in range(m):
        if lens[r] >= 0:
            rtime[r] = _route_time(t, routes[r], lens[r])
            for q in range(lens[r]):
                visited[routes[r, q]] = True
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


def format_routes(sol):
    """Routes as a CSV string: customers joined by "-", routes by ";"."""
    return ";".join("-".join(map(str, route)) for route in sol)


def parse_routes(s):
    """Inverse of format_routes: list of tuples of customer indices ([] for an empty or missing value)."""
    return [tuple(int(v) for v in route.split("-")) for route in str(s).split(";") if route and route != "nan"]


# ------------------------------------------------------------- F3-F5 PJS + BR

def reachable(inst):
    """Boolean array: customer k can be served alone within tmax (source -> k -> sink)."""
    return _reachable(inst.t, inst.tmax)


def savings_arcs(inst, alpha, reach=None):
    """Arcs (i, j) between reachable customers sorted by decreasing enriched savings."""
    t, sink = inst.t, inst.n - 1
    reach = reachable(inst) if reach is None else reach
    cust = np.flatnonzero(reach).astype(np.int64)
    a, b = np.triu_indices(len(cust), 1)
    ii = np.empty(2 * len(a), np.int64)
    jj = np.empty(2 * len(a), np.int64)
    ii[0::2], jj[0::2] = cust[a], cust[b]          # generation order of TOP: (i, j), (j, i)
    ii[1::2], jj[1::2] = cust[b], cust[a]
    ts = t[ii, sink] + t[0, jj] - t[ii, jj]
    sav = alpha * ts + (1.0 - alpha) * (inst.reward[ii] + inst.reward[jj])
    order = np.argsort(-sav, kind="stable")
    return ii[order], jj[order]


def pjs_construct(inst, alpha, selector="greedy", rng=None, beta=BETA, arcs=None, reach=None):
    """PJS savings heuristic; selector "greedy" (F4) or "br" (F5, geometric, needs rng)."""
    assert selector in ("greedy", "br"), f"unknown selector {selector!r}"
    use_br = selector == "br"
    assert not use_br or rng is not None, 'selector "br" needs an rng'
    reach = reachable(inst) if reach is None else reach
    arc_i, arc_j = savings_arcs(inst, alpha, reach) if arcs is None else arcs
    u = rng.random(len(arc_i)) if use_br else np.zeros(1)
    routes, lens = _construct(inst.t, inst.reward, inst.tmax, inst.m, arc_i, arc_j, u, beta, use_br, reach)
    return _unpack(routes, lens)


# ------------------------------------------------------------ F6 local search

def local_search(inst, sol, reach=None):
    """2-opt (first improvement) + best-reward insertion, repeated; never worsens `sol`."""
    # empty vehicle slots (fewer than m routes in `sol`) stay empty: PJS never leaves one that could be used
    reach = reachable(inst) if reach is None else reach
    routes, lens = _pack(inst, sol)
    return _unpack(*_local_search(inst.t, inst.reward, inst.tmax, routes, lens, reach))


# ------------------------------------------------------------- F7 simulation

def sample_scenarios(mu, sigma, t, R, rng):
    """Travel times T[r, i, j] = T[r, j, i] ~ LogNormal(mu_ij, sigma_ij), shape (R, N, N); 0 where t_ij = 0."""
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
    """SeedSequence of a case; idx_inst is the position in instance_names() (all 387 files, not the 141)."""
    return np.random.SeedSequence([base, idx_inst, idx_c, idx_alpha])


def _seed_sequence(seed):
    """Fresh copy of a seed: spawn() mutates a SeedSequence, and the same seed must give the same run."""
    if isinstance(seed, np.random.SeedSequence):
        return np.random.SeedSequence(seed.entropy, spawn_key=seed.spawn_key)
    return np.random.SeedSequence(seed)


def spawn_rngs(seed):
    """The three independent generators of a run, in this order: biased randomisation, fast, intensive."""
    return tuple(np.random.default_rng(s) for s in _seed_sequence(seed).spawn(3))


def generate_candidates(inst, alpha, rng_br, n_br=N_BR, beta=BETA):
    """Greedy + n_br BR constructions, each with local search; returns (all, distinct) in generation order."""
    reach = reachable(inst)
    arcs = savings_arcs(inst, alpha, reach)
    cands = [local_search(inst, pjs_construct(inst, alpha, "greedy", arcs=arcs, reach=reach), reach)]
    for _ in range(n_br):
        cands.append(local_search(inst, pjs_construct(inst, alpha, "br", rng_br, beta, arcs, reach), reach))
    seen, distinct = set(), []
    for k, sol in enumerate(cands):
        form = canonical(sol)
        if form not in seen:
            seen.add(form)
            distinct.append((k, sol))
    return cands, distinct


def deterministic_filter(inst, distinct, n_keep=N_FAST):
    """The n_keep best distinct candidates as (-reward, time, k, sol): reward desc, time asc, generation order."""
    rows = [(-det_reward(inst, sol), det_time(inst, sol), k, sol) for k, sol in distinct]
    rows.sort(key=lambda r: r[:3])
    return rows[:n_keep]


def fast_screen(inst, top, t_fast, n_keep=N_FINAL):
    """Common-random-numbers screen: (indices in `top` of the n_keep best fast means, all fast means)."""
    fast = [simulate(inst, r[3], t_fast)[0] for r in top]
    order = sorted(range(len(top)), key=lambda q: (-fast[q], top[q][0], q))[:n_keep]
    return order, fast


def intensive_winner(inst, top, order, t_int):
    """Simulate the finalists on fresh days; returns (position of the winner in `order`, all results)."""
    final = [simulate(inst, top[q][3], t_int) for q in order]
    return max(range(len(order)), key=lambda q: (final[q][0], -q)), final


def simheuristic(inst, c, alpha, seed):
    """One run: 51 candidates -> 7 (deterministic) -> 3 (fast simulation, CRN) -> 1 (intensive simulation)."""
    start = perf_counter()
    rng_br, rng_fast, rng_int = spawn_rngs(seed)
    cands, distinct = generate_candidates(inst, alpha, rng_br)
    top = deterministic_filter(inst, distinct)
    mu, sigma = lognormal_params(inst.t, c)
    order, fast = fast_screen(inst, top, sample_scenarios(mu, sigma, inst.t, R_FAST, rng_fast))
    w, final = intensive_winner(inst, top, order, sample_scenarios(mu, sigma, inst.t, R_INT, rng_int))
    q = order[w]
    return {
        "det_reward": -top[q][0], "fast_mean": fast[q], "exp_reward": final[w][0],
        "exp_std": final[w][1], "fail_rate": final[w][2], "n_distinct": len(distinct),
        "winner_is_greedy": top[q][2] == 0, "routes": top[q][3], "seed": str(_seed_sequence(seed).entropy),
        "max_det_reward": -top[0][0], "greedy_ls_reward": det_reward(inst, cands[0]),
        "time_s": perf_counter() - start,
    }
