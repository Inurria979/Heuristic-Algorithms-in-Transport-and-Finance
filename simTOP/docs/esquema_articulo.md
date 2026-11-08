# Esquema del artículo de simTOP (PDF, 3 páginas)

**Título:** *A Simheuristic for the Team Orienteering Problem with Stochastic Travel Times on the Small
Chao Instances*

Ficheros: `docs/Stochastic Team Orienteering Problem.tex` → `docs/Javier S Jesus I Stochastic Team
Orienteering Problem.pdf`. Figura leída de `../results/figures/`.

## Requisito → dónde se cubre

No hay enunciado (diapositiva de *Homework*). El alcance lo fijó el usuario en §1 de
`explicacion_paper.md`; las «decisiones que deben aparecer en el artículo» (§11, puntos 1–9), el trabajo
futuro y §6.1 hacen de enunciado.

| Requisito | Dónde se cubre |
|---|---|
| Implementar la simheurística del §5 del paper (PJS + BR + búsqueda local + simulación rápida e intensiva) | §3 (enumerate de 6 pasos), §4 |
| Tres niveles de varianza, solo p1–p3 (< 35 nodos) | §1 (instancias), §2 (niveles), Tabla 1 |
| SAA fuera del alcance | §1, última frase del primer párrafo |
| Comparar con la Fig. 3 del paper, valores leídos a ojo | §5 *Comparison with the paper*, Tabla 1, Figura 1 y sus leyendas |
| §11.1 Var = c·t y su consecuencia (desviación relativa √(c/E_r)); analizar por conjunto / tmax | §2 segundo párrafo; §5 *Set and t_max*; columnas p1/p2/p3 de la Tabla 1 |
| §11.2 Arcos independientes (supuesto nuestro) | §2 («We assume independent arcs, which the paper does not state») |
| §11.3 Arco origen–destino de longitud 0 (solo p7) | No se menciona por decisión del usuario (r2): solo se estudian p1–p3 |
| §11.4 α, 50 iteraciones, 7 → 3, parada por iteraciones: valores nuestros | §4 segundo párrafo |
| §11.5 Búsqueda local de TOP en lugar de destrucción–reconstrucción + 2-opt | §3 paso 1 y §4 segundo párrafo |
| §11.6 y §6.1 CRN en la rápida, días nuevos en la intensiva; «comunes ≠ reutilizados»; sesgo residual del mejor de 3 | §3 párrafo *Common days to compare, new days to measure* |
| §11.7 Greedy como candidata extra (qué garantiza y qué no); una semilla por caso | §3 último párrafo; §4 tercer párrafo |
| §11.8 Descomposición del gap (gap determinista + pérdida por incertidumbre, ambas sobre la BKS, suman el gap) | §4 (fórmulas), Tabla 1, §5 *Most of the gap is lost to uncertainty* |
| §11.9 Rutas al límite de tmax; margen de seguridad como trabajo futuro | §3 último párrafo, §5 (holgura 0.93 tmax), §6 (margen con las tres ideas) |
| Sesgo del mejor de 3 α | §5 *The weight α hardly matters* |
| Nota del orquestador: el filtro de 7 a veces no actúa (27 % < 7 distintas, 9 % solo una) | §5 *Most of the gap...* |
| Nota del orquestador: cautela causal sobre gap y tmax | §5 *Set and t_max* («observation, not an explanation») y «A likely reason» |
| Código | No se pide apéndice ni fichero aparte (no hay enunciado); los ficheros se nombran en §4 |

## Presupuesto por sección (palabras de prosa medidas en el `.tex`)

