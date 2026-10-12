//! Comparison of the grid (results/best_solution.csv) with the best known solution (BKS) of
//! every instance, per set, next to the mean gaps of Panadero et al. (2021), Figure 4.
//!
//! BKS: src/data/bks_chao.csv, column BEST of MA10-PSOMA-PSOiA-Chao-benchmark.xls, published
//! with Dang, Guibadj and Moukrim (PSOiA) at utc.fr/~moukrim (archive kept in
//! ignore/downloads/solutions-psoia.zip). On the 80 instances of Panadero's Tables 3 and 4 it
//! equals their BKS (Ke et al. 2016).
//!
//! Writes a CSV per set and a Markdown report (in Spanish).

use crate::io::write_file;
use crate::panadero::read_csv;
use std::collections::HashMap;
use std::path::Path;

/// Panadero et al. (2021), Figure 4: mean gap to the BKS per set, 128 threads of 1 s.
pub const PANADERO_FIG4: [(&str, f64); 7] =
    [("p1", 0.69), ("p2", 0.11), ("p3", 1.44), ("p4", 7.97), ("p5", 1.46), ("p6", 1.42), ("p7", 5.10)];

/// Our three levels, as columns of best_solution.csv.
const LEVELS: [&str; 3] = ["deterministic", "best_no_ls", "best_ls"];

/// Results of one set (or of all of them) against the BKS.
#[derive(Default)]
pub struct SetRow {
    pub set: String,
    /// Instances with BKS > 0.
    pub n: usize,
    /// Mean gap to the BKS per instance, in %, for each level.
    pub gap: [f64; 3],
    /// Instances where each level reaches the BKS.
    pub hit: [usize; 3],
    /// Instances where each level is above the BKS (should never happen).
    pub above: [usize; 3],
    /// Sum of the BKS and of each level's reward.
    pub bks_total: f64,
    pub total: [f64; 3],
    /// Panadero's mean gap with 128 threads (Figure 4); for "all", weighted by instances.
    pub panadero: f64,
}

impl SetRow {
    fn add(&mut self, bks: f64, ours: &[f64]) {
        self.n += 1;
        self.bks_total += bks;
        for l in 0..3 {
            self.gap[l] += 100.0 * (bks - ours[l]) / bks;
            self.hit[l] += usize::from(ours[l] == bks);
            self.above[l] += usize::from(ours[l] > bks);
            self.total[l] += ours[l];
        }
    }

    fn finish(&mut self) {
        for g in &mut self.gap {
            *g /= self.n as f64;
        }
    }
}

/// Per-set rows (p1..p7) and a last row "all" over every instance with a BKS.
pub fn compare(bks_csv: &Path, best_csv: &Path) -> Result<Vec<SetRow>, String> {
    let bks: HashMap<String, f64> = read_csv(bks_csv, &["bks"])?.into_iter().map(|(n, v)| (n, v[0])).collect();
    let ours = read_csv(best_csv, &LEVELS)?;
    let mut rows: Vec<SetRow> = PANADERO_FIG4.iter().map(|&(s, p)| SetRow { set: s.to_string(), panadero: p, ..SetRow::default() }).collect();
    let mut all = SetRow { set: "all".to_string(), ..SetRow::default() };
    let mut matched = 0;
    for (name, v) in &ours {
        let Some(&b) = bks.get(name) else { continue };
        matched += 1;
        let set = name.split('.').next().unwrap_or_default();
        let row = rows.iter_mut().find(|r| r.set == set).ok_or(format!("{name}: unknown set"))?;
        row.add(b, v);
        all.add(b, v);
    }
    if matched != bks.len() {
        return Err(format!("{} instances of {} are missing from {}", bks.len() - matched, bks_csv.display(), best_csv.display()));
    }
    for r in &mut rows {
        r.finish();
    }
    all.finish();
    all.panadero = rows.iter().map(|r| r.panadero * r.n as f64).sum::<f64>() / all.n as f64;
    rows.push(all);
    Ok(rows)
}

/// bks_by_set.csv: one row per set and a last row "all".
pub fn bks_csv(rows: &[SetRow]) -> String {
    let mut csv = String::from(
        "set,instances,bks_total,gap_deterministic,gap_no_ls,gap_final,gap_panadero128,bks_hit_deterministic,bks_hit_no_ls,bks_hit_final,above_bks_final\n",
    );
    for r in rows {
        csv += &format!(
            "{},{},{},{:.4},{:.4},{:.4},{:.4},{},{},{},{}\n",
            r.set, r.n, r.bks_total, r.gap[0], r.gap[1], r.gap[2], r.panadero, r.hit[0], r.hit[1], r.hit[2], r.above[2]
        );
    }
    csv
}

