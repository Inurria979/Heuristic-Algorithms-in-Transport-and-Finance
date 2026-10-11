//! Markdown report (in Spanish) on the alpha grid experiment: best deterministic alpha of
//! every instance and the GRASP runs of every alpha, with its figures.

use crate::alpha_grid::{best_grasp, grid_alpha, Best, GridConfig, GridResult, ALPHAS, GRID_TEXT};
use crate::io::write_file;
use crate::model::Instance;
use crate::plot::{plot_heatmap, plot_total_reward};
use std::fs;
use std::path::Path;

/// Median of a list of numbers (0 for an empty list).
pub fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    let h = v.len() / 2;
    if v.len() % 2 == 1 {
        v[h]
    } else {
        (v[h - 1] + v[h]) / 2.0
    }
}

/// Path of `target` relative to the folder `base`, with forward slashes (for Markdown links).
pub fn relative_to(base: &Path, target: &Path) -> Result<String, String> {
    let canon = |p: &Path| fs::canonicalize(p).map_err(|e| format!("{}: {e}", p.display()));
    let (base, target) = (canon(base)?, canon(target)?);
    let (b, t): (Vec<_>, Vec<_>) = (base.components().collect(), target.components().collect());
    let common = b.iter().zip(&t).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<String> = vec!["..".into(); b.len() - common];
    parts.extend(t[common..].iter().map(|c| c.as_os_str().to_string_lossy().into_owned()));
    Ok(parts.join("/"))
}

/// Percentage that `a` falls short of `b`.
fn pct(a: f64, b: f64) -> f64 {
    if b > 0.0 {
        100.0 * (b - a) / b
    } else {
        0.0
    }
}

/// Percentage that `a` exceeds `b`.
pub fn gain(a: f64, b: f64) -> f64 {
    if b > 0.0 {
        100.0 * (a - b) / b
    } else {
        0.0
    }
}

/// Index of the largest value (the first one on ties).
fn argmax(v: &[f64]) -> usize {
    (0..v.len()).fold(0, |b, k| if v[k] > v[b] { k } else { b })
}

