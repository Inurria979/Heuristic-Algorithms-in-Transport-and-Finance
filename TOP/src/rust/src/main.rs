//! Team Orienteering Problem (TOP) with the savings heuristic of Panadero et al. (2021).
//!
//! Modules:
//!   model       Instance, Route, Solution and the feasibility check
//!   heuristic   savings construction with a pluggable arc selector (Greedy = PJS)
//!   alpha_grid  alpha grid 0, 0.05, ..., 1: deterministic + GRASP runs per alpha, in parallel
//!   grasp       GRASP construction phase: geometric or softmax arc choice, multi-start
//!   elite       elite solutions of the grid (best alphas, best GRASP runs) without duplicates
//!   local_search  2-opt inside every route + insertion of the best unvisited customer
//!   plot        PNG figures (routes of a solution, alpha grid)
//!   report      Markdown report on the best alpha
//!   ls_report   Markdown report on the local search over the elite solutions
//!   panadero    comparison with Panadero et al. (2021), Tables 3-4, and a 1 s run per thread as there
//!   bks         comparison of the grid with the best known solution of every instance, per set,
//!               next to Panadero's Figure 4
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
//!   pjs grid [GRID] [--png-dir DIR] [--report FILE.md] [--ls-png-dir DIR] [--ls-report FILE.md]
//!            <data_dir> <out_dir>
//!       grid experiment on every instance; writes <out_dir>/alpha_grid.csv (one row per
//!       instance and alpha), <out_dir>/grasp_runs.csv (one row per GRASP run) and
//!       <out_dir>/best_alpha.csv, the best deterministic routes to DIR and the Markdown
//!       report (figures in <out_dir>/figures). Then the local search on the elite
//!       solutions: <out_dir>/local_search.csv (one row per elite), <out_dir>/best_solution.csv
//!       (one row per instance), <out_dir>/best_routes.csv (the node sequence of every route
//!       of the best solution, checked by the separate verify_routes binary), the routes of
//!       the best solution to --ls-png-dir and its report to --ls-report
//!   GRID options: for every alpha of the grid 0, 0.05, ..., 1, one deterministic
//!   construction and --reps R GRASP runs (default 10) with each randomisation, every run
//!     with --iters N randomised constructions (default 10) and its own seed;
//!     --geometric B (default 0.3), --softmax T (default 1), --seed S (default 1)
//!   pjs grasp [GRASP] [--png out.png] <instance.txt>
//!       GRASP on one instance: the best routes found
//!   pjs grasp [GRASP] [--png-dir DIR] <data_dir> <out.csv>
//!       GRASP on every instance, one CSV row each
//!   pjs panadero [--alpha best|grid] [--time S | --iters N] [--runs R] [--geometric B] [--softmax T] [--seed S]
//!                [--best-csv FILE] [--report FILE.md] [--resume] <data_dir> <panadero.csv> <best_solution.csv> <out_dir>
//!       only for the instances of the paper's tables: with the best grid alpha (default) or
//!       every alpha of the grid, R GRASP runs (default 10) per alpha and randomisation lasting
//!       S seconds of wall-clock time each (default 1) or N iterations each, elite and local
//!       search; writes <out_dir>/panadero_<1s|Nit>[_grid].csv and ..._runs.csv and the report
//!       comparing the paper, the grid (best_solution.csv), this experiment and, with
//!       --best-csv, another run of this command (e.g. panadero_1s.csv). With --resume, the rows
//!       already in the output CSVs are kept and only the instances missing there are run; the
//!       report is always rebuilt from the CSVs
//!   pjs bks [--report FILE.md] <bks.csv> <best_solution.csv> <out.csv>
//!       mean gap to the BKS per set of the grid (deterministic, best without local search,
//!       final) against Panadero's Figure 4; writes the CSV per set and the report
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
mod bks;
mod elite;
mod grasp;
mod heuristic;
mod io;
mod local_search;
mod ls_report;
mod model;
mod panadero;
mod plot;
mod report;

