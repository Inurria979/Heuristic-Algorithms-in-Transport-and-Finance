# Scratchpad del repositorio — traspaso entre sesiones

Notas internas, en español, para que otra sesión (de Claude o de un compañero) siga desde
aquí. Última actualización: **28-10-2026**. Las notas de un solo proyecto van en su propio
`docs/` (p. ej. [`TOP/docs/scratchpad.md`](TOP/docs/scratchpad.md)); aquí va lo que afecta a
varios proyectos o al repositorio entero.

**Qué leer al empezar:** este fichero → `CLAUDE.md` → el `README.md` del subproyecto →
`<P>/docs/preguntas_abiertas.md`.

---

## 1. Agentes y comandos de `.claude/` (creados el 28-10-2026)

Todo está en `.claude/` y se sube a git para que lo tengan los compañeros
(`settings.local.json` no se sube).

| Pieza | Fichero | Qué hace |
|---|---|---|
| `/paper-loop <P> [rondas]` | `.claude/skills/paper-loop/SKILL.md` (+ `guia_estilo.md`, `plantilla.tex`) | Bucle escritor–revisor del artículo de 3 páginas. Ya existía. |
| `paper-writer` / `paper-reviewer` | `.claude/agents/` | Escriben y revisan el artículo. No ejecutan nada del proyecto: solo leen salidas guardadas, `results/` y documentos. Ya existían. |
| `/orquestar <P> <objetivo> [rondas]` | `.claude/skills/orquestar/SKILL.md` | Convierte la sesión en orquestador: lee `.claude/agents/orquestador.md` y sigue su protocolo. |
| `orquestador` | `.claude/agents/orquestador.md` | Planifica (código → `results/` → README → artículo), **espera el visto bueno del usuario** antes de lanzar, reparte entre los agentes y no escribe código ni artículos. Plan en `<P>/ignore/orquestador/plan.md`. Solo funciona como sesión principal (`/orquestar` o `claude --agent orquestador`): un subagente no puede lanzar a otros. |
| `code-writer` | `.claude/agents/code-writer.md` | Implementa y verifica ejecutando. No cambia parámetros publicados, no toca `data/` ni los artículos, no arregla desviaciones de estructura previas. Lista en `cambios_r{k}.md` todos los ficheros que toca. |
| `code-reviewer` | `.claude/agents/code-reviewer.md` | Revisa solo lo que cambia la tarea (lo compara con `estado_inicial.txt`, que guarda el orquestador). Puede ejecutar pruebas pequeñas, pero nunca escribe en `src/`, `data/` ni `results/` (salidas a `<D>/tmp/`, `uv run --locked`, nada de `--inplace`). Veredicto APROBADO / CAMBIOS NECESARIOS. |

Bucle de código: carpeta por tarea, `<P>/ignore/code_loop/<tarea>/` (`estado_inicial.txt`,
`cambios_r{k}.md`, `revision_r{k}.md`). Dentro de un subproyecto, las tareas de código van una
detrás de otra. Todavía **no se ha probado** ningún bucle de código ni `/orquestar` de verdad.

## 2. Reorganización del 28-10-2026

TOP, GRASP y PFSP no seguían la estructura de `CLAUDE.md`. Cambios hechos (sin commit hasta
el commit de esta fecha):

- **TOP:** `src/data/` → `data/` con `git mv`. Se cambió `src/data/` por `data/` en 11 `.md` de
  `docs/`, en el `.tex` y en los textos y comentarios de 6 ficheros del crate (solo cadenas: el
  crate compila y no cambia ningún resultado). PDF recompilado: 3 páginas. Nuevo `README.md`.
  No tiene `pyproject.toml` a propósito: es solo Rust.
- **GRASP:** el notebook leía de `ALL_tsp`, que no existe. Se copiaron las 10 instancias que usa
  (y sus 9 `.opt.tour`; `kroB100` no tiene) de `TSP/data/` a `GRASP/data/`, y en los dos
  notebooks `TSP_DIR` pasó a `"../data"` (solo esa línea; salidas intactas). Nuevos
  `pyproject.toml`, `uv.lock`, `.python-version` y `README.md`.
- **PFSP:** `notebook/` → `src/` con `git mv`; ruta actualizada en el `.tex` y en el esquema.
  PDF recompilado: 3 páginas. Nuevos `pyproject.toml`, `uv.lock`, `.python-version` y `README.md`.
