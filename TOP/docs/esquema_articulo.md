# Guion del artículo de 3 páginas: TOP

## Título propuesto

*The PJS Savings Heuristic for the Team Orienteering Problem:\ Biased Randomisation and Local Search on the Chao Instances* (ronda 2: segunda línea acortada, M1)

Fichero: `docs/Team Orienteering Problem.tex` → `docs/Javier S Jesus I Team Orienteering Problem.pdf`.

## Requisito del enunciado → Dónde se cubre

Enunciado: `docs/homework.png` (diapositiva *Homework / Lab Activities*). No pide un artículo de forma expresa; lo pide el usuario con el formato de POP y VRP.

| Requisito del enunciado | Dónde se cubre |
|---|---|
| 1. *Construct your own Python program to implement the PJS heuristic for solving the TOP* | §3.1 (PJS), §4 (implementación). **Desviación:** el código es Rust, no Python; §4 lo dice y lo justifica. Los autores confirman que no es un problema (`preguntas_abiertas.md`, pregunta 1, 02-11-2026). |
| 1. *... and test it in different instances* | §1 (387 instancias de Chao, 353 factibles), Tabla 1 (por conjunto), Tabla 2 (81 instancias de p1/p2) |
| 2. *Complete a data analysis on the results for the different instances tested* | §5: ganancia por capa y por conjunto (Tabla 1), dependencia del tamaño (p4, p7), regla de α (§3.2 y §5), ejemplo p4.3.k (Figura 1), élite que no era la mejor (163 de 353) |
| 3. *(Optional) Combine GRASP concepts with the PJS heuristic and analyze the results* | §3.3 (multiarranque, Algoritmo 1), §5 párrafos *Each layer adds reward* y *Many α vs. many iterations* |
| 4. *(Optional) Enhance the GRASP-PJS algorithm by using biased randomization concepts* | §3.3 (geométrica β = 0.3, la del paper, y softmax τ = 1, propuesta nuestra); §3.4 búsqueda local sobre las élite como mejora adicional |
| Referencias del enunciado: Panadero et al. (2021) y Li et al. (2022) | Citadas en §1, §3, §4 y §5; Panadero es la referencia de comparación (tablas 3–4, figura 4) |

## Presupuesto por sección

| § | Sección | Espacio |
|---|---|---|
| — | Abstract | 120–135 palabras |
| 1 | Introduction | ~0.35 pág. |
| 2 | Problem formulation and enriched savings | ~0.45 pág. |
| 3 | PJS, GRASP and local search (3 subsecciones) | ~0.55 pág. |
| 4 | Implementation | ~0.4 pág. |
| 5 | Computational results (2 tablas y figura) | ~1 pág. |
| 6 | Conclusions + Code availability + referencias | ~0.25 pág. |

## Mensajes clave

1. PJS adapta los ahorros de Clarke y Wright al TOP con un ahorro enriquecido que pondera tiempo y recompensa con α; el paper no da α y su regla cualitativa no se cumple en Chao (r = +0.19 con la media de los α empatados, +0.43 con el menor), así que se prueba una rejilla de 21 α.
2. Con origen y destino distintos el ahorro depende del sentido y puede ser negativo (12.8 % de los arcos de p4.3.k): se consideran los dos sentidos.
3. Cada capa añade recompensa sobre las 353 instancias: PJS 157 868 → GRASP 170 469 (+7.98 %) → final 189 854 (+20.26 %); la ganancia crece con el tamaño (p4 +38.75 %, p7 +28.07 %).
4. La búsqueda local necesita la diversidad de GRASP: solo sobre las deterministas se queda en 2.13 % en p1/p2; en 163 de 353 instancias la final sale de una élite que no era la mejor.
5. Frente a Panadero: 2.04 % de distancia media al BKS, o 2.17 % con el denominador del paper, frente a 2.85 % y mejor en los siete conjuntos; en p1/p2, 0.34 % (10 it) y 0.15 % (1 s) frente a 0.44 % con 128 hilos; nunca por encima del BKS.
6. Repartir el presupuesto entre α rinde más que iterar con uno: 1.45 % con 42 000 construcciones frente a 1.49 % con ~880 000 (sin la trivial p2.4.a): unas 20 veces menos construcciones y una quinta parte del tiempo de hilo.
7. Salvedades: una ejecución nuestra de 1 s rinde menos que un hilo de Panadero (2.33–2.45 % frente a 1.73 %); las variantes de 1 s no son reproducibles; el cómputo no es igual.

## Cifras y fuentes