use alpha_grid::{
    best_alpha, best_csv, best_grasp, best_grid_alpha, grasp_runs_csv, grid_alpha, grid_csv, mean_time_us, solve_all, Best, GridConfig, ALPHAS,
    GRID_TEXT,
};
use elite::{best_routes_csv, best_solution_csv, local_search_csv, origin_text, run_local_search, LsResult};
use grasp::{grasp_runs, GraspConfig, GraspResult, Randomization};
use heuristic::pjs;
use model::{Instance, Solution};
use plot::plot_solution;
use rayon::prelude::*;
use std::collections::HashSet;
use std::env;
use std::fs;
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
    let ls = run_local_search(std::slice::from_ref(&inst), &res, cfg)?;
    let ie = &ls.instances[0];
    println!("local search on {} elite solutions ({} selected, duplicates dropped), {:.3} s", ie.elite.len(), ie.selected, ls.seconds);
    println!("  origin          alpha  run   before    after  inserted  2-opt");
    for e in &ie.elite {
        let (origin, rep) = origin_text(e.origin, &cfg.methods);
        println!(
            "  {origin:<15} {:.2} {rep:>4} {:>8} {:>8} {:>9} {:>6}",
            grid_alpha(e.k),
            e.before.reward,
            e.after.reward,
            e.stats.inserted,
            e.stats.two_opt
        );
    }
    let b = &ie.elite[ie.best()];
    println!(
        "best     {} after local search ({} before, {}, alpha {:.2})",
        b.after.reward,
        b.before.reward,
        origin_text(b.origin, &cfg.methods).0,
        grid_alpha(b.k)
    );
    print_routes(&inst, &b.after);
    Ok(())
}

/// Subtitle of the route plot of the best solution after the local search.
fn ls_note(ls: &LsResult, f: usize, cfg: &GridConfig) -> String {
    let ie = &ls.instances[f];
    let b = &ie.elite[ie.best()];
    let (origin, rep) = origin_text(b.origin, &cfg.methods);
    let run = if rep.is_empty() { String::new() } else { format!(", run {rep}") };
    format!(
        "Best of {} elite solutions after local search (2-opt + insertion)  ·  from {origin}, alpha = {:.2}{run}  ·  \
         reward {} before local search, {} customers inserted",
        ie.elite.len(),
        grid_alpha(b.k),
        b.before.reward,
        b.stats.inserted
    )
}

/// Run the grid experiment on every instance of a folder, all tasks in one parallel list;
/// write alpha_grid.csv, grasp_runs.csv and best_alpha.csv to `out_dir`; with `png_dir`,
/// draw the best deterministic solution of every instance; with `report`, write the
/// Markdown report (its figures go to <out_dir>/figures).
fn grid_all(dir: &Path, out_dir: &Path, cfg: &GridConfig, png_dir: Option<&Path>, report: Option<&Path>, ls_out: &LsOutputs) -> Result<(), String> {
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

    let ls = run_local_search(&instances, &res, cfg)?;
    let (ls_path, sol_path) = (out_dir.join("local_search.csv"), out_dir.join("best_solution.csv"));
    let routes_path = out_dir.join("best_routes.csv");
    io::write_file(&ls_path, &local_search_csv(&instances, &ls, cfg))?;
    io::write_file(&sol_path, &best_solution_csv(&instances, &res, &ls, cfg))?;
    io::write_file(&routes_path, &best_routes_csv(&instances, &ls))?;
    let elite: usize = ls.instances.iter().map(|ie| ie.elite.len()).sum();
    let selected: usize = ls.instances.iter().map(|ie| ie.selected).sum();
    let best: f64 = ls.instances.iter().map(|ie| ie.elite[ie.best()].after.reward).sum();
    println!("local search on {elite} elite solutions ({selected} selected, duplicates dropped) in {:.1} s", ls.seconds);
    println!("total reward, best elite of every instance after local search: {best}");
    println!("-> {}\n-> {}\n-> {}", ls_path.display(), sol_path.display(), routes_path.display());
    if let Some(dir) = ls_out.png_dir {
        (0..instances.len())
            .into_par_iter()
            .map(|f| {
                let (inst, ie) = (&instances[f], &ls.instances[f]);
                plot_solution(inst, &ie.elite[ie.best()].after, &ls_note(&ls, f, cfg), &dir.join(format!("{}.png", inst.name)))
            })
            .collect::<Result<(), String>>()?;
        println!("-> {} plots in {}", instances.len(), dir.display());
    }
    if let Some(report) = ls_out.report {
        ls_report::write_ls_report(&instances, &res, &ls, cfg, report, ls_out.png_dir)?;
    }
    Ok(())
}

/// Optional outputs of the local search: route plots and Markdown report.
struct LsOutputs<'a> {
    png_dir: Option<&'a Path>,
    report: Option<&'a Path>,
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

