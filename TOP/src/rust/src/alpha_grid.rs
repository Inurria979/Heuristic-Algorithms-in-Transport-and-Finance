//! Alpha grid experiment. The paper gives no value for alpha, so every instance is solved
//! with the 21 values alpha = 0, 0.05, 0.10, ..., 0.95, 1. For every (instance, alpha) pair there
//! is one deterministic construction and `reps` independent GRASP runs with each
//! randomisation (geometric and softmax), every run with its own seed. All tasks of all
//! instances form one flat list solved in parallel with rayon.
//!
//! The best deterministic alpha of an instance is the one with the highest total reward
//! (route times only have to respect tmax); when several alphas reach it they are all
//! optimal, and the smallest one is reported together with how many there are.

use crate::grasp::{grasp, stream_seed, GraspConfig, Randomization};
use crate::heuristic::pjs;
use crate::model::Instance;
use rayon::prelude::*;
use std::time::Instant;

/// Number of values of the alpha grid.
pub const ALPHAS: usize = 21;

/// k-th value of the alpha grid: 0 (k = 0), 0.05, ..., 1 (k = 20).
pub fn grid_alpha(k: usize) -> f64 {
    k as f64 / 20.0
}

/// Description of the grid for texts.
pub const GRID_TEXT: &str = "0, 0.05, 0.10, ..., 0.95, 1";

/// GRASP runs done for every alpha of the grid.
#[derive(Clone, Copy)]
pub struct GridConfig {
    /// Independent GRASP runs per alpha and randomisation (0 = deterministic only).
    pub reps: usize,
    /// Randomised constructions of every GRASP run.
    pub iterations: usize,
    /// The two randomisations: [geometric, softmax].
    pub methods: [Randomization; 2],
    pub seed: u64,
}

/// One GRASP run of the grid, as written to grasp_runs.csv.
#[derive(Clone, Copy)]
pub struct GraspRun {
    pub seed: u64,
    pub reward: f64,
    pub best_iteration: usize,
    pub iterations: usize,
    pub seconds: f64,
}

/// Summary of the rewards of the GRASP runs of one (instance, alpha, randomisation).
#[derive(Clone, Copy, Default)]
pub struct RepStats {
    pub best: f64,
    pub mean: f64,
    pub std: f64,
    pub worst: f64,
}

impl RepStats {
    /// Best, mean, population standard deviation and worst of `v` (zeros if empty).
    fn of(v: &[f64]) -> RepStats {
        if v.is_empty() {
            return RepStats::default();
        }
        let n = v.len() as f64;
        let mean = v.iter().sum::<f64>() / n;
        let std = (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n).sqrt();
        let best = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let worst = v.iter().copied().fold(f64::INFINITY, f64::min);
        RepStats { best, mean, std, worst }
    }
}

/// Result of one (instance, alpha) pair: the deterministic construction and the summary
/// of the GRASP runs of each randomisation ([geometric, softmax]).
#[derive(Clone, Copy)]
pub struct Run {
    pub reward: f64,
    pub time: f64,
    pub routes: usize,
    pub visited: usize,
    pub grasp: [RepStats; 2],
}

/// Best deterministic alpha of the grid for one instance and how the grid behaves around it.
pub struct Best {
    pub k: usize,
    pub optimal: usize,
    pub alpha_lo: f64,
    pub alpha_hi: f64,
    pub worst: f64,
}

/// Everything the grid experiment produces: `runs[f * ALPHAS + k]` is instance f with
/// alpha_k, and `grasp[((f * ALPHAS + k) * 2 + m) * reps + r]` is its GRASP run r with
/// randomisation m.
pub struct GridResult {
    pub runs: Vec<Run>,
    pub grasp: Vec<GraspRun>,
    pub seconds: f64,
}

/// Outcome of one task of the flat list.
enum Task {
    Deterministic { reward: f64, time: f64, routes: usize, visited: usize },
    Grasp(GraspRun),
}

