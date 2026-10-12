//! Elite solutions of the grid and the local search on them (docs/busqueda_local_diseno.md).
//!
//! Per instance, the elite are:
//! - deterministic: the solutions of the alphas with the 3 highest rewards;
//! - for every randomisation and alpha: the best solutions of the 3 best GRASP runs;
//!
//! with ties at the third reward all included (reward is the only criterion). They are
//! visited in that order (deterministic, geometric, softmax; alpha ascending; run ascending)
//! and a solution that visits the same set of customers as an earlier one is dropped, with no
//! replacement. Every remaining elite goes through the local search.

use crate::alpha_grid::{grid_alpha, GridConfig, GridResult, ALPHAS};
use crate::grasp::Randomization;
use crate::local_search::{local_search, LsStats};
use crate::model::{Instance, Solution};
use rayon::prelude::*;
use std::collections::HashSet;
use std::time::Instant;

/// Group an elite solution comes from.
#[derive(Clone, Copy, PartialEq)]
pub enum Origin {
    Deterministic,
    /// GRASP with randomisation m (index into `GridConfig::methods`) and run r.
    Grasp { m: usize, r: usize },
}

/// One elite solution before and after the local search.
pub struct Elite {
    pub origin: Origin,
    /// Index of alpha in the grid.
    pub k: usize,
    pub before: Solution,
    pub after: Solution,
    pub stats: LsStats,
}

/// Elite solutions of one instance.
pub struct InstanceElite {
    /// Elite selected before dropping the duplicates.
    pub selected: usize,
    /// The elite left, in visiting order.
    pub elite: Vec<Elite>,
}

impl InstanceElite {
    /// Index of the best elite after the local search (the first one on ties).
    pub fn best(&self) -> usize {
        (0..self.elite.len()).fold(0, |b, i| if self.elite[i].after.reward > self.elite[b].after.reward { i } else { b })
    }

    /// Highest reward after (or before) the local search over the elite that satisfy `keep`.
    pub fn max_reward(&self, after: bool, keep: impl Fn(&Elite) -> bool) -> f64 {
        self.elite
            .iter()
            .filter(|e| keep(e))
            .map(|e| if after { e.after.reward } else { e.before.reward })
            .fold(0.0, f64::max)
    }
}

/// Local search on the elite of every instance, with its wall-clock time.
pub struct LsResult {
    pub instances: Vec<InstanceElite>,
    pub seconds: f64,
}

/// Indices of the candidates whose reward reaches the third highest one (all of them if
/// there are at most 3), in their original order.
pub fn top_with_ties(rewards: &[f64]) -> Vec<usize> {
    let mut sorted = rewards.to_vec();
    sorted.sort_by(|a, b| b.total_cmp(a));
    let threshold = sorted.get(2).copied().unwrap_or(f64::NEG_INFINITY);
    (0..rewards.len()).filter(|&i| rewards[i] >= threshold).collect()
}

/// Set of customers a solution visits, as a sorted list (the key for duplicates).
fn customer_set(sol: &Solution) -> Vec<usize> {
    let mut v: Vec<usize> = sol.routes.iter().flat_map(|r| r.nodes.iter().copied()).collect();
    v.sort_unstable();
    v
}

/// Candidate elite solution: its group, its alpha index and the solution.
pub type Candidate<'a> = (Origin, usize, &'a Solution);

/// Drop the candidates that visit the same set of customers as an earlier one (no
/// replacement); returns how many there were before.
pub fn drop_duplicates(list: &mut Vec<Candidate>) -> usize {
    let selected = list.len();
    let mut seen = HashSet::new();
    list.retain(|(_, _, sol)| seen.insert(customer_set(sol)));
    selected
}

/// Elite of instance f of the grid in visiting order, without duplicates: returns the number
/// selected and the ones kept.
fn select_elite<'a>(res: &'a GridResult, f: usize, cfg: &GridConfig) -> (usize, Vec<Candidate<'a>>) {
    let mut list: Vec<Candidate> = Vec::new();
    let det = &res.det_solutions[f * ALPHAS..(f + 1) * ALPHAS];
    for k in top_with_ties(&det.iter().map(|s| s.reward).collect::<Vec<_>>()) {
        list.push((Origin::Deterministic, k, &det[k]));
    }
    for m in 0..2 {
        for k in 0..ALPHAS {
            let start = ((f * ALPHAS + k) * 2 + m) * cfg.reps;
            let runs = &res.grasp_solutions[start..start + cfg.reps];
            for r in top_with_ties(&runs.iter().map(|s| s.reward).collect::<Vec<_>>()) {
                list.push((Origin::Grasp { m, r }, k, &runs[r]));
            }
        }
    }
    let selected = drop_duplicates(&mut list);
    (selected, list)
}

/// Local search on the elite of the grid, every instance.
pub fn run_local_search(instances: &[Instance], res: &GridResult, cfg: &GridConfig) -> Result<LsResult, String> {
    search_all(instances, (0..instances.len()).map(|f| select_elite(res, f, cfg)).collect())
}

