# Esquema del artículo (PDF, 3 páginas): GRASP con ahorros y aleatorización sesgada para el TSP

> Fuentes: salidas guardadas de `src/GRASP_TSP_savings_biased_EN.ipynb` (no se ha vuelto a ejecutar),
> extraídas como JSON a `results/*.csv` y `results/figures/*.png` con
> `ignore/paper_loop/tmp/extract.py`; presentación `docs/GRASP_TSP_biased_randomization.{pdf,pptx}`
> y `docs/presentation_script.md`. Estadísticos derivados con `ignore/paper_loop/tmp/derived.py`.
> Ronda 3: correcciones de redacción de `ignore/paper_loop/verificacion_codigo.md` (`cambios_r3.md`).
> Ronda 4: correcciones de `ignore/paper_loop/revision_r3.md` (`cambios_r4.md`).
> Las celdas se citan por su índice en el notebook, no por el contador de ejecución (In [9] es la celda 16).

**Título propuesto:** *Biased Randomisation in a Savings-Based GRASP:\\ Four Selection Rules for the Travelling Salesman Problem*

---

## Correspondencia con el enunciado

No hay enunciado de esta actividad en el proyecto (no hay imagen en `docs/`, ni diapositiva
*Homework / Lab Activities* en los PDF de clase de `TSP/`, `POP/`, `VRP/`, ni en `GRASP/README.md`, que existe desde el 28-10-2026).
La tabla usa la pregunta de la presentación («Does biased randomization make GRASP better?»)
y los requisitos comunes de los enunciados anteriores.

| Requisito (supuesto) | Dónde se cubre |
|---|---|
| Artículo de 3 páginas en PDF, en inglés | Todo el documento |
| Describir GRASP (construcción aleatoria + búsqueda local + mejor solución) | §1, §3 |
| Construcción por ahorros de Clarke y Wright para el TSP | §2 (ahorros), §3 (filtrar, elegir, fusionar) |
| Comparar RCL clásica con aleatorización sesgada (4 reglas) | §3, §5 |
| Instancias TSPLIB con óptimo conocido, gap | §1, §4, Tabla 1 |
| Tiempos de cómputo | §5 *Time*, fila de tiempos de la Tabla 1 |
| Representación gráfica | Figura 1 (`gap_boxplots.png`) |
| Código | *Code availability*: notebook `src/GRASP_TSP_savings_biased_EN.ipynb` (no se sabe si se pide como apéndice) |

## Presupuesto por sección

| § | Sección | Espacio |
|---|---|---|
| — | Abstract | 120–135 palabras |
| 1 | Introduction | ~0.35 pág. |
| 2 | The TSP and Clarke–Wright savings | ~0.4 pág. (ecuación del TSP + ahorros) |
| 3 | GRASP with savings: RCL vs. biased randomisation | ~0.6 pág. (construcción en `enumerate`, 5 reglas en `itemize`) |
| 4 | Implementation | ~0.4 pág. (ajuste, protocolo, test estadísticos) |
| 5 | Computational results | ~1 pág. con Tabla 1 y Figura 1 |
| 6 | Conclusions + Code availability + References | resto |

## Mensajes clave

1. GRASP funciona: de 5.07 % (CW determinista + 2-opt) a 2.22 % con la RCL clásica (56 % menos error).
2. La regla geométrica (β = 0.4) da el menor gap medio, 1.70 % (23 % menos que la RCL), es significativamente mejor en berlin52 y a280 y nunca significativamente peor.
3. Causa probable, no medida: el corte de la RCL se fija sobre el valor normalizado del ahorro, así que, con un solo α, el número de candidatos depende del reparto de valores en la cabeza de la lista: en eil51 las 20 ejecuciones de la RCL acaban con el mismo coste, 432 (media = mejor = 1.41 %; el notebook no guarda los tours, así que no está probado que sea el mismo tour); las reglas por rango fijan la probabilidad de cada posición. Es la redacción del revisor en r2 (I1); falta que la confirmen los autores (pregunta 7).
4. Softmax es una versión suave de la misma RCL (mismo $z_k$; con T = 0.02 el borde de la RCL pesa ≈0.08), y por eso no difiere significativamente de ella en ninguna de las 7 instancias.
5. Zipf y softmax cuestan unas 1.3–1.75 veces el tiempo de la RCL (pesos sobre toda la lista); geométrica y triangular, lo mismo o menos (unas 0.8–0.95 veces).
6. Cautela: Friedman p = 0.76; evidencia moderada.

## Cifras y fuentes

