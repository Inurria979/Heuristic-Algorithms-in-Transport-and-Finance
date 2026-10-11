//! Markdown report (in Spanish) on the local search over the elite solutions of the grid:
//! best solution of every instance, gain of GRASP over the deterministic heuristic and gain
//! of the local search over the same solutions without it.

use crate::alpha_grid::{grid_alpha, GridConfig, GridResult, ALPHAS};
use crate::elite::{grasp_best, origin_text, InstanceElite, LsResult, Origin};
use crate::io::write_file;
use crate::model::Instance;
use crate::report::{gain, relative_to};
use std::fs;
use std::path::Path;

/// Rewards of one instance that the report compares.
struct Summary {
    /// Best deterministic solution (best alpha), no local search.
    det: f64,
    /// Best GRASP run of any randomisation, no local search.
    grasp: f64,
    /// Best deterministic elite after the local search.
    det_ls: f64,
    /// Best elite of any origin after the local search.
    best_ls: f64,
}

impl Summary {
    fn of(ie: &InstanceElite, res: &GridResult, f: usize, cfg: &GridConfig) -> Summary {
        let is_det = |e: &crate::elite::Elite| e.origin == Origin::Deterministic;
        Summary {
            det: ie.max_reward(false, is_det),
            grasp: grasp_best(res, f, cfg),
            det_ls: ie.max_reward(true, is_det),
            best_ls: ie.elite[ie.best()].after.reward,
        }
    }

    /// Best without local search (GRASP starts from the deterministic solution, so it is
    /// never below it, but the max keeps this true with 0 GRASP runs too).
    fn no_ls(&self) -> f64 {
        self.det.max(self.grasp)
    }
}

