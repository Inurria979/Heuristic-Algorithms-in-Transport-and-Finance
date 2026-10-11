//! Comparison with Panadero et al. (2021), Tables 3 and 4 (sets p1 and p2): their best-known
//! solutions (BKS) and their results with 1 and 128 threads of 1 s each, against
//! - our grid (results/best_solution.csv): best without local search and final, and
//! - a timed experiment on the instances of the tables only: `runs` GRASP runs of each
//!   randomisation, each iterating for `time` seconds of wall-clock time, either with the
//!   best deterministic alpha of the grid (mode best) or with every alpha of the grid (mode
//!   grid); then the elite (deterministic solutions of the 3 best alphas tried and, for every
//!   alpha and randomisation, the 3 best runs; ties included, duplicates dropped) go through
//!   the local search.
//!
//! Writes a CSV per instance, a CSV per run and a Markdown report (in Spanish).

use crate::alpha_grid::{best_grid_k, grid_alpha, ALPHAS};
use crate::elite::{drop_duplicates, origin_text, search_all, top_with_ties, Candidate, InstanceElite, LsResult, Origin};
use crate::grasp::{grasp, stream_seed, GraspConfig, GraspResult, Randomization};
use crate::heuristic::pjs;
use crate::io::write_file;
use crate::model::{Instance, Solution};
use crate::report::gain;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

/// One row of the paper's tables: BKS and the rewards with 1 and 128 threads.
pub struct Reference {
    pub name: String,
    pub bks: f64,
    pub thr1: f64,
    pub thr128: f64,
}

/// How long every GRASP run lasts.
#[derive(Clone, Copy)]
pub enum Budget {
    /// Wall-clock seconds (not reproducible: the iterations depend on the machine's load).
    Seconds(f64),
    /// Fixed number of randomised constructions (reproducible).
    Iterations(usize),
}

impl Budget {
    /// Text for the report ("1 s", "100 iteraciones").
    pub fn label(&self) -> String {
        match self {
            Budget::Seconds(t) => format!("{t} s"),
            Budget::Iterations(n) => format!("{n} iteraciones"),
        }
    }

    /// Tag for file names and CSVs ("1s", "100it").
    pub fn tag(&self) -> String {
        match self {
            Budget::Seconds(t) => format!("{t}s"),
            Budget::Iterations(n) => format!("{n}it"),
        }
    }

    /// Command-line flag that gives this budget.
    pub fn flag(&self) -> String {
        match self {
            Budget::Seconds(t) => format!("--time {t}"),
            Budget::Iterations(n) => format!("--iters {n}"),
        }
    }
}

/// Settings of the timed experiment.
#[derive(Clone, Copy)]
pub struct TimedConfig {
    /// Length of every GRASP run.
    pub budget: Budget,
    /// GRASP runs per alpha and randomisation.
    pub runs: usize,
    pub methods: [Randomization; 2],
    pub seed: u64,
    /// Every alpha of the grid (true) or only the best deterministic one (false).
    pub grid: bool,
}

impl TimedConfig {
    /// Number of alphas tried per instance.
    fn alphas(&self) -> usize {
        if self.grid {
            ALPHAS
        } else {
            1
        }
    }

    /// Base name of the output files ("panadero_1s", "panadero_100it_grid").
    pub fn stem(&self) -> String {
        format!("panadero_{}{}", self.budget.tag(), if self.grid { "_grid" } else { "" })
    }
}

/// Timed experiment on one instance.
pub struct Timed {
    /// Best deterministic alpha of the grid (index).
    pub k: usize,
    /// Alpha indices tried: [k] or 0..ALPHAS.
    pub alphas: Vec<usize>,
    /// Deterministic solution of every alpha tried.
    pub det: Vec<Solution>,
    /// GRASP runs: `runs[(a * 2 + m) * cfg.runs + r]` for alpha position a in `alphas`.
    pub runs: Vec<GraspResult>,
}