| Cifra | Valor | Fuente |
|---|---|---|
| Instancias test y n | eil51 51, berlin52 52, st70 70, eil76 76, kroA100 100, ch130 130, a280 280 | notebook celda 3 (salida) |
| Óptimos | 426, 7542, 675, 538, 21282, 6110, 2579 | notebook celda 3 (salida); evaluados de `.opt.tour` |
| Instancias de ajuste | kroB100 (opt 22141, fijado en código), eil101 (629), ch150 (6528) | celdas 1 y 3 |
| Pares candidatos kroA100 | 4851, hub 57 | celda 5 (salida) |
| ITERS, RUNS | 150, 20 | celda 1 |
| TUNE_ITERS, TUNE_RUNS, semillas | 80, 6; test 0…19, ajuste 10000+r | celda 1 |
| Rejillas | α {0,0.002,0.005,0.01,0.02,0.05,0.1,0.2}; β {0.01,0.02,0.05,0.1,0.2,0.4,0.7}; f {0.0002,…,0.02}; e {1,1.25,1.5,2,3,5}; T {0.001,…,0.05} | celda 16 |
| Parámetros elegidos | α 0.05, β 0.4, f 0.002, e 3, T 0.02 | celda 16 (salida) |
| Tiempo ajuste / experimento | 306 s / 1011 s, 700 ejecuciones | celdas 16 y 18 (salida) |
| Gap CW determinista (sin / con 2-opt) por instancia | 5.40/4.46 … 13.65/9.46; una sola construcción por instancia, no media de 20 (leyenda de la Tabla 1: «a single run») | `results/deterministic_cw.csv`; celda 14 (`default_rng(0)`) |
| Media CW det. sin 2-opt | 7.60 % | `derived.py` sobre `deterministic_cw.csv` |
| Media CW det. + 2-opt | 5.07 % | `results/mean_gap.csv` fila MEAN |
| Gap medio por instancia y variante | tabla | `results/mean_gap.csv` |
| Medias: RCL 2.22, geo 1.70, tri 2.10, Zipf 2.18, softmax 2.25 | | `mean_gap.csv` fila MEAN |
| Reducción geo vs RCL | 23.4 % | 1 − 1.70/2.22 (`derived.py`) |
| Reducción RCL vs CW+2-opt | 56.2 % (respecto al 5.07 %, no al 7.60 %: «of the remaining error») | 1 − 2.22/5.07 |
| berlin52 RCL 3.69 → geo 0.93; a280 5.16 → 4.31 | | `mean_gap.csv` |
| eil51: RCL media = mejor = 1.41; softmax igual | | `mean_gap.csv`, `best_gap.csv` |
| eil51: coste común de las 20 ejecuciones de la RCL | 432 | cálculo con el óptimo 426: 432 da 1.408 %; una unidad de coste vale 0.235 puntos de gap, así que una sola ejecución en 433 subiría la media a 1.42 (cuenta directa sobre `mean_gap.csv` y `best_gap.csv`, ronda 3) |
| berlin52 RCL mejor 3.62 | | `best_gap.csv` |
| Mejor de las 20 ejecuciones, promediado sobre las 7 instancias: RCL 1.70, geo 0.81 | 1.696 / 0.813 | `derived.py` sobre `best_gap.csv` |
| Veredictos (Holm 5 %) | geo 2/5/0, tri 2/4/1, Zipf 0/6/1, softmax 0/7/0 | `results/verdict_summary.csv`, `verdicts_vs_rcl.csv` |
| P(superior) medio | geo 0.602, tri 0.512, Zipf 0.449, softmax 0.488 | `verdict_summary.csv` |
| Friedman | χ² = 1.87, p = 0.7596 | celda 21 (salida) |
| Tiempo por ejecución a280 | RCL 5.81, geo 5.33, tri 5.53, Zipf 10.18, softmax 9.93 s | `results/mean_time.csv` |
| Rango tiempos por ejecución | 0.14–10.18 s: medias de 20 ejecuciones por regla e instancia («on average») | mínimo y máximo de `mean_time.csv` |
| Cociente tiempo Zipf/RCL y softmax/RCL | 1.258–1.752 y 1.294–1.709 (media 1.47 ambos); en el artículo, «about 1.3–1.75». El mínimo sale de tiempos redondeados a 0.01 s: Zipf/RCL en st70 está en [1.222, 1.295] | `derived.py` y `vc_data.py` sobre `mean_time.csv` |
| Cociente geo/RCL, tri/RCL | 0.774–0.941, 0.824–0.952; en el artículo, «about 0.8–0.95». El mínimo geo/RCL en st70 está en [0.746, 0.803] | `derived.py` y `vc_data.py` sobre `mean_time.csv` |
| a280 mejores tours | RCL 2682 (3.99 %), geo 2664 (3.30 %) | `results/figures/tours_a280.png` (títulos) |
| Python | 3.13.16 | metadatos del notebook |
| Posiciones de la regla triangular en el primer paso | a280: ⌈0.002·38781⌉ = 78; kroB100: ⌈0.002·4851⌉ = 10 | cálculo con `L = min(m, max(3, ceil(f·m)))` (celda 7) y m = (n−1)(n−2)/2 |
| Prosa / abstract (ronda 4) | 1 950 / 132 palabras; PDF de 3 páginas | §11 de la guía |
| β de Juan et al. | U(0.05, 0.25) en cada selección; sin ajuste | `VRP/docs/On the use of Monte Carlo…pdf`, p. 1088 y p. 1086 (d) |
| Peso softmax en el borde de la RCL | $e^{-0.05/0.02}=e^{-2.5}\approx0.08$ | cálculo con T = 0.02 y α = 0.05 (celda 16) |