- **GRASP y PFSP fijan Python 3.13**, no 3.14: es la versión con la que se ejecutaron los
  notebooks (3.13.16 en sus metadatos) y la que citan los artículos. Numba 0.68.1 resuelve bien.
- `CLAUDE.md` y el `README.md` raíz tienen ya las filas de TOP, GRASP y PFSP y sus excepciones.
- Las cifras de los tres README nuevos salen de los CSV de `results/`, no de los artículos.

## 3. Verificación artículos ↔ código (28-10-2026)

### Cómo se hizo

Se lanzaron en paralelo cinco agentes `paper-reviewer`, uno por artículo (POP, VRP, GRASP,
PFSP, TOP), con un encargo **reducido**: comprobar solo que lo que dice el artículo coincide con
lo que produce el código, sin revisar estilo, longitud, inglés ni enunciado. Reglas del encargo:

- **No ejecutar nada del proyecto.** Fuentes de verdad, en orden: código (`src/`, el crate de
  TOP), salidas guardadas de los notebooks (leídas con scripts que vuelcan el JSON), CSV de
  `results/` y, en TOP, los informes Markdown que genera `pjs`.
- Comprobar todas las cifras del abstract, las tablas, el texto, las leyendas y las conclusiones,
  y recalcular los agregados desde los CSV crudos con scripts de un solo uso en
  `<P>/ignore/paper_loop/tmp/`.
- Comprobar los parámetros citados frente al código, la descripción del algoritmo frente a lo que
  hace el código y que las figuras y tablas existen.
- Gravedad: **B** = cifra o parámetro falso; **I** = impreciso, redondeo engañoso o sin fuente en
  el código; **M** = trivial.
- Informe en `<P>/ignore/paper_loop/verificacion_codigo.md`, con otro nombre que `revision_r*.md`
  para no romper la numeración de `/paper-loop`. **`ignore/` no se sube**: los informes y
  scripts solo están en el equipo de Jesús. El resumen de abajo recoge todo lo necesario.

Para repetirlo en otro artículo, basta lanzar `paper-reviewer` con un prompt que diga
«verificación puntual de datos, NO es una ronda de /paper-loop», las fuentes del proyecto y las
reglas de arriba.

### Resultado

| Artículo | Estado | B | I | M |
|---|---|---|---|---|
| GRASP | por entregar | 0 | 1 | 6 |
| TOP | por entregar | 0 | 4 | 5 |
| PFSP | por entregar | 0 | 6 | 6 |
| VRP | entregado | 1 | 3 | 4 |
| POP | entregado | 1 | 8 | 4 |

Las cifras de las tablas principales coinciden con el código en los cinco. Los problemas son de
redacción, de cifras sin fuente en el código o de detalles del algoritmo mal descritos.
Líneas referidas al `.tex` del 28-10-2026.

#### GRASP (`GRASP/docs/GRASP for the Travelling Salesman Problem.tex`)

- **I1, Conclusions, l. 242:** «more than halves the gap of the greedy savings tour (5.07 to
  2.22 %)». El 5.07 % es el tour voraz **seguido de 2-opt**; sin 2-opt es 7.60 % (Tabla 1,
  l. 210). → «greedy savings tour followed by 2-opt».
- M: l. 162–164, el código solo comprueba que el tour es una permutación antes del 2-opt, no
  después. l. 221–222, en eil51 está probado que el coste es el mismo (432), no que lo sea el
  tour. l. 233, 0.14–10.18 s son medias de 20 ejecuciones, y el segundo decimal de 0.77 y 1.26
  no se sostiene con tiempos redondeados a 0.01 s (→ «about 0.8–0.95», «about 1.3–1.75»).
  l. 148, «same effort», pero las rejillas tienen 6–8 valores (→ «every grid value receives
  the same effort»). l. 205, en español están el título general y la leyenda de la Figura 1,
  no los paneles. l. 173, la leyenda de la Tabla 1 no dice que CW es una sola ejecución.
- Comprobado: los notebooks EN y ES tienen el mismo código; las salidas del EN son la ejecución
  española traducida a mano. Los CSV y PNG de `results/` coinciden con las salidas (los PNG,
  byte a byte). Friedman χ² = 1.87, p = 0.7596.