| Sección | Objetivo guía | Medido r1 |
|---|---|---|
| Abstract | 120–135 | 132 |
| 1 Introduction | ~0.35 pág. | ≈ 260 |
| 2 The stochastic model | ~0.5 pág. | ≈ 295 |
| 3 The simheuristic | ~0.5 pág. | ≈ 530 (más larga: incluye §6.1 y punto 7) |
| 4 Implementation | ~0.4 pág. | ≈ 280 |
| 5 Computational results | ~1 pág. con tabla y figura | ≈ 580 |
| 6 Conclusions | ~0.15 pág. | ≈ 125 |
| **Total prosa** | 1 700–1 950 | **2 267** en r3 (2 266 en r2, 2 208 en r1; el revisor acepta el exceso) |

## Mensajes clave

1. La simheurística separa construir (con tiempos esperados) de evaluar (con simulación); BR explora, la
   simulación evalúa.
2. Días comunes para comparar, días nuevos para medir: comunes no es lo mismo que reutilizados.
3. Gap medio 14.65 / 25.02 / 30.51 % (L / M / H): mismo orden que el paper, medianas 7–9 puntos por
   encima de las leídas a ojo.
4. Entre el 80 y el 90 % del gap es pérdida por incertidumbre; la ganadora suele alcanzar la BKS en
   determinista (mediana del gap determinista 0).
5. Causa probable: rutas construidas hasta 0.93 tmax; fallan 13–29 % de las rutas. La simulación elige
   bien, pero solo entre rutas sin holgura.
6. α apenas importa; el gap crece con tmax, pero con confusión conjunto/tmax: observación, no explicación.
7. Trabajo futuro: margen de seguridad (rejilla, a partir de la varianza, adaptativo).

## Cifras y fuentes

