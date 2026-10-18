//! ADDED LATER (13-10-2026), after the presentation, only to check its p4.3.k example: the
//! slide "One instance, before and after" claims 614 (PJS, best alpha) -> 874 (+42 %), found
//! by the main grid (10 GRASP runs of 10 iterations per alpha and randomisation). This
//! command repeats the grid on a single instance with a different budget per GRASP run
//! (by default 1 s of wall-clock time instead of 10 iterations), to see whether the same
//! final solution, or a better one, comes out. It is not part of the published results.
//!
//! It reuses the timed experiment of `panadero` in grid mode: every alpha of the grid, R runs
//! of each randomisation with the grid's seeds (so a run starts like the grid's 10-iteration
//! run and goes on), the same elite rules and the same local search. Optionally it writes a
//! Markdown document (in Spanish) comparing the deterministic solution, the main grid's final
//! solution, this run's final solution and the best known solution of the literature (BKS),
//! whose routes come from the PSOiA solution files (Dang, Guibadj and Moukrim).
//!
//! A separate binary so that pjs (src/main.rs) stays as it was: there is no library crate,
//! so it compiles pjs's modules again from their files (#[path]), unchanged.
//!
//! Usage, from TOP/:
//!   ignore/target/release/recheck.exe [--time S | --iters N] [--runs R] [--seed S]
//!       [--grid-csv results/best_solution.csv] [--png out.png]
//!       [--bks-sol src/data/psoia_p4.3.k.sol.txt --bks-png out.png] [--report docs/FILE.md]
//!       <instance.txt> <out_dir>
//!   defaults: 1 s per run, 10 runs per alpha and randomisation, geometric beta = 0.3,
//!   softmax tau = 1, seed 1 (the grid's). Checks that every solution is feasible and writes
//!   <out_dir>/recheck_<instance>_<budget>_grid.csv (summary), ..._runs.csv (one row per run),
//!   ..._routes.csv and ..._solution.csv (final routes, for verify_routes). With --grid-csv the
//!   main grid's routes are read from best_routes.csv next to it, and its figures from
//!   figures/routes and figures/routes_ls there. --report needs --grid-csv and --bks-sol.

#![allow(dead_code)]

#[path = "../alpha_grid.rs"]
mod alpha_grid;
#[path = "../elite.rs"]
mod elite;
#[path = "../grasp.rs"]
mod grasp;
#[path = "../heuristic.rs"]
mod heuristic;
#[path = "../io.rs"]
mod io;
#[path = "../local_search.rs"]
mod local_search;
#[path = "../model.rs"]
mod model;
#[path = "../panadero.rs"]
mod panadero;
#[path = "../plot.rs"]
mod plot;
#[path = "../report.rs"]
mod report;

use alpha_grid::{grid_alpha, ALPHAS};
use elite::{best_routes_csv, origin_text};
use grasp::Randomization;
use io::write_file;
use model::{Instance, Route, Solution};
use panadero::{read_csv, run_timed, timed_csvs, Budget, TimedConfig};
use plot::plot_solution;
use report::relative_to;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::{env, process};

/// Optional inputs and outputs besides the instance and the output folder.
struct Opts {
    grid_csv: Option<PathBuf>,
    png: Option<PathBuf>,
    bks_sol: Option<PathBuf>,
    bks_png: Option<PathBuf>,
    report: Option<PathBuf>,
}

/// Remove `flag VALUE` from the arguments and return VALUE, if the flag is present.
fn take_flag(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let k = args.iter().position(|a| a == flag)?;
    let value = args.get(k + 1).cloned().unwrap_or_default();
    args.drain(k..(k + 2).min(args.len()));
    Some(value)
}

/// Parse the value of a flag, or exit with a usage message.
fn parse_flag<T: std::str::FromStr>(args: &mut Vec<String>, flag: &str) -> Option<T> {
    take_flag(args, flag).map(|v| v.parse().unwrap_or_else(|_| usage(&format!("bad value for {flag}"))))
}