impl Timed {
    /// Runs of alpha position a and randomisation m.
    fn group(&self, a: usize, m: usize, runs: usize) -> &[GraspResult] {
        let start = (a * 2 + m) * runs;
        &self.runs[start..start + runs]
    }

    /// Position of the best deterministic alpha in `alphas`.
    fn best_pos(&self) -> usize {
        self.alphas.iter().position(|&k| k == self.k).unwrap()
    }

    /// Best reward without local search (deterministic or any run).
    fn no_ls(&self) -> f64 {
        let det = self.det.iter().map(|s| s.reward).fold(0.0, f64::max);
        self.runs.iter().map(|g| g.solution.reward).fold(det, f64::max)
    }

    fn mean_iterations(&self) -> f64 {
        self.runs.iter().map(|g| g.iterations as f64).sum::<f64>() / self.runs.len() as f64
    }
}

/// Columns of a CSV header, by name.
fn columns(header: &str, path: &Path, names: &[&str]) -> Result<Vec<usize>, String> {
    let cols: Vec<&str> = header.split(',').collect();
    names
        .iter()
        .map(|n| cols.iter().position(|c| c == n).ok_or(format!("{}: no column '{n}'", path.display())))
        .collect()
}

/// Numeric fields of every data row of a CSV, by column name, keyed by the first field.
fn read_csv(path: &Path, names: &[&str]) -> Result<Vec<(String, Vec<f64>)>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let idx = columns(lines.next().unwrap_or_default(), path, names)?;
    lines
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            let v = idx
                .iter()
                .map(|&i| f.get(i).and_then(|x| x.parse().ok()).ok_or(format!("{}: bad row '{l}'", path.display())))
                .collect::<Result<_, String>>()?;
            Ok((f[0].to_string(), v))
        })
        .collect()
}

/// The paper's tables (src/data/panadero2021_p1_p2.csv).
pub fn load_reference(path: &Path) -> Result<Vec<Reference>, String> {
    Ok(read_csv(path, &["bks", "thr1", "thr128"])?
        .into_iter()
        .map(|(name, v)| Reference { name, bks: v[0], thr1: v[1], thr128: v[2] })
        .collect())
}

/// Best reward without and with local search of every instance, from a CSV with columns
/// best_no_ls and best_ls (results/best_solution.csv or results/panadero_1s.csv).
pub fn load_best(path: &Path) -> Result<HashMap<String, (f64, f64)>, String> {
    Ok(read_csv(path, &["best_no_ls", "best_ls"])?.into_iter().map(|(n, v)| (n, (v[0], v[1]))).collect())
}