/// Comparison with Panadero et al. (2021): the timed experiment on the instances of the
/// paper's tables, its CSVs and the Markdown report.
fn panadero_all(paths: [&Path; 4], cfg: &panadero::TimedConfig, report: Option<&Path>, best_csv: Option<&Path>, resume: bool) -> Result<(), String> {
    let [data_dir, reference, grid_best, out_dir] = paths;
    let refs = panadero::load_reference(reference)?;
    let grid = panadero::load_best(grid_best)?;
    let best_1s = best_csv.map(panadero::load_best).transpose()?;
    let (per_path, runs_path) = (out_dir.join(format!("{}.csv", cfg.stem())), out_dir.join(format!("{}_runs.csv", cfg.stem())));
    // --resume: keep the rows already in the CSVs and run only the instances missing there.
    let read = |p: &Path| if resume { fs::read_to_string(p).unwrap_or_default() } else { String::new() };
    let (old_per, old_runs) = panadero::upgrade_csvs(&read(&per_path), &read(&runs_path));
    let done: HashSet<&str> = old_per.lines().skip(1).filter_map(|l| l.split(',').next()).collect();
    let todo: Vec<&str> = refs.iter().map(|r| r.name.as_str()).filter(|n| !done.contains(n)).collect();
    let instances: Vec<Instance> =
        todo.par_iter().map(|n| Instance::load(&data_dir.join(format!("{n}.txt")))).collect::<Result<_, String>>()?;
    let (mut new_per, mut new_runs, mut seconds) = (String::new(), String::new(), 0.0);
    if !instances.is_empty() {
        let (timed, ls, s) = panadero::run_timed(&instances, cfg)?;
        (new_per, new_runs) = panadero::timed_csvs(&instances, &timed, &ls, cfg);
        seconds = s;
        println!(
            "{} instances x {} GRASP runs of {} in {seconds:.1} s on {} threads",
            instances.len(),
            timed.first().map_or(0, |t| t.runs.len()),
            cfg.budget.label(),
            rayon::current_num_threads()
        );
    }
    let names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
    let per = panadero::merge_csv(&old_per, &new_per, &names);
    let runs = panadero::merge_csv(&old_runs, &new_runs, &names);
    // Read the merged rows back before writing anything: a CSV the report cannot use is
    // never written over the old one.
    let outcomes = panadero::outcomes(&per, &runs, cfg)?;
    io::write_file(&per_path, &per)?;
    io::write_file(&runs_path, &runs)?;
    let best: f64 = outcomes.iter().map(|o| o.final_ls).sum();
    if resume {
        println!("resume: {} instances already done, {} run now: {}", done.len(), todo.len(), todo.join(", "));
    }
    println!("total reward after local search, {} instances: {best}", outcomes.len());
    println!("-> {}\n-> {}", per_path.display(), runs_path.display());
    if let Some(report) = report {
        // With --resume the time reported adds the earlier run's, and the instances added later
        // are those run now plus those the earlier report already listed; both are read from it.
        let mut added: Vec<String> = Vec::new();
        if resume {
            let old = fs::read_to_string(report).unwrap_or_default();
            // "Tardó N s. La instancia X se añadió ..." / "... Las instancias X, Y se añadieron ..."
            let line = old.split("Tardó ").nth(1).and_then(|t| t.lines().next()).unwrap_or_default();
            seconds += line.split(' ').next().and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
            let earlier = line
                .split_once("instancia")
                .and_then(|(_, t)| t.split_once(" se añadi"))
                .map(|(list, _)| list.trim_start_matches('s').trim())
                .unwrap_or_default();
            added.extend(earlier.split(',').map(str::trim).filter(|n| !n.is_empty()).map(String::from));
            let new: Vec<String> = todo.iter().map(|n| n.to_string()).filter(|n| !added.contains(n)).collect();
            added.extend(new);
            added.sort_by_key(|n| names.iter().position(|m| m == n));
        }
        let p = |path: &Path| path.display().to_string().replace('\\', "/");
        let extra = best_csv.map(|b| format!(" --best-csv {}", p(b))).unwrap_or_default();
        let cmd = format!(
            "ignore/target/release/pjs.exe panadero --alpha {} {} --runs {} --geometric {} --softmax {} --seed {}{extra} \\\n    --report {} {} {} {} {}",
            if cfg.grid { "grid" } else { "best" },
            cfg.budget.flag(),
            cfg.runs,
            cfg.methods[0].param(),
            cfg.methods[1].param(),
            cfg.seed,
            p(report),
            p(data_dir),
            p(reference),
            p(grid_best),
            p(out_dir)
        );
        let best_name = best_csv.and_then(|b| b.file_name()).map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
        let input =
            panadero::ReportInputs { refs: &refs, grid: &grid, best_1s: best_1s.as_ref(), best_name: &best_name, seconds, cmd: &cmd, added: &added };
        panadero::write_comparison(&outcomes, cfg, &input, report)?;
    }
    Ok(())
}

