//! Independent check of the final routes written by `pjs grid` (results/best_routes.csv).
//!
//! On purpose this binary shares no code with pjs (there is no library crate, so it cannot):
//! it reads the instance files with its own parser and recomputes every travel time from the
//! coordinates. For every instance of best_solution.csv it checks that
//! - every node of a route is a customer of the instance (not the source, not the sink);
//! - no customer is visited twice, in the same route or in different ones;
//! - there are at most m routes;
//! - every route, source -> customers -> sink, takes at most tmax (tolerance 1e-9);
//! - the reward and time stated in best_routes.csv match the recomputed ones;
//! - best_ls, routes and visited in best_solution.csv match the routes.
//!
//! Usage, from TOP/ (exit code 1 if any check fails):
//!   ignore/target/release/verify_routes.exe src/data/top-data results/best_routes.csv results/best_solution.csv

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// Tolerance on tmax, the same as pjs uses, so that a route of exactly tmax passes.
const TMAX_TOL: f64 = 1e-9;
/// Largest accepted difference between a stated and a recomputed route time.
const TIME_TOL: f64 = 1e-6;

/// Instance data: vehicles, time limit, coordinates and rewards (node 0 source, last sink).
struct Inst {
    m: usize,
    tmax: f64,
    pts: Vec<(f64, f64)>,
    reward: Vec<f64>,
}

/// One row of best_routes.csv.
struct RouteRow {
    route: usize,
    reward: f64,
    time: f64,
    nodes: Vec<usize>,
}

/// Totals of one instance after the checks.
#[derive(Default)]
struct Summary {
    reward: f64,
    visited: usize,
    /// Smallest tmax - time over the routes (how close the tightest route is to the limit).
    min_slack: f64,
    /// Largest |stated - recomputed| route time.
    max_time_diff: f64,
}

/// Read an instance: "n;N", "m;M", "tmax;T", then N lines "x;y;reward".
fn read_instance(path: &Path) -> Result<Inst, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let rows: Vec<Vec<&str>> =
        text.lines().map(str::trim).filter(|l| !l.is_empty()).map(|l| l.split(';').map(str::trim).collect()).collect();
    let num = |i: usize, j: usize| -> Result<f64, String> {
        let cell = rows.get(i).and_then(|r| r.get(j)).ok_or(format!("{}: line {} too short", path.display(), i + 1))?;
        cell.parse().map_err(|_| format!("{}: '{cell}' is not a number", path.display()))
    };
    for (i, key) in ["n", "m", "tmax"].iter().enumerate() {
        if rows.get(i).map(|r| r[0]) != Some(*key) {
            return Err(format!("{}: line {} should start with '{key};'", path.display(), i + 1));
        }
    }
    let (n, m, tmax) = (num(0, 1)? as usize, num(1, 1)? as usize, num(2, 1)?);
    let (mut pts, mut reward) = (Vec::new(), Vec::new());
    for i in 3..rows.len() {
        if rows[i].len() != 3 {
            return Err(format!("{}: line {} is not x;y;reward", path.display(), i + 1));
        }
        pts.push((num(i, 0)?, num(i, 1)?));
        reward.push(num(i, 2)?);
    }
    if pts.len() != n {
        return Err(format!("{}: header says {n} nodes, found {}", path.display(), pts.len()));
    }
    Ok(Inst { m, tmax, pts, reward })
}

/// Euclidean distance, computed with hypot (pjs uses sqrt(dx² + dy²) from a matrix).
fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// Time of the route source -> nodes -> sink.
fn route_time(inst: &Inst, nodes: &[usize]) -> f64 {
    let sink = inst.pts.len() - 1;
    let path: Vec<usize> = std::iter::once(0).chain(nodes.iter().copied()).chain(std::iter::once(sink)).collect();
    path.windows(2).map(|w| dist(inst.pts[w[0]], inst.pts[w[1]])).sum()
}