#### TOP (`TOP/docs/Team Orienteering Problem.tex`)

- **I1, §4 l. 179–180 y Tabla 1:** llama «the paper's» a gap = 100 (BKS − reward)/BKS. El
  código divide por el BKS, pero Panadero divide por **su propia recompensa** en las 81 filas
  de las tablas 3–4. Con ese denominador nuestra final da **2.17 %** en total (no 2.04 %); por
  conjunto, p1 0.60, p2 0.00, p3 0.70, p4 6.38, p5 1.35, p6 1.22, p7 3.61. Sigue por debajo de
  Panadero en los siete. → quitar «the paper's» y decir que el paper divide por su recompensa.
- **I2, §5 l. 277–280:** «960 000 construcciones… 23 veces menos trabajo» es cierto en media,
  pero la infla p2.4.a, una instancia trivial con 7.4 M construcciones. Sin ella, 876 008 y un
  cociente de 20.9; en tiempo de hilo, 20.0 s frente a 4.09 s por instancia, unas 4.9 veces.
  → «about 20 times fewer constructions and a fifth of the thread time».
- **I3, §4 l. 185–186:** «an independent Python implementation reproduced 38 700 deterministic
  constructions». No hay código ni salida en el repo (solo el scratchpad de TOP), y 38 700 =
  387 × 100 es la rejilla **antigua** de 100 α, no la de 21. → precisarlo o quitarlo.
  **Decisión de los autores.**
- **I4, §3.1 l. 135–136:** «rules based on the CV or on instance features did not beat a fixed
  α». Sin respaldo en código ni informes, solo en notas a mano. → «in preliminary trials, not
  part of the published code» o quitarlo. **Decisión de los autores.**
- M: la leyenda de la Figura 1 dice «marker area proportional to reward», pero el radio es
  3 + 6√(u/umax) (`plot.rs` l. 128; → «marker size grows with reward»). El 68 % de p4 es sobre
  54 instancias (69.1 % sobre 56). r = +0.19 es con la media de los α empatados; con el menor
  α óptimo da +0.43 (sobra «weak»). *Code availability* está incompleto (ejecutable en
  `ignore/target/release/`, `--png-dir` y tres ejecuciones de `pjs panadero`). El *Protocol*
  no menciona el experimento de 1 s con solo el mejor α.
- Comprobado: parámetros, descripción del algoritmo y las 162 540 semillas de `grasp_runs.csv`
  (reimplementando `stream_seed`); el CSV de Panadero es idéntico a sus tablas 3–4.

#### PFSP (`PFSP/docs/Permutation Flow-Shop Problem.tex`)

- **I1, Code availability, l. 310–313:** dice que todo sale del notebook `_en`, pero «32 of 35»
  sale de `results/v1_una_construccion/`, y el chequeo de β de 0.3 a 0.9 solo está en
  `results/chequeo_beta_alto.csv`, que **no genera ningún notebook** del repo. → citar las
  versiones anteriores.
- **I2, §5 l. 290–291:** «changing the set of 10 seeds moved tai117 from 0.71 to 0.63». El 0.63
  no está en ningún CSV, y el notebook v2 lo atribuye a dos ejecuciones con las mismas semillas
  (ruido del presupuesto en tiempo real). **Decisión de los autores:** quitar la frase o
  confirmar la causa.
- **I3, §5 l. 237:** «its cost is that of Geo(0.1)–Geo(0.2)». En 4 de 5 instancias Geo-adapt
  cuesta más que ambas; se parece a Geo(0.05). → «close to Geo(0.05)–Geo(0.1)».
- **I4, §3 l. 145–146:** el *enhanced swap* elige la posición con el makespan de la secuencia
  **parcial** `perm[0..k]` (`insert_best_position`), no con el completo. → describirlo así.
- **I5, §3 l. 150–151:** «a worse solution is accepted only right after an improvement». En el
  código la bandera sigue activa durante las iteraciones rechazadas hasta que se acepta un
  empeoramiento. → «only if the last accepted change was an improvement».
- **I6, §5 l. 233–234:** «differences of 0.1–0.2 points are within the noise». No hay test en
  el experimento A: sin corregir, 20 de esos 41 pares dan p < 0.05; con Bonferroni sobrevive
  uno. → «Experiment A has no significance test…» o añadir el test al notebook.
