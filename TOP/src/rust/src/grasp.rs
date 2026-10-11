//! GRASP construction phase: the savings construction of crate::heuristic with a randomised
//! choice of the next arc, repeated in a multi-start loop that keeps the best solution
//! (Li et al. 2022, Algorithm 1: the deterministic heuristic gives the initial solution and
//! randomised constructions follow until the iteration or time limit). Independent runs
//! with different seeds can be solved in parallel and the best one kept, as in Panadero et
//! al. (2021, Algorithm 1). No local search yet.
//!
//! Randomisation of the arc choice:
//! - Geometric (biased randomisation, Panadero et al. 2021): the next arc is the k-th one
//!   left in the list, counted from the top, with k ~ Geom(beta), P(k) = beta (1 - beta)^k,
//!   drawn by inversion as floor(ln U / ln(1 - beta)) and taken modulo the number of arcs
//!   left. The paper uses beta = 0.3; beta -> 1 is the greedy heuristic.
//! - Softmax: the next arc is drawn with probability softmax(s'/tau) over the enriched
//!   savings s' of the arcs left, P(arc) = exp(s'/tau) / sum exp(s'/tau). tau = 1 is the
//!   plain softmax of the savings; tau -> 0 is the greedy heuristic, large tau is uniform.

use crate::heuristic::{construct, ArcSelector, Greedy, SavingsList};
use crate::model::{Instance, Solution};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use rayon::prelude::*;
use std::time::Instant;

/// How the next arc of the savings list is drawn.
#[derive(Clone, Copy)]
pub enum Randomization {
    /// Position from the top ~ Geom(beta).
    Geometric { beta: f64 },
    /// Probability softmax(savings / tau) over the arcs left.
    Softmax { tau: f64 },
}

impl Randomization {
    /// Short name used in the output and the CSV ("geometric", "softmax").
    pub fn name(self) -> &'static str {
        match self {
            Randomization::Geometric { .. } => "geometric",
            Randomization::Softmax { .. } => "softmax",
        }
    }

    /// Value of its parameter (beta or tau).
    pub fn param(self) -> f64 {
        match self {
            Randomization::Geometric { beta } => beta,
            Randomization::Softmax { tau } => tau,
        }
    }

    /// Description for printouts and plots.
    pub fn describe(self) -> String {
        match self {
            Randomization::Geometric { beta } => format!("geometric, beta = {beta}"),
            Randomization::Softmax { tau } => format!("softmax, tau = {tau}"),
        }
    }
}

/// Settings of one GRASP run.
#[derive(Clone, Copy)]
pub struct GraspConfig {
    pub alpha: f64,
    pub randomization: Randomization,
    /// Randomised constructions after the initial deterministic one.
    pub iterations: usize,
    /// Optional wall-clock limit in seconds (the run stops at whichever limit comes first).
    pub time_limit: Option<f64>,
}

/// Outcome of one GRASP run.
pub struct GraspResult {
    pub solution: Solution,
    /// Reward of the deterministic heuristic with the same alpha (the initial solution).
    pub initial_reward: f64,
    /// Iteration that found the best solution (0 = the deterministic one).
    pub best_iteration: usize,
    /// Randomised constructions done.
    pub iterations: usize,
    pub seconds: f64,
}

/// Geometric selector: keeps the indices of the arcs left with the top of the list at the
/// end of the vector, so that taking the k-th from the top costs O(k).
struct GeometricSelector {
    beta: f64,
    left: Vec<usize>,
    rng: StdRng,
}

impl ArcSelector for GeometricSelector {
    fn start(&mut self, savings: &[f64]) {
        self.left = (0..savings.len()).rev().collect();
    }

    fn next(&mut self) -> usize {
        let u = 1.0 - self.rng.random::<f64>(); // in (0, 1], so ln u is finite
        let k = ((u.ln() / (1.0 - self.beta).ln()) as usize) % self.left.len();
        self.left.remove(self.left.len() - 1 - k)
    }
}

/// Sum tree over non-negative weights: draws an index with probability proportional to its
/// weight and sets weights in O(log n). Parent sums are recomputed from their children, so
/// removed items have exactly zero mass (no drift from repeated subtractions).
struct SumTree {
    size: usize,
    node: Vec<f64>,
}

impl SumTree {
    /// Tree over `weights` (leaves beyond them weigh 0).
    fn new(weights: &[f64]) -> SumTree {
        let size = weights.len().next_power_of_two().max(1);
        let mut node = vec![0.0; 2 * size];
        node[size..size + weights.len()].copy_from_slice(weights);
        for p in (1..size).rev() {
            node[p] = node[2 * p] + node[2 * p + 1];
        }
        SumTree { size, node }
    }