/// Check the routes of one instance; returns the totals and the violations found.
fn check(inst: &Inst, routes: &[RouteRow]) -> (Summary, Vec<String>) {
    let mut errors = Vec::new();
    let mut sum = Summary { min_slack: f64::INFINITY, ..Summary::default() };
    if routes.len() > inst.m {
        errors.push(format!("{} routes for {} vehicles", routes.len(), inst.m));
    }
    let sink = inst.pts.len() - 1;
    let mut seen = HashSet::new();
    for r in routes {
        for &k in &r.nodes {
            if k == 0 || k >= sink {
                errors.push(format!("route {}: node {k} is not a customer (customers are 1..{})", r.route, sink - 1));
            } else if !seen.insert(k) {
                errors.push(format!("route {}: customer {k} visited twice", r.route));
            }
        }
        let nodes: Vec<usize> = r.nodes.iter().copied().filter(|&k| k < inst.pts.len()).collect();
        let time = route_time(inst, &nodes);
        if time > inst.tmax + TMAX_TOL {
            errors.push(format!("route {}: time {time} exceeds tmax {}", r.route, inst.tmax));
        }
        if (time - r.time).abs() > TIME_TOL {
            errors.push(format!("route {}: stated time {} but recomputed {time}", r.route, r.time));
        }
        let reward: f64 = nodes.iter().map(|&k| inst.reward[k]).sum();
        if reward != r.reward {
            errors.push(format!("route {}: stated reward {} but recomputed {reward}", r.route, r.reward));
        }
        sum.reward += reward;
        sum.visited += r.nodes.len();
        sum.min_slack = sum.min_slack.min(inst.tmax - time);
        sum.max_time_diff = sum.max_time_diff.max((time - r.time).abs());
    }
    (sum, errors)
}

/// Rows of a CSV file as maps column -> value (no quoting: none of our fields need it).
fn read_csv(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<&str> = lines.next().ok_or(format!("{}: empty", path.display()))?.split(',').collect();
    lines
        .enumerate()
        .map(|(i, l)| {
            let cells: Vec<&str> = l.split(',').collect();
            if cells.len() != header.len() {
                return Err(format!("{}: line {} has {} fields, header has {}", path.display(), i + 2, cells.len(), header.len()));
            }
            Ok(header.iter().zip(cells).map(|(h, c)| (h.to_string(), c.to_string())).collect())
        })
        .collect()
}

/// Field `col` of a CSV row, parsed.
fn field<T: std::str::FromStr>(row: &BTreeMap<String, String>, col: &str) -> Result<T, String> {
    let v = row.get(col).ok_or(format!("missing column '{col}'"))?;
    v.parse().map_err(|_| format!("column '{col}': cannot parse '{v}'"))
}