/// Comparison of the grid with the BKS of every instance, per set: CSV and Markdown report.
fn bks_all(paths: [&Path; 3], report: Option<&Path>) -> Result<(), String> {
    let [bks_csv, best_csv, out] = paths;
    let rows = bks::compare(bks_csv, best_csv)?;
    io::write_file(out, &bks::bks_csv(&rows))?;
    for r in &rows {
        println!(
            "{:>4}: {:>3} instances, mean gap to BKS: deterministic {:.2} %, no local search {:.2} %, final {:.2} % (Panadero 128 threads {:.2} %); BKS reached {}, above BKS {}",
            r.set, r.n, r.gap[0], r.gap[1], r.gap[2], r.panadero, r.hit[2], r.above[2]
        );
    }
    println!("-> {}", out.display());
    if let Some(report) = report {
        let p = |path: &Path| path.display().to_string().replace('\\', "/");
        let cmd = format!("ignore/target/release/pjs.exe bks --report {} {} {} {}", p(report), p(bks_csv), p(best_csv), p(out));
        bks::write_report(&rows, &cmd, report)?;
        println!("-> {}", report.display());
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

/// Remove a flag without value from the arguments and return whether it was there.
fn take_switch(args: &mut Vec<String>, flag: &str) -> bool {
    let k = args.iter().position(|a| a == flag);
    if let Some(k) = k {
        args.remove(k);
    }
    k.is_some()
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
         pjs grid [GRID] [--png-dir DIR] [--report FILE.md] [--ls-png-dir DIR] [--ls-report FILE.md] <data_dir> <out_dir>\n       \
         pjs panadero [--alpha best|grid] [--time S | --iters N] [--runs R] [--geometric B] [--softmax T] [--seed S] [--best-csv FILE] [--report FILE.md] [--resume] <data_dir> <panadero.csv> <best_solution.csv> <out_dir>\n       \
         pjs bks [--report FILE.md] <bks.csv> <best_solution.csv> <out.csv>\n       \
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
            let ls_png_dir = take_flag(&mut args, "--ls-png-dir").map(PathBuf::from);
            let ls_report = take_flag(&mut args, "--ls-report").map(PathBuf::from);
            let ls_out = LsOutputs { png_dir: ls_png_dir.as_deref(), report: ls_report.as_deref() };
            let cfg = grid_args(&mut args);
            match args.as_slice() {
                [file] => grid_one(Path::new(file), &cfg, png.as_deref()),
                [dir, out_dir] => grid_all(Path::new(dir), Path::new(out_dir), &cfg, png_dir.as_deref(), report.as_deref(), &ls_out),
                _ => usage("wrong arguments for grid"),
            }
        }
        "panadero" => {
            let report = take_flag(&mut args, "--report").map(PathBuf::from);
            let best_csv = take_flag(&mut args, "--best-csv").map(PathBuf::from);
            let resume = take_switch(&mut args, "--resume");
            let grid = match take_flag(&mut args, "--alpha").as_deref() {
                None | Some("best") => false,
                Some("grid") => true,
                Some(_) => usage("--alpha needs best or grid"),
            };
            let beta = parse_flag(&mut args, "--geometric", |b: &f64| *b > 0.0 && *b < 1.0, "a beta in (0, 1)").unwrap_or(0.3);
            let tau = parse_flag(&mut args, "--softmax", |t: &f64| *t > 0.0, "a temperature > 0").unwrap_or(1.0);
            let time = parse_flag(&mut args, "--time", |t: &f64| *t > 0.0, "seconds > 0");
            let iters = parse_flag(&mut args, "--iters", |n: &usize| *n > 0, "a number of iterations > 0");
            let budget = match (time, iters) {
                (Some(_), Some(_)) => usage("choose --time or --iters, not both"),
                (_, Some(n)) => panadero::Budget::Iterations(n),
                (t, None) => panadero::Budget::Seconds(t.unwrap_or(1.0)),
            };
            let cfg = panadero::TimedConfig {
                grid,
                budget,
                runs: parse_flag(&mut args, "--runs", |r: &usize| *r > 0, "a number of runs > 0").unwrap_or(10),
                methods: [Randomization::Geometric { beta }, Randomization::Softmax { tau }],
                seed: parse_flag(&mut args, "--seed", |_: &u64| true, "an integer seed").unwrap_or(1),
            };
            match args.as_slice() {
                [d, r, g, o] => panadero_all([Path::new(d), Path::new(r), Path::new(g), Path::new(o)], &cfg, report.as_deref(), best_csv.as_deref(), resume),
                _ => usage("wrong arguments for panadero"),
            }
        }
        "bks" => {
            let report = take_flag(&mut args, "--report").map(PathBuf::from);
            match args.as_slice() {
                [b, g, o] => bks_all([Path::new(b), Path::new(g), Path::new(o)], report.as_deref()),
                _ => usage("wrong arguments for bks"),
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