/// Exit with a usage message.
fn usage(msg: &str) -> ! {
    eprintln!("{msg}");
    eprintln!(
        "usage: recheck [--time S | --iters N] [--runs R] [--geometric B] [--softmax T] [--seed S] [--grid-csv FILE] [--png out.png] \
         [--bks-sol FILE --bks-png out.png] [--report FILE.md] <instance.txt> <out_dir>"
    );
    process::exit(2);
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut path = |flag: &str| take_flag(&mut args, flag).map(PathBuf::from);
    let opts = Opts { grid_csv: path("--grid-csv"), png: path("--png"), bks_sol: path("--bks-sol"), bks_png: path("--bks-png"), report: path("--report") };
    if opts.report.is_some() && (opts.grid_csv.is_none() || opts.bks_sol.is_none()) {
        usage("--report needs --grid-csv and --bks-sol");
    }
    let budget = match (parse_flag::<f64>(&mut args, "--time"), parse_flag::<usize>(&mut args, "--iters")) {
        (Some(_), Some(_)) => usage("choose --time or --iters, not both"),
        (_, Some(n)) => Budget::Iterations(n),
        (t, None) => Budget::Seconds(t.unwrap_or(1.0)),
    };
    let cfg = TimedConfig {
        grid: true,
        budget,
        runs: parse_flag(&mut args, "--runs").unwrap_or(10),
        methods: [
            Randomization::Geometric { beta: parse_flag(&mut args, "--geometric").unwrap_or(0.3) },
            Randomization::Softmax { tau: parse_flag(&mut args, "--softmax").unwrap_or(1.0) },
        ],
        seed: parse_flag(&mut args, "--seed").unwrap_or(1),
    };
    let result = match args.as_slice() {
        [file, out_dir] => Instance::load(Path::new(file)).and_then(|inst| recheck(&inst, &cfg, &opts, Path::new(out_dir))),
        _ => usage("wrong arguments"),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

/// Solution made of routes given as customer lists, with times and rewards recomputed.
fn solution_from(inst: &Instance, routes: Vec<Vec<usize>>) -> Solution {
    let reward = |nodes: &[usize]| nodes.iter().fold(0.0, |acc, &k| acc + inst.reward[k]);
    Solution::new(routes.into_iter().map(|nodes| Route { time: inst.route_time(&nodes), reward: reward(&nodes), nodes }).collect())
}

/// Final routes of the main grid, from best_routes.csv ("instance,route,reward,time,nodes").
fn load_grid_routes(inst: &Instance, path: &Path) -> Result<Solution, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let routes = text
        .lines()
        .skip(1)
        .filter(|l| l.split(',').next() == Some(inst.name.as_str()))
        .map(|l| l.rsplit(',').next().unwrap_or_default().split(' ').map(|s| s.parse().map_err(|_| format!("{}: bad node '{s}'", path.display()))).collect())
        .collect::<Result<Vec<Vec<usize>>, String>>()?;
    let sol = solution_from(inst, routes);
    sol.check(inst).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(sol)
}

/// Best known solution from a PSOiA solution file: "profit = P", "client = C", then one line
/// per tour "tour = i; profit = p; length = l; visit = {a, b, ...}". Customers there are
/// numbered from 0, so customer a is node a + 1 of the instance file. The stated profit,
/// customers and every tour's profit and length must match the recomputation.
fn load_psoia(inst: &Instance, path: &Path) -> Result<Solution, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let err = |what: &str| format!("{}: {what}", path.display());
    let value = |line: &str, key: &str| -> Result<f64, String> {
        let v = line.split(';').find_map(|p| p.trim().strip_prefix(key)).ok_or(err(&format!("no '{key}' in '{line}'")))?;
        v.trim().trim_start_matches('=').trim().parse().map_err(|_| err(&format!("bad '{key}' in '{line}'")))
    };
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let (profit, clients) = (value(lines.first().copied().unwrap_or_default(), "profit")?, value(lines.get(1).copied().unwrap_or_default(), "client")?);
    let mut routes = Vec::new();
    let mut stated = Vec::new();
    for l in lines.iter().filter(|l| l.starts_with("tour")) {
        let visit = l.split_once('{').and_then(|(_, r)| r.split_once('}')).ok_or(err(&format!("no visit list in '{l}'")))?.0;
        let nodes = visit.split(',').map(|s| s.trim().parse::<usize>().map(|a| a + 1).map_err(|_| err(&format!("bad customer '{s}'")))).collect::<Result<Vec<_>, _>>()?;
        stated.push((value(l, "profit")?, value(l, "length")?));
        routes.push(nodes);
    }
    let sol = solution_from(inst, routes);
    sol.check(inst).map_err(|e| err(&e))?;
    for (r, (p, len)) in sol.routes.iter().zip(&stated) {
        if r.reward != *p || (r.time - len).abs() > 1e-3 {
            return Err(err(&format!("tour stated profit {p} length {len}, recomputed {} and {:.4}", r.reward, r.time)));
        }
    }
    if sol.reward != profit || sol.visited() as f64 != clients {
        return Err(err(&format!("stated profit {profit} and {clients} customers, the tours give {} and {}", sol.reward, sol.visited())));
    }
    Ok(sol)
}

