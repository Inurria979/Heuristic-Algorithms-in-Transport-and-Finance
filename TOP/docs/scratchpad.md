# TOP — scratchpad de la sesión (11-10-2026)

Notas internas para retomar el trabajo en otra sesión: qué se ha hecho, por qué, cómo se
ejecuta y qué falta. El informe con resultados está en [`informe_alpha.md`](informe_alpha.md).

## 1. La actividad

`docs/homework.png`:

1. **Implementar la heurística PJS para el TOP y probarla en varias instancias.** Hecho.
2. **Análisis de datos de los resultados.** Hecho para la rejilla de α y GRASP (`informe_alpha.md`).
3. *(Opcional)* **Combinar GRASP con PJS.** Hecha la construcción (multiarranque aleatorizado) y la búsqueda local sobre las soluciones élite (sección 6b, `informe_busqueda_local.md`).
4. *(Opcional)* **Mejorar GRASP-PJS con aleatorización sesgada.** Hecho con la geométrica; también hay softmax.

**Documentos de la búsqueda local** (12-10-2026):

- [`busqueda_local.md`](busqueda_local.md): todas las opciones de movimientos, con sus pros y contras.
- [`busqueda_local_diseno.md`](busqueda_local_diseno.md): las decisiones acordadas y el pseudocódigo.
- [`informe_busqueda_local.md`](informe_busqueda_local.md): los resultados, generado por `pjs grid`.

Papers, en `docs/`; su texto extraído está en `ignore/papers_txt/`, con las letras griegas intactas:

- **Panadero et al. (2021)**, *Applied Sciences* 11, 12092 (`applsci-11-12092-v2.pdf`). Es el paper base.
  - Sección 4: la heurística.
  - Sección 5: multiarranque y paralelismo (Algoritmo 1).
  - Tablas 3 y 4: BKS y resultados con 1 a 128 hilos, solo para p1 y p2.
- **Li et al. (2022)**, *Mathematics* 10, 982 (`mathematics-10-00982-v2.pdf`).
  - TOP dinámico con visitas obligatorias; usa la misma heurística.
  - Su Algoritmo 1 es el multiarranque que seguimos.

## 2. Datos

- **Ubicación:** `TOP/src/data/top-data/`, 387 instancias de Chao et al. (1996). Están en `src/` y no en `data/` como pide el `CLAUDE.md`; moverlas sigue pendiente.
- **Formato:** `n;N`, `m;M`, `tmax;T`, y después N líneas `x;y;recompensa`. El primer nodo es el origen y el último el destino. Los tiempos son distancias euclídeas.
- **Instancias infactibles:** 34, en las que ningún cliente cabe en `tmax` ni siquiera solo. Quedan **353 instancias útiles**. El paper dice 354 en el texto y su tabla 1 suma 389; no cuadra.
- **Origen y destino:** son puntos distintos en p1–p6 y coinciden en p7.

## 3. Código: `TOP/src/rust/` (proyecto Cargo)

**No se usa Python en el flujo: Rust con rayon es la base**, porque se quieren lanzar muchas instancias. Python solo se usó una vez como comprobación independiente.