/// Local search on the chosen elite of every instance (`chosen[f]` = number selected and the
/// candidates kept), all of them in one parallel list. Every solution is checked after it.
pub fn search_all(instances: &[Instance], chosen: Vec<(usize, Vec<Candidate>)>) -> Result<LsResult, String> {
    let start = Instant::now();
    let jobs: Vec<(usize, Candidate)> = chosen.iter().enumerate().flat_map(|(f, (_, list))| list.iter().map(move |&c| (f, c))).collect();
    let done: Vec<Elite> = jobs
        .par_iter()
        .map(|&(f, (origin, k, sol))| {
            let inst = &instances[f];
            let (after, stats) = local_search(inst, sol);
            after.check(inst).map_err(|e| format!("{} alpha {} local search: {e}", inst.name, grid_alpha(k)))?;
            Ok(Elite { origin, k, before: sol.clone(), after, stats })
        })
        .collect::<Result<_, String>>()?;
    let mut done = done.into_iter();
    let instances = chosen.iter().map(|(selected, list)| InstanceElite { selected: *selected, elite: done.by_ref().take(list.len()).collect() }).collect();
    Ok(LsResult { instances, seconds: start.elapsed().as_secs_f64() })
}

/// Name of an origin ("deterministic", "geometric", "softmax") and its run ("" if none).
pub fn origin_text(origin: Origin, methods: &[Randomization; 2]) -> (&'static str, String) {
    match origin {
        Origin::Deterministic => ("deterministic", String::new()),
        Origin::Grasp { m, r } => (methods[m].name(), r.to_string()),
    }
}

/// local_search.csv: one row per elite solution (duplicates already dropped).
pub fn local_search_csv(instances: &[Instance], ls: &LsResult, cfg: &GridConfig) -> String {
    let mut csv = String::from("instance,origin,k,alpha,rep,reward_before,time_before,reward_after,time_after,two_opt_moves,inserted\n");
    for (inst, ie) in instances.iter().zip(&ls.instances) {
        for e in &ie.elite {
            let (origin, rep) = origin_text(e.origin, &cfg.methods);
            csv += &format!(
                "{},{origin},{},{:.2},{rep},{},{:.6},{},{:.6},{},{}\n",
                inst.name,
                e.k,
                grid_alpha(e.k),
                e.before.reward,
                e.before.time,
                e.after.reward,
                e.after.time,
                e.stats.two_opt,
                e.stats.inserted
            );
        }
    }
    csv
}

/// Best reward of any GRASP run of instance f (no local search). It is read from the grid,
/// not from the elite: a GRASP run that repeats a deterministic elite was dropped there.
pub fn grasp_best(res: &GridResult, f: usize, cfg: &GridConfig) -> f64 {
    let n = ALPHAS * 2 * cfg.reps;
    res.grasp[f * n..(f + 1) * n].iter().map(|g| g.reward).fold(0.0, f64::max)
}

/// best_solution.csv: one row per instance with the best deterministic and GRASP rewards of
/// the grid (no local search), the best deterministic elite after the local search and the
/// best solution overall, with the elite it comes from.
pub fn best_solution_csv(instances: &[Instance], res: &GridResult, ls: &LsResult, cfg: &GridConfig) -> String {
    let mut csv = String::from(
        "instance,m,tmax,deterministic,grasp,best_no_ls,deterministic_ls,best_ls,origin,k,alpha,rep,reward_before_ls,inserted,routes,visited,total_time,elite_selected,elite_kept\n",
    );
    for (f, (inst, ie)) in instances.iter().zip(&ls.instances).enumerate() {
        let det = ie.max_reward(false, |e| e.origin == Origin::Deterministic);
        let grasp = grasp_best(res, f, cfg);
        let det_ls = ie.max_reward(true, |e| e.origin == Origin::Deterministic);
        let b = &ie.elite[ie.best()];
        let (origin, rep) = origin_text(b.origin, &cfg.methods);
        csv += &format!(
            "{},{},{},{det},{grasp},{},{det_ls},{},{origin},{},{:.2},{rep},{},{},{},{},{:.6},{},{}\n",
            inst.name,
            inst.m,
            inst.tmax,
            det.max(grasp),
            b.after.reward,
            b.k,
            grid_alpha(b.k),
            b.before.reward,
            b.stats.inserted,
            b.after.routes.len(),
            b.after.visited(),
            b.after.time,
            ie.selected,
            ie.elite.len()
        );
    }
    csv
}

/// best_routes.csv: the routes of the best solution of every instance, one row per route,
/// for an outside check (src/bin/verify_routes.rs). `nodes` are the customers in visiting
/// order, as 0-based node indices of the instance file separated by spaces; the source (0)
/// and the sink (last node) are implicit. Times use the shortest exact representation.
pub fn best_routes_csv(instances: &[Instance], ls: &LsResult) -> String {
    let mut csv = String::from("instance,route,reward,time,nodes\n");
    for (inst, ie) in instances.iter().zip(&ls.instances) {
        for (i, r) in ie.elite[ie.best()].after.routes.iter().enumerate() {
            let nodes: Vec<String> = r.nodes.iter().map(|k| k.to_string()).collect();
            csv += &format!("{},{},{},{},{}\n", inst.name, i + 1, r.reward, r.time, nodes.join(" "));
        }
    }
    csv
}

#[cfg(test)]
mod tests {
    use super::top_with_ties;

    #[test]
    fn top_three_with_ties() {
        assert_eq!(top_with_ties(&[80.0, 100.0, 90.0, 95.0]), vec![1, 2, 3]);
        assert_eq!(top_with_ties(&[100.0, 100.0, 90.0, 100.0, 100.0]), vec![0, 1, 3, 4]);
        assert_eq!(top_with_ties(&[90.0, 100.0, 80.0, 95.0, 90.0]), vec![0, 1, 3, 4]);
        assert_eq!(top_with_ties(&[100.0, 90.0]), vec![0, 1]);
    }
}
