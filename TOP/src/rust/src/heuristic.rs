//! Savings-based constructive heuristic for the TOP, as described in Panadero et al. (2021),
//! Section 4 "From a Heuristic to a Biased-Randomized Algorithm":
//!
//! 1. Dummy solution: one route source -> i -> sink for every customer i. A customer whose
//!    dummy route already exceeds the driving range tmax can never be served and is dropped.
//! 2. Enriched savings: for every arc (i, j) between two customers,
//!        s_ij  = t_{i,sink} + t_{source,j} - t_ij          (time saved by merging)
//!        s'_ij = alpha * s_ij + (1 - alpha) * (u_i + u_j)  (enriched with the rewards)
//!    and the arcs are sorted by decreasing enriched savings.
//! 3. Merging: an arc is taken from the list; arc (i, j) joins the route that ends in i with
//!    the route that starts in j (as ... -> i -> j -> ...) if the merged route does not exceed
//!    tmax. The arc is then removed from the list, whether it was used or not.
//! 4. Selection: the routes are sorted by decreasing reward and the best m are kept.
//!
//! Source and sink are different nodes (except in set p7), so s_ij != s_ji: both directions
//! of every pair are put in the savings list, as in Li et al. (2022).
//!
//! Which arc is taken in step 3 is decided by an [`ArcSelector`]: [`Greedy`] always takes
//! the first one left (the deterministic heuristic); randomised selectors (crate::grasp)
//! plug into [`construct`] without touching the rest.

use crate::model::{Instance, Route, Solution, EPS};

/// Arc between two customers with its enriched savings.
pub struct Arc {
    pub i: usize,
    pub j: usize,
    pub saving: f64,
}

/// Counters of each step of one construction, for printouts.
pub struct Stats {
    pub reachable: usize,
    pub arcs: usize,
    pub merges: usize,
    pub routes_built: usize,
}

/// Rule that decides in which order the arcs of the savings list are taken.
pub trait ArcSelector {
    /// Start a new construction on a savings list sorted by decreasing savings (`savings[i]`
    /// is the enriched savings of the i-th arc).
    fn start(&mut self, savings: &[f64]);

    /// Index in that list of the next arc. It is called once per arc and must return every
    /// index exactly once (the arc is removed from the list once taken).
    fn next(&mut self) -> usize;
}

/// Deterministic heuristic: always the arc with the highest savings left.
#[derive(Default)]
pub struct Greedy {
    taken: usize,
}

impl ArcSelector for Greedy {
    fn start(&mut self, _savings: &[f64]) {
        self.taken = 0;
    }

    fn next(&mut self) -> usize {
        self.taken += 1;
        self.taken - 1
    }
}

/// Step 1: one route source -> i -> sink per reachable customer. Returns the routes and,
/// for every node, the index of the route that holds it (None for unreachable customers).
pub fn dummy_solution(inst: &Instance) -> (Vec<Option<Route>>, Vec<Option<usize>>) {
    let mut routes = Vec::new();
    let mut route_of = vec![None; inst.n()];
    for i in inst.customers() {
        let route = Route::single(inst, i);
        if route.time <= inst.tmax + EPS {
            route_of[i] = Some(routes.len());
            routes.push(Some(route));
        }
    }
    (routes, route_of)
}

/// Step 2: enriched savings of every arc between two reachable customers, sorted from
/// highest to lowest (stable sort, so ties keep the generation order). It depends only on
/// the instance and alpha, so repeated constructions (GRASP) compute it once.
pub struct SavingsList {
    pub arcs: Vec<Arc>,
    pub savings: Vec<f64>,
}

impl SavingsList {
    pub fn new(inst: &Instance, alpha: f64) -> SavingsList {
        let arcs = savings_arcs(inst, alpha);
        let savings = arcs.iter().map(|a| a.saving).collect();
        SavingsList { arcs, savings }
    }
}