/// Run the timed experiment on every instance and the local search on its elite. All GRASP
/// runs of all instances form one parallel list; every run gets its own wall-clock budget.
/// Seeds: mode grid uses the grid's (alpha, randomisation, run), so a timed run starts like
/// the grid's 10-iteration run and goes on; mode best keeps (randomisation, run).
pub fn run_timed(instances: &[Instance], cfg: &TimedConfig) -> Result<(Vec<Timed>, LsResult, f64), String> {
    let start = Instant::now();
    let base: Vec<(usize, Vec<usize>, Vec<Solution>)> = instances
        .par_iter()
        .map(|inst| {
            let k = best_grid_k(inst);
            let alphas: Vec<usize> = if cfg.grid { (0..ALPHAS).collect() } else { vec![k] };
            let det = alphas.iter().map(|&a| pjs(inst, grid_alpha(a)).0).collect();
            (k, alphas, det)
        })
        .collect();
    let per = cfg.alphas() * 2 * cfg.runs;
    let runs: Vec<GraspResult> = (0..instances.len() * per)
        .into_par_iter()
        .map(|job| {
            let (f, j) = (job / per, job % per);
            let (a, m, r) = (j / (2 * cfg.runs), j / cfg.runs % 2, j % cfg.runs);
            let (inst, k) = (&instances[f], base[f].1[a]);
            let (iterations, time_limit) = match cfg.budget {
                Budget::Seconds(t) => (usize::MAX, Some(t)),
                Budget::Iterations(n) => (n, None),
            };
            let gcfg = GraspConfig { alpha: grid_alpha(k), randomization: cfg.methods[m], iterations, time_limit };
            let labels: Vec<u64> = if cfg.grid { vec![k as u64, m as u64, r as u64] } else { vec![m as u64, r as u64] };
            let res = grasp(inst, &gcfg, stream_seed(cfg.seed, &inst.name, &labels));
            res.solution.check(inst).map_err(|e| format!("{} alpha {} {}: {e}", inst.name, grid_alpha(k), cfg.methods[m].name()))?;
            Ok(res)
        })
        .collect::<Result<_, String>>()?;
    let mut runs = runs.into_iter();
    let timed: Vec<Timed> =
        base.into_iter().map(|(k, alphas, det)| Timed { k, alphas, det, runs: runs.by_ref().take(per).collect() }).collect();

    let chosen: Vec<(usize, Vec<Candidate>)> = timed
        .iter()
        .map(|t| {
            let mut list: Vec<Candidate> = Vec::new();
            for a in top_with_ties(&t.det.iter().map(|s| s.reward).collect::<Vec<_>>()) {
                list.push((Origin::Deterministic, t.alphas[a], &t.det[a]));
            }
            for m in 0..2 {
                for (a, &k) in t.alphas.iter().enumerate() {
                    let group = t.group(a, m, cfg.runs);
                    for r in top_with_ties(&group.iter().map(|g| g.solution.reward).collect::<Vec<_>>()) {
                        list.push((Origin::Grasp { m, r }, k, &group[r].solution));
                    }
                }
            }
            let selected = drop_duplicates(&mut list);
            (selected, list)
        })
        .collect();
    let ls = search_all(instances, chosen)?;
    Ok((timed, ls, start.elapsed().as_secs_f64()))
}

/// "mejor" / "igual" / "peor" for a against b, with the difference.
fn versus(a: f64, b: f64) -> String {
    if a > b {
        format!("mejor (+{})", a - b)
    } else if a == b {
        "igual".into()
    } else {
        format!("peor ({})", a - b)
    }
}

/// Everything the report compares on one instance.
struct Row<'a> {
    reference: &'a Reference,
    inst: &'a Instance,
    timed: &'a Timed,
    elite: &'a InstanceElite,
    grid_no_ls: f64,
    grid_ls: f64,
    /// Timed experiment with the best alpha (mode best), when comparing mode grid with it.
    best_1s: Option<(f64, f64)>,
}

impl Row<'_> {
    fn final_ls(&self) -> f64 {
        self.elite.elite[self.elite.best()].after.reward
    }

    /// Mean reward of one run of randomisation m with the best deterministic alpha (no
    /// local search): what a single run like Panadero's 1 thread gives.
    fn mean_run(&self, m: usize, runs: usize) -> f64 {
        self.timed.group(self.timed.best_pos(), m, runs).iter().map(|g| g.solution.reward).sum::<f64>() / runs as f64
    }
}

/// Distance to the BKS in %: over the total, and the mean over the instances (the paper's).
fn gaps(rows: &[Row], v: &dyn Fn(&Row) -> f64) -> (f64, f64, f64) {
    let total: f64 = rows.iter().map(v).sum();
    let bks: f64 = rows.iter().map(|r| r.reference.bks).sum();
    let mean = rows.iter().map(|r| 100.0 * (r.reference.bks - v(r)) / r.reference.bks).sum::<f64>() / rows.len() as f64;
    (total, 100.0 * (bks - total) / bks, mean)
}

