# CLAUDE.md

Prácticas del máster *Heuristic Algorithms in Transport and Finance*. Una carpeta por
problema, cada una es un proyecto uv independiente. El `README.md` de la raíz es la
referencia general; cada subproyecto tiene su propio `README.md` con modelo, método y
resultados, y puede tener su propio `CLAUDE.md` (hoy solo `POP/`), que manda sobre este.

| Carpeta | Problema | Estado |
|---|---|---|
| `POP/` | Portfolio Optimization (Markowitz + OSQP, OR-Library port1–5) | Terminado |
| `TSP/` | Travelling Salesman (vecino más próximo y greedy edge, TSPLIB) | Código terminado |
| `VRP/` | Capacitated Vehicle Routing, heurística de ahorros de Clarke & Wright (33 instancias A/B/E/F/M/P) | En curso: CWS secuencial hecho, falta paralelo y artículo |
| `DTOP/` | Dynamic Team Orienteering (Transformers + RL) | Terminado, sin código (presentación en clase) |

## Estructura obligatoria de cada subproyecto

Todo subproyecto con código (`POP/`, `TSP/`, `VRP/` y los que se añadan) tiene estas carpetas:

- `src/` — código: notebooks y módulos `.py`. Nada más.
- `data/` — instancias del problema (benchmarks, soluciones de referencia), sin modificar.
- `docs/` — PDFs del profesor (transparencias, enunciados, papers que da) y documentos
  propios (artículos `.tex` + PDF, guiones, notas). **Se sube a GitHub, PDFs incluidos.**
- `results/` — salidas generadas: CSV y figuras. Se sube.
- `ignore/` — todo lo que no se quiere subir (descargas originales, zips, scripts
  archivados, pruebas). Está en `.gitignore`.

En la raíz de cada subproyecto solo quedan `README.md`, `CLAUDE.md` (opcional),
`pyproject.toml`, `uv.lock` y `.python-version`. No dejes ficheros sueltos: colócalos en
una de las cinco carpetas y, si no está claro cuál, pregunta.

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

- Python 3.14 y **uv** en todos los proyectos. No hay `pyproject.toml` en la raíz ni
  workspace de uv: cada carpeta resuelve y bloquea sus dependencias por separado.
- Trabaja siempre desde dentro del subproyecto: `cd TSP && uv sync`.
- Dependencias con `uv add` / `uv add --dev`, nunca con `pip`. Los `uv.lock` se suben.
- Ejecutar un notebook sin interfaz (desde la carpeta del subproyecto):
  `uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/<nb>.ipynb`
- Windows: si uv falla al enlazar ficheros, usa `UV_LINK_MODE=copy`.
- Algunos notebooks tardan (POP ~6 min, TSP ~75 s): no los relances sin necesidad.

## Git

- Se sube: `src/`, `data/`, `docs/` (con PDFs), `results/`, `pyproject.toml`, `uv.lock`,
  `.python-version`, README y CLAUDE.md.
- No se sube: `ignore/`, `.venv/`, cachés de Python y herramientas, auxiliares de LaTeX.
- No añadas reglas al `.gitignore` que excluyan PDFs de `docs/`.
- Git está en Windows con `core.ignorecase=true`: los nombres de carpeta van en mayúsculas
  (`POP`, `TSP`, `VRP`, `DTOP`); para cambiar solo mayúsculas/minúsculas hay que pasar por
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
