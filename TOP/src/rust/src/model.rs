//! Problem data and solutions of the Team Orienteering Problem: instances in the format of
//! Chao et al. (1996), open routes source -> customers -> sink, and solutions, with the
//! feasibility check shared by every solving method.

use std::fs;
use std::ops::Range;
use std::path::Path;

/// Tolerance for the driving-range check, so that floating-point noise in a sum of
/// distances never rejects a route whose exact length equals tmax.
pub const EPS: f64 = 1e-9;

/// One TOP instance: node 0 is the source, node n-1 the sink, the rest are customers.
/// Travel times are Euclidean distances, precomputed in a flat n x n matrix.
pub struct Instance {
    pub name: String,
    pub m: usize,
    pub tmax: f64,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub reward: Vec<f64>,
    dist: Vec<f64>,
}

impl Instance {
    /// Read an instance: "n;N", "m;M", "tmax;T", then N lines "x;y;reward".
    pub fn load(path: &Path) -> Result<Instance, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
        let mut header = |key: &str| -> Result<f64, String> {
            let line = lines.next().ok_or(format!("{}: missing '{key}'", path.display()))?;
            match line.split_once(';') {
                Some((k, v)) if k == key => v
                    .parse()
                    .map_err(|_| format!("{}: bad value in '{line}'", path.display())),
                _ => Err(format!("{}: expected '{key};...', got '{line}'", path.display())),
            }
        };
        let n = header("n")? as usize;
        let m = header("m")? as usize;
        let tmax = header("tmax")?;

        let (mut x, mut y, mut reward) = (Vec::new(), Vec::new(), Vec::new());
        for line in lines {
            let v: Vec<f64> = line
                .split(';')
                .map(|s| s.parse())
                .collect::<Result<_, _>>()
                .map_err(|_| format!("{}: bad node line '{line}'", path.display()))?;
            if v.len() != 3 {
                return Err(format!("{}: bad node line '{line}'", path.display()));
            }
            x.push(v[0]);
            y.push(v[1]);
            reward.push(v[2]);
        }
        if x.len() != n || n < 3 {
            return Err(format!("{}: header says n={n}, found {} nodes", path.display(), x.len()));
        }
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let dist = distance_matrix(&x, &y);
        Ok(Instance { name, m, tmax, x, y, reward, dist })
    }

    /// Number of nodes, source and sink included.
    pub fn n(&self) -> usize {
        self.x.len()
    }

    /// Index of the sink (destination depot).
    pub fn sink(&self) -> usize {
        self.n() - 1
    }

    /// Indices of the customers.
    pub fn customers(&self) -> Range<usize> {
        1..self.sink()
    }

    /// Set of the instance ("p4" for "p4.3.k").
    pub fn set(&self) -> &str {
        set_of(&self.name)
    }

    /// Travel time from node i to node j.
    pub fn t(&self, i: usize, j: usize) -> f64 {
        self.dist[i * self.n() + j]
    }

    /// Travel time of the open route source -> nodes... -> sink.
    pub fn route_time(&self, nodes: &[usize]) -> f64 {
        let mut time = 0.0;
        let mut prev = 0;
        for &k in nodes {
            time += self.t(prev, k);
            prev = k;
        }
        time + self.t(prev, self.sink())
    }

    /// Whether customer i can be served at all: its single-customer route fits in tmax.
    pub fn reachable(&self, i: usize) -> bool {
        self.route_time(&[i]) <= self.tmax + EPS
    }

    /// Number of customers that can be served at all.
    pub fn reachable_count(&self) -> usize {
        self.customers().filter(|&i| self.reachable(i)).count()
    }

    /// Sum of the rewards of all customers.
    pub fn total_reward(&self) -> f64 {
        self.reward.iter().sum()
    }
}

/// Set of an instance name ("p4" for "p4.3.k").
pub fn set_of(name: &str) -> &str {
    name.split('.').next().unwrap()
}

/// Euclidean distance between every pair of nodes (flat n x n matrix).
fn distance_matrix(x: &[f64], y: &[f64]) -> Vec<f64> {
    let n = x.len();
    let mut dist = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let (dx, dy) = (x[i] - x[j], y[i] - y[j]);
            dist[i * n + j] = (dx * dx + dy * dy).sqrt();
        }
    }
    dist
}

/// Open route source -> nodes... -> sink; `nodes` holds only the customers.
#[derive(Clone)]
pub struct Route {
    pub nodes: Vec<usize>,
    pub time: f64,
    pub reward: f64,
}

impl Route {
    /// Route source -> i -> sink.
    pub fn single(inst: &Instance, i: usize) -> Route {
        Route { nodes: vec![i], time: inst.route_time(&[i]), reward: inst.reward[i] }
    }
}

/// Set of at most m routes with its total reward and total time.
#[derive(Clone)]
pub struct Solution {
    pub routes: Vec<Route>,
    pub reward: f64,
    pub time: f64,
}

impl Solution {
    /// Solution made of `routes`, with the totals computed from them.
    pub fn new(routes: Vec<Route>) -> Solution {
        // fold from +0.0: since Rust 1.81 an empty f64 sum() is -0.0, printed as "-0"
        let reward = routes.iter().fold(0.0, |acc, r| acc + r.reward);
        let time = routes.iter().fold(0.0, |acc, r| acc + r.time);
        Solution { routes, reward, time }
    }

    /// Number of customers visited.
    pub fn visited(&self) -> usize {
        self.routes.iter().map(|r| r.nodes.len()).sum()
    }

    /// Independent feasibility check: at most m routes, every customer visited at most once,
    /// every route within tmax, and the stored times and rewards match a recomputation.
    pub fn check(&self, inst: &Instance) -> Result<(), String> {
        if self.routes.len() > inst.m {
            return Err(format!("{} routes for {} vehicles", self.routes.len(), inst.m));
        }
        let mut seen = vec![false; inst.n()];
        let mut total = 0.0;
        for r in &self.routes {
            for &k in &r.nodes {
                if k == 0 || k >= inst.sink() || seen[k] {
                    return Err(format!("node {k} is a depot or visited twice"));
                }
                seen[k] = true;
            }
            let time = inst.route_time(&r.nodes);
            if time > inst.tmax + EPS || (time - r.time).abs() > 1e-9 {
                return Err(format!("route time {time} (stored {}) vs tmax {}", r.time, inst.tmax));
            }
            let reward: f64 = r.nodes.iter().map(|&k| inst.reward[k]).sum();
            if reward != r.reward {
                return Err(format!("route reward {reward} (stored {})", r.reward));
            }
            total += reward;
        }
        if total != self.reward {
            return Err(format!("total reward {total} (stored {})", self.reward));
        }
        Ok(())
    }
}
