//! Team Orienteering Problem (TOP) with the savings heuristic of Panadero et al. (2021).
//!
//! Modules:
//!   model       Instance, Route, Solution and the feasibility check
//!   heuristic   savings construction with a pluggable arc selector (Greedy = PJS)
//!   alpha_grid  alpha grid 0, 0.05, ..., 1: deterministic + GRASP runs per alpha, in parallel
//!   grasp       GRASP construction phase: geometric or softmax arc choice, multi-start
//!   plot        PNG figures (routes of a solution, alpha grid)
//!   report      Markdown report on the best alpha
//!   io          loading instances and writing files
//!
//! Build (from TOP/src/rust, so that rust-toolchain.toml and .cargo/config.toml apply):
//!   cargo build --release
//! rust-toolchain.toml selects the GNU toolchain, which ships its own MinGW linker (no
//! Visual Studio needed); .cargo/config.toml sends the build to the git-ignored TOP/ignore/.
//!
//! Usage (exe in TOP/ignore/target/release/):
//!   pjs grid [GRID] [--png out.png] <instance.txt>
//!       grid experiment on one instance: a table per alpha, then the steps and routes of
//!       the best deterministic alpha
//!   pjs grid [GRID] [--png-dir DIR] [--report FILE.md] <data_dir> <out_dir>
//!       grid experiment on every instance; writes <out_dir>/alpha_grid.csv (one row per
//!       instance and alpha), <out_dir>/grasp_runs.csv (one row per GRASP run) and
//!       <out_dir>/best_alpha.csv, the best deterministic routes to DIR and the Markdown
//!       report (figures in <out_dir>/figures)
//!   GRID options: for every alpha of the grid 0, 0.05, ..., 1, one deterministic
//!   construction and --reps R GRASP runs (default 10) with each randomisation, every run
//!     with --iters N randomised constructions (default 10) and its own seed;
//!     --geometric B (default 0.3), --softmax T (default 1), --seed S (default 1)
//!   pjs grasp [GRASP] [--png out.png] <instance.txt>
//!       GRASP on one instance: the best routes found
//!   pjs grasp [GRASP] [--png-dir DIR] <data_dir> <out.csv>
//!       GRASP on every instance, one CSV row each
//!   GRASP options:
//!     --alpha A|best   alpha of the enriched savings (default best: the best of the grid)
//!     --geometric B    arc position ~ Geom(B) (default, B = 0.3 as in the paper)
//!     --softmax T      arc drawn with probability softmax(savings / T)
//!     --iters N        randomised constructions per run (default 1000)
//!     --time S         wall-clock limit per run in seconds (optional)
//!     --runs R         independent runs per instance, in parallel; the best is kept (default 1)
//!     --seed S         base seed (default 1)
//!   --threads N limits the worker threads (default: one per logical core).
//!
//! Instance format (Chao et al. 1996): "n;N", "m;M", "tmax;T", then N lines "x;y;reward".
//! The first node is the source, the last one the sink; travel times are Euclidean distances.

mod alpha_grid;
mod grasp;
mod heuristic;
mod io;
mod model;
mod plot;
mod report;

use alpha_grid::{
    best_alpha, best_csv, best_grasp, best_grid_alpha, grasp_runs_csv, grid_alpha, grid_csv, mean_time_us, solve_all, Best, GridConfig, ALPHAS,
    GRID_TEXT,
};
use grasp::{grasp_runs, GraspConfig, GraspResult, Randomization};
use heuristic::pjs;
use model::{Instance, Solution};
use plot::plot_solution;
use rayon::prelude::*;
use std::env;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

/// Subtitle of the route plot: the best deterministic alpha and how many grid values reach
/// its reward.
fn best_note(best: &Best) -> String {
    format!(
        "PJS savings heuristic, smallest alpha of the grid {GRID_TEXT} with the highest reward: \
         alpha = {:.2}  ·  {} of {ALPHAS} alphas reach it, between {:.2} and {:.2}",
        grid_alpha(best.k),
        best.optimal,
        best.alpha_lo,
        best.alpha_hi
    )
}