    /// Total weight.
    fn total(&self) -> f64 {
        self.node[1]
    }

    /// Set the weight of item i.
    fn set(&mut self, i: usize, w: f64) {
        let mut p = i + self.size;
        self.node[p] = w;
        while p > 1 {
            p /= 2;
            self.node[p] = self.node[2 * p] + self.node[2 * p + 1];
        }
    }

    /// Item whose cumulative weight interval contains u * total, u in [0, 1); never an item
    /// of zero weight while the total is positive.
    fn sample(&self, u: f64) -> usize {
        let mut p = 1;
        let mut x = u * self.total();
        while p < self.size {
            let (l, r) = (self.node[2 * p], self.node[2 * p + 1]);
            if r <= 0.0 || (l > 0.0 && x < l) {
                p = 2 * p;
            } else {
                x -= l;
                p = 2 * p + 1;
            }
        }
        p - self.size
    }
}

/// Softmax selector: weights exp((s' - s'_top) / tau) in a sum tree; taking an arc sets its
/// weight to 0, so every draw is the softmax over the arcs left. Weights are taken relative
/// to the best arc left; if they all underflow to 0 they are rebuilt relative to it.
struct SoftmaxSelector {
    tau: f64,
    savings: Vec<f64>,
    taken: Vec<bool>,
    tree: SumTree,
    rng: StdRng,
}

impl SoftmaxSelector {
    /// Weights of the arcs left relative to the best of them.
    fn rebuild(&mut self) {
        let top = (0..self.savings.len()).find(|&i| !self.taken[i]).map_or(0.0, |i| self.savings[i]);
        let weights: Vec<f64> = (0..self.savings.len())
            .map(|i| if self.taken[i] { 0.0 } else { ((self.savings[i] - top) / self.tau).exp() })
            .collect();
        self.tree = SumTree::new(&weights);
    }
}

impl ArcSelector for SoftmaxSelector {
    fn start(&mut self, savings: &[f64]) {
        self.savings = savings.to_vec();
        self.taken = vec![false; savings.len()];
        self.rebuild();
    }

    fn next(&mut self) -> usize {
        if self.tree.total() <= 0.0 || !self.tree.total().is_finite() {
            self.rebuild();
        }
        let i = self.tree.sample(self.rng.random::<f64>());
        self.taken[i] = true;
        self.tree.set(i, 0.0);
        i
    }
}

/// Randomised selector of either kind, seeded.
enum RandomSelector {
    Geometric(GeometricSelector),
    Softmax(SoftmaxSelector),
}

impl RandomSelector {
    fn new(randomization: Randomization, seed: u64) -> RandomSelector {
        let rng = StdRng::seed_from_u64(seed);
        match randomization {
            Randomization::Geometric { beta } => RandomSelector::Geometric(GeometricSelector { beta, left: Vec::new(), rng }),
            Randomization::Softmax { tau } => RandomSelector::Softmax(SoftmaxSelector {
                tau,
                savings: Vec::new(),
                taken: Vec::new(),
                tree: SumTree::new(&[]),
                rng,
            }),
        }
    }
}

impl ArcSelector for RandomSelector {
    fn start(&mut self, savings: &[f64]) {
        match self {
            RandomSelector::Geometric(s) => s.start(savings),
            RandomSelector::Softmax(s) => s.start(savings),
        }
    }

    fn next(&mut self) -> usize {
        match self {
            RandomSelector::Geometric(s) => s.next(),
            RandomSelector::Softmax(s) => s.next(),
        }
    }
}

/// One GRASP run (construction phase only): the deterministic heuristic, then randomised
/// constructions until the iteration or time limit; returns the best solution found.
pub fn grasp(inst: &Instance, cfg: &GraspConfig, seed: u64) -> GraspResult {
    let start = Instant::now();
    let list = SavingsList::new(inst, cfg.alpha);
    let (mut best, _) = construct(inst, &list, &mut Greedy::default());
    let initial_reward = best.reward;
    let mut selector = RandomSelector::new(cfg.randomization, seed);
    let (mut best_iteration, mut iterations) = (0, 0);
    while iterations < cfg.iterations && cfg.time_limit.map_or(true, |t| start.elapsed().as_secs_f64() < t) {
        iterations += 1;
        let (sol, _) = construct(inst, &list, &mut selector);
        if sol.reward > best.reward {
            best = sol;
            best_iteration = iterations;
        }
    }
    GraspResult { solution: best, initial_reward, best_iteration, iterations, seconds: start.elapsed().as_secs_f64() }
}