## Figuras y tablas

- **Tabla 1**: gap medio (%) por instancia: n, óptimo, CW det., CW det. + 2-opt, RCL, geométrica, triangular, Zipf, softmax; marcas de significación (`verdicts_vs_rcl.csv`); filas Mean, B/=/W y tiempo por ejecución en a280.
- **Figura 1**: `results/figures/gap_boxplots.png` (cajas de los 20 gaps por instancia y regla).
- Extraídas pero sin usar: `selection_pmf_kroA100.png`, `tuning.png`, `convergence.png`, `tours_a280.png`.

## Referencias

- Feo y Resende (1995), *J. Global Optim.* 6:109–133: comprobado en la bibliografía de Juan et al. (2011), `VRP/docs/On the use of Monte Carlo...pdf`.
- Clarke y Wright (1964), *Oper. Res.* 12(4):568–581: como en el artículo VRP.
- Juan et al. (2011), *J. Oper. Res. Soc.* 62(6):1085–1097: primera página del PDF en `VRP/docs/`; distribución geométrica con β ~ U(0.05, 0.25) en p. 1088.
- Reinelt (1991), *ORSA J. Comput.* 3(4):376–384: comprobado en `TSP/docs/TSP_heuristic.pdf` (ref. [3]) y `TSP/README.md`.

## Pendiente

- **Enunciado no encontrado**: no se sabe si pide el código como apéndice o en fichero aparte. Se ha puesto *Code availability* (estilo POP). Si se pide apéndice, el código es un notebook: hay que decidir qué listar.
- **Figuras con rótulos en español** («Gap por instancia», «GRASP estándar», «Sesgada geométrica»…): las salidas del notebook EN vienen de la ejecución en español. Regenerarlas exige ejecutar la celda de gráficos; no se ha hecho.
- **Equipo de cómputo desconocido**: el notebook no registra CPU ni hilos (Python 3.13.16; dirección `0x7f…` sugiere Linux). Falta el dato para la frase de tiempos.
- **Discrepancia**: el notebook y la presentación dicen «≈1.7 veces más lentos» (Zipf/softmax); eso solo vale en a280 (1.75/1.71); en media sobre las 7 instancias es 1.47 (rango 1.258–1.752). El artículo da el rango, como «about 1.3–1.75» desde la ronda 3. El texto de la pregunta 4 de `preguntas_abiertas.md` sigue diciendo 1.26–1.75.
- ~~No hay carpeta `data/` en GRASP (`TSP_DIR = "ALL_tsp"`).~~ Resuelto el 28-10-2026: las 10 instancias (y sus `.opt.tour`) se copiaron de `TSP/data/` a `GRASP/data/` y el notebook lee de `../data`.
- La explicación del peor resultado de la triangular en a280 (78 frente a 10 posiciones) es una hipótesis, marcada con «possibly» en el texto.
- Nombre de los ficheros: `docs/GRASP for the Travelling Salesman Problem.tex` y `docs/Javier S Jesus I GRASP for the Travelling Salesman Problem.pdf`; cambiar si el usuario prefiere otro.
- `results/` se ha creado con CSV y PNG extraídos de las salidas guardadas (con permiso del usuario).
- **Corte de la triangular**: el artículo ya dice que la regla triangular tiene corte (sobre el rango), como `select_triangular` (celda 7). La celda 6 del notebook, la diapositiva 6 y `docs/presentation_script.md`, l. 109, siguen diciendo que ninguna regla sesgada lo tiene; el orquestador avisa a los autores.