/// Print the routes of a solution, one line each.
fn print_routes(inst: &Instance, sol: &Solution) {
    for (k, r) in sol.routes.iter().enumerate() {
        let path: Vec<String> = r.nodes.iter().map(|v| v.to_string()).collect();
        println!(
            "  route {}: 0 -> {} -> {}   reward {}  time {:.4}",
            k + 1,
            path.join(" -> "),
            inst.sink(),
            r.reward,
            r.time
        );
    }
}

/// Run the grid experiment on one instance: a table with every alpha (deterministic reward
/// and the GRASP runs of each randomisation), then the steps and routes of the best
/// deterministic alpha; optionally plot them to a PNG.
fn grid_one(path: &Path, cfg: &GridConfig, png: Option<&Path>) -> Result<(), String> {
    let inst = Instance::load(path)?;
    let res = solve_all(std::slice::from_ref(&inst), cfg)?;
    let best = best_alpha(&res.runs);
    let alpha = grid_alpha(best.k);
    let (sol, st) = pjs(&inst, alpha);
    let customers = inst.n() - 2;
    println!("instance {}  ({customers} customers, m = {}, tmax = {})", inst.name, inst.m, inst.tmax);
    if cfg.reps > 0 {
        let (g0, g1) = (cfg.methods[0].describe(), cfg.methods[1].describe());
        println!("grasp    {} runs x {} iterations per alpha: {g0} | {g1}  ({:.2} s)", cfg.reps, cfg.iterations, res.seconds);
        println!("  alpha  determ.   geometric best / mean / std     softmax best / mean / std");
        for (k, r) in res.runs.iter().enumerate() {
            let [a, s] = r.grasp;
            println!(
                "  {:.2} {:>9} {:>11} {:>9.1} {:>7.2} {:>13} {:>9.1} {:>7.2}",
                grid_alpha(k),
                r.reward,
                a.best,
                a.mean,
                a.std,
                s.best,
                s.mean,
                s.std
            );
        }
    }
    println!(
        "alpha    smallest of {ALPHAS} values ({GRID_TEXT}) with the highest deterministic reward: {alpha:.2}; \
         {} alphas reach reward {} (alpha {:.2}..{:.2}); worst alpha gives {}",
        best.optimal, sol.reward, best.alpha_lo, best.alpha_hi, best.worst
    );
    println!("step 1   dummy routes: {} (customers out of range: {})", st.reachable, customers - st.reachable);
    println!("step 2   savings list: {} arcs", st.arcs);
    println!("step 3   merges done: {}  -> {} routes", st.merges, st.routes_built);
    println!("step 4   routes kept: {} of {}", sol.routes.len(), st.routes_built);
    print_routes(&inst, &sol);
    println!("reward   {} of {}   (total time {:.4})", sol.reward, inst.total_reward(), sol.time);
    println!("time     {:.1} us per deterministic run", mean_time_us(&inst, alpha));
    if let Some(png) = png {
        plot_solution(&inst, &sol, &best_note(&best), png)?;
        println!("plot     {}", png.display());
    }
    Ok(())
}

