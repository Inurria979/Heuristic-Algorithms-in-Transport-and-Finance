"""Clarke & Wright savings heuristic (CWS) for the Capacitated Vehicle Routing Problem.

Instances live in ``../data/<name>_input_nodes.txt``: one tab-separated row per node,
``x y demand``, the first row being the depot at the origin. The vehicle capacity is not
in those files; it is taken from Table 1 of Juan et al. (2011, JORS 62(6)).

Only the classic CWS is implemented, i.e. lines 1-2 of Figure 4 of that paper (build the
savings list, construct the CWS solution); the construction loop has the structure of
Figure 5 (take an edge, find the routes of its nodes, check the merging conditions,
merge) but always takes the first edge of the list instead of a random one. A route is
stored as the list of its customers; the depot at both ends is implicit.
"""

from pathlib import Path

import numpy as np

DATA_DIR = Path(__file__).resolve().parent.parent / "data"

# Vehicle capacity of each instance (Juan et al. 2011, Table 1).
CAPACITY = {
    "A-n32-k5": 100, "A-n38-k5": 100, "A-n45-k7": 100, "A-n55-k9": 100,
    "A-n60-k9": 100, "A-n61-k9": 100, "A-n65-k9": 100, "A-n80-k10": 100,
    "B-n50-k7": 100, "B-n52-k7": 100, "B-n57-k9": 100, "B-n78-k10": 100,
    "E-n22-k4": 6000, "E-n30-k3": 4500, "E-n33-k4": 8000, "E-n51-k5": 160,
    "E-n76-k7": 220, "E-n76-k10": 140, "E-n76-k14": 100,
    "F-n45-k4": 2010, "F-n72-k4": 30000, "F-n135-k7": 2210,
    "M-n101-k10": 200, "M-n121-k7": 200,
    "P-n22-k8": 3000, "P-n40-k5": 140, "P-n50-k10": 100, "P-n55-k15": 70,
    "P-n65-k10": 130, "P-n70-k10": 135, "P-n76-k4": 350, "P-n76-k5": 280,
    "P-n101-k4": 400,
}


def load_instance(name):
    """Return coordinates (n, 2), demands (n,) and vehicle capacity of an instance."""
    data = np.loadtxt(DATA_DIR / f"{name}_input_nodes.txt")
    return data[:, :2], data[:, 2], CAPACITY[name]


def distance_matrix(coords):
    """Euclidean distance between every pair of nodes, not rounded."""
    diff = coords[:, None, :] - coords[None, :, :]
    return np.sqrt((diff**2).sum(axis=-1))


def make_savings_list(dist):
    """Customer pairs (i < j) sorted by decreasing savings s(i,j) = c0i + c0j - cij."""
    i, j = np.triu_indices(len(dist), k=1)
    customers = i > 0
    i, j = i[customers], j[customers]
    savings = dist[0, i] + dist[0, j] - dist[i, j]
    order = np.argsort(-savings, kind="stable")  # ties keep the (i, j) generation order
    return i[order], j[order], savings[order]


def construct_initial_sol(demand):
    """Dummy solution: one route (0, i, 0) per customer, with its route index and load."""
    n = len(demand)
    routes = {i: [i] for i in range(1, n)}
    route_of = np.arange(n)
    load = demand.astype(float).copy()
    return routes, route_of, load


def is_exterior(node, route):
    """True if the node is adjacent to the depot, i.e. at one end of its route."""
    return route[0] == node or route[-1] == node


def check_merging_conditions(i, j, routes, route_of, load, capacity):
    """CWS conditions: different routes, both nodes exterior and capacity respected."""
    ri, rj = route_of[i], route_of[j]
    return (
        ri != rj
        and is_exterior(i, routes[ri])
        and is_exterior(j, routes[rj])
        and load[ri] + load[rj] <= capacity
    )


def merge_routes_using_edge(i, j, routes, route_of, load):
    """Join the route ending at i with the route starting at j; return the merged index."""
    ri, rj = route_of[i], route_of[j]
    route_i, route_j = routes[ri], routes.pop(rj)
    if route_i[-1] != i:
        route_i.reverse()
    if route_j[0] != j:
        route_j.reverse()
    route_i.extend(route_j)
    route_of[route_j] = ri
    load[ri] += load[rj]
    return ri


def sequential_cws(dist, demand, capacity):
    """Sequential CWS: grow one route at a time with the best feasible saving at its ends.

    The first feasible edge of the savings list seeds a route; the list is then scanned
    again and again for the first feasible edge touching the current route, until none is
    left. Edges that touch the current route and fail the conditions are deleted, since
    they can never become feasible later; edges that do not touch it are kept for the
    next routes.
    """
    i_list, j_list, _ = make_savings_list(dist)
    alive = np.ones(len(i_list), dtype=bool)
    routes, route_of, load = construct_initial_sol(demand)
    current = None
    while True:
        merged = False
        for e in np.flatnonzero(alive):
            i, j = i_list[e], j_list[e]
            if current is not None and current not in (route_of[i], route_of[j]):
                continue
            alive[e] = False
            if check_merging_conditions(i, j, routes, route_of, load, capacity):
                current = merge_routes_using_edge(i, j, routes, route_of, load)
                merged = True
                break
        if not merged:
            if current is None:
                break
            current = None  # the current route is closed: no remaining edge touches it
    return list(routes.values())


def parallel_cws(dist, demand, capacity):
    """Parallel CWS: one pass over the savings list, any two routes may be merged.

    This is the classic CWS used by Juan et al. (2011, Figure 4 line 2): all routes grow
    at the same time, and every edge of the list is tried once, in order of decreasing
    saving, merging the routes of its two nodes if the conditions hold. A discarded edge is
    never tried again, since its conditions can only get worse as routes grow.
    """
    i_list, j_list, _ = make_savings_list(dist)
    routes, route_of, load = construct_initial_sol(demand)
    for i, j in zip(i_list, j_list):
        if check_merging_conditions(i, j, routes, route_of, load, capacity):
            merge_routes_using_edge(i, j, routes, route_of, load)
    return list(routes.values())

def route_cost(route, dist):
    """Length of the closed tour depot -> route -> depot."""
    tour = [0, *route, 0]
    return dist[tour[:-1], tour[1:]].sum()


def solution_cost(routes, dist):
    """Total length of all routes."""
    return sum(route_cost(r, dist) for r in routes)


def is_feasible(routes, demand, capacity):
    """True if every customer is visited exactly once and no route exceeds the capacity."""
    visited = sorted(c for r in routes for c in r)
    loads_ok = all(demand[r].sum() <= capacity for r in routes)
    return visited == list(range(1, len(demand))) and loads_ok


if __name__ == "__main__":
    for name in CAPACITY:
        coords, demand, capacity = load_instance(name)
        dist = distance_matrix(coords)
        seq = sequential_cws(dist, demand, capacity)
        par = parallel_cws(dist, demand, capacity)
        assert is_feasible(seq, demand, capacity) and is_feasible(par, demand, capacity)
        print(f"{name:12s} sequential: {len(seq):3d} routes {solution_cost(seq, dist):9.2f}"
              f" | parallel: {len(par):3d} routes {solution_cost(par, dist):9.2f}")