| Fichero | Contenido |
|---|---|
| `model.rs` | `Instance` (lectura, matriz de distancias, `t(i,j)`, `route_time`, `reachable`), `Route`, `Solution` y `Solution::check()`, la comprobación independiente de factibilidad |
| `heuristic.rs` | La heurística PJS por pasos, el trait `ArcSelector`, `Greedy`, `SavingsList` (lista de ahorros reutilizable), `construct()` y `pjs()` |
| `grasp.rs` | Los selectores aleatorios (geométrica, softmax con árbol de sumas), `grasp()` (multiarranque), `stream_seed()` y `grasp_runs()` (ejecuciones en paralelo); incluye tests unitarios |
| `alpha_grid.rs` | El experimento de la rejilla: tareas planas en paralelo, el mejor α y los CSV. Guarda también las soluciones (`det_solutions`, `grasp_solutions`) para la búsqueda local |
| `elite.rs` | Elección de las élite (`top_with_ties`), eliminación de duplicados, `run_local_search()` en paralelo, `local_search.csv` y `best_solution.csv` |
| `local_search.rs` | La búsqueda local: `two_opt()`, `insert_one()` y `local_search()`, con sus tests |
| `plot.rs` | Figuras PNG con plotters: rutas, recompensa total frente a α y mapa de calor |
| `report.rs` | El informe Markdown `docs/informe_alpha.md`, generado desde Rust |
| `ls_report.rs` | El informe Markdown `docs/informe_busqueda_local.md` |
| `panadero.rs` | El subcomando `pjs panadero`: experimento de 1 s por ejecución con el mejor α, élite y búsqueda local, y la comparación con las tablas 3 y 4 de Panadero (`docs/comparacion_panadero.md`) |
| `io.rs` | Cargar las instancias de una carpeta y escribir ficheros |
| `main.rs` | La línea de comandos, con los subcomandos `grid` y `grasp` |

**Compilación**, siempre desde `TOP/src/rust`, para que se lean `rust-toolchain.toml` y `.cargo/config.toml`:

```
cd TOP/src/rust
cargo build --release      # ejecutable: TOP/ignore/target/release/pjs.exe
cargo test --release       # tests de los selectores, la búsqueda local y la elección de élite
```

Decisiones de entorno y por qué:

- **Toolchain GNU** (`rust-toolchain.toml` → `stable-x86_64-pc-windows-gnu`, rustc 1.99). En el equipo no hay Visual Studio C++, y el toolchain GNU trae su propio linker. El de MSVC sigue siendo el predeterminado fuera de este proyecto.
- **`target-dir = ../../ignore/target`**, para que no haya compilados dentro de `src/`.
- **Optimizaciones agresivas en `release`:** `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` y `target-cpu=native`. El usuario quiere un solo perfil.
  - Dan entre un 5 y un 10 % más de velocidad. Una recompilación tras tocar el código pasa de unos 10 s a unos 38 s.
  - No cambian ningún resultado: verificado byte a byte.
- **Dependencias** (la mayoría sin opciones por defecto, para evitar `dlltool`, que el toolchain GNU no trae):
  - `plotters` sin opciones por defecto, con `ab_glyph` y la fuente Arial de Windows leída al ejecutar.
  - `rand` 0.10 sin opciones por defecto, solo `std_rng`. No necesitamos aleatoriedad del sistema operativo, porque todas las semillas son explícitas.
  - `rayon`.

## 4. La heurística PJS (Panadero, sección 4)

1. **Solución dummy:** una ruta 0 → i → destino por cliente. Se descartan los clientes que ni solos caben en `tmax`.
2. **Ahorro enriquecido:** s'ᵢⱼ = α·sᵢⱼ + (1 − α)·(uᵢ + uⱼ), con sᵢⱼ = t(i, destino) + t(0, j) − t(i, j). La lista se ordena de mayor a menor con un orden estable, así que en los empates se mantiene el orden de generación.
3. **Fusión:** el arco (i, j) une la ruta que termina en i con la que empieza en j si el resultado cabe en `tmax`. El arco se quita de la lista tanto si se fusiona como si no.
4. **Selección:** se ordenan las rutas por recompensa (en empate, la más corta primero) y se quedan las m mejores.

Decisiones nuestras, porque el paper no las fija:

- **Arcos en los dos sentidos.** Panadero dice que el ahorro no depende del sentido, pero con origen y destino distintos s_ij ≠ s_ji. Li et al. lo definen para cada par (i, j) con i ≠ j, que es lo que hacemos.
- **Tolerancia `EPS = 1e-9`** en la comprobación de `tmax`.
- **Filtro de velocidad en `try_merge`:** primero una estimación incremental del tiempo y después el cálculo exacto, que es el que decide. No cambia ningún resultado.