/// Run the grid experiment on every instance of a folder, all tasks in one parallel list;
/// write alpha_grid.csv, grasp_runs.csv and best_alpha.csv to `out_dir`; with `png_dir`,
/// draw the best deterministic solution of every instance; with `report`, write the
/// Markdown report (its figures go to <out_dir>/figures).
fn grid_all(dir: &Path, out_dir: &Path, cfg: &GridConfig, png_dir: Option<&Path>, report: Option<&Path>) -> Result<(), String> {
    let instances = io::load_all(dir)?;
    let res = solve_all(&instances, cfg)?;
    let bests: Vec<Best> = res.runs.chunks(ALPHAS).map(best_alpha).collect();

    let grid_path = out_dir.join("alpha_grid.csv");
    let runs_path = out_dir.join("grasp_runs.csv");
    let best_path = out_dir.join("best_alpha.csv");
    io::write_file(&grid_path, &grid_csv(&instances, &res, cfg))?;
    io::write_file(&best_path, &best_csv(&instances, &res, &bests, cfg))?;
    if cfg.reps > 0 {
        io::write_file(&runs_path, &grasp_runs_csv(&instances, &res, cfg))?;
    }
    let total: f64 = bests.iter().zip(res.runs.chunks(ALPHAS)).map(|(b, row)| row[b.k].reward).sum();
    println!(
        "{} instances x {ALPHAS} alphas x (1 deterministic + {} GRASP runs of {} iterations) = {} tasks in {:.1} s on {} threads",
        instances.len(),
        2 * cfg.reps,
        cfg.iterations,
        instances.len() * ALPHAS * (1 + 2 * cfg.reps),
        res.seconds,
        rayon::current_num_threads()
    );
    println!("total reward, deterministic with the best alpha of every instance: {total}");
    if cfg.reps > 0 {
        for (m, method) in cfg.methods.iter().enumerate() {
            let best: f64 = res.runs.chunks(ALPHAS).map(|row| best_grasp(row, m).0).sum();
            println!("total reward, GRASP {} with the best run of every instance: {best}", method.describe());
        }
    }
    println!("-> {}\n-> {}", grid_path.display(), best_path.display());
    if cfg.reps > 0 {
        println!("-> {}", runs_path.display());
    }

    if let Some(png_dir) = png_dir {
        instances
            .par_iter()
            .zip(&bests)
            .map(|(inst, best)| {
                let (sol, _) = pjs(inst, grid_alpha(best.k));
                plot_solution(inst, &sol, &best_note(best), &png_dir.join(format!("{}.png", inst.name)))
            })
            .collect::<Result<(), String>>()?;
        println!("-> {} plots in {}", instances.len(), png_dir.display());
    }
    if let Some(report) = report {
        report::write_report(&instances, &res, &bests, cfg, report, &out_dir.join("figures"), png_dir)?;
    }
    Ok(())
}

/// GRASP settings from the command line; `alpha` is None for "best".
struct GraspArgs {
    alpha: Option<f64>,
    randomization: Randomization,
    iterations: usize,
    time_limit: Option<f64>,
    runs: usize,
    seed: u64,
}

impl GraspArgs {
    /// Configuration for one instance, resolving alpha = best with the alpha grid.
    fn config(&self, inst: &Instance) -> Result<GraspConfig, String> {
        let alpha = match self.alpha {
            Some(a) => a,
            None => best_grid_alpha(inst)?,
        };
        Ok(GraspConfig { alpha, randomization: self.randomization, iterations: self.iterations, time_limit: self.time_limit })
    }

    /// Subtitle of the route plot.
    fn note(&self, cfg: &GraspConfig, run: usize, res: &GraspResult) -> String {
        let alpha = if self.alpha.is_some() { format!("alpha = {:.4}", cfg.alpha) } else { format!("alpha = {:.4} (best of the grid)", cfg.alpha) };
        format!(
            "GRASP construction ({}), {alpha}, {} runs x {} iterations, seed {}  ·  best in run {run}, iteration {}  ·  deterministic heuristic {}",
            self.randomization.describe(),
            self.runs,
            res.iterations,
            self.seed,
            res.best_iteration,
            res.initial_reward
        )
    }
}

/// GRASP on one instance: print the best solution and how it compares with the
/// deterministic heuristic; optionally plot it.
fn grasp_one(path: &Path, args: &GraspArgs, png: Option<&Path>) -> Result<(), String> {
    let inst = Instance::load(path)?;
    let cfg = args.config(&inst)?;
    let (run, res) = grasp_runs(&inst, &cfg, args.runs, args.seed)?;
    let sol = &res.solution;
    println!("instance {}  ({} customers, m = {}, tmax = {})", inst.name, inst.n() - 2, inst.m, inst.tmax);
    println!(
        "grasp    {}, alpha = {:.4}{}, {} runs x {} iterations, seed {}",
        args.randomization.describe(),
        cfg.alpha,
        if args.alpha.is_none() { " (best of the grid)" } else { "" },
        args.runs,
        res.iterations,
        args.seed
    );
    println!(
        "result   reward {} (deterministic {}, +{:.2} %), best found in run {run} at iteration {}, {:.3} s",
        sol.reward,
        res.initial_reward,
        gain_pct(res.initial_reward, sol.reward),
        res.best_iteration,
        res.seconds
    );
    print_routes(&inst, sol);
    println!("reward   {} of {}   (total time {:.4})", sol.reward, inst.total_reward(), sol.time);
    if let Some(png) = png {
        plot_solution(&inst, sol, &args.note(&cfg, run, &res), png)?;
        println!("plot     {}", png.display());
    }
    Ok(())
}

