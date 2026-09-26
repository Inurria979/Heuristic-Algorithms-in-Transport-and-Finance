# CLAUDE.md

Práctica de máster (Heurísticas): modelo básico de Markowitz resuelto con OSQP sobre las
instancias OR-Library port1–port5, comparando la frontera eficiente con portef1–portef5.
El entregable es un artículo tutorial de 3 páginas en PDF (en inglés) más el código Python.
Enunciado en `docs/image.png`; guion del artículo en `docs/esquema_articulo.md`.

## Entorno

- Python 3.14 gestionado con `uv` (`uv sync`). Windows + OneDrive: usar `UV_LINK_MODE=copy`
  si `uv` falla al enlazar ficheros.
- Ejecutar el notebook sin interfaz:
  `UV_LINK_MODE=copy uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/markowitz_osqp.ipynb`
  Tarda ~5–6 min (casi todo port5); no relanzarlo sin necesidad.

## Estructura

- `src/markowitz_osqp.ipynb`: entregable principal. Se ejecuta con `src/` como directorio de
  trabajo: lee `../data/`, resuelve 2000 QP por instancia (los niveles de retorno de
  `portefK`) y guarda `../results/figures/frontiers.{pdf,png}`.
- `data/`: `portK.txt` (N; μ_i, σ_i por activo; `i j ρ_ij` para i ≤ j, índices desde 1) y
  `portefK.txt` (2000 pares retorno, varianza).
- `docs/`: artículo (`Portfolio Optimization Problem.tex` + PDF; el `.tex` busca la figura en
  `../results/figures/`), guion y contexto del artículo, papers de referencia, transparencias.
- `results/`: CSV y figuras del script extendido antiguo, más la figura del notebook.
- `ignore/` (en `.gitignore`): descarga original de OR-Library, zips, script extendido
  `markowitz_osqp.py` archivado y `main.py` (importaba ese script; no funciona).
  `pyproject.toml` apunta pytest a `tests/`, que no existe.

## Convenciones

- Notebook, código y README en inglés; el guion del artículo y la conversación en español.
- Estilo del código: funciones cortas con docstring de una línea, numpy/scipy.sparse, sin
  clases. Mantener la configuración de OSQP (`eps_abs = eps_rel = 1e-6`, polishing,
  `max_iter = 20000`) salvo que se pida cambiarla: las cifras del artículo dependen de ella.
- Si cambian tiempos o resultados, actualizar a la vez el README, la tabla del notebook y
  `docs/esquema_articulo.md`.