## 5. El parámetro α: historia y decisiones

- **El paper no da ningún valor de α.** Solo dice que va cerca de 0 si las recompensas son heterogéneas y cerca de 1 si son homogéneas. Li da el rango (0.1, 0.9). El único parámetro con valor es **β = 0.3**.
- **Reglas probadas y descartadas:**
  - α a partir del coeficiente de variación de las recompensas: α = 1/(1+CV), 1/(1+CV²) y exp(−k·CV).
  - Un estudio con predictores sobre rasgos de la instancia (regresión, árboles, kNN, validando con un conjunto fuera).
  - **Ninguna superó a un α fijo.** La recompensa salta de forma irregular con α: pequeños cambios reordenan la lista de ahorros y la heurística voraz construye rutas distintas.
- **Decisión: rejilla de α y quedarse con el mejor de cada instancia.** Primero se probaron 100 valores (k/99). Ahora son **21 valores: 0, 0.05, 0.10, …, 0.95, 1** (`ALPHAS = 21`, `grid_alpha(k) = k/20`).
- **Criterio:** solo cuenta la **recompensa total**; el tiempo de las rutas solo tiene que respetar `tmax`. Si varios α empatan, todos son óptimos. Se informa del menor, de cuántos son y de su rango.

## 6. GRASP: construcción

**Qué es una iteración y qué es una ejecución.** Así lo plantean los papers:

- **Una iteración** es una construcción aleatorizada completa: dummy, lista de ahorros, arcos elegidos al azar, fusión y selección de m rutas. Equivale a `BRA(...)` en Li et al.
- **Una ejecución de GRASP** es el multiarranque (Li et al., Algoritmo 1). Parte de la solución determinista y repite iteraciones, quedándose con la mejor, hasta un número de iteraciones (`--iters`) o un tiempo (`--time`). Equivale a un hilo del Algoritmo 1 de Panadero, que itera durante 1 s.
- **Dentro de GRASP no hay mejora de soluciones.** Solo se generan muchas soluciones y se guarda la mejor. La búsqueda local se aplica después, sobre las élite (sección 6b).

**Selectores** (trait `ArcSelector`: `start(savings)` y `next()`, que devuelve el índice del siguiente arco, cada uno una sola vez):

- **`Greedy`:** siempre el arco de mayor ahorro que quede. Es la heurística determinista.
- **Geométrica** (Panadero): se toma el k-ésimo arco de los que quedan, contando desde arriba, con k ~ Geom(β), P(k) = β(1−β)ᵏ.
  - k se obtiene por inversión: k = ⌊ln U / ln(1−β)⌋, y se toma módulo el número de arcos que quedan, como en el código del grupo de Juan.
  - β = 0.3 por defecto. β → 1 equivale al voraz.
- **Softmax** (propuesta del usuario): cada arco que queda se elige con probabilidad exp(s'/τ) / Σ exp(s'/τ). τ = 1 es el softmax "puro".
  - Se implementa con un árbol de sumas: elegir y quitar un arco cuesta O(log n).
  - Si los pesos se van a cero por desbordamiento, se reconstruyen tomando como referencia el mejor arco que quede.

**Semillas.**

- La semilla fija el punto de partida del generador pseudoaleatorio. Con la misma semilla, la ejecución se repite exactamente; sirve para reproducir y verificar resultados.
- Dentro de una ejecución, el generador avanza con cada elección, así que todas las iteraciones son distintas. Se siembra **una vez por ejecución**, como en el Algoritmo 1 de Panadero: una semilla por hilo.
- `stream_seed(base, nombre_instancia, etiquetas)` deriva una semilla distinta por (instancia, α, aleatorización, ejecución). Así el resultado no depende del orden de los hilos ni de si la instancia se resuelve sola o con toda la carpeta.

## 6b. Búsqueda local sobre las élite