/// Relative gain of `new` over `old`, in %.
fn gain_pct(old: f64, new: f64) -> f64 {
    if old > 0.0 {
        100.0 * (new - old) / old
    } else {
        0.0
    }
}

/// GRASP on every instance of a folder, instances (and their runs) in parallel; one CSV
/// row per instance and, with `png_dir`, the best routes of every instance.
fn grasp_all(dir: &Path, out: &Path, args: &GraspArgs, png_dir: Option<&Path>) -> Result<(), String> {
    let instances = io::load_all(dir)?;
    let start = Instant::now();
    let results: Vec<(GraspConfig, usize, GraspResult)> = instances
        .par_iter()
        .map(|inst| {
            let cfg = args.config(inst)?;
            let (run, res) = grasp_runs(inst, &cfg, args.runs, args.seed)?;
            if let Some(png_dir) = png_dir {
                plot_solution(inst, &res.solution, &args.note(&cfg, run, &res), &png_dir.join(format!("{}.png", inst.name)))?;
            }
            Ok((cfg, run, res))
        })
        .collect::<Result<_, String>>()?;
    let wall = start.elapsed().as_secs_f64();

    let mut csv = String::from(
        "instance,m,tmax,randomization,param,alpha,runs,iterations,seed,deterministic_reward,grasp_reward,gain_pct,best_run,best_iteration,seconds,routes,visited\n",
    );
    for (inst, (cfg, run, res)) in instances.iter().zip(&results) {
        csv += &format!(
            "{},{},{},{},{},{:.6},{},{},{},{},{},{:.4},{run},{},{:.4},{},{}\n",
            inst.name,
            inst.m,
            inst.tmax,
            cfg.randomization.name(),
            cfg.randomization.param(),
            cfg.alpha,
            args.runs,
            res.iterations,
            args.seed,
            res.initial_reward,
            res.solution.reward,
            gain_pct(res.initial_reward, res.solution.reward),
            res.best_iteration,
            res.seconds,
            res.solution.routes.len(),
            res.solution.visited()
        );
    }
    io::write_file(out, &csv)?;
    let det: f64 = results.iter().map(|r| r.2.initial_reward).sum();
    let gr: f64 = results.iter().map(|r| r.2.solution.reward).sum();
    let improved = results.iter().filter(|r| r.2.solution.reward > r.2.initial_reward).count();
    println!(
        "{} instances, {}, {} runs x {} iterations in {wall:.2} s on {} threads",
        instances.len(),
        args.randomization.describe(),
        args.runs,
        args.iterations,
        rayon::current_num_threads()
    );
    println!("total reward: deterministic {det}, GRASP {gr} (+{:.2} %); improved {improved} instances", gain_pct(det, gr));
    println!("-> {}", out.display());
    if let Some(png_dir) = png_dir {
        println!("-> {} plots in {}", instances.len(), png_dir.display());
    }
    Ok(())
}

/// Remove `flag VALUE` from the arguments and return VALUE, if the flag is present.
fn take_flag(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let k = args.iter().position(|a| a == flag)?;
    let value = args.get(k + 1).cloned().unwrap_or_default();
    args.drain(k..(k + 2).min(args.len()));
    Some(value)
}

/// Parse the value of a flag, or exit with a usage message.
fn parse_flag<T: std::str::FromStr>(args: &mut Vec<String>, flag: &str, ok: impl Fn(&T) -> bool, what: &str) -> Option<T> {
    take_flag(args, flag).map(|v| v.parse().ok().filter(|x| ok(x)).unwrap_or_else(|| usage(&format!("{flag} needs {what}"))))
}

/// Exit with a usage message.
fn usage(msg: &str) -> ! {
    eprintln!("{msg}");
    eprintln!(
        "usage: pjs grid [GRID] [--png out.png] <instance.txt>\n       \
         pjs grid [GRID] [--png-dir DIR] [--report FILE.md] <data_dir> <out_dir>\n       \
         pjs grasp [GRASP] [--png out.png] <instance.txt>\n       \
         pjs grasp [GRASP] [--png-dir DIR] <data_dir> <out.csv>\n\
         GRID: --reps R  --iters N  --geometric BETA  --softmax TAU  --seed S\n\
         GRASP: --alpha A|best  --geometric BETA | --softmax TAU  --iters N  --time S  --runs R  --seed S\n\
         --threads N limits the worker threads"
    );
    process::exit(2);
}