/// Mean of a list (0 if empty).
fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// Write the Markdown report; `routes_dir`, if given, holds the route plot of the best
/// solution of every instance.
pub fn write_ls_report(
    instances: &[Instance],
    res: &GridResult,
    ls: &LsResult,
    cfg: &GridConfig,
    report: &Path,
    routes_dir: Option<&Path>,
) -> Result<(), String> {
    let n = instances.len();
    let sums: Vec<Summary> = (0..n).map(|f| Summary::of(&ls.instances[f], res, f, cfg)).collect();
    let feasible: Vec<usize> = (0..n).filter(|&f| sums[f].det > 0.0).collect();
    let infeasible: Vec<&str> = (0..n).filter(|&f| sums[f].det == 0.0).map(|f| instances[f].name.as_str()).collect();
    let total = |v: &dyn Fn(&Summary) -> f64| -> f64 { feasible.iter().map(|&f| v(&sums[f])).sum() };
    let count = |v: &dyn Fn(&Summary) -> bool| feasible.iter().filter(|&&f| v(&sums[f])).count();
    let (det, grasp, det_ls, best_ls) = (total(&|s| s.det), total(&|s| s.no_ls()), total(&|s| s.det_ls), total(&|s| s.best_ls));
    let selected: usize = feasible.iter().map(|&f| ls.instances[f].selected).sum();
    let kept: usize = feasible.iter().map(|&f| ls.instances[f].elite.len()).sum();

    let report_dir = report.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    fs::create_dir_all(report_dir).map_err(|e| format!("{}: {e}", report_dir.display()))?;
    let link_routes = routes_dir.map(|d| relative_to(report_dir, d)).transpose()?;

    let mut md = String::new();
    let mut w = |s: String| {
        md += &s;
        md.push('\n');
    };
    w("# Búsqueda local sobre las soluciones élite de la rejilla".into());
    w(String::new());
    w("Informe generado por `pjs` (`TOP/src/rust`); se regenera con\n".into());
    w(format!(
        "```\ncd TOP\nignore/target/release/pjs.exe grid --reps {} --iters {} --geometric {} --softmax {} --seed {} \\\n    \
         --png-dir results/figures/routes --report docs/informe_alpha.md \\\n    \
         --ls-png-dir results/figures/routes_ls --ls-report docs/informe_busqueda_local.md src/data/top-data results\n```",
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
        "Las decisiones y el pseudocódigo están en [`busqueda_local_diseno.md`](busqueda_local_diseno.md). En resumen:\n\n\
         - **Rejilla** (la de [`informe_alpha.md`](informe_alpha.md)): por instancia y por cada uno de los {ALPHAS} α, \
         una construcción determinista y {reps} ejecuciones de GRASP de {iters} iteraciones con cada aleatorización \
         (geométrica β = {beta} y softmax τ = {tau}).\n\
         - **Élite por instancia:** las soluciones deterministas de las α con las 3 mejores recompensas y, para cada \
         aleatorización y cada α, las mejores soluciones de las 3 mejores ejecuciones. Solo cuenta la recompensa y los \
         empates en la tercera entran todos.\n\
         - **Duplicados:** dos élite son iguales si visitan el mismo conjunto de clientes; se conserva la primera en el \
         orden determinista → geométrica → softmax, α creciente, ejecución creciente, y la repetida no se sustituye. \
         Se eligieron {selected} élite en las {fea} instancias con ruta factible y quedaron **{kept}** tras quitar \
         duplicados ({:.1} por instancia).\n\
         - **Búsqueda local** sobre cada élite, en bucle: 2-opt dentro de cada ruta (primera mejora) y después insertar, \
         entre los clientes no visitados que caben en alguna ruta, el de mayor recompensa (en empate, el que menos tiempo \
         añade) en su posición más barata; se repite mientras se inserte alguno. Tarda {secs:.1} s en paralelo con rayon \
         y cada solución se comprueba de forma independiente.",
        kept as f64 / feasible.len().max(1) as f64,
        reps = cfg.reps,
        iters = cfg.iterations,
        beta = cfg.methods[0].param(),
        tau = cfg.methods[1].param(),
        fea = feasible.len(),
        secs = ls.seconds
    ));
    w(String::new());

    w("## Resultados globales".into());
    w(String::new());
    w(format!("Recompensa total de las {} instancias con alguna ruta factible:", feasible.len()));
    w(String::new());
    w("| | Búsqueda local | Recompensa total | Frente al determinista sin búsqueda local | Instancias que mejora |".into());
    w("|---|---|---|---|---|".into());
    w(format!("| Determinista, mejor α de cada instancia | no | {det} | — | — |"));
    w(format!(
        "| Mejor de GRASP (geométrica o softmax), mejor ejecución | no | {grasp} | +{:.2} % | {} |",
        gain(grasp, det),
        count(&|s| s.no_ls() > s.det)
    ));
    w(format!(
        "| Mejor élite determinista | sí | {det_ls} | +{:.2} % | {} |",
        gain(det_ls, det),
        count(&|s| s.det_ls > s.det)
    ));
    w(format!(
        "| **Mejor élite de cualquier origen (solución final)** | sí | **{best_ls}** | **+{:.2} %** | {} |",
        gain(best_ls, det),
        count(&|s| s.best_ls > s.det)
    ));
    w(String::new());
    w(format!(
        "- **GRASP frente a la determinista.** Sin búsqueda local, GRASP sube la recompensa total un **{:.2} %** y \
         mejora {} instancias. Con búsqueda local en las dos, añadir las élite de GRASP a las deterministas la sube un \
         **{:.2} %** (de {det_ls} a {best_ls}) y mejora {} instancias.\n\
         - **Búsqueda local frente a la misma solución sin ella.** Sobre las deterministas sube la recompensa un \
         **{:.2} %** ({} instancias mejoran). Sobre la mejor solución disponible sin búsqueda local (la de GRASP), la \
         solución final sube un **{:.2} %** ({} instancias mejoran).",
        gain(grasp, det),
        count(&|s| s.no_ls() > s.det),
        gain(best_ls, det_ls),
        count(&|s| s.best_ls > s.det_ls),
        gain(det_ls, det),
        count(&|s| s.det_ls > s.det),
        gain(best_ls, grasp),
        count(&|s| s.best_ls > s.no_ls())
    ));
    w(String::new());

    w("## Efecto de la búsqueda local en cada élite".into());
    w(String::new());
    w("Cada élite comparada consigo misma antes y después de la búsqueda local.".into());
    w(String::new());
    w("| Origen | Élite | Mejoradas | Ganancia media | Ganancia media de las mejoradas | Ganancia máxima | Clientes insertados (media) | Movimientos 2-opt (media) |".into());
    w("|---|---|---|---|---|---|---|---|".into());
    let origins: [(&str, Box<dyn Fn(Origin) -> bool>); 4] = [
        ("Determinista", Box::new(|o| o == Origin::Deterministic)),
        (cfg.methods[0].name(), Box::new(|o| matches!(o, Origin::Grasp { m: 0, .. }))),
        (cfg.methods[1].name(), Box::new(|o| matches!(o, Origin::Grasp { m: 1, .. }))),
        ("**Todas**", Box::new(|_| true)),
    ];
    for (name, keep) in &origins {
        let es: Vec<&crate::elite::Elite> = feasible.iter().flat_map(|&f| &ls.instances[f].elite).filter(|e| keep(e.origin)).collect();
        let gains: Vec<f64> = es.iter().map(|e| gain(e.after.reward, e.before.reward)).collect();
        let improved: Vec<f64> = gains.iter().copied().filter(|&g| g > 0.0).collect();
        w(format!(
            "| {name} | {} | {} ({:.1} %) | +{:.2} % | +{:.2} % | +{:.2} % | {:.2} | {:.2} |",
            es.len(),
            improved.len(),
            100.0 * improved.len() as f64 / es.len().max(1) as f64,
            mean(&gains),
            mean(&improved),
            gains.iter().copied().fold(0.0, f64::max),
            mean(&es.iter().map(|e| e.stats.inserted as f64).collect::<Vec<_>>()),
            mean(&es.iter().map(|e| e.stats.two_opt as f64).collect::<Vec<_>>())
        ));
    }
    w(String::new());

    w("## De dónde sale la mejor solución".into());
    w(String::new());
    let from = |keep: &dyn Fn(Origin) -> bool| feasible.iter().filter(|&&f| keep(ls.instances[f].elite[ls.instances[f].best()].origin)).count();
    // Best after the local search among the elite that were already the best before it.
    let from_top = |ie: &InstanceElite| {
        let top = ie.max_reward(false, |_| true);
        ie.max_reward(true, |e| e.before.reward == top)
    };
    let other = feasible.iter().filter(|&&f| sums[f].best_ls > from_top(&ls.instances[f])).count();
    w(format!(
        "- **Origen de la solución final** (la primera élite con la mayor recompensa tras la búsqueda local, en el orden \
         determinista → geométrica → softmax): determinista en {}, {} en {} y {} en {} instancias.\n\
         - **¿Bastaba con la mejor antes de la búsqueda local?** En **{other} de {}** instancias la solución final sale \
         de una élite que antes no era la mejor: aplicar la búsqueda local solo a la mejor solución habría dado menos. \
         En esas instancias, la diferencia suma {} de recompensa.",
        from(&|o| o == Origin::Deterministic),
        cfg.methods[0].name(),
        from(&|o| matches!(o, Origin::Grasp { m: 0, .. })),
        cfg.methods[1].name(),
        from(&|o| matches!(o, Origin::Grasp { m: 1, .. })),
        feasible.len(),
        feasible.iter().map(|&f| sums[f].best_ls - from_top(&ls.instances[f])).sum::<f64>()
    ));
    w(String::new());

    w("## Por conjunto".into());
    w(String::new());
    w("| Conjunto | Instancias | Determinista | GRASP | GRASP frente a determinista | Determinista + búsqueda local | Final | Búsqueda local (final frente a GRASP) | Final frente a determinista |".into());
    w("|---|---|---|---|---|---|---|---|---|".into());
    let mut sets: Vec<&str> = Vec::new();
    for &f in &feasible {
        if !sets.contains(&instances[f].set()) {
            sets.push(instances[f].set());
        }
    }
    for set in &sets {
        let fs_: Vec<usize> = feasible.iter().copied().filter(|&f| instances[f].set() == *set).collect();
        let t = |v: &dyn Fn(&Summary) -> f64| -> f64 { fs_.iter().map(|&f| v(&sums[f])).sum() };
        let (d, g, dl, b) = (t(&|s| s.det), t(&|s| s.no_ls()), t(&|s| s.det_ls), t(&|s| s.best_ls));
        w(format!(
            "| {set} | {} | {d} | {g} | +{:.2} % | {dl} | {b} | +{:.2} % | +{:.2} % |",
            fs_.len(),
            gain(g, d),
            gain(b, g),
            gain(b, d)
        ));
    }
    w(String::new());

    w("## Por instancia".into());
    w(String::new());
    w(
        "*Determinista* y *GRASP*: la mejor recompensa de la rejilla sin búsqueda local. *Final*: la mejor élite tras la \
         búsqueda local, con su origen, su α, su ejecución (solo GRASP), su recompensa antes de la búsqueda local y los \
         clientes que insertó. *Élite*: elegidas / que quedan tras quitar duplicados."
            .into(),
    );
    w(String::new());
    let route_col = if link_routes.is_some() { " Rutas |" } else { "" };
    let route_sep = if link_routes.is_some() { "---|" } else { "" };
    w(format!(
        "| Instancia | m | tmax | Determinista | GRASP | Determinista + búsqueda local | Final | Origen | α | Ejecución | Antes | Insertados | Élite |{route_col}"
    ));
    w(format!("|---|---|---|---|---|---|---|---|---|---|---|---|---|{route_sep}"));
    for &f in &feasible {
        let (inst, ie, s) = (&instances[f], &ls.instances[f], &sums[f]);
        let b = &ie.elite[ie.best()];
        let (origin, rep) = origin_text(b.origin, &cfg.methods);
        let png = link_routes.as_ref().map(|d| format!(" [png]({d}/{}.png) |", inst.name)).unwrap_or_default();
        w(format!(
            "| {} | {} | {} | {} | {} | {} | **{}** | {origin} | {:.2} | {rep} | {} | {} | {} / {} |{png}",
            inst.name,
            inst.m,
            inst.tmax,
            s.det,
            s.no_ls(),
            s.det_ls,
            s.best_ls,
            grid_alpha(b.k),
            b.before.reward,
            b.stats.inserted,
            ie.selected,
            ie.elite.len()
        ));
    }
    w(String::new());
    w(format!("Instancias sin ninguna ruta factible ({}): {}.", infeasible.len(), infeasible.join(", ")));
    write_file(report, &md)?;
    println!("-> {}", report.display());
    Ok(())
}