| Cifra en el artículo | Valor | Fuente |
|---|---|---|
| Instancias factibles | 141 (48 / 33 / 60) | `results/best.csv` (141 instancias distintas; recuento por prefijo, script `ignore/paper_loop/tmp/stats.py`); README |
| Seis instancias de p1 sin cliente alcanzable | 6 | README §Instances; `explicacion_paper.md` §2.1 |
| Nodos por conjunto | 32 / 21 / 33 | `explicacion_paper.md` §2.1 |
| Rango de tmax | 3.8–55 | `results/best.csv` columna `tmax` (stats.py); README |
| Niveles de c | 0.05 / 0.25 / 0.75 | paper p. 1913; `src/simtop.py` `C_LEVELS` |
| CV de un arco t = 10 | 7.1 / 15.8 / 27.4 % | `explicacion_paper.md` §2.2 (cálculo √(c t)/t) |
| Desviación relativa de una ruta llena, c = 0.25 | 25.6 % (tmax 3.8), 6.7 % (tmax 55) | cálculo √(0.25/3.8) = 0.2565, √(0.25/55) = 0.0674 |
| Ruta con E = tmax falla | 47.18 % de 20 000 réplicas (4 arcos de 2.5, tmax 10, c 0.25) | `src/checks.py` l. 125–129; valor impreso citado en `ignore/code_loop/readme/cambios_r1.md` l. 17 |
| β, iteraciones BR, filtro 7 → 3, réplicas 200 / 1000, α | 0.3, 50, 7/3, 200/1000, {0.25, 0.5, 0.75} | `src/simtop.py` l. 17–22 |
| Semilla | `SeedSequence([2019, idx_inst, idx_c, idx_alpha])`, 3 generadores | `src/simtop.py` `BASE_SEED`, `case_seed`, `simheuristic` (`spawn(3)`) |
| Versiones | Python 3.14, NumPy 2.5.4, Numba 0.68.1 | `.python-version`, `uv.lock` |
| 18 comprobaciones; greedy PJS = TOP para 3 α en 141 instancias | — | README §Usage; `src/checks.py` l. 54–63 |
| Ejecuciones / casos mejor α | 1 269 / 423 | `results/runs.csv` (1 269 filas), `results/best.csv` (423) |
| Gap mejor α: mediana (Q1–Q3), media | L 15.69 (9.90–20.23) 14.65; M 26.02 (20.04–31.97) 25.02; H 31.56 (26.35–36.44) 30.51 | `results/gap_best_by_level.csv` |
| Paper Fig. 3, SimHeu (a ojo, ±1–2) | ≈ 7 (5–10), ≈ 17 (9–20), ≈ 24 (19–28) | `results/gap_best_by_level.csv` columnas `paper_*`; `explicacion_paper.md` §9 |
| «7–9 puntos por encima» | 8.69 / 9.02 / 7.56 | diferencia de medianas, `gap_best_by_level.csv` |
| Gap por conjunto (medias) | p1 18.38/29.37/33.87; p2 10.36/19.33/25.19; p3 14.02/24.68/30.75 | `results/gap_by_set_level.csv` |
| Gap determinista medio | 3.15 / 2.96 / 2.41 (mediana 0) | `results/gap_decomposition_by_level.csv` |
| Pérdida por incertidumbre media sobre la BKS (Tabla 1) | 11.50 / 22.06 / 28.10 (= gap − gap det.) | `best.csv`, media de 100 (det_reward − exp_reward)/bks por nivel (`ignore/paper_loop/tmp/lossbks.py`) |
| Pérdida media sobre la recompensa determinista de la ganadora (texto) | 11.86 / 22.72 / 28.79 | `results/gap_decomposition_by_level.csv` (`uncert_loss_pct`) |
| Ganancia del mejor α sobre cada α fijo | 1.3–2.2 puntos (L 1.64–2.21, M 1.81–1.90, H 1.27–1.46) | medias de `gap_alpha_c_matrix.csv` menos medias de `gap_best_by_level.csv` |
| «≈ 80 % del gap en L, 90 % en M y H» | 78.5 / 88.2 / 92.1 % | pérdida sobre la BKS / gap (lossbks.py) |
| Rutas que fallan (media) | 12.73 / 23.57 / 29.45 % | `results/fail_rate_by_level.csv` |
| Ganadoras con gap determinista 0 | 72 / 72 / 82 de 141 | `results/best.csv`, `det_gap_pct == 0` por nivel (stats.py) |
| Holgura de las rutas ganadoras | mediana 0.928 / 0.927 / 0.932 tmax; 3.8 / 6.1 / 8.4 % a < 1 % de tmax | `results/route_slack_by_level.csv` |
| Menos de 7 candidatas distintas / solo una | 27.1 % / 9.0 % de 1 269 | `results/runs.csv`, `n_distinct` (stats.py) |
| Ganadora = greedy | 13.2 % de 423 | `best.csv`; salida guardada de `analysis.ipynb` celda 19 |
| Mayor gap | 42.7 % (p1.2.k, H); ninguno > 50 % | `analysis.ipynb` celda 19; README |
| Matriz α × c (medias) | L 16.29–16.86; M 26.83–26.92; H 31.78–31.97 | `results/gap_alpha_c_matrix.csv` |
| Mejor α | 142 / 138 / 143 de 423 | `analysis.ipynb` celda 19; README |
| Spearman gap–tmax | 0.49 / 0.56 / 0.67 | `analysis.ipynb` celda 10 |
| Spearman gap det.–tmax | 0.56 / 0.54 / 0.59 | `analysis.ipynb` celda 11 |
| Spearman pérdida sobre la BKS–tmax (r3) | 0.30 / 0.39 / 0.44 (0.299 / 0.391 / 0.438) | `best.csv`, 100 (det_reward − exp_reward)/bks frente a tmax, rangos con empates promediados (`ignore/paper_loop/tmp/sp.py`); la pérdida sobre det_reward da 0.33 / 0.44 / 0.52 (notebook celda 11, README) y no se usa |
| Mediana de tmax por conjunto | p1 17.5, p2 10.0, p3 20.6 | `analysis.ipynb` celda 10 |
| Tiempo por ejecución | 0.053–0.234 s; medias p1 0.159–0.165, p2 0.071–0.074, p3 0.164–0.165 | `results/runs.csv` `time_s` (stats.py); `results/time_by_set_level.csv` |
| Tiempo total y equipo | ≈ 25 s en 12 hilos de un AMD Ryzen 5 5500U de 6 núcleos | README §Usage; `ignore/code_loop/experimento/cambios_r2.md` (24.3 s); equipo confirmado por el orquestador en r2 |
| Paper Fig. 4 | de unos segundos a ≈ 400 s, Java, Xeon 3.7 GHz | paper p. 1912 (máquina) y Fig. 4 p. 1915; `explicacion_paper.md` §9 |
| Paper: SAA solo resuelve ≤ 35 nodos en 100 000 s | — | paper p. 1914 |
| Paper: simulación de la solución base dentro del bucle, parada por criterio | — | paper p. 1912, §5, segunda viñeta |