El detalle está en [`busqueda_local_diseno.md`](busqueda_local_diseno.md). Lo acordó el usuario el 12-10-2026:

- **Va al final, no dentro de las iteraciones de GRASP.**
- **Élite por instancia:**
  - deterministas: las α con las 3 mejores recompensas;
  - por aleatorización y α: las 3 mejores ejecuciones, de cada una su mejor solución.
- **Criterio:** solo la recompensa. Los empates en la tercera entran todos, aunque sean más de 3.
- **Duplicados:** dos élite son iguales si visitan el mismo conjunto de clientes. Se descarta la repetida sin sustituirla, también entre grupos, en el orden determinista → geométrica → softmax, α creciente, ejecución creciente.
- **Búsqueda local:** en bucle,
  1. 2-opt dentro de cada ruta, con primera mejora;
  2. insertar un cliente: entre los no visitados que caben, el de mayor recompensa (en empate, el que menos tiempo añade), en su posición más barata de cualquier ruta.

  Se repite mientras se inserte alguno.
- **Primera ejecución completa:** 125 490 élite elegidas, 18 791 tras quitar duplicados. La búsqueda local tarda 0,4 s.

## 7. Experimentos actuales y cómo se ejecutan

Todo desde `TOP/`, con `E=ignore/target/release/pjs.exe`.

**Experimento principal (rejilla + búsqueda local):**

```
$E grid --png-dir results/figures/routes --report docs/informe_alpha.md \
    --ls-png-dir results/figures/routes_ls --ls-report docs/informe_busqueda_local.md src/data/top-data results
```

La búsqueda local siempre se ejecuta. `--ls-png-dir` y `--ls-report` son opcionales; tarda unos 2 min 10 s en total.

- **Por cada instancia y cada uno de los 21 α:**
  - 1 construcción determinista;
  - **10 ejecuciones de GRASP geométrico** (β = 0.3), cada una de **10 iteraciones**;
  - **10 ejecuciones de GRASP softmax** (τ = 1), cada una de **10 iteraciones**.
- **Opciones:** `--reps 10 --iters 10 --geometric 0.3 --softmax 1 --seed 1` son los valores por defecto.
- **Escala:** 387 × 21 × 21 = 170 667 tareas en una sola lista paralela de rayon, unos 140 s con 12 hilos.
- **Salidas:**
  - `results/alpha_grid.csv`: una fila por instancia y α. Determinista, y para cada aleatorización la mejor, media, desviación y peor recompensa.
  - `results/grasp_runs.csv`: una fila por ejecución de GRASP (162 540 filas, 11 MB), con semilla, recompensa e iteración de la mejor.
  - `results/best_alpha.csv`: una fila por instancia.
  - `results/figures/alpha_total_reward.png` y `alpha_heatmap.png`.
  - `results/figures/routes/<instancia>.png`: rutas con el mejor α determinista.
  - `docs/informe_alpha.md`.
  - `results/local_search.csv`: una fila por élite, con la recompensa y el tiempo antes y después, los movimientos 2-opt y los clientes insertados.
  - `results/best_solution.csv`: una fila por instancia, con la determinista, GRASP, la determinista con búsqueda local, la final y su origen.
  - `results/figures/routes_ls/<instancia>.png` (59 MB) y `docs/informe_busqueda_local.md`.
- **Con una sola instancia:** `$E grid src/data/top-data/p4.4.t.txt [--png out.png]` imprime una tabla por α, las rutas y una tabla de las élite antes y después de la búsqueda local.

**GRASP con una configuración concreta:**

```
$E grasp [--alpha A|best] [--geometric B | --softmax T] [--iters N] [--time S] [--runs R] [--seed S] <instancia|carpeta> [salida.csv]
```

Por defecto: `--alpha best` (el mejor α determinista de la rejilla), geométrica con β = 0.3, **1000 iteraciones**, 1 ejecución y semilla 1.

**Resultados actuales** (en `informe_alpha.md`):