/// Task t of an (instance, alpha) pair: t = 0 is the deterministic construction, t = 1 +
/// m * reps + r is GRASP run r with randomisation m. Every solution is checked.
fn run_task(inst: &Instance, k: usize, t: usize, cfg: &GridConfig) -> Result<Task, String> {
    let alpha = grid_alpha(k);
    let fail = |what: &str, e: String| format!("{} alpha {alpha} {what}: {e}", inst.name);
    if t == 0 {
        let (sol, _) = pjs(inst, alpha);
        sol.check(inst).map_err(|e| fail("deterministic", e))?;
        return Ok(Task::Deterministic { reward: sol.reward, time: sol.time, routes: sol.routes.len(), visited: sol.visited() });
    }
    let (m, r) = ((t - 1) / cfg.reps, (t - 1) % cfg.reps);
    let method = cfg.methods[m];
    let seed = stream_seed(cfg.seed, &inst.name, &[k as u64, m as u64, r as u64]);
    let gcfg = GraspConfig { alpha, randomization: method, iterations: cfg.iterations, time_limit: None };
    let res = grasp(inst, &gcfg, seed);
    res.solution.check(inst).map_err(|e| fail(method.name(), e))?;
    Ok(Task::Grasp(GraspRun {
        seed,
        reward: res.solution.reward,
        best_iteration: res.best_iteration,
        iterations: res.iterations,
        seconds: res.seconds,
    }))
}

/// The grid experiment on every instance: all (instance, alpha, task) triples in one flat
/// parallel list, then grouped back per (instance, alpha).
pub fn solve_all(instances: &[Instance], cfg: &GridConfig) -> Result<GridResult, String> {
    let tasks = 1 + 2 * cfg.reps;
    let start = Instant::now();
    let out: Vec<Task> = (0..instances.len() * ALPHAS * tasks)
        .into_par_iter()
        .map(|job| {
            let (pair, t) = (job / tasks, job % tasks);
            run_task(&instances[pair / ALPHAS], pair % ALPHAS, t, cfg)
        })
        .collect::<Result<_, String>>()?;
    let seconds = start.elapsed().as_secs_f64();

    let mut runs = Vec::with_capacity(instances.len() * ALPHAS);
    let mut grasp_runs = Vec::with_capacity(instances.len() * ALPHAS * 2 * cfg.reps);
    for pair in out.chunks(tasks) {
        let Task::Deterministic { reward, time, routes, visited } = pair[0] else { unreachable!() };
        let g: Vec<GraspRun> = pair[1..]
            .iter()
            .map(|t| match t {
                Task::Grasp(g) => *g,
                Task::Deterministic { .. } => unreachable!(),
            })
            .collect();
        let stats = |m: usize| RepStats::of(&g[m * cfg.reps..(m + 1) * cfg.reps].iter().map(|x| x.reward).collect::<Vec<_>>());
        runs.push(Run { reward, time, routes, visited, grasp: [stats(0), stats(1)] });
        grasp_runs.extend(g);
    }
    Ok(GridResult { runs, grasp: grasp_runs, seconds })
}

/// Smallest alpha of the grid with the highest deterministic reward on one instance,
/// solved sequentially (for callers that already run instances in parallel).
pub fn best_grid_alpha(inst: &Instance) -> Result<f64, String> {
    let rewards: Vec<f64> = (0..ALPHAS).map(|k| pjs(inst, grid_alpha(k)).0.reward).collect();
    let best = rewards.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Ok(grid_alpha(rewards.iter().position(|&r| r == best).unwrap()))
}

/// Best deterministic alpha from the runs of one instance (runs[k] is alpha_k): only the
/// total reward counts. Every alpha that reaches the highest reward is optimal; the
/// smallest one is reported, with how many there are and the range they span (they need
/// not be contiguous).
pub fn best_alpha(runs: &[Run]) -> Best {
    let best = runs.iter().map(|r| r.reward).fold(f64::NEG_INFINITY, f64::max);
    let optimal: Vec<usize> = (0..runs.len()).filter(|&k| runs[k].reward == best).collect();
    let worst = runs.iter().map(|r| r.reward).fold(f64::INFINITY, f64::min);
    Best {
        k: optimal[0],
        optimal: optimal.len(),
        alpha_lo: grid_alpha(optimal[0]),
        alpha_hi: grid_alpha(*optimal.last().unwrap()),
        worst,
    }
}

/// For randomisation m on one instance (`row` = its runs): the best reward of any GRASP run
/// over the grid with the smallest alpha that reaches it, and the highest mean over the
/// runs of one alpha with that alpha.
pub fn best_grasp(row: &[Run], m: usize) -> (f64, f64, f64, f64) {
    let pick = |v: &dyn Fn(&Run) -> f64| {
        let best = row.iter().map(v).fold(f64::NEG_INFINITY, f64::max);
        (best, grid_alpha(row.iter().position(|r| v(r) == best).unwrap()))
    };
    let (best, best_a) = pick(&|r: &Run| r.grasp[m].best);
    let (mean, mean_a) = pick(&|r: &Run| r.grasp[m].mean);
    (best, best_a, mean, mean_a)
}