- M: l. 286–287, 0.36 debería ser 0.35 (0.3549). En tai117 los bloques son de unos 80 ms, y en
  3 de 300 ejecuciones el resultado mejora después del presupuesto, entre ellas el 1.42 de Tri
  en tai101 (1.43 al agotarse). «Shared machine» no tiene fuente (pregunta 4 de
  `preguntas_abiertas.md`). En tai117 la opción más barata es Geo(0.2). Las comprobaciones de
  validez se hicieron sobre una muestra. Las salidas del `_en` son la ejecución española.
- Comprobado: Tablas 1 y 2, abstract y conclusiones; los p-valores de Mann–Whitney salen
  idénticos al recalcularlos.

#### VRP (entregado; solo para saberlo)

- **B1, §4 l. 161:** `Appendix~\ref{app:code}` sin apéndice: el PDF dice «Appendix ??».
- **I:** l. 285, «dominated by sorting» es probablemente falso: el tiempo por arista en Python es
  casi constante (1.8–3.3 µs), lo que apunta al bucle interpretado. l. 192–193, «costs match
  bit for bit» no lo prueba ninguna salida, y las sumas de NumPy (por pares) y de C++ (en orden)
  pueden diferir en los últimos bits. l. 296, «about 10 % worse» choca con el 11.6 % del
  abstract (el 9.97 % mide otra cosa).
- M: «exactly 27 of 33» usa una tolerancia de 0.015 (con igualdad estricta, 26). 4.91 % queda
  justo por debajo de 5–10 %. F-n45-k4 y F-n135-k7 no tienen coordenadas enteras. El
  secuencial también borra las aristas rechazadas al buscar la semilla.
- Fuera del artículo: **`VRP/README.md` dice que C++ es 15–65 veces más rápido; los CSV dan
  11.5–58** (lo que pone el artículo).

#### POP (entregado; solo para saberlo)

- **B1, ec. (1) l. 85:** pone `= R_min`, pero el código impone `≥` (`u = [inf, 1, inf]`), y el
  propio artículo dice «at least».
- **I:** el notebook guarda salidas de **dos ejecuciones** (celda 11: port5 274.96 s; celda 9:
  183.9 s). `results/figures/frontiers.png` **no existe** y nunca se subió, así que el `.tex` no
  compila desde el repo; la figura del PDF es idéntica a la salida de la celda 14. «Coinciden
  hasta el redondeo (~1e-5 %)» contradice los picos de 1e-2 a 1.1e-1 % de la Tabla 2. Sin
  fuente en el código: la violación de ~1e-8 (que además no explica los picos), el setup de
  5–12 ms, los errores de 3.1/11.2/6.7 % con los ajustes por defecto, el perfilado (96 % / 73 %)
  y los puntos perdidos con `max_iter = 4000`; el estado real es *primal infeasible
  inaccurate*.
- **Los CSV de `POP/results/` no corresponden al notebook actual:** los escribió el script
  archivado `ignore/markowitz_osqp.py`, con otra formulación.

## 4. Pendiente

1. **Instancias de Taillard de PFSP** (`tai*_inputs.txt`, para `PFSP/data/`): nunca se han
   subido y no están en el equipo. Sin ellas el notebook no arranca, ni siquiera en modo lectura
   (`PFSP_RERUN=0`), porque la celda 3 lee las instancias. Preguntar a los autores de dónde
   salieron.
2. **Corregir los artículos por entregar** (GRASP, TOP, PFSP): `/paper-loop <P>` pasando como
   notas la lista de su sección 3. Antes, los autores deciden TOP I3 e I4 y PFSP I2 y M3.
3. **POP:** sacar `results/figures/frontiers.png` de la salida guardada de la celda 14 (sin
   ejecutar nada) y decidir qué hacer con los CSV antiguos de `results/`.
4. **VRP:** corregir en su README la velocidad de C++ (15–65 → 11.5–58).
5. Las carpetas `PFSP/results/v1_una_construccion/` y `v2_antes_geoadapt/` llevan notebooks
   dentro de `results/`. Se dejan ahí porque son la única fuente de cifras que cita el
   artículo.
6. TOP: lo que queda de [`TOP/docs/scratchpad.md`](TOP/docs/scratchpad.md) §pendiente
   (tamaño de `results/`, unos 130 MB).