/// <stem>.csv (one row per instance) and <stem>_runs.csv (one row per GRASP run).
pub fn timed_csvs(instances: &[Instance], timed: &[Timed], ls: &LsResult, cfg: &TimedConfig) -> (String, String) {
    let mut per = String::from(
        "instance,best_k,best_alpha,deterministic,best_no_ls,best_ls,origin,k,alpha,rep,reward_before_ls,inserted,mean_iterations,elite_selected,elite_kept\n",
    );
    let mut runs = String::from("instance,k,alpha,randomization,param,rep,budget,iterations,best_iteration,reward,seconds\n");
    for ((inst, t), ie) in instances.iter().zip(timed).zip(&ls.instances) {
        let b = &ie.elite[ie.best()];
        let (origin, rep) = origin_text(b.origin, &cfg.methods);
        per += &format!(
            "{},{},{:.2},{},{},{},{origin},{},{:.2},{rep},{},{},{:.1},{},{}\n",
            inst.name,
            t.k,
            grid_alpha(t.k),
            t.det[t.best_pos()].reward,
            t.no_ls(),
            b.after.reward,
            b.k,
            grid_alpha(b.k),
            b.before.reward,
            b.stats.inserted,
            t.mean_iterations(),
            ie.selected,
            ie.elite.len()
        );
        for (i, g) in t.runs.iter().enumerate() {
            let (k, method) = (t.alphas[i / (2 * cfg.runs)], cfg.methods[i / cfg.runs % 2]);
            runs += &format!(
                "{},{k},{:.2},{},{},{},{},{},{},{},{:.4}\n",
                inst.name,
                grid_alpha(k),
                method.name(),
                method.param(),
                i % cfg.runs,
                cfg.budget.tag(),
                g.iterations,
                g.best_iteration,
                g.solution.reward,
                g.seconds
            );
        }
    }
    (per, runs)
}

/// Inputs of the report besides the experiment itself.
pub struct ReportInputs<'a> {
    pub refs: &'a [Reference],
    pub grid: &'a HashMap<String, (f64, f64)>,
    /// Results of another run of this command (e.g. results/panadero_1s.csv), to compare with.
    pub best_1s: Option<&'a HashMap<String, (f64, f64)>>,
    /// File name of best_1s, for the labels.
    pub best_name: &'a str,
    pub seconds: f64,
    pub cmd: &'a str,
}