/// alpha_grid.csv: one row per instance and alpha, with the summary of its GRASP runs.
pub fn grid_csv(instances: &[Instance], res: &GridResult, cfg: &GridConfig) -> String {
    let mut csv = String::from("instance,k,alpha,reward,total_time,routes,visited");
    if cfg.reps > 0 {
        for m in cfg.methods {
            let n = m.name();
            csv += &format!(",{n}_best,{n}_mean,{n}_std,{n}_worst");
        }
    }
    csv.push('\n');
    for (inst, row) in instances.iter().zip(res.runs.chunks(ALPHAS)) {
        for (k, r) in row.iter().enumerate() {
            csv += &format!("{},{k},{:.2},{},{:.6},{},{}", inst.name, grid_alpha(k), r.reward, r.time, r.routes, r.visited);
            if cfg.reps > 0 {
                for s in r.grasp {
                    csv += &format!(",{},{:.4},{:.4},{}", s.best, s.mean, s.std, s.worst);
                }
            }
            csv.push('\n');
        }
    }
    csv
}

/// grasp_runs.csv: one row per GRASP run of the grid.
pub fn grasp_runs_csv(instances: &[Instance], res: &GridResult, cfg: &GridConfig) -> String {
    let mut csv = String::from("instance,k,alpha,randomization,param,iterations,rep,seed,reward,best_iteration,seconds\n");
    if cfg.reps == 0 {
        return csv;
    }
    for (p, runs) in res.grasp.chunks(2 * cfg.reps).enumerate() {
        let (inst, k) = (&instances[p / ALPHAS], p % ALPHAS);
        for (i, g) in runs.iter().enumerate() {
            let method = cfg.methods[i / cfg.reps];
            csv += &format!(
                "{},{k},{:.2},{},{},{},{},{},{},{},{:.4}\n",
                inst.name,
                grid_alpha(k),
                method.name(),
                method.param(),
                g.iterations,
                i % cfg.reps,
                g.seed,
                g.reward,
                g.best_iteration,
                g.seconds
            );
        }
    }
    csv
}

/// best_alpha.csv: one row per instance with its best deterministic alpha and, for each
/// randomisation, the best GRASP reward over the grid and the best mean of one alpha.
pub fn best_csv(instances: &[Instance], res: &GridResult, bests: &[Best], cfg: &GridConfig) -> String {
    let mut csv = String::from(
        "instance,customers,m,tmax,reachable,best_k,best_alpha,optimal_alphas,alpha_lo,alpha_hi,reward,worst_reward,total_time,routes,visited",
    );
    if cfg.reps > 0 {
        for m in cfg.methods {
            let n = m.name();
            csv += &format!(",{n}_best_reward,{n}_best_alpha,{n}_best_mean,{n}_best_mean_alpha");
        }
    }
    csv.push('\n');
    for ((inst, row), best) in instances.iter().zip(res.runs.chunks(ALPHAS)).zip(bests) {
        let r = row[best.k];
        csv += &format!(
            "{},{},{},{},{},{},{:.2},{},{:.2},{:.2},{},{},{:.6},{},{}",
            inst.name,
            inst.n() - 2,
            inst.m,
            inst.tmax,
            inst.reachable_count(),
            best.k,
            grid_alpha(best.k),
            best.optimal,
            best.alpha_lo,
            best.alpha_hi,
            r.reward,
            best.worst,
            r.time,
            r.routes,
            r.visited
        );
        if cfg.reps > 0 {
            for m in 0..2 {
                let (b, ba, mean, ma) = best_grasp(row, m);
                csv += &format!(",{b},{ba:.2},{mean:.4},{ma:.2}");
            }
        }
        csv.push('\n');
    }
    csv
}

/// Mean wall-clock time of one deterministic run in microseconds (repeated for at least
/// 20 ms).
pub fn mean_time_us(inst: &Instance, alpha: f64) -> f64 {
    let start = Instant::now();
    let mut runs = 0;
    while runs == 0 || start.elapsed().as_millis() < 20 {
        std::hint::black_box(pjs(inst, alpha));
        runs += 1;
    }
    start.elapsed().as_secs_f64() * 1e6 / runs as f64
}