/// Write the Markdown report on the alpha grid experiment, with its two figures in
/// `fig_dir` and, if given, a link to the route plot of every instance in `routes_dir`.
/// `res.runs[f * ALPHAS + k]` is instance f with alpha_k and `bests[f]` its best
/// deterministic alpha.
pub fn write_report(
    instances: &[Instance],
    res: &GridResult,
    bests: &[Best],
    cfg: &GridConfig,
    report: &Path,
    fig_dir: &Path,
    routes_dir: Option<&Path>,
) -> Result<(), String> {
    let runs = &res.runs;
    let row = |f: usize| &runs[f * ALPHAS..(f + 1) * ALPHAS];
    let best_reward = |f: usize| runs[f * ALPHAS + bests[f].k].reward;
    let n = instances.len();
    let feasible: Vec<usize> = (0..n).filter(|&f| best_reward(f) > 0.0).collect();
    let infeasible: Vec<&str> = (0..n).filter(|&f| best_reward(f) == 0.0).map(|f| instances[f].name.as_str()).collect();
    let matters: Vec<usize> = feasible.iter().copied().filter(|&f| bests[f].optimal < ALPHAS).collect();
    let oracle: f64 = (0..n).map(best_reward).sum();
    let det_totals: Vec<f64> = (0..ALPHAS).map(|k| (0..n).map(|f| runs[f * ALPHAS + k].reward).sum()).collect();
    let k_star = argmax(&det_totals);
    let star_optimal = feasible.iter().filter(|&&f| runs[f * ALPHAS + k_star].reward == best_reward(f)).count();
    let det_gain = |f: usize| pct(bests[f].worst, best_reward(f));
    let with_grasp = cfg.reps > 0;
    // Per instance and randomisation: (best run, its alpha, best mean of one alpha, its alpha).
    let g: Vec<[(f64, f64, f64, f64); 2]> = (0..n).map(|f| [best_grasp(row(f), 0), best_grasp(row(f), 1)]).collect();
    let mean_totals = |m: usize| -> Vec<f64> { (0..ALPHAS).map(|k| (0..n).map(|f| runs[f * ALPHAS + k].grasp[m].mean).sum()).collect() };

    fs::create_dir_all(fig_dir).map_err(|e| format!("{}: {e}", fig_dir.display()))?;
    let (fig_total, fig_heat) = (fig_dir.join("alpha_total_reward.png"), fig_dir.join("alpha_heatmap.png"));
    let mut series = vec![("deterministic".to_string(), det_totals.clone())];
    let mut references = vec![("deterministic, best alpha per instance".to_string(), oracle)];
    if with_grasp {
        for (m, method) in cfg.methods.iter().enumerate() {
            series.push((format!("GRASP {}, mean of {} runs", method.name(), cfg.reps), mean_totals(m)));
            references.push((format!("GRASP {}, best run per instance", method.name()), (0..n).map(|f| g[f][m].0).sum()));
        }
    }
    plot_total_reward(&series, &references, &fig_total)?;
    let names: Vec<&str> = feasible.iter().map(|&f| instances[f].name.as_str()).collect();
    let rel: Vec<Vec<f64>> = feasible.iter().map(|&f| row(f).iter().map(|r| r.reward / best_reward(f)).collect()).collect();
    plot_heatmap(&names, &rel, &fig_heat)?;
    let report_dir = report.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    fs::create_dir_all(report_dir).map_err(|e| format!("{}: {e}", report_dir.display()))?;
    let (link_total, link_heat) = (relative_to(report_dir, &fig_total)?, relative_to(report_dir, &fig_heat)?);
    let link_routes = routes_dir.map(|d| relative_to(report_dir, d)).transpose()?;

    let mut md = String::new();
    let mut w = |s: String| {
        md += &s;
        md.push('\n');
    };
    w("# Rejilla de α: heurística PJS determinista y GRASP".into());
    w(String::new());
    w("Informe generado por `pjs` (`TOP/src/rust`); se regenera con\n".into());
    w(format!(
        "```\ncd TOP\nignore/target/release/pjs.exe grid --reps {} --iters {} --geometric {} --softmax {} --seed {} \\\n    \
         --png-dir results/figures/routes --report docs/informe_alpha.md src/data/top-data results\n```",
        cfg.reps,
        cfg.iterations,
        cfg.methods[0].param(),
        cfg.methods[1].param(),
        cfg.seed
    ));
    w(String::new());
    w("## Método".into());
    w(String::new());
    w(format!(
        "La heurística de ahorros de Panadero et al. (2021, sección 4) construye una ruta \
         *dummy* origen → i → destino por cliente (descartando los que ni solos caben en `tmax`), \
         ordena los arcos por el ahorro enriquecido s'ᵢⱼ = α·sᵢⱼ + (1 − α)·(uᵢ + uⱼ), fusiona rutas \
         siguiendo esa lista mientras no se supere `tmax` y se queda con las m rutas de más \
         recompensa. El paper no da ningún valor de α, así que cada instancia se resuelve con los \
         **{ALPHAS} valores α = {GRID_TEXT}**."
    ));
    w(String::new());
    w(format!(
        "- **Determinista:** una construcción por α, tomando siempre el arco de mayor ahorro.\n\
         - **GRASP** (solo construcción, sin búsqueda local): para cada α, {reps} ejecuciones con la \
         **geométrica** (β = {beta}) y {reps} con el **softmax** (τ = {tau}). Cada ejecución sigue el \
         multiarranque de Li et al. (2022, Algoritmo 1) y cada hilo del Algoritmo 1 de Panadero: parte de \
         la solución determinista y hace **{iters} construcciones aleatorias**, quedándose con la mejor. \
         Cada ejecución tiene su propia semilla (derivada de la semilla base {seed}, la instancia, α, la \
         aleatorización y el número de ejecución), así que sus resultados son distintos entre sí y el \
         experimento es reproducible.\n\
         - **Geométrica:** el siguiente arco es el k-ésimo de los que quedan, con k ~ Geom(β). \
         **Softmax:** cada arco que queda se elige con probabilidad exp(s'/τ) / Σ exp(s'/τ).\n\
         - **Criterio:** solo cuenta la recompensa total; las rutas solo tienen que respetar `tmax`.\n\
         - **Ejecución:** las {tasks} tareas (instancia, α, construcción determinista o ejecución de GRASP) \
         se reparten en paralelo con rayon ({secs:.0} s). Cada solución se comprueba de forma independiente \
         (como mucho m rutas, ningún cliente repetido, cada ruta dentro de `tmax`, tiempos y recompensas \
         recalculados).\n\
         - **Datos:** {n} instancias de Chao et al. (1996). En {inf} de ellas ningún cliente es alcanzable \
         ni con una ruta de un solo cliente, así que su recompensa es 0; el análisis usa las {fea} restantes.",
        reps = cfg.reps,
        beta = cfg.methods[0].param(),
        tau = cfg.methods[1].param(),
        iters = cfg.iterations,
        seed = cfg.seed,
        tasks = n * ALPHAS * (1 + 2 * cfg.reps),
        secs = res.seconds,
        inf = infeasible.len(),
        fea = feasible.len()
    ));
    w(String::new());
    w("## Resultados globales".into());
    w(String::new());
    w(format!(
        "**Determinista.** α influye en **{} de {}** instancias. Con el mejor α de cada instancia la \
         recompensa total es **{oracle}**; el mejor α común a todas es **α = {:.2}**, que da {} \
         ({:.2} % menos) y solo es óptimo en {star_optimal} instancias.",
        matters.len(),
        feasible.len(),
        grid_alpha(k_star),
        det_totals[k_star],
        pct(det_totals[k_star], oracle)
    ));
    w(String::new());
    if with_grasp {
        w("| | Recompensa total | Frente al determinista con su mejor α | Instancias que mejora |".into());
        w("|---|---|---|---|".into());
        w(format!("| Determinista, mejor α de cada instancia | {oracle} | — | — |"));
        for (m, method) in cfg.methods.iter().enumerate() {
            let best: f64 = (0..n).map(|f| g[f][m].0).sum();
            let mean: f64 = (0..n).map(|f| g[f][m].2).sum();
            let improved = feasible.iter().filter(|&&f| g[f][m].0 > best_reward(f)).count();
            let improved_mean = feasible.iter().filter(|&&f| g[f][m].2 > best_reward(f)).count();
            let totals = mean_totals(m);
            let km = argmax(&totals);
            w(format!(
                "| GRASP {}, mejor ejecución de cada instancia | {best} | +{:.2} % | {improved} |",
                method.name(),
                gain(best, oracle)
            ));
            w(format!(
                "| GRASP {}, mejor media de {} ejecuciones (mejor α de cada instancia) | {mean:.1} | +{:.2} % | {improved_mean} |",
                method.name(),
                cfg.reps,
                gain(mean, oracle)
            ));
            w(format!(
                "| GRASP {}, media de {} ejecuciones con el mejor α común (α = {:.2}) | {:.1} | {:+.2} % | — |",
                method.name(),
                cfg.reps,
                grid_alpha(km),
                totals[km],
                gain(totals[km], oracle)
            ));
        }
        w(String::new());
        w(
            "*Mejor ejecución*: la mayor recompensa de cualquier ejecución de GRASP de la instancia, sobre todos \
             los α. *Mejor media*: para cada α, la media de sus ejecuciones; se toma el α con la media más alta. \
             La media es lo que cabe esperar de **una** ejecución de GRASP con ese α; la mejor ejecución, lo que \
             se obtiene lanzando todas y quedándose con la mejor."
                .into(),
        );
        w(String::new());
    }
    w(format!("![Recompensa total frente a α]({link_total})"));
    w(String::new());
    w(
        "Cada línea es la recompensa total de todas las instancias si todas usan el mismo α (para GRASP, la \
         media de sus ejecuciones); las discontinuas, lo que se obtiene eligiendo lo mejor de cada instancia."
            .into(),
    );
    w(String::new());
    w(format!("![Mapa de recompensa relativa]({link_heat})"));
    w(String::new());
    w(
        "Heurística determinista: cada fila es una instancia y cada columna un α; el azul más oscuro marca los \
         α óptimos. Las zonas óptimas cambian de una instancia a otra y a menudo están partidas: pequeños \
         cambios de α reordenan la lista de ahorros y la heurística voraz construye rutas distintas."
            .into(),
    );
    w(String::new());
    w("## Por conjunto".into());
    w(String::new());
    let mut sets: Vec<&str> = Vec::new();
    for inst in instances {
        if !sets.contains(&inst.set()) {
            sets.push(inst.set());
        }
    }
    let members = |set: &str| -> Vec<usize> { (0..n).filter(|&f| instances[f].set() == set).collect() };
    w("### Determinista".into());
    w(String::new());
    w("| Conjunto | Instancias | Con ruta factible | α influye | Mediana del menor α óptimo (donde α influye) | Recompensa (mejor α por instancia) | Mejor α común del conjunto | Recompensa con él | Pérdida |".into());
    w("|---|---|---|---|---|---|---|---|---|".into());
    for set in &sets {
        let all = members(set);
        let fs_: Vec<usize> = all.iter().copied().filter(|f| feasible.contains(f)).collect();
        let ms: Vec<usize> = fs_.iter().copied().filter(|&f| bests[f].optimal < ALPHAS).collect();
        let total: f64 = all.iter().map(|&f| best_reward(f)).sum();
        let set_totals: Vec<f64> = (0..ALPHAS).map(|k| all.iter().map(|&f| runs[f * ALPHAS + k].reward).sum()).collect();
        let ks = argmax(&set_totals);
        w(format!(
            "| {set} | {} | {} | {} | {:.2} | {total} | {:.2} | {} | {:.2} % |",
            all.len(),
            fs_.len(),
            ms.len(),
            median(ms.iter().map(|&f| grid_alpha(bests[f].k)).collect()),
            grid_alpha(ks),
            set_totals[ks],
            pct(set_totals[ks], total)
        ));
    }
    w(String::new());
    if with_grasp {
        w("### GRASP".into());
        w(String::new());
        let (g0, g1) = (cfg.methods[0].name(), cfg.methods[1].name());
        w(format!(
            "| Conjunto | Determinista (mejor α) | {g0}: mejor ejecución | {g0}: mejor media | {g1}: mejor ejecución | {g1}: mejor media |"
        ));
        w("|---|---|---|---|---|---|".into());
        for set in &sets {
            let all = members(set);
            let det: f64 = all.iter().map(|&f| best_reward(f)).sum();
            let col = |m: usize, i: usize| -> f64 { all.iter().map(|&f| if i == 0 { g[f][m].0 } else { g[f][m].2 }).sum() };
            w(format!(
                "| {set} | {det} | {} (+{:.2} %) | {:.1} (+{:.2} %) | {} (+{:.2} %) | {:.1} (+{:.2} %) |",
                col(0, 0),
                gain(col(0, 0), det),
                col(0, 1),
                gain(col(0, 1), det),
                col(1, 0),
                gain(col(1, 0), det),
                col(1, 1),
                gain(col(1, 1), det)
            ));
        }
        w(String::new());
    }
    w("## Por instancia".into());
    w(String::new());
    w(
        "Instancias con alguna ruta factible. *Menor α óptimo* es el menor α con la mayor recompensa \
         determinista (el de la figura de rutas) y *α óptimos* cuántos de la rejilla la alcanzan. Para GRASP, \
         cada celda es la recompensa y, entre paréntesis, su α."
            .into(),
    );
    w(String::new());
    let route_col = if link_routes.is_some() { " Rutas |" } else { "" };
    let route_sep = if link_routes.is_some() { "---|" } else { "" };
    if with_grasp {
        let (g0, g1) = (cfg.methods[0].name(), cfg.methods[1].name());
        w(format!(
            "| Instancia | m | tmax | Menor α óptimo | α óptimos | Determinista | Peor α | {g0}: mejor ejecución | {g0}: mejor media | {g1}: mejor ejecución | {g1}: mejor media |{route_col}"
        ));
        w(format!("|---|---|---|---|---|---|---|---|---|---|---|{route_sep}"));
    } else {
        w(format!("| Instancia | m | tmax | Menor α óptimo | α óptimos | Determinista | Peor α | Mejora |{route_col}"));
        w(format!("|---|---|---|---|---|---|---|---|{route_sep}"));
    }
    for &f in &feasible {
        let (inst, b) = (&instances[f], &bests[f]);
        let png = link_routes.as_ref().map(|d| format!(" [png]({d}/{}.png) |", inst.name)).unwrap_or_default();
        let head = format!(
            "| {} | {} | {} | {:.2} | {} | {} | {} |",
            inst.name,
            inst.m,
            inst.tmax,
            grid_alpha(b.k),
            b.optimal,
            best_reward(f),
            b.worst
        );
        if with_grasp {
            let [a, s] = g[f];
            w(format!(
                "{head} {} ({:.2}) | {:.1} ({:.2}) | {} ({:.2}) | {:.1} ({:.2}) |{png}",
                a.0, a.1, a.2, a.3, s.0, s.1, s.2, s.3
            ));
        } else {
            w(format!("{head} {:.2} % |{png}", det_gain(f)));
        }
    }
    w(String::new());
    w(format!("Instancias sin ninguna ruta factible ({}): {}.", infeasible.len(), infeasible.join(", ")));
    write_file(report, &md)?;
    println!("-> {}\n-> {}\n-> {}", report.display(), fig_total.display(), fig_heat.display());
    Ok(())
}