/// Write the Markdown report comparing the paper with our grid and our timed experiment.
pub fn write_comparison(instances: &[Instance], timed: &[Timed], ls: &LsResult, cfg: &TimedConfig, input: &ReportInputs, report: &Path) -> Result<(), String> {
    let rows: Vec<Row> = input
        .refs
        .iter()
        .zip(instances)
        .zip(timed)
        .zip(&ls.instances)
        .map(|(((reference, inst), t), ie)| {
            let &(grid_no_ls, grid_ls) = input.grid.get(&reference.name).ok_or(format!("{} is not in best_solution.csv", reference.name))?;
            let best_1s = match input.best_1s {
                Some(m) => Some(*m.get(&reference.name).ok_or(format!("{} is not in the best-alpha CSV", reference.name))?),
                None => None,
            };
            Ok(Row { reference, inst, timed: t, elite: ie, grid_no_ls, grid_ls, best_1s })
        })
        .collect::<Result<_, String>>()?;
    let with_best = input.best_1s.is_some();
    let above: Vec<&str> = rows
        .iter()
        .filter(|r| r.final_ls() > r.reference.bks || r.grid_ls > r.reference.bks)
        .map(|r| r.reference.name.as_str())
        .collect();
    let count = |v: &dyn Fn(&Row) -> bool| rows.iter().filter(|r| v(r)).count();
    let n = rows.len();
    let (g0, g1) = (cfg.methods[0].name(), cfg.methods[1].name());
    let budget = cfg.budget.label();
    let ts = if cfg.grid { format!("{budget}, rejilla de α") } else { format!("{budget}, mejor α") };
    let tb = format!("`{}`", input.best_name);
    let run_text = match cfg.budget {
        Budget::Seconds(t) => format!(
            "cada una iterando durante {t} s de reloj**. Las {} ejecuciones se reparten entre {} hilos en un equipo de 6 núcleos \
             físicos (Ryzen 5 5500U), así que cada ejecución hace menos iteraciones que si estuviera sola. Por usar tiempo de \
             reloj, este experimento **no es reproducible exactamente**: el número de iteraciones depende de la carga de la \
             máquina.",
            n * cfg.alphas() * 2 * cfg.runs,
            rayon::current_num_threads()
        ),
        Budget::Iterations(k) => format!(
            "cada una de {k} iteraciones** (construcciones aleatorias). Con un número fijo de iteraciones el experimento es \
             reproducible."
        ),
    };

    let mut md = String::new();
    let mut w = |s: String| {
        md += &s;
        md.push('\n');
    };
    w(format!("# Comparación con Panadero et al. (2021), conjuntos p1 y p2: {ts}"));
    w(String::new());
    w(format!("Informe generado por `pjs` (`TOP/src/rust`); se regenera con\n\n```\ncd TOP\n{}\n```", input.cmd));
    w(String::new());
    w("## Qué se compara".into());
    w(String::new());
    let alpha_text = if cfg.grid {
        format!("**cada uno de los {ALPHAS} α de la rejilla**")
    } else {
        "el mejor α determinista de la rejilla (el menor si empatan)".to_string()
    };
    let elite_text = if cfg.grid {
        "las deterministas de las 3 mejores α y, para cada α y aleatorización, las 3 mejores ejecuciones"
    } else {
        "la determinista de ese α y las 3 mejores ejecuciones de cada aleatorización"
    };
    let seeds_text = if cfg.grid {
        " Cada ejecución usa la misma semilla que en la rejilla, así que empieza igual que la de 10 iteraciones y sigue iterando."
    } else {
        ""
    };
    w(format!(
        "Solo las {n} instancias de las tablas 3 y 4 de Panadero et al. (2021), extraídas en `src/data/panadero2021_p1_p2.csv`.\n\n\
         - **BKS:** la mejor solución conocida de la literatura (columna [a] de las tablas).\n\
         - **Panadero, 1 hilo / 128 hilos:** su algoritmo (ahorros con aleatorización sesgada, multiarranque), en el que cada \
         hilo itera durante 1 s; con 128 hilos se queda con el mejor (*OBS*).\n\
         - **Rejilla:** nuestro experimento principal ([`informe_busqueda_local.md`](informe_busqueda_local.md)): 21 α, \
         10 ejecuciones de 10 iteraciones por α y aleatorización, élite y búsqueda local.\n\
         - **{ts}:** para cada instancia, {alpha_text}, con **{runs} ejecuciones de GRASP {g0} (β = {beta}) y {runs} {g1} \
         (τ = {tau}) por α, {run_text}{seeds_text} Después, élite con las mismas reglas que la rejilla ({elite_text}; con \
         empates y sin duplicados) y búsqueda local (2-opt + inserción). Tardó {seconds:.0} s.\n\
         - **Origen:** de qué élite sale la solución final; si una ejecución de GRASP repite una determinista, cuenta como \
         determinista.\n\
         - **¿La búsqueda local mejora?** Sí, si la final supera a la mejor sin búsqueda local de la instancia. *Antes* e \
         *Insertados* se refieren a la élite de la que sale la final.",
        runs = cfg.runs,
        beta = cfg.methods[0].param(),
        tau = cfg.methods[1].param(),
        seconds = input.seconds
    ));
    w(String::new());

    w("## Resumen".into());
    w(String::new());
    w("| | Recompensa total | Distancia al BKS (sobre el total) | Distancia media al BKS por instancia (la del paper) |".into());
    w("|---|---|---|---|".into());
    let mut lines: Vec<(String, Box<dyn Fn(&Row) -> f64>)> = vec![
        ("Panadero, 1 hilo".into(), Box::new(|r| r.reference.thr1)),
        ("Panadero, 128 hilos".into(), Box::new(|r| r.reference.thr128)),
        ("Rejilla: sin búsqueda local".into(), Box::new(|r| r.grid_no_ls)),
        ("Rejilla: final (con búsqueda local)".into(), Box::new(|r| r.grid_ls)),
    ];
    if with_best {
        lines.push((format!("{tb}: sin búsqueda local"), Box::new(|r| r.best_1s.unwrap().0)));
        lines.push((format!("{tb}: final (con búsqueda local)"), Box::new(|r| r.best_1s.unwrap().1)));
    }
    lines.extend::<[(String, Box<dyn Fn(&Row) -> f64>); 4]>([
        (format!("{ts}: una ejecución {g0} con el mejor α (media de {}), sin búsqueda local", cfg.runs), Box::new(|r| r.mean_run(0, cfg.runs))),
        (format!("{ts}: una ejecución {g1} con el mejor α (media de {}), sin búsqueda local", cfg.runs), Box::new(|r| r.mean_run(1, cfg.runs))),
        (format!("{ts}: mejor de las {} ejecuciones, sin búsqueda local", cfg.alphas() * 2 * cfg.runs), Box::new(|r| r.timed.no_ls())),
        (format!("**{ts}: final (con búsqueda local)**"), Box::new(|r| r.final_ls())),
    ]);
    w(format!("| BKS | {} | — | — |", rows.iter().map(|r| r.reference.bks).sum::<f64>()));
    for (name, v) in &lines {
        let (total, gt, gm) = gaps(&rows, v.as_ref());
        w(format!("| {name} | {} | {gt:.2} % | {gm:.2} % |", (total * 10.0).round() / 10.0));
    }
    w(String::new());
    w(format!(
        "*Una ejecución (media)* es lo que da, de media, una sola ejecución de {budget} con el mejor α (con 1 s es lo comparable \
         con Panadero con 1 hilo). La distancia media por instancia es la medida de la fila *Average* de las tablas del paper; \
         la del total da más peso a las instancias con más recompensa."
    ));
    w(String::new());
    let cmp_line = |name: &str, v: &dyn Fn(&Row) -> f64, against: &dyn Fn(&Row) -> f64| {
        format!(
            "- **{name}:** mejor en {}, igual en {} y peor en {} instancias.",
            count(&|r| v(r) > against(r)),
            count(&|r| v(r) == against(r)),
            count(&|r| v(r) < against(r))
        )
    };
    w(cmp_line(&format!("{ts} final frente a Panadero con 128 hilos"), &|r| r.final_ls(), &|r| r.reference.thr128));
    w(cmp_line(&format!("{ts} sin búsqueda local frente a Panadero con 128 hilos"), &|r| r.timed.no_ls(), &|r| r.reference.thr128));
    w(cmp_line("Rejilla final frente a Panadero con 128 hilos", &|r| r.grid_ls, &|r| r.reference.thr128));
    w(cmp_line(&format!("{ts} final frente a la rejilla final"), &|r| r.final_ls(), &|r| r.grid_ls));
    if with_best {
        w(cmp_line(&format!("{ts} final frente a {tb} final"), &|r| r.final_ls(), &|r| r.best_1s.unwrap().1));
    }
    w(format!(
        "- **BKS alcanzado:** {ts} final en {}, rejilla final en {}, Panadero con 128 hilos en {} de {n}. {}",
        count(&|r| r.final_ls() == r.reference.bks),
        count(&|r| r.grid_ls == r.reference.bks),
        count(&|r| r.reference.thr128 == r.reference.bks),
        if above.is_empty() { "Ninguna solución nuestra lo supera.".to_string() } else { format!("**Superan el BKS (revisar): {}.**", above.join(", ")) }
    ));
    w(format!(
        "- **La búsqueda local mejora** la mejor solución de {ts} en {} instancias; en {} de ellas es lo que hace alcanzar o superar \
         a Panadero con 128 hilos.",
        count(&|r| r.final_ls() > r.timed.no_ls()),
        count(&|r| r.final_ls() > r.timed.no_ls() && r.timed.no_ls() < r.reference.thr128 && r.final_ls() >= r.reference.thr128)
    ));
    let iters = |set: &str| -> f64 {
        let v: Vec<f64> = rows.iter().filter(|r| r.inst.set() == set).map(|r| r.timed.mean_iterations()).collect();
        v.iter().sum::<f64>() / v.len().max(1) as f64
    };
    if let Budget::Seconds(t) = cfg.budget {
        w(format!(
            "- **Iteraciones en {t} s** (media por ejecución, con los {} hilos ocupados): {:.0} en p1 y {:.0} en p2, frente a las 10 de la rejilla.",
            rayon::current_num_threads(),
            iters("p1"),
            iters("p2")
        ));
    }
    w(String::new());

    w(format!("## Según el origen de la solución final de {ts}"));
    w(String::new());
    w("| Origen | Instancias | La búsqueda local mejora | Mejor que Panadero 128 | Igual | Peor |".into());
    w("|---|---|---|---|---|---|".into());
    let origin_of = |r: &Row| r.elite.elite[r.elite.best()].origin;
    let groups: [(&str, Box<dyn Fn(Origin) -> bool>); 3] = [
        ("determinista", Box::new(|o| o == Origin::Deterministic)),
        (g0, Box::new(|o| matches!(o, Origin::Grasp { m: 0, .. }))),
        (g1, Box::new(|o| matches!(o, Origin::Grasp { m: 1, .. }))),
    ];
    for (name, keep) in &groups {
        let of = |v: &dyn Fn(&Row) -> bool| count(&|r| keep(origin_of(r)) && v(r));
        w(format!(
            "| {name} | {} | {} | {} | {} | {} |",
            of(&|_| true),
            of(&|r| r.final_ls() > r.timed.no_ls()),
            of(&|r| r.final_ls() > r.reference.thr128),
            of(&|r| r.final_ls() == r.reference.thr128),
            of(&|r| r.final_ls() < r.reference.thr128)
        ));
    }
    w(String::new());

    w("## Por instancia".into());
    w(String::new());
    w("*α* es el de la élite de la que sale la solución final.".into());
    w(String::new());
    let best_col = if with_best { format!(" {tb} final |") } else { String::new() };
    let best_sep = if with_best { "---|" } else { "" };
    w(format!(
        "| Instancia | BKS | Panadero 1 hilo | Panadero 128 hilos | Rejilla final |{best_col} {ts} sin búsqueda local | **{ts} final** | Origen | α | Ejecución | Antes | Insertados | ¿La búsqueda local mejora? | Frente a Panadero 128 | Frente a BKS | Iteraciones por ejecución |"
    ));
    w(format!("|---|---|---|---|---|{best_sep}---|---|---|---|---|---|---|---|---|---|---|"));
    for r in &rows {
        let b = &r.elite.elite[r.elite.best()];
        let (origin, rep) = origin_text(b.origin, &cfg.methods);
        let origin = if origin == "deterministic" { "determinista" } else { origin };
        let (fin, nols) = (r.final_ls(), r.timed.no_ls());
        let best_cell = r.best_1s.map(|b| format!(" {} |", b.1)).unwrap_or_default();
        w(format!(
            "| {} | {} | {} | {} | {} |{best_cell} {nols} | **{fin}** | {origin} | {:.2} | {} | {} | {} | {} | {} | {} | {:.0} |",
            r.reference.name,
            r.reference.bks,
            r.reference.thr1,
            r.reference.thr128,
            r.grid_ls,
            grid_alpha(b.k),
            if rep.is_empty() { "—".to_string() } else { rep },
            b.before.reward,
            b.stats.inserted,
            if fin > nols { format!("sí (+{})", fin - nols) } else { "no".into() },
            versus(fin, r.reference.thr128),
            if fin == r.reference.bks { "igual".to_string() } else { format!("{}", fin - r.reference.bks) },
            r.timed.mean_iterations()
        ));
    }
    w(String::new());
    w(format!(
        "Ganancia total de {ts} final sobre la rejilla final: {:+.2} %.",
        gain(rows.iter().map(|r| r.final_ls()).sum(), rows.iter().map(|r| r.grid_ls).sum())
    ));
    write_file(report, &md)?;
    println!("-> {}", report.display());
    Ok(())
}