| | Recompensa total | Frente al determinista |
|---|---|---|
| Determinista, mejor α de cada instancia | 157 868 | — |
| Geométrica, mejor de las 10 ejecuciones | 169 571 | +7.41 % (mejora 243 instancias) |
| Geométrica, mejor media de 10 | 164 835 | +4.41 % |
| Softmax, mejor de las 10 ejecuciones | 164 434 | +4.16 % |
| Softmax, mejor media de 10 | 159 692 | +1.16 % |

- El mejor α común determinista es 0.40, con 150 585.
- El softmax con τ = 1 casi no aporta en p4 (+0.9 %). Con ahorros que se diferencian en decenas de unidades, τ = 1 deja el softmax casi como el voraz. Una idea pendiente es una rejilla de τ, o τ proporcional a la desviación típica de los ahorros.
- **Comparación con el paper** (solo p1 y p2, BKS de las tablas 3 y 4): el determinista queda a un 6–8 % del BKS. Panadero, con 1 hilo durante 1 s, queda a un 1.73 %, y con 128 hilos a un 0.44 %.

**Resultados con búsqueda local** (en `informe_busqueda_local.md`, sobre las 353 instancias factibles):

| | Recompensa total | Frente al determinista |
|---|---|---|
| Determinista, mejor α | 157 868 | — |
| Mejor GRASP (cualquier tipo), sin búsqueda local | 170 469 | +7.98 % |
| Élite deterministas + búsqueda local | 181 103 | +14.72 % |
| **Final: todas las élite + búsqueda local** | **189 854** | **+20.26 %** |

- La búsqueda local mejora el 93.6 % de las élite, con una media del +18 % e inserta 7.5 clientes de media. Donde más aporta es en p4 (+28 % sobre GRASP) y en p7 (+18.5 %).
- En 163 de 353 instancias la mejor final sale de una élite que **no** era la mejor antes de la búsqueda local.
- **Frente a Panadero** (tablas 3 y 4, 80 instancias de p1/p2), detallado en [`comparacion_panadero.md`](comparacion_panadero.md):
  - ninguna solución supera el BKS y 72 lo igualan;
  - distancia media al BKS por instancia, la medida del paper: **0.35 % la nuestra**, 0.45 % Panadero con 128 hilos y 1.75 % con 1 hilo;
  - frente a Panadero con 128 hilos, la nuestra es mejor en 10 instancias, igual en 64 y peor en 6;
  - **cómo se genera:** los datos del paper están extraídos en `src/data/panadero2021_p1_p2.csv`, y la comparación la escribe `pjs panadero` (en Rust).

**Experimento de 1 s como Panadero** (`pjs panadero`, 12-10-2026), sobre las 80 instancias de sus tablas:

- **Configuración:** el mejor α de la rejilla y 10 ejecuciones geométricas más 10 softmax, de **1 s de reloj** cada una, en paralelo con 12 hilos. Después, élite (la determinista y las 3 mejores ejecuciones de cada tipo) y búsqueda local. Tarda 134 s.
- **No es reproducible exactamente**, porque usa tiempo de reloj; el usuario lo eligió así a sabiendas.
- **Iteraciones por ejecución:** unas 39 000 en p1 y unas 50 000 en p2.
- **Resultado:** distancia media al BKS del **0.72 %**, peor que la rejilla (0.35 %) y que Panadero con 128 hilos (0.45 %). Una sola ejecución de 1 s, sin búsqueda local, da de media un 2.4–2.5 %, frente al 1.75 % de Panadero con 1 hilo.
- **Lectura:** con un solo α, miles de iteraciones rinden menos que las 21 α × 20 ejecuciones × 10 iteraciones de la rejilla. Probar varios α aporta más que iterar más con uno fijo.
- **Salidas:** `results/panadero_1s.csv` (una fila por instancia), `results/panadero_1s_runs.csv` (una por ejecución) y `docs/comparacion_panadero.md`.