fn run(data_dir: &Path, routes_csv: &Path, solution_csv: &Path) -> Result<bool, String> {
    let mut by_instance: BTreeMap<String, Vec<RouteRow>> = BTreeMap::new();
    for row in read_csv(routes_csv)? {
        let nodes = row
            .get("nodes")
            .ok_or("best_routes.csv: missing column 'nodes'")?
            .split_whitespace()
            .map(|s| s.parse().map_err(|_| format!("node '{s}' is not an index")))
            .collect::<Result<Vec<usize>, String>>()?;
        let r = RouteRow { route: field(&row, "route")?, reward: field(&row, "reward")?, time: field(&row, "time")?, nodes };
        by_instance.entry(field(&row, "instance")?).or_default().push(r);
    }

    let (mut ok, mut instances, mut n_routes, mut visited, mut total) = (true, 0, 0, 0, 0.0);
    let (mut min_slack, mut max_time_diff, mut tightest) = (f64::INFINITY, 0.0_f64, String::new());
    for row in read_csv(solution_csv)? {
        let name: String = field(&row, "instance")?;
        let inst = read_instance(&data_dir.join(format!("{name}.txt")))?;
        let routes = by_instance.remove(&name).unwrap_or_default();
        let (sum, mut errors) = check(&inst, &routes);
        let (best_ls, n, v): (f64, usize, usize) = (field(&row, "best_ls")?, field(&row, "routes")?, field(&row, "visited")?);
        if sum.reward != best_ls {
            errors.push(format!("best_solution.csv says reward {best_ls}, the routes add up to {}", sum.reward));
        }
        if routes.len() != n || sum.visited != v {
            errors.push(format!("best_solution.csv says {n} routes and {v} customers, found {} and {}", routes.len(), sum.visited));
        }
        for e in &errors {
            println!("FAIL {name}: {e}");
        }
        ok &= errors.is_empty();
        instances += 1;
        n_routes += routes.len();
        visited += sum.visited;
        total += sum.reward;
        if sum.min_slack < min_slack {
            (min_slack, tightest) = (sum.min_slack, name.clone());
        }
        max_time_diff = max_time_diff.max(sum.max_time_diff);
    }
    for name in by_instance.keys() {
        println!("FAIL {name}: has routes in best_routes.csv but no row in best_solution.csv");
        ok = false;
    }

    println!("{instances} instances, {n_routes} routes, {visited} customers visited, total reward {total}");
    println!("tightest route: {tightest}, {min_slack:.3e} below tmax");
    println!("largest difference between stated and recomputed route time: {max_time_diff:.3e}");
    println!("{}", if ok { "OK: every route is feasible" } else { "FAILED: see the lines above" });
    Ok(ok)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [data_dir, routes_csv, solution_csv] = args.as_slice() else {
        eprintln!("usage: verify_routes <data_dir> <best_routes.csv> <best_solution.csv>");
        return ExitCode::from(2);
    };
    match run(Path::new(data_dir), Path::new(routes_csv), Path::new(solution_csv)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Source (0,0), sink (10,0), customers at (5,0) u=3, (5,3) u=7 and (2,0) u=1; m = 1.
    fn line() -> Inst {
        Inst { m: 1, tmax: 10.5, pts: vec![(0.0, 0.0), (5.0, 0.0), (5.0, 3.0), (2.0, 0.0), (10.0, 0.0)], reward: vec![0.0, 3.0, 7.0, 1.0, 0.0] }
    }

    fn row(route: usize, nodes: &[usize], inst: &Inst) -> RouteRow {
        let reward = nodes.iter().map(|&k| inst.reward[k]).sum();
        RouteRow { route, reward, time: route_time(inst, nodes), nodes: nodes.to_vec() }
    }

    #[test]
    fn feasible_route_passes() {
        let inst = line();
        let (sum, errors) = check(&inst, &[row(1, &[3, 1], &inst)]);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!((sum.reward, sum.visited), (4.0, 2));
        assert!((sum.min_slack - 0.5).abs() < 1e-12);
    }

    #[test]
    fn route_over_tmax_fails() {
        let inst = line();
        let (_, errors) = check(&inst, &[row(1, &[2], &inst)]);
        assert!(errors.iter().any(|e| e.contains("exceeds tmax")), "{errors:?}");
    }

    #[test]
    fn repeated_customer_depot_and_extra_route_fail() {
        let inst = line();
        let (_, errors) = check(&inst, &[row(1, &[1], &inst), row(2, &[1, 4], &inst)]);
        assert!(errors.iter().any(|e| e.contains("2 routes for 1 vehicles")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("visited twice")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("not a customer")), "{errors:?}");
    }

    #[test]
    fn wrong_stated_values_fail() {
        let inst = line();
        let mut r = row(1, &[1], &inst);
        (r.reward, r.time) = (5.0, 9.0);
        let (_, errors) = check(&inst, &[r]);
        assert!(errors.iter().any(|e| e.contains("stated reward")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("stated time")), "{errors:?}");
    }
}