/// Grid options from the command line (defaults: 10 GRASP runs of 10 iterations per alpha
/// and randomisation, geometric beta = 0.3, softmax tau = 1, seed 1).
fn grid_args(args: &mut Vec<String>) -> GridConfig {
    let beta = parse_flag(args, "--geometric", |b: &f64| *b > 0.0 && *b < 1.0, "a beta in (0, 1)").unwrap_or(0.3);
    let tau = parse_flag(args, "--softmax", |t: &f64| *t > 0.0, "a temperature > 0").unwrap_or(1.0);
    GridConfig {
        reps: parse_flag(args, "--reps", |_: &usize| true, "a number of GRASP runs").unwrap_or(10),
        iterations: parse_flag(args, "--iters", |_: &usize| true, "a number of iterations").unwrap_or(10),
        methods: [Randomization::Geometric { beta }, Randomization::Softmax { tau }],
        seed: parse_flag(args, "--seed", |_: &u64| true, "an integer seed").unwrap_or(1),
    }
}

/// GRASP options from the command line (defaults: alpha best, geometric beta = 0.3,
/// 1000 iterations, 1 run, seed 1).
fn grasp_args(args: &mut Vec<String>) -> GraspArgs {
    let alpha = match take_flag(args, "--alpha").as_deref() {
        None | Some("best") => None,
        Some(v) => Some(v.parse().ok().filter(|a| (0.0..=1.0).contains(a)).unwrap_or_else(|| usage("--alpha needs best or a value in [0, 1]"))),
    };
    let beta = parse_flag(args, "--geometric", |b: &f64| *b > 0.0 && *b < 1.0, "a beta in (0, 1)");
    let tau = parse_flag(args, "--softmax", |t: &f64| *t > 0.0, "a temperature > 0");
    let randomization = match (beta, tau) {
        (Some(_), Some(_)) => usage("choose --geometric or --softmax, not both"),
        (_, Some(tau)) => Randomization::Softmax { tau },
        (beta, None) => Randomization::Geometric { beta: beta.unwrap_or(0.3) },
    };
    GraspArgs {
        alpha,
        randomization,
        iterations: parse_flag(args, "--iters", |_: &usize| true, "a number of iterations").unwrap_or(1000),
        time_limit: parse_flag(args, "--time", |t: &f64| *t > 0.0, "seconds > 0"),
        runs: parse_flag(args, "--runs", |r: &usize| *r > 0, "a number of runs > 0").unwrap_or(1),
        seed: parse_flag(args, "--seed", |_: &u64| true, "an integer seed").unwrap_or(1),
    }
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if let Some(n) = parse_flag(&mut args, "--threads", |n: &usize| *n > 0, "N > 0") {
        rayon::ThreadPoolBuilder::new().num_threads(n).build_global().expect("thread pool");
    }
    let png = take_flag(&mut args, "--png").map(PathBuf::from);
    let png_dir = take_flag(&mut args, "--png-dir").map(PathBuf::from);
    if args.is_empty() {
        usage("missing command");
    }
    let command = args.remove(0);
    let result = match command.as_str() {
        "grid" => {
            let report = take_flag(&mut args, "--report").map(PathBuf::from);
            let cfg = grid_args(&mut args);
            match args.as_slice() {
                [file] => grid_one(Path::new(file), &cfg, png.as_deref()),
                [dir, out_dir] => grid_all(Path::new(dir), Path::new(out_dir), &cfg, png_dir.as_deref(), report.as_deref()),
                _ => usage("wrong arguments for grid"),
            }
        }
        "grasp" => {
            let g = grasp_args(&mut args);
            match args.as_slice() {
                [file] => grasp_one(Path::new(file), &g, png.as_deref()),
                [dir, out] => grasp_all(Path::new(dir), Path::new(out), &g, png_dir.as_deref()),
                _ => usage("wrong arguments for grasp"),
            }
        }
        _ => usage(&format!("unknown command '{command}'")),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