| Cifra | Valor | Fuente |
|---|---|---|
| Instancias, factibles, infactibles | 387, 353, 34 | `docs/informe_alpha.md` (Datos); ficheros de `data/top-data/` |
| Instancias por conjunto (ficheros) | p1 54, p2 33, p3 60, p4 60, p5 78, p6 42, p7 60 | `informe_alpha.md`, tabla *Por conjunto / Determinista*, columna Instancias |
| Factibles por conjunto | 48, 33, 60, 56, 74, 24, 58 | `results/bks_by_set.csv`, columna `instances` |
| Nodos por conjunto | 32, 21, 33, 100, 66, 64, 102 | primera línea `n;` de cada fichero de `data/top-data/` (`p$s.*`); Panadero Tabla 1 |
| m entre 2 y 4 | 2–4 | segunda línea de los ficheros; Panadero §6.1 |
| Panadero dice 354 instancias | 354 | Panadero §6.1 y §7 (texto) |
| Origen ≠ destino en p1–p6, iguales en p7 | — | `docs/scratchpad.md` §2 |
| p4.3.k: 98 clientes, m = 3, tmax = 50, recompensa total 1306, BKS recoge 70.4 % | — | `docs/recheck_p4.3.k.md` §1; cabecera de `p4.3.k.txt` |
| Ahorros de p4.3.k: 9506 arcos, 12.8 % negativos, s ∈ [−19.74, 42.57], u ∈ [1, 29] | — | cálculo propio `ignore/paper_loop/tmp/savings.sh` sobre `data/top-data/p4.3.k.txt` |
| p7.2.a: ahorros ≥ 0 salvo redondeo (origen = destino) | 4 de 9900 con −0.00 | mismo script sobre `p7.2.a.txt` |
| β = 0.3 | 0.3 | Panadero §4; `src/rust/src/main.rs` l. 594 (default) |
| τ = 1, semilla 1, 10 ejecuciones, 10 iteraciones | — | `informe_busqueda_local.md` (comando); `main.rs` l. 592 |
| Rejilla α = k/20, 21 valores | 21 | `src/rust/src/alpha_grid.rs` l. 18–21 |
| EPS = 1e-9 | 1e-9 | `src/rust/src/model.rs` l. 11 |
| Orden estable de la lista | — | `heuristic.rs` l. 83, 113 |
| Selección de rutas: recompensa, en empate la más corta | — | `heuristic.rs` l. 154 |
| Geométrica por inversión ⌊ln U / ln(1−β)⌋ mód. arcos restantes | — | `grasp.rs` l. 11, 97 |
| Li et al.: α en (0.1, 0.9), pares i ≠ j | — | Li et al. (2022) §4 (texto extraído l. 538–543) |
| Tareas paralelas | 170 667 (441 por instancia, 21 por α) | `informe_alpha.md` (Ejecución); `best_solution.csv`, `elite_selected` = 441 |
| Tiempo de la rejilla | 127 s (12 hilos) | `informe_alpha.md` (Ejecución) |
| Búsqueda local: élite elegidas / tras duplicados / por instancia, tiempo | 110 496 / 18 757 / 53.1, 0.4 s | `informe_busqueda_local.md` (Método) |
| Equipo | Ryzen 5 5500U, 6 núcleos, 12 hilos | `metodos_y_comparacion_panadero.md` §2 (nota bajo la tabla de presupuestos) |
| Recompensa total: PJS mejor α / GRASP / det.+LS / final | 157 868 / 170 469 (+7.98 %) / 181 103 (+14.72 %) / 189 854 (+20.26 %) | `informe_busqueda_local.md`, *Resultados globales* |
| Mejor α común y su recompensa | 0.40, 150 585 (−4.61 %), óptimo en 170 | `informe_alpha.md`, *Resultados globales* |
| α influye en | 285 de 353 | `informe_alpha.md` |
| Correlación CV–α óptimo | +0.19 (media de los empatados), +0.43 (menor α óptimo, columna `best_alpha`); 285 inst. | `tmp/cv_alpha2.sh` y `tmp/r4_check.py` sobre `results/alpha_grid.csv` y los ficheros de instancia: 0.1929 y 0.4255. El primero coincide con la diapositiva 4 |
| CV de las recompensas p4 / p5 | 0.64 / 0.35 | `tmp/cv.txt` (mismo script) |
| α = 0 en p4: % de la mejor | 68 % (media por instancia, sobre las 54 en que α influye; 69.1 % sobre las 56 factibles) | `tmp/cv_alpha2.sh`, `tmp/r4_check.py`: 0.6796 y 0.6910 |
| LS mejora élite / ganancia media / insertados | 93.6 % / +18.16 % / 7.48 | `informe_busqueda_local.md`, *Efecto de la búsqueda local* |
| Instancias en que aplicar la LS solo a la mejor previa habría dado menos | 163 de 353 (8 187 de recompensa) | `informe_busqueda_local.md`, *De dónde sale* (la lectura «la élite final no era la mejor» daría 244, según el revisor) |
| Ganancia final vs determinista por conjunto | p1 +8.70, p2 +2.09, p3 +8.40, p4 +38.75, p5 +13.61, p6 +17.44, p7 +28.07 % | `informe_busqueda_local.md`, *Por conjunto* |
| Gaps por conjunto (det., sin LS, final, P-128) y BKS alcanzado | ver Tabla 1 | `results/bks_by_set.csv` (redondeado a 2 decimales) |
| Total BKS / final | 195 931 / 189 854 | `bks_by_set.csv`, fila `all` |
| Denominador del *Gap* de Panadero | Su recompensa (OBS con 128 hilos): cuadra en 81 de 81 filas; dividir por el BKS, solo en 69 | `tmp/r4_check.py` sobre `data/panadero2021_p1_p2.csv` (`bks`, `thr128`, `gap_pct`); medias p1 0.69 y p2 0.11, las de su figura 4 |
| Final con el denominador del paper | p1 0.60, p2 0.00, p3 0.70, p4 6.38, p5 1.35, p6 1.22, p7 3.61; total 2.17 (Tabla 1 y §4, §5, abstract) | `tmp/vc_gapobs.py` y `tmp/r4_check.py` sobre `results/best_solution.csv` (`best_ls`) y `data/bks_chao.csv` |
| p1/p2: BKS total 10 685; Panadero 1 hilo 1.73 % (52 BKS), 128 hilos 0.44 % (69) | — | `metodos_y_comparacion_panadero.md` §0 (de Panadero tablas 3–4) |
| p1/p2 nuestras variantes (gap, BKS, vs P-128) | det 4.82/44/0-45-36; 10 it 1.99/53/1-56-24; 1 s rejilla 0.71/65/4-67-10; 10 it+LS 0.34/73/10-65-6; 100 it+LS 0.29/74/12-62-7; 1 s+LS 0.15/77/12-65-4; det+LS 2.13 | `metodos_y_comparacion_panadero.md` §4 |
| Una ejecución de 1 s (media de 10, mejor α) | geom. 2.45 %, softmax 2.33 % | `docs/comparacion_panadero.md` l. 31–32 (informe generado; la síntesis daba 2.43/2.29, desfasada) |
| 1 s mejor α vs 100 it rejilla | 1.49 % (~880 000 constr./instancia: 876 008 sin p2.4.a, que suma 7 431 281; 956 937 con ella) vs 1.45 % (42 000); cociente 20.9 ("about 20 times" en el artículo) | `tmp/r4_check.py` y `tmp/vc_work.py` sobre `results/panadero_1s_runs.csv` y `results/panadero_100it_grid_runs.csv`, columna `iterations` |
| Tiempo de hilo por instancia, 1 s mejor α vs 100 it rejilla | 20.00 s vs 4.09 s (cociente 4.9, "a fifth"); 20 ejecuciones de 1 s por instancia (1620 filas / 81) | mismos scripts, suma de `seconds` / 81 |
| Iteraciones de 1 s | ~40 600 p1, ~56 200 p2 (rejilla) | `scratchpad.md` §7 |
| Tiempo 1 s rejilla | 2 836 s (47 min) | `metodos_y_comparacion_panadero.md` §2 |
| p4.3.k det. / final / BKS | 614 (32 cl., α = 0.40) / 874 (54 cl., +42.3 %, geom. α = 0.45 run 7, 625 antes, 19 insertados) / 919 (61 cl.) | `recheck_p4.3.k.md` §3; subtítulos de las figuras |
| verify_routes | 387 inst., 1000 rutas, 9679 clientes, todo factible | `scratchpad.md` §7 |
| Panadero: Java SE 8, Xeon E5-2650 v4 16 núcleos, 1 s por hilo | — | Panadero §6.2 |
| Cómputo por instancia de la rejilla | < 4 hilo·s (127 × 12 / 387 = 3.9) | cálculo propio |
| p1.4.r 210 → 205 | — | `metodos_y_comparacion_panadero.md` §3 |