```
$E panadero --report docs/comparacion_panadero.md src/data/top-data src/data/panadero2021_p1_p2.csv results/best_solution.csv results
```

Opciones: `--alpha best --time 1 --runs 10 --geometric 0.3 --softmax 1 --seed 1` son los valores por defecto. Necesita el `best_solution.csv` de `pjs grid`.

**Variante con la rejilla de α** (`--alpha grid`, 12-10-2026). Solo para las instancias de Panadero; `pjs grid` no cambia.

- **Configuración:** los 21 α con 10 + 10 ejecuciones de 1 s cada uno. Élite con las reglas de la rejilla y búsqueda local.
- **Semillas:** las mismas que en la rejilla, así que cada ejecución empieza igual que la de 10 iteraciones y sigue iterando.
- **Coste:** 33 600 ejecuciones, unos 47 min.
- **Salidas:** `results/panadero_1s_grid.csv`, `results/panadero_1s_grid_runs.csv` y `docs/comparacion_panadero_rejilla.md`. Con `--best-csv results/panadero_1s.csv` incluye también los resultados con el mejor α.

```
$E panadero --alpha grid --best-csv results/panadero_1s.csv --report docs/comparacion_panadero_rejilla.md \
    src/data/top-data src/data/panadero2021_p1_p2.csv results/best_solution.csv results
```

**Resultado (2 801 s ≈ 47 min):**

- **Distancia media al BKS: 0.15 %.** Es la mejor de todas: la rejilla de 10 iteraciones da 0.35 %, Panadero con 128 hilos 0.45 % y 1 s con el mejor α 0.72 %.
- **BKS alcanzado en 76 de 80 instancias.** Solo se quedan cortas p1.2.i, p1.2.l, p1.2.m y p1.3.l, que son también las 4 en las que pierde frente a Panadero con 128 hilos. Le gana en 12 instancias y empata en 64.
- **Frente a la rejilla final:** mejor en 5 instancias e igual en 75. Frente a 1 s con el mejor α: mejor en 14 e igual en 66.
- **Las 33 600 ejecuciones:** ninguna peor que su equivalente de 10 iteraciones y 12 996 mejores, como corresponde a semillas compartidas.
- **Iteraciones por ejecución:** unas 40 600 en p1 y unas 56 200 en p2.

**Variante con iteraciones fijas** (`--iters N` en lugar de `--time`, 12-10-2026). Es reproducible.

- **Comprobación:** `--alpha grid --iters 10` reproduce exactamente la rejilla principal en las 80 instancias (0 diferencias en `best_no_ls` y `best_ls`).
- **Con `--iters 100`** (29 s):
  - distancia media al BKS del **0.29 %**, frente al 0.35 % con 10 iteraciones y el 0.15 % con 1 s;
  - BKS en 73 de 80 instancias;
  - frente a la rejilla de 10 iteraciones: mejor en 2, igual en 77 y peor en 1 (p1.4.r: 205 frente a 210).
- **Por qué p1.4.r empeora**, aunque ninguna de las 33 600 ejecuciones es peor que su equivalente de 10 iteraciones: la élite se elige por la recompensa antes de la búsqueda local. Con 10 iteraciones, el 210 salía de una élite softmax que antes de la búsqueda local solo valía 165. Con 100 iteraciones esa ejecución cambia y la búsqueda local ya no llega a 210. La búsqueda local no es monótona respecto a la calidad de partida.
- **Salidas:** `results/panadero_100it_grid.csv`, `results/panadero_100it_grid_runs.csv` y `docs/comparacion_panadero_rejilla_100it.md` (con `--best-csv results/panadero_1s_grid.csv`).

```
$E panadero --alpha grid --iters 100 --best-csv results/panadero_1s_grid.csv --report docs/comparacion_panadero_rejilla_100it.md \
    src/data/top-data src/data/panadero2021_p1_p2.csv results/best_solution.csv results
```

