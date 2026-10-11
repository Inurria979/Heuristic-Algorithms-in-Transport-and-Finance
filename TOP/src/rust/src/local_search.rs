//! Local search applied to the elite solutions of the grid (docs/busqueda_local_diseno.md):
//! repeat { 2-opt inside every route; insert one unvisited customer } until no customer fits.
//!
//! - 2-opt (first improvement): reverse the segment v_i..v_j of a route when that shortens
//!   it. Distances are Euclidean, hence symmetric, so only the two end arcs change.
//! - Insertion: among the unvisited reachable customers that fit in some route, the one with
//!   the highest reward (ties: smallest added time), at its cheapest position in any route.
//!
//! Neither move can worsen the solution (2-opt keeps the reward and shortens; an insertion
//! raises the reward within tmax), so there is no acceptance criterion. A new route is never
//! needed: PJS keeps fewer than m routes only when they already hold every reachable customer.

use crate::model::{Instance, Route, Solution, EPS};

/// Moves done by one local search.
#[derive(Clone, Copy, Default)]
pub struct LsStats {
    pub two_opt: usize,
    pub inserted: usize,
}

/// Node before position p of the route (the source for p = 0) and node at position p (the
/// sink for p = len): the arc (prev, next) of gap p.
fn gap(inst: &Instance, nodes: &[usize], p: usize) -> (usize, usize) {
    let prev = if p == 0 { 0 } else { nodes[p - 1] };
    let next = if p == nodes.len() { inst.sink() } else { nodes[p] };
    (prev, next)
}

/// 2-opt with first improvement until no reversal shortens the route; returns the number of
/// reversals. The route time is then recomputed leg by leg.
pub fn two_opt(inst: &Instance, route: &mut Route) -> usize {
    let mut moves = 0;
    let len = route.nodes.len();
    'search: loop {
        for i in 0..len {
            for j in i + 1..len {
                let v = &route.nodes;
                let (a, d) = (gap(inst, v, i).0, gap(inst, v, j + 1).1);
                let delta = inst.t(a, v[j]) + inst.t(v[i], d) - inst.t(a, v[i]) - inst.t(v[j], d);
                if delta < -EPS {
                    route.nodes[i..=j].reverse();
                    moves += 1;
                    continue 'search;
                }
            }
        }
        break;
    }
    if moves > 0 {
        route.time = inst.route_time(&route.nodes);
    }
    moves
}

/// Candidate insertion: customer k in gap p of route r, adding `delta` time.
#[derive(Clone, Copy)]
struct Insertion {
    k: usize,
    r: usize,
    p: usize,
    delta: f64,
}

impl Insertion {
    /// Whether `self` beats `other`: higher reward, then smaller added time (the scan order
    /// k, r, p ascending settles the remaining ties, since only strict wins replace).
    fn beats(&self, other: &Insertion, inst: &Instance) -> bool {
        let (u, w) = (inst.reward[self.k], inst.reward[other.k]);
        u > w || (u == w && self.delta < other.delta)
    }
}

/// Insert the best unvisited customer that fits (see the module docs); returns whether one
/// was inserted. A position accepted by the incremental time but rejected by the exact one
/// (rounding) is banned and the search repeated.
fn insert_one(inst: &Instance, routes: &mut [Route], visited: &mut [bool]) -> bool {
    let mut banned: Vec<(usize, usize, usize)> = Vec::new();
    loop {
        let mut best: Option<Insertion> = None;
        for k in inst.customers().filter(|&k| !visited[k] && inst.reachable(k)) {
            for (r, route) in routes.iter().enumerate() {
                for p in 0..=route.nodes.len() {
                    let (a, b) = gap(inst, &route.nodes, p);
                    let delta = inst.t(a, k) + inst.t(k, b) - inst.t(a, b);
                    if route.time + delta > inst.tmax + EPS || banned.contains(&(k, r, p)) {
                        continue;
                    }
                    let cand = Insertion { k, r, p, delta };
                    if best.map_or(true, |b| cand.beats(&b, inst)) {
                        best = Some(cand);
                    }
                }
            }
        }
        let Some(ins) = best else { return false };
        let route = &mut routes[ins.r];
        let mut nodes = route.nodes.clone();
        nodes.insert(ins.p, ins.k);
        let time = inst.route_time(&nodes);
        if time > inst.tmax + EPS {
            banned.push((ins.k, ins.r, ins.p));
            continue;
        }
        route.reward = nodes.iter().map(|&v| inst.reward[v]).sum();
        route.nodes = nodes;
        route.time = time;
        visited[ins.k] = true;
        return true;
    }
}