## Figuras y tablas

- **Tabla 1** (única): una fila por nivel L / M / H; columnas: c, nuestra mediana (Q1–Q3), nuestra media,
  paper mediana (Q1–Q3), medias p1 / p2 / p3, gap determinista, pérdida, rutas que fallan, ganadoras con
  BKS determinista. Sin fila *Mean*: cada nivel es un problema distinto (se dice en la leyenda).
  Fuentes: `gap_best_by_level.csv`, `gap_by_set_level.csv`, `gap_decomposition_by_level.csv`,
  `fail_rate_by_level.csv`, `best.csv`.
- **Figura 1**: `results/figures/gap_best_vs_paper.png` (0.85 `\textwidth`).
- No se usan: `gap_decomposition.png`, `gap_by_set_tmax.png`, `gap_by_c_alpha.png`,
  `fail_rate_by_level.png` (sus cifras van en la tabla y en el texto).

## Referencias (comprobadas)

1. Juan et al. 2019, *Proc. WSC*, pp. 1906–1917 — primera página y pie de página del PDF de `docs/`.
2. Chao, Golden, Wasil 1996, *The team orienteering problem*, EJOR 88(3):464–474 — lista de referencias
   del PDF de Juan et al.
3. Panadero et al. 2021, *Appl. Sci.* 11(24):12092 — primera página de `TOP/docs/applsci-11-12092-v2.pdf`
   (volumen 11, artículo 12092; el número 24 sale del DOI `app112412092` y del artículo de TOP).
4. Grasas et al. 2017, *Comput. Ind. Eng.* 110:216–228 — lista de referencias del PDF de Juan et al.
5. Dang, Guibadj, Moukrim 2013, EJOR 229:332–344 — **no comprobada contra PDF** (no está en `docs/`);
   copiada del artículo de TOP, que la usa como fuente de `bks_chao.csv` (`TOP/README.md`).

## Pendiente

- **Prosa 2 208 palabras**, por encima del rango de la guía (1 700–1 950), aunque el PDF cumple las 3
  páginas (márgenes superior e inferior de 1.8 cm, dentro de la holgura de la guía). El exceso viene de
  los requisitos del usuario (§11 puntos 1–9, §6.1, trabajo futuro). Si hay que recortar, candidatos: el
  párrafo de la greedy (§3), los valores de la matriz α y la frase del CV de un arco.
- **Equipo**: resuelto en r2 (Ryzen 5 5500U, 6 núcleos / 12 hilos, confirmado por el orquestador).
- **Dang et al. 2013**: datos bibliográficos sin comprobar contra el PDF.
- **Discrepancia menor de fuentes**: el artículo de TOP cita para Chao et al. (1996) otro trabajo (*A fast
  and effective heuristic for the orienteering problem*, EJOR 88:475–489); aquí se cita *The team
  orienteering problem* (88(3):464–474), que es el que cita Juan et al. No afecta a simTOP.
- §11.3 (arco de longitud 0 en p7): se omite por decisión del usuario (r2).
- El texto de la Fig. 3 del paper («lower expected time») no cuadra con su eje (gap de recompensa); no se
  comenta por espacio.