/// Repeat the grid on one instance with `cfg` (grid mode); print the result per alpha and the
/// final solution; check every solution; write the CSVs, the figures and the document asked for.
fn recheck(inst: &Instance, cfg: &TimedConfig, opts: &Opts, out_dir: &Path) -> Result<(), String> {
    let (timed, ls, seconds) = run_timed(std::slice::from_ref(inst), cfg)?;
    let (t, ie) = (&timed[0], &ls.instances[0]);
    println!("instance {}  ({} customers, m = {}, tmax = {})", inst.name, inst.n() - 2, inst.m, inst.tmax);
    println!(
        "recheck  {ALPHAS} alphas x 2 randomisations x {} runs of {} each, {:.1} s on {} threads",
        cfg.runs,
        cfg.budget.label(),
        seconds,
        rayon::current_num_threads()
    );
    // Per alpha: deterministic reward, best run of each randomisation, mean iterations per run.
    let per_alpha: Vec<(usize, f64, f64, f64, f64)> = t
        .alphas
        .iter()
        .enumerate()
        .map(|(a, &k)| {
            let g = &t.runs[a * 2 * cfg.runs..(a + 1) * 2 * cfg.runs];
            let best = |m: usize| g[m * cfg.runs..(m + 1) * cfg.runs].iter().map(|r| r.solution.reward).fold(0.0, f64::max);
            (k, t.det[a].reward, best(0), best(1), g.iter().map(|r| r.iterations as f64).sum::<f64>() / g.len() as f64)
        })
        .collect();
    println!("  alpha  determ.   {} best   {} best   mean iterations", cfg.methods[0].name(), cfg.methods[1].name());
    for &(k, d, g0, g1, it) in &per_alpha {
        println!("  {:.2} {d:>9} {g0:>14} {g1:>12} {it:>16.0}", grid_alpha(k));
    }
    let no_ls = t.runs.iter().map(|g| g.solution.reward).fold(t.det.iter().map(|s| s.reward).fold(0.0, f64::max), f64::max);
    let b = &ie.elite[ie.best()];
    let (origin, rep) = origin_text(b.origin, &cfg.methods);
    let det = &t.det[t.alphas.iter().position(|&k| k == t.k).unwrap()];
    println!("deterministic, best alpha {:.2}: {}", grid_alpha(t.k), det.reward);
    println!("best without local search: {no_ls}");
    println!(
        "final    {} after local search on {} elite ({} selected): from {origin}, alpha {:.2}, run {}, {} before, {} inserted, {} customers  (+{:.1} % over the deterministic)",
        b.after.reward,
        ie.elite.len(),
        ie.selected,
        grid_alpha(b.k),
        if rep.is_empty() { "-" } else { rep.as_str() },
        b.before.reward,
        b.stats.inserted,
        b.after.visited(),
        100.0 * (b.after.reward - det.reward) / det.reward
    );
    for (r, route) in b.after.routes.iter().enumerate() {
        let path: Vec<String> = route.nodes.iter().map(|v| v.to_string()).collect();
        println!("  route {}: {}   reward {}  time {:.4}", r + 1, path.join(" "), route.reward, route.time);
    }
    let mut grid = None;
    if let Some(path) = &opts.grid_csv {
        let cols = ["deterministic", "best_no_ls", "best_ls"];
        if let Some((_, v)) = read_csv(path, &cols)?.into_iter().find(|(n, _)| *n == inst.name) {
            println!("main grid (10 iterations per run, {}): deterministic {}, without local search {}, final {}", path.display(), v[0], v[1], v[2]);
            let routes = load_grid_routes(inst, &path.with_file_name("best_routes.csv"))?;
            if routes.reward != v[2] {
                return Err(format!("best_routes.csv gives {} for {}, best_solution.csv says {}", routes.reward, inst.name, v[2]));
            }
            grid = Some((v[1], routes));
        }
    }
    let bks = opts.bks_sol.as_deref().map(|p| load_psoia(inst, p)).transpose()?;
    if let Some(s) = &bks {
        println!("best known (PSOiA): {} with {} customers, routes feasible", s.reward, s.visited());
    }

    // Feasibility of every solution of the experiment, not only the final one: deterministic
    // of every alpha, best of every GRASP run, and every elite before and after the local search.
    let mut all: Vec<(String, &Solution)> = t.det.iter().zip(&t.alphas).map(|(s, &k)| (format!("deterministic alpha {:.2}", grid_alpha(k)), s)).collect();
    all.extend(t.runs.iter().enumerate().map(|(i, g)| (format!("GRASP run {i}"), &g.solution)));
    for (i, e) in ie.elite.iter().enumerate() {
        all.push((format!("elite {i} before local search"), &e.before));
        all.push((format!("elite {i} after local search"), &e.after));
    }
    let failed: Vec<String> = all.iter().filter_map(|(what, s)| s.check(inst).err().map(|e| format!("{what}: {e}"))).collect();
    println!("check    {} of {} solutions feasible (at most m routes, no customer twice, every route within tmax)", all.len() - failed.len(), all.len());
    if !failed.is_empty() {
        return Err(format!("infeasible solutions:\n{}", failed.join("\n")));
    }

    let (per, runs) = timed_csvs(std::slice::from_ref(inst), &timed, &ls, cfg);
    let stem = format!("recheck_{}_{}_grid", inst.name, cfg.budget.tag());
    let path = |suffix: &str| out_dir.join(format!("{stem}{suffix}.csv"));
    // Final routes and summary in the format of best_routes.csv / best_solution.csv, for the
    // outside check: verify_routes <data_dir> <..._routes.csv> <..._solution.csv>.
    let solution = format!("instance,best_ls,routes,visited\n{},{},{},{}\n", inst.name, b.after.reward, b.after.routes.len(), b.after.visited());
    let (routes_path, solution_path) = (path("_routes"), path("_solution"));
    for (p, text) in [(path(""), per), (path("_runs"), runs), (routes_path.clone(), best_routes_csv(std::slice::from_ref(inst), &ls)), (solution_path.clone(), solution)] {
        write_file(&p, &text)?;
        println!("-> {}", p.display());
    }
    if let Some(png) = &opts.png {
        let note = format!(
            "Recheck added after the presentation: grid with {} per GRASP run  ·  from {origin}, alpha = {:.2}  ·  reward {} before local search, {} inserted",
            cfg.budget.label(),
            grid_alpha(b.k),
            b.before.reward,
            b.stats.inserted
        );
        plot_solution(inst, &b.after, &note, png)?;
        println!("-> {}", png.display());
    }
    if let (Some(png), Some(s)) = (&opts.bks_png, &bks) {
        let note = format!("Best known solution of the literature (PSOiA, Dang, Guibadj and Moukrim): reward {}, {} customers", s.reward, s.visited());
        plot_solution(inst, s, &note, png)?;
        println!("-> {}", png.display());
    }
    if let (Some(report), Some((grid_no_ls, grid_sol)), Some(bks_sol)) = (&opts.report, &grid, &bks) {
        let d = Doc {
            inst,
            cfg,
            seconds,
            per_alpha: &per_alpha,
            alpha: grid_alpha(t.k),
            det,
            grid_no_ls: *grid_no_ls,
            grid: grid_sol,
            no_ls,
            fin: &b.after,
            fin_text: format!(
                "{origin}, α = {:.2}{}: {} antes de la búsqueda local, {} clientes insertados",
                grid_alpha(b.k),
                if rep.is_empty() { String::new() } else { format!(", ejecución {rep}") },
                b.before.reward,
                b.stats.inserted
            ),
            elite: (ie.elite.len(), ie.selected),
            checked: all.len(),
            bks: bks_sol,
            files: Files {
                grid_csv: opts.grid_csv.as_deref().unwrap(),
                bks_sol: opts.bks_sol.as_deref().unwrap(),
                png: opts.png.as_deref(),
                bks_png: opts.bks_png.as_deref(),
                routes: &routes_path,
                solution: &solution_path,
                out_dir,
            },
        };
        write_doc(&d, report)?;
        println!("-> {}", report.display());
    }
    Ok(())
}