## Figuras y tablas

- **Tabla 1** (`tab:results`): una fila por conjunto + fila *All*. Columnas: nodos, instancias factibles, gap determinista, gap sin LS, gap final, gap Panadero 128 (figura 4), BKS alcanzados, ganancia final vs determinista; la leyenda da además la final con el denominador del paper. Fuente: `results/bks_by_set.csv` + `informe_busqueda_local.md` + `tmp/r4_check.py`.
- **Tabla 2** (`tab:p1p2`): 81 instancias de p1/p2; método, gap medio, BKS alcanzados, mejor/igual/peor frente a P-128. Fuente: `metodos_y_comparacion_panadero.md` §4.
- **Figura 1** (`fig:p4`): `results/figures/routes/p4.3.k.png` y `results/figures/routes_ls/p4.3.k.png`, lado a lado.

## Referencias

- Panadero et al. (2021), *Appl. Sci.* 11(24):12092 — primera página de `docs/applsci-11-12092-v2.pdf`.
- Li et al. (2022), *Mathematics* 10(6):982 — primera página de `docs/mathematics-10-00982-v2.pdf` y diapositiva del enunciado.
- Clarke y Wright (1964), *Oper. Res.* 12(4):568–581 — ref. 56 de Panadero y artículo de VRP.
- Chao, Golden y Wasil (1996), «The team orienteering problem», *EJOR* 88(3):464–474 — lista de referencias de Juan et al. (2019), `simTOP/docs/SIMULATION-BASED OPTIMIZATION IN TRANSPORTATION AND LOGISTICS.pdf` (corregido el 08-11-2026; antes se citaba «A fast and effective heuristic for the orienteering problem», *EJOR* 88:475–489, ref. 39 de Panadero, que es el artículo del OP).
- Dang, Guibadj y Moukrim (2013), *EJOR* 229:332–344 — ref. 49 de Panadero (PSOiA, fuente del BKS).

