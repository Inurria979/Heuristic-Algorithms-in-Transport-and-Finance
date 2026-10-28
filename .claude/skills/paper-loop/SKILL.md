---
name: paper-loop
description: Escribe y revisa en bucle el artículo de 3 páginas (LaTeX, en inglés) de un subproyecto del máster (TOP, PFSP, TSP…) con los agentes paper-writer y paper-reviewer, hasta que el revisor lo apruebe o se agoten las rondas. Úsalo cuando se pida escribir, redactar, terminar o revisar el paper o artículo de una práctica.
argument-hint: <CARPETA> [rondas máx., por defecto 3] [notas]
---

# Bucle escritor–revisor del artículo de 3 páginas

Argumentos: `$ARGUMENTS`. El primero es la carpeta del subproyecto `<P>` (`TOP`, `PFSP`…); el segundo, si es un número, el máximo de rondas `N` (por defecto 3); el resto son notas del usuario que se pasan tal cual a los dos agentes. Si falta la carpeta, pregunta cuál.

Las reglas de estilo están en `guia_estilo.md` y el esqueleto LaTeX en `plantilla.tex`, ambos en esta carpeta. Los agentes los leen por su cuenta: tú no los cargues, salvo para resolver una duda concreta.

## Preparación (tú)

1. Comprueba que `<P>/` existe y tiene resultados (`results/` o equivalente). Sin resultados no hay artículo: avisa al usuario y para.
2. Motor de LaTeX: `command -v tectonic latexmk pdflatex`. Apunta el primero que exista o «ninguno». **No instales nada.**
3. Crea `<P>/ignore/paper_loop/` (está en `.gitignore`). Si ya contiene revisiones de un bucle anterior, sigue la numeración: la ronda inicial es la siguiente a la última `revision_r*.md`.
4. Di al usuario en una línea qué vas a hacer: carpeta, rondas, motor.

## Ronda k (k = 1 … N)

1. **Escritor.**
   - k = 1: lanza `paper-writer` con un prompt que diga: carpeta `<P>`, ronda 1, motor, notas del usuario, y «primera ronda: escribe el guion y el artículo».
   - k ≥ 2: continúa al **mismo** escritor con `SendMessage` (conserva el contexto y es más barato): «ronda k: corrige según `<P>/ignore/paper_loop/revision_r{k-1}.md` y escribe `cambios_r{k}.md`». Si no se puede continuar, lanza uno nuevo con la misma información.
2. **Revisor.** Cuando el escritor termine, lanza un `paper-reviewer` **nuevo** en cada ronda (ojos frescos) con: carpeta `<P>`, ronda k, motor y notas del usuario.
3. Espera a cada agente (te llega una notificación al terminar; no consultes su salida mientras tanto). No leas los informes completos salvo que haga falta para decidir: los mensajes finales de los agentes bastan.
4. **Decide:**
   - Veredicto **APROBADO** → fin del bucle.
   - Hay puntos en **`DESACUERDO`**, o un B o un I que exige datos o experimentos que no existen, o un requisito ambiguo del enunciado → para y pregunta al usuario. No sigas iterando sobre un desacuerdo.
   - k = N sin aprobar → fin, con lo que queda pendiente.
   - En otro caso → ronda k + 1.

Durante el bucle **no edites tú el artículo**: los papeles deben quedar separados para que la revisión sea independiente.

Ni tú ni los agentes **ejecutáis nada del proyecto** (notebooks, código de `src/`, experimentos, compilaciones de Rust/C++, `uv sync`/`uv run`): solo se miran los notebooks con sus salidas guardadas, `results/` y los documentos. Lo único que se lanza es el motor de LaTeX y comandos o scripts de solo lectura. Si faltan resultados, se pregunta al usuario; no se generan.

Las dudas que solo pueden resolver los autores (requisitos ambiguos, datos o referencias que faltan, discrepancias entre fuentes) se apuntan en `<P>/docs/preguntas_abiertas.md`, una sección por pregunta con un hueco **Respuesta:** para que la rellenen. Los agentes leen ese fichero al empezar cada ronda y aplican las respuestas que haya.

## Cierre (tú, en español)

- Veredicto final y rondas usadas.
- Páginas del PDF (o «sin compilar: no hay motor de LaTeX; compílalo en Overleaf y comprueba que da 3 páginas»).
- Ficheros: `<P>/docs/<Problem Name>.tex`, el PDF, `<P>/docs/esquema_articulo.md` y los informes de `<P>/ignore/paper_loop/`.
- Menores que quedan abiertos, datos que faltan y preguntas para el usuario.
- No hagas commit salvo que se pida.