/// Files the document cites (paths relative to TOP/, as given on the command line).
struct Files<'a> {
    grid_csv: &'a Path,
    bks_sol: &'a Path,
    png: Option<&'a Path>,
    bks_png: Option<&'a Path>,
    routes: &'a Path,
    solution: &'a Path,
    out_dir: &'a Path,
}

/// Everything the comparison document needs.
struct Doc<'a> {
    inst: &'a Instance,
    cfg: &'a TimedConfig,
    seconds: f64,
    per_alpha: &'a [(usize, f64, f64, f64, f64)],
    /// Best deterministic alpha and its solution.
    alpha: f64,
    det: &'a Solution,
    /// Main grid: best without local search and final solution.
    grid_no_ls: f64,
    grid: &'a Solution,
    /// This run: best without local search, final solution and where it comes from.
    no_ls: f64,
    fin: &'a Solution,
    fin_text: String,
    /// Elite kept and selected.
    elite: (usize, usize),
    /// Solutions checked for feasibility.
    checked: usize,
    bks: &'a Solution,
    files: Files<'a>,
}

/// Customers visited by a solution.
fn visited_set(s: &Solution) -> BTreeSet<usize> {
    s.routes.iter().flat_map(|r| r.nodes.iter().copied()).collect()
}

/// Path with forward slashes, for commands and links.
fn slash(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

/// Write the Markdown document comparing the four solutions of the instance.
fn write_doc(d: &Doc, report: &Path) -> Result<(), String> {
    let inst = d.inst;
    let base = report.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    write_file(report, "")?; // the folder must exist for the relative links
    let link = |p: &Path| relative_to(base, p);
    let figures = d.files.grid_csv.parent().unwrap_or(Path::new(".")).join("figures");
    let (det_png, grid_png) = (figures.join("routes").join(format!("{}.png", inst.name)), figures.join("routes_ls").join(format!("{}.png", inst.name)));
    let bks_reward = d.bks.reward;
    let pct = |v: f64| 100.0 * v / bks_reward;
    let sols: [(&str, &Solution); 4] = [
        ("PJS determinista (mejor α)", d.det),
        ("Rejilla, 10 iteraciones + búsqueda local (la de la presentación)", d.grid),
        (&format!("Recheck, {} + búsqueda local", d.cfg.budget.label()), d.fin),
        ("Mejor conocida (BKS, PSOiA)", d.bks),
    ];
    let reachable = inst.customers().filter(|&i| inst.reachable(i)).count();
    let budget = d.cfg.budget.label();
    let mut md = String::new();
    let mut w = |s: String| {
        md += &s;
        md.push('\n');
    };

    w(format!("# {}: solución determinista, GRASP + búsqueda local y mejor solución conocida", inst.name));
    w(String::new());
    w(format!(
        "> **Documento añadido después de la presentación**, solo para comprobar el ejemplo de la diapositiva *One instance, \
         before and after: {}* (614 → 874, +42 %). Lo genera el ejecutable aparte `recheck` \
         (`src/rust/src/bin/recheck.rs`), que no forma parte del flujo principal (`pjs`) ni cambia sus resultados.",
        inst.name
    ));
    w(String::new());
    w("Se regenera, desde `TOP/`, con:".into());
    w(String::new());
    let png = d.files.png.map(|p| format!(" --png {}", slash(p))).unwrap_or_default();
    let bks_png = d.files.bks_png.map(|p| format!(" --bks-png {}", slash(p))).unwrap_or_default();
    w(format!(
        "```\nignore/target/release/recheck.exe {} --runs {} --seed {} --grid-csv {}{png} \\\n    --bks-sol {}{bks_png} --report {} \\\n    src/data/top-data/{}.txt {}\n```",
        d.cfg.budget.flag(),
        d.cfg.runs,
        d.cfg.seed,
        slash(d.files.grid_csv),
        slash(d.files.bks_sol),
        slash(report),
        inst.name,
        slash(d.files.out_dir)
    ));
    w(String::new());
    if let Budget::Seconds(_) = d.cfg.budget {
        w(format!(
            "Con tiempo de reloj por ejecución el recheck **no es reproducible exactamente**: el número de iteraciones depende de la \
             carga de la máquina, así que al regenerarlo pueden cambiar las cifras de la sección 4 y la solución del recheck."
        ));
        w(String::new());
    }

    w("## 1. La instancia".into());
    w(String::new());
    w(format!(
        "- {} clientes ({} alcanzables solos dentro de tmax), m = {} vehículos, tmax = {}.\n\
         - Recompensa total de todos los clientes: {}. Ninguna solución puede recogerla entera: el BKS recoge el {:.1} %.\n\
         - Los tiempos son distancias euclídeas; cada ruta sale del origen (nodo 0) y acaba en el destino (nodo {}).",
        inst.n() - 2,
        reachable,
        inst.m,
        inst.tmax,
        inst.total_reward(),
        100.0 * bks_reward / inst.total_reward(),
        inst.sink()
    ));
    w(String::new());

    w("## 2. Las cuatro soluciones".into());
    w(String::new());
    w(format!(
        "- **PJS determinista:** la heurística de ahorros de Panadero et al. (2021) con el mejor α de la rejilla 0, 0.05, …, 1 \
         (α = {:.2}).\n\
         - **Rejilla (la de la presentación):** el experimento principal, `results/best_solution.csv` y `results/best_routes.csv`: \
         para cada α, 10 ejecuciones de GRASP de 10 iteraciones con cada aleatorización (geométrica y softmax), élite y búsqueda \
         local (2-opt + inserción).\n\
         - **Recheck:** lo mismo, pero cada ejecución de GRASP itera durante **{budget}** en lugar de 10 iteraciones (unas {:.0} \
         iteraciones de media), con las mismas semillas, las mismas reglas de élite y la misma búsqueda local. Tardó {:.0} s. \
         La final sale de {}.\n\
         - **BKS:** la mejor solución conocida de la literatura, {bks_reward}, en la columna BEST de \
         `MA10-PSOMA-PSOiA-Chao-benchmark.xls` (Dang, Guibadj y Moukrim; es la de `src/data/bks_chao.csv`). Sus rutas son las de \
         su fichero de solución, copiado sin cambios en [`{}`]({}); allí los clientes se numeran desde 0, así que el cliente *a* es \
         el nodo *a* + 1 del fichero de la instancia. Recalculadas aquí, las tres rutas dan exactamente la recompensa y la \
         longitud que declara el fichero.",
        d.alpha,
        d.per_alpha.iter().map(|r| r.4).sum::<f64>() / d.per_alpha.len() as f64,
        d.seconds,
        d.fin_text,
        slash(d.files.bks_sol),
        link(d.files.bks_sol)?
    ));
    w(String::new());

    w("## 3. Comparación".into());
    w(String::new());
    w(format!("| Solución | Recompensa | % del BKS | Distancia al BKS | Mejora sobre PJS | Clientes | Tiempo total (máx. {}) |", inst.m as f64 * inst.tmax));
    w("|---|---|---|---|---|---|---|".into());
    for (name, s) in &sols {
        w(format!(
            "| {name} | **{}** | {:.1} % | {:.1} % | {:+.1} % | {} | {:.2} |",
            s.reward,
            pct(s.reward),
            100.0 - pct(s.reward),
            100.0 * (s.reward - d.det.reward) / d.det.reward,
            s.visited(),
            s.time
        ));
    }
    w(String::new());
    w(format!(
        "Sin búsqueda local, lo mejor era {} en la rejilla y {} en el recheck. La mejora de la presentación (614 → 874, +42 %) \
         se confirma: con {budget} por ejecución la final llega a {} ({:+.1} % sobre PJS). El BKS está un {:.1} % por encima de \
         nuestra mejor solución.",
        d.grid_no_ls,
        d.no_ls,
        d.fin.reward,
        100.0 * (d.fin.reward - d.det.reward) / d.det.reward,
        100.0 * (bks_reward - d.fin.reward.max(d.grid.reward)) / d.fin.reward.max(d.grid.reward)
    ));
    w(String::new());

    // Panadero et al. (2021) give results per instance only for p1 and p2 (Tables 3 and 4); for
    // the other sets only the mean gap to the BKS with 128 threads of 1 s (Figure 4).
    let set = inst.set().to_string();
    let by_set = d.files.grid_csv.with_file_name("bks_by_set.csv");
    let row = read_csv(&by_set, &["instances", "gap_final", "gap_panadero128"])?.into_iter().find(|(s, _)| *s == set);
    let (n_set, ours_set, paper_set) = row.map(|(_, v)| (v[0], v[1], v[2])).ok_or(format!("{}: no row for set {set}", by_set.display()))?;
    w("### Lo que reporta el paper".into());
    w(String::new());
    w(format!(
        "**Panadero et al. (2021) no publica la solución de {name}.** Sus tablas 3 y 4 dan la recompensa instancia a instancia \
         (BKS, 1 a 128 hilos) solo para los conjuntos p1 y p2. Para {set} solo da, en la figura 4, la distancia media al BKS con \
         128 hilos de 1 s: **{paper_set:.2} %** sobre las instancias del conjunto, y comenta que {set} es el conjunto más difícil \
         (\"a mean value close to 8%\"). Tampoco publica sus rutas. Li et al. (2022) usa la misma heurística sobre otras \
         instancias (TOP dinámico) y no da resultados de {set}.\n\n\
         - **Estimación, no un dato del paper:** si en {name} tuviera la distancia media de su conjunto, Panadero con 128 hilos \
         obtendría unos {bks_reward} × (1 − {paper_set:.2} %) ≈ **{:.0}**.\n\
         - **Nuestras distancias en {name}:** {:.2} % la rejilla ({}, la de la presentación) y {:.2} % el recheck ({}).\n\
         - **Comparación válida, por conjuntos** ([`comparacion_bks_conjuntos.md`]({})): en las {n_set:.0} instancias de {set}, \
         nuestra rejilla final queda de media a un **{ours_set:.2} %** del BKS, frente al {paper_set:.2} % de Panadero con 128 hilos.",
        bks_reward * (1.0 - paper_set / 100.0),
        100.0 - pct(d.grid.reward),
        d.grid.reward,
        100.0 - pct(d.fin.reward),
        d.fin.reward,
        link(&base.join("comparacion_bks_conjuntos.md"))?,
        name = inst.name
    ));
    w(String::new());

    w("### Clientes frente al BKS".into());
    w(String::new());
    let bset = visited_set(d.bks);
    let sum = |set: &BTreeSet<usize>| set.iter().map(|&k| inst.reward[k]).sum::<f64>();
    w("| Solución | Clientes | En común con el BKS | Solo en esta (recompensa) | Solo en el BKS (recompensa) |".into());
    w("|---|---|---|---|---|".into());
    for (name, s) in &sols[..3] {
        let set = visited_set(s);
        let (common, mine, theirs): (BTreeSet<usize>, BTreeSet<usize>, BTreeSet<usize>) =
            (set.intersection(&bset).copied().collect(), set.difference(&bset).copied().collect(), bset.difference(&set).copied().collect());
        w(format!("| {name} | {} | {} | {} ({}) | {} ({}) |", set.len(), common.len(), mine.len(), sum(&mine), theirs.len(), sum(&theirs)));
    }
    w(String::new());
    let fset = visited_set(d.fin);
    let mut missing: Vec<usize> = bset.difference(&fset).copied().collect();
    missing.sort_by(|a, b| inst.reward[*b].total_cmp(&inst.reward[*a]).then(a.cmp(b)));
    let extra: Vec<usize> = fset.difference(&bset).copied().collect();
    let list = |v: &[usize]| v.iter().map(|&k| format!("{k} ({})", inst.reward[k])).collect::<Vec<_>>().join(", ");
    w(format!(
        "Clientes del BKS que no visita la solución del recheck, de mayor a menor recompensa: {}.\n\n\
         Clientes que visita el recheck y no el BKS: {}.",
        list(&missing),
        if extra.is_empty() { "ninguno".to_string() } else { list(&extra) }
    ));
    w(String::new());

    w("### Rutas".into());
    w(String::new());
    w("Clientes en orden de visita, con los índices de nodo del fichero de la instancia (el origen 0 y el destino quedan implícitos).".into());
    w(String::new());
    w("| Solución | Ruta | Recompensa | Clientes | Tiempo | Holgura hasta tmax | Clientes visitados |".into());
    w("|---|---|---|---|---|---|---|".into());
    for (name, s) in &sols {
        for (i, r) in s.routes.iter().enumerate() {
            let nodes: Vec<String> = r.nodes.iter().map(|k| k.to_string()).collect();
            w(format!("| {name} | {} | {} | {} | {:.4} | {:.4} | {} |", i + 1, r.reward, r.nodes.len(), r.time, inst.tmax - r.time, nodes.join(" ")));
        }
    }
    w(String::new());

    w("### Figuras".into());
    w(String::new());
    let mut figs: Vec<(String, &Path)> = vec![(format!("PJS determinista, α = {:.2}", d.alpha), &det_png), ("Rejilla + búsqueda local".into(), &grid_png)];
    if let Some(p) = d.files.png {
        figs.push((format!("Recheck, {budget} + búsqueda local"), p));
    }
    if let Some(p) = d.files.bks_png {
        figs.push(("BKS (PSOiA)".into(), p));
    }
    for (title, p) in figs {
        if p.exists() {
            w(format!("**{title}**\n\n![{title}]({})\n", link(p)?));
        }
    }

    w(format!("## 4. Recheck por α ({budget} por ejecución)"));
    w(String::new());
    w(format!(
        "Recompensa de la determinista y de la mejor de las {} ejecuciones de cada aleatorización, sin búsqueda local.",
        d.cfg.runs
    ));
    w(String::new());
    w(format!("| α | Determinista | Mejor {} | Mejor {} | Iteraciones por ejecución (media) |", d.cfg.methods[0].name(), d.cfg.methods[1].name()));
    w("|---|---|---|---|---|".into());
    for &(k, det, g0, g1, it) in d.per_alpha {
        w(format!("| {:.2} | {det} | {g0} | {g1} | {it:.0} |", grid_alpha(k)));
    }
    w(String::new());

    w("## 5. Comprobación de factibilidad".into());
    w(String::new());
    w(format!(
        "- **Todas las soluciones del recheck:** {} de {} factibles con `Solution::check` (como mucho m rutas, ningún cliente \
         repetido ni depósito dentro de una ruta, cada ruta dentro de tmax, tiempos y recompensas iguales a los recalculados): las \
         {} deterministas, la mejor de cada una de las {} ejecuciones de GRASP y las {} élite (de {} seleccionadas, sin \
         duplicados), antes y después de la búsqueda local. Si alguna fallara, `recheck` terminaría con error sin escribir nada.\n\
         - **La rejilla y el BKS** pasan la misma comprobación al leerlos.\n\
         - **Comprobación externa** de la final del recheck con `verify_routes`, que no comparte código con `pjs` (lee la \
         instancia con su propio parser y recalcula las distancias):\n\n\
         ```\nignore/target/release/verify_routes.exe src/data/top-data {} {}\n```",
        d.checked,
        d.checked,
        d.per_alpha.len(),
        d.per_alpha.len() * 2 * d.cfg.runs,
        d.elite.0,
        d.elite.1,
        slash(d.files.routes),
        slash(d.files.solution)
    ));
    write_file(report, &md)
}
