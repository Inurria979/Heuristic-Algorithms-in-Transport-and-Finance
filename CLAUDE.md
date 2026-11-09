# CLAUDE.md

Prácticas del máster *Heuristic Algorithms in Transport and Finance*. Una carpeta por
problema, cada una es un proyecto uv independiente (salvo `TOP/`, que es Rust, y `DTOP/`, sin
código). El `README.md` de la raíz es la
referencia general; cada subproyecto tiene su propio `README.md` con modelo, método y
resultados, y puede tener su propio `CLAUDE.md` (hoy solo `POP/`), que manda sobre este.

**Al empezar una sesión, lee `scratchpad.md` (raíz):** notas de traspaso con el estado del
trabajo, las revisiones hechas y lo pendiente. Actualízalo al terminar algo que afecte a varios
proyectos; lo de un solo proyecto va en su `docs/` (p. ej. `TOP/docs/scratchpad.md`).

| Carpeta | Problema | Estado |
|---|---|---|
| `POP/` | Portfolio Optimization (Markowitz + OSQP, OR-Library port1–5) | Terminado |
| `TSP/` | Travelling Salesman (vecino más próximo y greedy edge, TSPLIB) | Código terminado |
| `VRP/` | Capacitated Vehicle Routing, heurística de ahorros de Clarke & Wright (33 instancias A/B/E/F/M/P) | Terminado (código, artículo y presentación) |
| `DTOP/` | Dynamic Team Orienteering (Transformers + RL) | Terminado, sin código (presentación en clase) |
| `TOP/` | Team Orienteering, heurística PJS + GRASP + búsqueda local, en Rust (387 instancias de Chao) | Código, artículo y presentación terminados |
| `GRASP/` | GRASP con ahorros y aleatorización sesgada para el TSP (10 instancias TSPLIB) | Código, artículo y presentación terminados |
| `PFSP/` | Permutation Flow-Shop, NEH / RandNEH / ILS-ESP con β adaptativo (5 instancias de Taillard) | Código, artículo y presentación terminados; faltan las instancias en `data/` |
| `simTOP/` | TOP con tiempos de viaje estocásticos: simheurística (PJS + BR + búsqueda local + simulación Monte Carlo) del paper de Juan et al. (2019), 141 instancias de p1–p3 (Numba) | Código, artículo y presentación terminados |

## Estructura obligatoria de cada subproyecto

Todo subproyecto con código (`POP/`, `TSP/`, `VRP/`, `TOP/`, `GRASP/`, `PFSP/`, `simTOP/` y los que se
añadan) tiene estas carpetas:

- `src/` — código: notebooks y módulos `.py`. Nada más, salvo dos excepciones:
  - `VRP/src/cpp/`: fuentes C++; se compilan con Zig instalado vía uv y el ejecutable va a
    `ignore/bin/`.
  - `TOP/src/rust/`: proyecto Cargo con todo el flujo de TOP. `cargo` se lanza siempre desde
    `TOP/src/rust`, para que `.cargo/config.toml` mande los compilados a `TOP/ignore/target`.
- `data/` — instancias del problema (benchmarks, soluciones de referencia), sin modificar.
- `docs/` — PDFs del profesor (transparencias, enunciados, papers que da) y documentos
  propios (artículos `.tex` + PDF, guiones, notas). **Se sube a GitHub, PDFs incluidos.**
- `results/` — salidas generadas: CSV y figuras. Se sube.
- `ignore/` — todo lo que no se quiere subir (descargas originales, zips, scripts
  archivados, pruebas). Está en `.gitignore`.

En la raíz de cada subproyecto solo quedan `README.md`, `CLAUDE.md` (opcional),
`pyproject.toml`, `uv.lock` y `.python-version` (en `TOP/`, solo `README.md`: no tiene entorno
de Python). No dejes ficheros sueltos: colócalos en una de las cinco carpetas y, si no está
claro cuál, pregunta.

`DTOP/` es la excepción: **no contiene código y está terminado**. La actividad consistía
solo en hacer una presentación en clase, así que la carpeta guarda únicamente los papers y
las transparencias en su raíz, sin subcarpetas. No crees código ni la estructura anterior
en `DTOP/` salvo que se pida expresamente.

## Rutas en el código

- Los notebooks se ejecutan con `src/` como directorio de trabajo: leen de `../data/` y
  escriben en `../results/` (figuras en `../results/figures/`).
- Los módulos `.py` construyen las rutas a partir de su propia ubicación, p. ej.
  `Path(__file__).resolve().parent.parent / "data"`.
- Si mueves ficheros, actualiza las rutas del código, del `.tex` y de los README.

## Entorno

- Python 3.14 y **uv** en todos los proyectos, salvo `GRASP/` y `PFSP/`, que fijan 3.13 porque
  sus resultados se generaron con esa versión (y usan Numba). No hay `pyproject.toml` en la
  raíz ni workspace de uv: cada carpeta resuelve y bloquea sus dependencias por separado.
- `TOP/` es solo Rust (rayon, perfil release agresivo único): experimentos, figuras e informes
  van en el crate; no se añade Python a su flujo.
- Trabaja siempre desde dentro del subproyecto: `cd TSP && uv sync`.
- Dependencias con `uv add` / `uv add --dev`, nunca con `pip`. Los `uv.lock` se suben.
- Ejecutar un notebook sin interfaz (desde la carpeta del subproyecto):
  `uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/<nb>.ipynb`