/// Arcs of the savings list, sorted by decreasing enriched savings.
fn savings_arcs(inst: &Instance, alpha: f64) -> Vec<Arc> {
    let sink = inst.sink();
    let customers: Vec<usize> = inst.customers().filter(|&i| inst.reachable(i)).collect();
    let enriched = |i: usize, j: usize| {
        let time_saving = inst.t(i, sink) + inst.t(0, j) - inst.t(i, j);
        alpha * time_saving + (1.0 - alpha) * (inst.reward[i] + inst.reward[j])
    };
    let mut arcs = Vec::with_capacity(customers.len() * customers.len());
    for (a, &i) in customers.iter().enumerate() {
        for &j in &customers[a + 1..] {
            arcs.push(Arc { i, j, saving: enriched(i, j) });
            arcs.push(Arc { i: j, j: i, saving: enriched(j, i) });
        }
    }
    arcs.sort_by(|a, b| b.saving.total_cmp(&a.saving));
    arcs
}

/// Step 3, one arc: merge the route ending in i with the route starting in j if the result
/// respects the driving range. Returns whether the merge was done.
fn try_merge(arc: &Arc, inst: &Instance, routes: &mut [Option<Route>], route_of: &mut [Option<usize>]) -> bool {
    let (Some(a), Some(b)) = (route_of[arc.i], route_of[arc.j]) else {
        return false;
    };
    if a == b {
        return false;
    }
    let (ra, rb) = (routes[a].as_ref().unwrap(), routes[b].as_ref().unwrap());
    if *ra.nodes.last().unwrap() != arc.i || rb.nodes[0] != arc.j {
        return false;
    }
    // Cheap filter: the merged time by difference, without building the route. Only merges
    // that clearly fit go on to the exact time, summed leg by leg as everywhere else, which
    // takes the final decision.
    let estimate = ra.time + rb.time - inst.t(arc.i, inst.sink()) - inst.t(0, arc.j) + inst.t(arc.i, arc.j);
    if estimate > inst.tmax + 1e-6 {
        return false;
    }
    let nodes: Vec<usize> = ra.nodes.iter().chain(&rb.nodes).copied().collect();
    let time = inst.route_time(&nodes);
    if time > inst.tmax + EPS {
        return false;
    }
    let reward = ra.reward + rb.reward;
    for &k in &rb.nodes {
        route_of[k] = Some(a);
    }
    routes[a] = Some(Route { nodes, time, reward });
    routes[b] = None;
    true
}

/// Step 4: sort the routes by decreasing reward (ties: shorter first) and keep the best m.
pub fn select_routes(routes: Vec<Option<Route>>, m: usize) -> Vec<Route> {
    let mut routes: Vec<Route> = routes.into_iter().flatten().collect();
    routes.sort_by(|a, b| b.reward.total_cmp(&a.reward).then(a.time.total_cmp(&b.time)));
    routes.truncate(m);
    routes
}

/// Savings construction on a precomputed savings list, taking the arcs in the order chosen
/// by `selector`: dummy solution, merging, selection of m routes.
pub fn construct(inst: &Instance, list: &SavingsList, selector: &mut impl ArcSelector) -> (Solution, Stats) {
    let (mut routes, mut route_of) = dummy_solution(inst);
    let reachable = routes.len();
    selector.start(&list.savings);
    let mut merges = 0;
    for _ in 0..list.arcs.len() {
        if try_merge(&list.arcs[selector.next()], inst, &mut routes, &mut route_of) {
            merges += 1;
        }
    }
    let routes_built = routes.iter().flatten().count();
    let solution = Solution::new(select_routes(routes, inst.m));
    (solution, Stats { reachable, arcs: list.arcs.len(), merges, routes_built })
}

/// Deterministic PJS heuristic: the construction with the greedy selector.
pub fn pjs(inst: &Instance, alpha: f64) -> (Solution, Stats) {
    construct(inst, &SavingsList::new(inst, alpha), &mut Greedy::default())
}