/// Local search on a copy of `sol`: 2-opt on every route, then one insertion, repeated until
/// no customer can be inserted.
pub fn local_search(inst: &Instance, sol: &Solution) -> (Solution, LsStats) {
    let mut routes = sol.routes.clone();
    let mut visited = vec![false; inst.n()];
    for r in &routes {
        for &k in &r.nodes {
            visited[k] = true;
        }
    }
    let mut stats = LsStats::default();
    loop {
        for r in routes.iter_mut() {
            stats.two_opt += two_opt(inst, r);
        }
        if !insert_one(inst, &mut routes, &mut visited) {
            break;
        }
        stats.inserted += 1;
    }
    (Solution::new(routes), stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Instance from its text (written to a temporary file, since Instance::load reads one).
    fn instance(name: &str, text: &str) -> Instance {
        let path = std::env::temp_dir().join(format!("{name}.txt"));
        fs::write(&path, text).unwrap();
        Instance::load(&path).unwrap()
    }

    fn route(inst: &Instance, nodes: &[usize]) -> Route {
        Route { nodes: nodes.to_vec(), time: inst.route_time(nodes), reward: nodes.iter().map(|&v| inst.reward[v]).sum() }
    }

    #[test]
    fn two_opt_removes_a_crossing() {
        // Source (0,0), sink (3,0); customers at (1,1), (2,1), (1,-1), (2,-1). The route
        // 0 -> (1,1) -> (2,-1) -> (1,-1) -> (2,1) -> sink zigzags; 2-opt must shorten it.
        let inst = instance("ls_cross", "n;6\nm;1\ntmax;100\n0;0;0\n1;1;1\n2;1;1\n1;-1;1\n2;-1;1\n3;0;0\n");
        let mut r = route(&inst, &[1, 4, 3, 2]);
        let before = r.time;
        assert!(two_opt(&inst, &mut r) > 0);
        assert!(r.time < before - 1e-9);
        assert!((r.time - inst.route_time(&r.nodes)).abs() < 1e-12);
        // Already 2-optimal: nothing more to do.
        assert_eq!(two_opt(&inst, &mut r), 0);
    }

    #[test]
    fn inserts_highest_reward_that_fits_then_stops() {
        // Line from source (0,0) to sink (10,0), tmax 10.5, route 0 -> (5,0) -> sink (time
        // 10). Customers: (5,0.1) u=5 (detour ~0.1, fits), (5,3) u=9 (detour ~3.8, does not).
        let inst = instance("ls_insert", "n;5\nm;1\ntmax;10.5\n0;0;0\n5;0;1\n5;0.1;5\n5;3;9\n10;0;0\n");
        let sol = Solution::new(vec![route(&inst, &[1])]);
        let (after, st) = local_search(&inst, &sol);
        after.check(&inst).unwrap();
        assert_eq!(st.inserted, 1);
        assert_eq!(after.reward, 6.0);
        assert!(after.routes[0].nodes.contains(&2));
        assert!(!after.routes[0].nodes.contains(&3));
    }

    #[test]
    fn reward_tie_goes_to_the_cheaper_insertion() {
        // Two customers with the same reward, each fits alone but not both; the cheaper one
        // must be chosen even though it has the higher index.
        let inst = instance("ls_tie", "n;4\nm;1\ntmax;11\n0;0;0\n5;2;3\n5;0.5;3\n10;0;0\n");
        let start = Solution::new(vec![Route { nodes: vec![], time: inst.t(0, 3), reward: 0.0 }]);
        let (after, st) = local_search(&inst, &start);
        assert_eq!(st.inserted, 1);
        assert_eq!(after.routes[0].nodes, vec![2]);
    }

    #[test]
    fn never_worse_and_idempotent() {
        let inst = instance("ls_idem", "n;6\nm;2\ntmax;7\n0;0;0\n1;1;4\n2;1;2\n1;-1;3\n2;-1;5\n3;0;0\n");
        let sol = Solution::new(vec![route(&inst, &[1])]);
        let (a, _) = local_search(&inst, &sol);
        a.check(&inst).unwrap();
        assert!(a.reward >= sol.reward);
        let (b, st) = local_search(&inst, &a);
        assert_eq!((st.two_opt, st.inserted), (0, 0));
        assert_eq!(b.reward, a.reward);
    }
}