- Windows: si uv falla al enlazar ficheros, usa `UV_LINK_MODE=copy`.
- Algunos experimentos tardan (POP ~6 min, TSP ~75 s, GRASP ~22 min, simTOP ~25 s, PFSP con
  `PFSP_RERUN=1` mucho más, rejilla de TOP ~2 min): no los relances sin necesidad.

## Git

- Se sube: `src/`, `data/`, `docs/` (con PDFs), `results/`, `pyproject.toml`, `uv.lock`,
  `.python-version`, README y CLAUDE.md.
- No se sube: `ignore/`, `.venv/`, cachés de Python y herramientas, auxiliares de LaTeX.
- No añadas reglas al `.gitignore` que excluyan PDFs de `docs/`.
- Git está en Windows con `core.ignorecase=true`: los nombres de carpeta van en mayúsculas
  (`POP`, `TSP`, `VRP`, `DTOP`, `TOP`, `GRASP`, `PFSP`), salvo las prácticas de simulación, que empiezan por `sim` en minúsculas
  seguido del problema en mayúsculas (`simTOP`); para cambiar solo mayúsculas/minúsculas hay que pasar por
  un nombre intermedio o por `git rm --cached`.
- Solo haz commit o push cuando se pida.

## Convenciones

- Código, notebooks y README en inglés; conversación, guiones y notas internas en español.
- Estilo: funciones cortas con docstring de una línea, numpy/scipy, sin clases salvo que
  hagan falta.
- Los resultados publicados (tablas de README, artículos) dependen de la configuración de
  cada método: no cambies parámetros de los solvers o heurísticas sin que se pida, y si
  cambian cifras, actualiza a la vez notebook, README y documentos de `docs/`.
- Nuevo proyecto: crea las cinco carpetas, sigue la receta de "Adding a new project" del
  README raíz y añade una fila a la tabla de proyectos del README y de este fichero.

## Artículos de 3 páginas

- Se escriben y revisan con `/paper-loop <CARPETA> [rondas]`, que alterna los agentes
  `paper-writer` y `paper-reviewer` de `.claude/agents/`. Las reglas de estilo, sacadas de los
  artículos de POP y VRP, están en `.claude/skills/paper-loop/guia_estilo.md`.
- Los informes de revisión quedan en `<CARPETA>/ignore/paper_loop/` y no se suben.

## Orquestador y agentes de código

- `lector-papers` (Opus, esfuerzo high) lee los PDFs de `<CARPETA>/docs/`, conversa con el usuario y
  escribe `<CARPETA>/docs/explicacion_paper.md`: explicación sencilla del paper y especificación para
  implementarlo. Como necesita hablar con el usuario, se abre en sesión propia:
  `claude --agent lector-papers`, o con `/leer-paper <CARPETA>` en la sesión actual. La skill
  (`.claude/skills/leer-paper/SKILL.md`) añade el proceso iterativo y las herramientas (PDFs, internet,
  código del repositorio) que se usaron con `simTOP/`. No escribe código.
- `/orquestar <CARPETA> <objetivo>` (o `claude --agent orquestador`) planifica el trabajo y lo
  reparte entre `code-writer`/`code-reviewer` (bucle de código), `paper-writer`/`paper-reviewer`
  (bucle de `/paper-loop`) y `Explore`. El orquestador no escribe código ni artículos.
- `code-writer` corre en Sonnet y, si existe `explicacion_paper.md`, lo sigue como especificación
  (`code-reviewer` lo usa como referencia de fidelidad).
- `revisor-final` (Opus, esfuerzo xhigh) revisa al cerrar una práctica todo el conjunto: paper
  original, especificación, código, resultados, README, artículo y presentación. No edita nada.
  Informe en `<CARPETA>/ignore/revision_final/informe.md`.
- Orden habitual: (`explicacion_paper.md` →) código → `results/` → README → artículo → presentación
  → `revisor-final`.
- Notebooks: la skill `cuaderno-explicado` (`.claude/skills/cuaderno-explicado/SKILL.md`) dice cómo debe quedar el
  notebook principal (introducción, recorrido paso a paso, experimento, conclusiones) sin cambiar `results/`.
  `code-writer` la sigue y `code-reviewer` la usa como lista de comprobación.
- Plan en `<CARPETA>/ignore/orquestador/plan.md`; informes de código en
  `<CARPETA>/ignore/code_loop/<tarea>/`. No se suben.

## Segunda opinión (MCP con modelos externos)

- Servidor MCP `segunda-opinion` en `.claude/mcp/segunda-opinion/` (proyecto uv propio, Python
  3.14), registrado en `.mcp.json` de la raíz. Herramientas `list_models` y `second_opinion`:
  manda los ficheros de una práctica a modelos de otras familias (Gemini por su API; DeepSeek y
  GLM por NVIDIA NIM), **solo planes gratuitos**, y devuelve sus hallazgos. Catálogo de modelos en
  `providers.toml`; para añadir uno basta un bloque nuevo.
- Claves en variables de entorno de usuario (`GEMINI_API_KEY`, `NVIDIA_API_KEY`), nunca en el repo.
  Comprobar con `uv run server.py --check` desde esa carpeta.
- `paper-reviewer`, `code-reviewer` y `revisor-final` la llaman **siempre al final de cada
  revisión**, después de su propia revisión. Verifican cada hallazgo; solo los confirmados cuentan.
  Si falla o no hay claves, se anota y se sigue. Respuestas en `<CARPETA>/ignore/segunda_opinion/`.