## Pendiente

1. ~~**Python frente a Rust**~~ (resuelto el 02-11-2026, `preguntas_abiertas.md`, pregunta 1): el artículo sigue diciendo que el enunciado pide Python y que el código es Rust; la frase de la comprobación en Python se ha quitado porque no hay código ni salida en el repositorio.
2. ~~**Referencia de Chao**~~ (resuelto el 08-11-2026: el artículo cita ya *The team orienteering problem*, EJOR 88(3):464–474, comprobado en las referencias de Juan et al. (2019), PDF en `simTOP/docs/`). Nota original: Panadero cita como fuente del benchmark «A fast and effective heuristic for the orienteering problem» (EJOR 88:475–489), que es el artículo del OP; el del TOP (*The team orienteering problem*) no está en ningún PDF de `docs/`. Se cita como lo da Panadero; decidir si se cambia.
3. **Tiempo de la rejilla:** 127 s en `informe_alpha.md` (tareas de construcción), ~140 s en la presentación y `comparacion_bks_conjuntos.md`, ~2 min 10 s en total según el scratchpad. El artículo usa 127 s + 0.4 s.
4. **Élite:** el scratchpad dice 125 490 / 18 791; el informe generado, 110 496 / 18 757. Se usa el informe.
5. **GRASP sin LS:** el scratchpad da 169 571 (+7.41 %, solo geométrica); el informe de búsqueda local, 170 469 (+7.98 %, cualquier aleatorización). Se usa 170 469.
6. `comparacion_bks_conjuntos.md` dice «80 instancias» de Panadero donde ya son 81 (se generó antes de añadir p2.4.a).
7. No hay transparencias del curso sobre el TOP en `docs/` (solo `homework.png`), así que no se citan.
8. Falta `TOP/README.md` (pendiente del proyecto, no del artículo).
9. **BKS de Panadero en p3–p7 (ronda 2):** sus gaps se miden frente al BKS de Ke et al. (§6.2); solo hemos comprobado que coincide con el de PSOiA en p1–p2. Salvedad añadida en §1 y en la leyenda de la Tabla 1.
10. **Prosa (ronda 4):** 2 015 palabras con el comando de la guía §11 (1 968 en la ronda 3, aprobada) y 1 923 con `tmp/r3_count.py`, que no cuenta fórmulas ni comandos (1 870 en la ronda 3); abstract, 135 y 133. El PDF tiene 3 páginas. La figura sigue en 2 × 0.45\textwidth.
11. Las reglas de α «basadas en el CV o en rasgos» solo las respalda `metodos_y_comparacion_panadero.md` §1 y `scratchpad.md` §5 (sin CSV); el artículo las presenta como «in preliminary trials, not part of the published code» (decisión de los autores).
12. **Denominador de la figura 4 de Panadero:** solo se puede comprobar en p1 y p2 (las medias de su columna *Gap* coinciden con la figura). En p3–p7 el paper no da datos por instancia y se supone el mismo denominador; la leyenda de la Tabla 1 lo presenta «as in its Tables 3–4».
13. **Presentación:** `guion_presentacion.md` (l. 185) y la diapositiva 8 siguen citando las 38 700 ejecuciones en Python, y `guion_presentacion.md` (l. 292) y `metodos_y_comparacion_panadero.md` (l. 118, l. 156) dan ~900 000 construcciones, frente a las ~880 000 del artículo. No se han tocado.