**Comprobación:** con 0,01 s, de 6 720 ejecuciones emparejadas con la rejilla solo 2 salieron peores, y fue porque hicieron menos de 10 iteraciones (arranque de los hilos). Las que llegan a 10 iteraciones nunca son peores, como corresponde a semillas compartidas.

**Aviso:** `docs/comparacion_panadero.md` (modo best) se generó con la versión anterior del informe, con una columna α distinta y otro título. Regenerarlo cambiaría las cifras, porque usa tiempo de reloj.

**Verificaciones hechas:**

- `Solution::check()` en cada solución, también después de la búsqueda local.
- Al añadir la búsqueda local, `alpha_grid.csv` y `best_alpha.csv` salen idénticos, y `grasp_runs.csv`, idéntico salvo la columna de segundos.
- Tests de la búsqueda local (cruce que el 2-opt deshace, inserción del mejor que cabe, desempate por menor tiempo, monotonía e idempotencia) y de `top_with_ties`.
- Las 38 700 ejecuciones deterministas de la rejilla antigua comparadas con una implementación independiente en Python: 0 discrepancias.
- Tests de los selectores: cada arco se elige una sola vez, Geom(0.3) y softmax con su distribución teórica, desbordamiento y árbol de sumas.
- Cada refactorización y cada optimización comprobada con salidas idénticas byte a byte.

## 8. Pendiente

1. ~~**Búsqueda local de GRASP**~~: hecha el 12-10-2026 (sección 6b). Ideas que siguen abiertas, recogidas en `busqueda_local.md`:
   - el reemplazo de un cliente visitado por uno no visitado;
   - los movimientos entre rutas;
   - meter la comparación con el BKS en el informe.
2. ~~Ficheros obsoletos~~: los dos CSV antiguos de `pjs grasp` (`grasp_geometric_0.3.csv`, `grasp_softmax_1.csv`) se borraron al cerrar la sesión.
3. **Andamiaje de `TOP/`:**
   - falta su `README.md`;
   - falta la fila de TOP en las tablas del README raíz y del `CLAUDE.md`;
   - hay que mover `src/data/top-data/` a `data/`, actualizando las rutas de los comandos.
4. **Tamaño de `results/`:** unos 60 MB de PNG de rutas deterministas, otros 59 MB de rutas tras la búsqueda local y los 11 MB de `grasp_runs.csv`, y `results/` se sube a GitHub. Hay que decidir si va todo o solo una selección.
5. **Commit:** la rejilla está en git (commit `b1f0fce`). La búsqueda local, sus documentos y sus resultados están sin commit.
6. ~~**Posible optimización:** compartir la lista de ahorros y la solución determinista entre las 20 ejecuciones de GRASP de cada (instancia, α).~~ **Descartada** tras medirla el 11-10-2026 (detalle en [`coste_lista_ahorros.md`](coste_lista_ahorros.md)):
   - la lista y la construcción determinista son solo el **3,5–5,4 %** de una ejecución de 10 iteraciones, tanto en p2 (21 nodos) como en p7 (102 nodos);
   - compartirlas ahorraría unos 6 s de los 140 s de la rejilla.

   **Lo que de verdad cuesta son las construcciones aleatorias:** cada una tarda entre 8 y 12 veces más que la voraz (en p7, ~1,3–1,6 ms frente a 135 µs). Si hace falta acelerar la rejilla, hay que mirar los selectores, sin investigar todavía.

## 9. Preferencias del usuario aprendidas en la sesión

- Conversación en español; código e informes generados por el código, en inglés salvo el informe, que está en español.
- **Nada de Python en el flujo**; todo en Rust con rayon, paralelizando todo lo posible.
- **No implementar mientras se está hablando**: si dice "estamos hablando", parar y responder.
- Un solo perfil de compilación, con las optimizaciones agresivas.
- Prefiere explicaciones apoyadas en el texto de los papers.