/// Markdown report (in Spanish).
pub fn write_report(rows: &[SetRow], cmd: &str, report: &Path) -> Result<(), String> {
    let all = rows.last().unwrap();
    let mut md = String::from("# Comparación con el BKS de todas las instancias, por conjunto\n\n");
    md += &format!("Informe generado por `pjs bks` (`TOP/src/rust`); se regenera con\n\n```\ncd TOP\n{cmd}\n```\n\n");
    md += "## Qué se compara\n\n";
    md += "- **BKS:** la mejor solución conocida de cada instancia, columna *BEST* de la hoja de resultados que publican Dang, Guibadj y Moukrim con PSOiA (`src/data/bks_chao.csv`; el original está en `ignore/downloads/solutions-psoia.zip`). En las 80 instancias de las tablas 3 y 4 de Panadero coincide con su BKS (Ke et al., 2016).\n";
    md += "- **Nuestros resultados:** la rejilla de [`informe_busqueda_local.md`](informe_busqueda_local.md) (`results/best_solution.csv`): la determinista con el mejor α, la mejor sin búsqueda local (determinista o GRASP de 10 iteraciones) y la final, con búsqueda local sobre las élite.\n";
    md += "- **Panadero, 128 hilos:** la distancia media al BKS de cada conjunto que da su figura 4 (1 s por hilo). El paper no publica los valores por instancia de p3 a p7, solo estas medias.\n";
    md += "- **Distancia:** 100 · (BKS − nuestra) / BKS, media por instancia, sobre las instancias con BKS > 0.\n\n";
    md += "## Distancia media al BKS por conjunto\n\n";
    md += "| Conjunto | Instancias | Determinista, mejor α | Mejor sin búsqueda local | **Final** | Panadero, 128 hilos (figura 4) | BKS alcanzado (final) |\n|---|---|---|---|---|---|---|\n";
    for r in rows {
        let name = if r.set == "all" { "**Todas**".to_string() } else { r.set.clone() };
        let pan = if r.set == "all" { format!("{:.2} % (media ponderada)", r.panadero) } else { format!("{:.2} %", r.panadero) };
        md += &format!(
            "| {name} | {} | {:.2} % | {:.2} % | **{:.2} %** | {pan} | {} de {} |\n",
            r.n, r.gap[0], r.gap[1], r.gap[2], r.hit[2], r.n
        );
    }
    let better = rows.iter().filter(|r| r.set != "all" && r.gap[2] < r.panadero).count();
    md += &format!(
        "\n- **Final frente a Panadero con 128 hilos:** menor distancia media en {better} de los 7 conjuntos; sobre todas las instancias, {:.2} % frente a {:.2} %.\n",
        all.gap[2], all.panadero
    );
    md += &format!(
        "- **Por encima del BKS:** {} instancias con la determinista, {} sin búsqueda local y {} con la final. Superar el BKS indicaría una solución no factible o un récord nuevo.\n",
        all.above[0], all.above[1], all.above[2]
    );
    md += &format!(
        "- **Recompensa total:** BKS {}, final {} ({:.2} % por debajo).\n",
        all.bks_total,
        all.total[2],
        100.0 * (all.bks_total - all.total[2]) / all.bks_total
    );
    md += "- **Cómputo:** la rejilla gasta unos 140 s para las 387 instancias con 12 hilos; Panadero, 1 s de reloj por instancia con 128 hilos. Las medias de la figura 4 son de 128 hilos, la mejor configuración del paper.\n";
    write_file(report, &md)
}

#[cfg(test)]
mod tests {
    use super::SetRow;

    #[test]
    fn gaps_hits_and_above() {
        let mut r = SetRow::default();
        r.add(100.0, &[90.0, 100.0, 100.0]);
        r.add(50.0, &[50.0, 45.0, 51.0]);
        r.finish();
        assert!((r.gap[0] - 5.0).abs() < 1e-12);
        assert!((r.gap[1] - 5.0).abs() < 1e-12);
        assert!((r.gap[2] + 1.0).abs() < 1e-12);
        assert_eq!((r.hit, r.above), ([1, 1, 1], [0, 0, 1]));
    }
}