/// SplitMix64 finaliser: nearby inputs give unrelated outputs.
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Seed of one GRASP run, derived from the base seed, the instance name and labels (alpha
/// index, randomisation, run...). Seeding every run from what it computes, not from the
/// order of execution, keeps results independent of the thread schedule and equal whether
/// an instance is solved alone or with the whole folder.
pub fn stream_seed(base: u64, name: &str, labels: &[u64]) -> u64 {
    let mut z = mix(base ^ 0x9E37_79B9_7F4A_7C15);
    for b in name.bytes() {
        z = mix(z ^ u64::from(b));
    }
    for &l in labels {
        z = mix(z ^ l.wrapping_add(0x9E37_79B9_7F4A_7C15));
    }
    z
}

/// `runs` independent GRASP runs of one instance in parallel; returns the best one (highest
/// reward, then lowest run number) with its run number, and checks its solution.
pub fn grasp_runs(inst: &Instance, cfg: &GraspConfig, runs: usize, seed: u64) -> Result<(usize, GraspResult), String> {
    let results: Vec<GraspResult> =
        (0..runs).into_par_iter().map(|r| grasp(inst, cfg, stream_seed(seed, &inst.name, &[r as u64]))).collect();
    let (run, best) = results
        .into_iter()
        .enumerate()
        .reduce(|a, b| if b.1.solution.reward > a.1.solution.reward { b } else { a })
        .unwrap();
    best.solution.check(inst).map_err(|e| format!("{} GRASP: {e}", inst.name))?;
    Ok((run, best))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Frequency of each index as the first pick of a fresh list, over `n` draws.
    fn first_pick_freq(sel: &mut RandomSelector, savings: &[f64], n: usize) -> Vec<f64> {
        let mut count = vec![0usize; savings.len()];
        for _ in 0..n {
            sel.start(savings);
            count[sel.next()] += 1;
        }
        count.iter().map(|&c| c as f64 / n as f64).collect()
    }

    #[test]
    fn selectors_return_every_index_once() {
        let savings: Vec<f64> = (0..257).rev().map(|v| v as f64 / 10.0).collect();
        for r in [Randomization::Geometric { beta: 0.3 }, Randomization::Softmax { tau: 1.0 }] {
            let mut sel = RandomSelector::new(r, 7);
            for _ in 0..20 {
                sel.start(&savings);
                let mut seen = vec![false; savings.len()];
                for _ in 0..savings.len() {
                    let i = sel.next();
                    assert!(!seen[i], "{} returned {i} twice", r.name());
                    seen[i] = true;
                }
            }
        }
    }

    #[test]
    fn geometric_first_pick_follows_geom_beta() {
        let beta = 0.3;
        let savings = vec![0.0; 1000];
        let freq = first_pick_freq(&mut RandomSelector::new(Randomization::Geometric { beta }, 1), &savings, 200_000);
        for k in 0..5 {
            let p = beta * (1.0 - beta).powi(k as i32);
            assert!((freq[k] - p).abs() < 0.005, "k = {k}: {} vs {p}", freq[k]);
        }
    }

    #[test]
    fn softmax_first_pick_follows_softmax() {
        let savings: [f64; 4] = [2.0, 1.0, 0.0, -1.0];
        for tau in [0.5f64, 1.0, 3.0] {
            let z: f64 = savings.iter().map(|s| (s / tau).exp()).sum();
            let freq = first_pick_freq(&mut RandomSelector::new(Randomization::Softmax { tau }, 3), &savings, 200_000);
            for (i, s) in savings.iter().enumerate() {
                let p = (s / tau).exp() / z;
                assert!((freq[i] - p).abs() < 0.005, "tau {tau}, arc {i}: {} vs {p}", freq[i]);
            }
        }
    }

    #[test]
    fn softmax_survives_underflow() {
        // Differences of thousands of units underflow exp(); the selector must rebuild the
        // weights relative to the best arc left instead of drawing a taken arc.
        let savings = [5000.0, 2000.0, 0.0];
        let mut sel = RandomSelector::new(Randomization::Softmax { tau: 1.0 }, 5);
        sel.start(&savings);
        let order: Vec<usize> = (0..3).map(|_| sel.next()).collect();
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn sum_tree_never_draws_zero_weight() {
        let mut tree = SumTree::new(&[0.0, 1.0, 0.0, 2.0, 0.0]);
        for u in [0.0, 0.1, 0.33, 0.34, 0.5, 0.99, 0.999_999_999] {
            assert!([1, 3].contains(&tree.sample(u)));
        }
        tree.set(3, 0.0);
        assert_eq!(tree.sample(0.999_999_999), 1);
    }
}
