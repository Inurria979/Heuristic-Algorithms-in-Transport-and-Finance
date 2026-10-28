# Guía de estilo de los artículos de 3 páginas (Javier Salvatierra y Jesús Inurria)

Reglas extraídas de los dos artículos ya entregados y de su guion previo:

- `VRP/docs/Vehicle Routing Problem.tex` → `VRP/docs/Javier S Jesus I Vehicle Routing Problem.pdf` (el más reciente: **ante la duda, imitar este**).
- `POP/docs/Portfolio Optimization Problem.tex` → `POP/docs/Javier S, Jesus I, Portfolio Optimization Problem.pdf`.
- `POP/docs/esquema_articulo.md`: guion previo (requisitos, presupuesto por sección, mensajes clave).

La lee entera tanto el escritor (`paper-writer`) como el revisor (`paper-reviewer`) antes de cada ronda.

---

## 1. El entregable

- Artículo (*tutorial article* / *paper*) en inglés, **exactamente 3 páginas A4**, una columna, LaTeX `article` 10 pt. Las 3 páginas incluyen título, abstract y referencias. Los dos artículos entregados llenan las 3 páginas justas.
- Lo pide un enunciado del profesor (A. A. Juan) en las transparencias de clase (diapositiva *Homework / Lab Activities*, en un PDF de `docs/`; se extrae con `pdftotext`) o en una imagen de `docs/` (`POP/docs/image.png`, `TOP/docs/homework.png`). Ejemplos literales:
  - POP: *"prepare a 3-page tutorial article in PDF introducing the basic version of the portfolio optimization problem, explaining its quadratic programming model, and solving the benchmark instances port1 to port5 with a solver like ... (explain your choice). Provide computational times and a graphical representation ... Include Python / Julia code in a separate file."*
  - VRP: *"Write a 3-page paper in which you describe the savings heuristics and implement it for solving the VRP, testing it in different instances and comparing your results with the ones provided in the JORS table. Include your code as appendix."*
- **Cada frase del enunciado es un requisito comprobable.** El guion `docs/esquema_articulo.md` empieza por la tabla *Requisito del enunciado → Dónde se cubre*.
- Código:
  - *"in a separate file"* → párrafo final `\paragraph{Code availability.}` que nombra el fichero y cómo ejecutarlo (POP).
  - *"as appendix"* → `\appendix` con `\lstinputlisting` **después** de la bibliografía. El límite de 3 páginas se aplica al cuerpo, hasta las referencias. En VRP el apéndice se quedó sin poner y el PDF salió con «Appendix ??»: no repetirlo. Si el código es un notebook y no hay un `.py` que listar, preguntar al usuario.

## 2. Formato LaTeX

- Partir de `.claude/skills/paper-loop/plantilla.tex`, cuyo preámbulo es idéntico al del VRP. **No tocar** paquetes, tamaño de letra, `\linespread`, `\titlespacing` ni márgenes laterales para ganar sitio: la longitud se ajusta recortando texto. Única holgura: márgenes superior e inferior entre 1.6 y 1.8 cm (los dos valores usados).
- Título: `\vspace{-1.2cm}\bfseries`, dos líneas separadas por `\\`. Nombra el problema, el método y el alcance o los datos:
  - *Tracing the Markowitz Efficient Frontier with OSQP:\\ a Short Tutorial on the OR-Library Portfolio Benchmarks*
  - *The Clarke \& Wright Savings Heuristic for the\\ Capacitated Vehicle Routing Problem: Sequential and Parallel Versions*
- Autores, literal: `Heuristic Algorithms in Transport and Finance --- Master's course assignment \\ Javier Salvatierra \\ Jesús Inurria`. Fecha: mes y año en inglés (`October 2026`).
- `\maketitle` seguido de `\vspace{-0.8cm}`; abstract con `\noindent\small`.
- Ficheros: el `.tex` en `docs/` con el nombre del problema en inglés (`Vehicle Routing Problem.tex`); el PDF en `docs/Javier S Jesus I <Problem Name>.pdf`. Las figuras se leen de `../results/figures/` con `\graphicspath{{../results/figures/}}` dentro del entorno `figure`.
- El `.tex` final no lleva comentarios ni texto en español.

## 3. Estructura y presupuesto

| § | Sección (título en inglés) | Espacio | Contenido |
|---|---|---|---|
| — | Abstract | 120–135 palabras | Problema en una frase · qué hacemos (método, implementación, instancias) · cifras principales (precisión o gap, tiempos) · una limitación o hallazgo |
| 1 | Introduction | ~0.35 pág. | Ver 3.1 |
| 2 | Modelo: *The quadratic programming model*, *Problem formulation and savings*… | ~0.5 pág. | Ver 3.2 |
| 3 | Método: *Choice of solver*, *The Clarke and Wright savings heuristic*… | ~0.5 pág. | Ver 3.3 |
| 4 | Implementation | ~0.4 pág. | Ver 3.4 |
| 5 | Computational results | ~1 pág. con tabla y figura | Ver 3.5 |
| 6 | Conclusions | ~0.15 pág. | Ver 3.6 |
| — | Code availability / References | resto | 3–5 referencias |

- Prosa: **1 700–1 950 palabras** medidas sobre el `.tex` sin tablas, figuras ni bibliografía (POP 1 689, VRP 1 957; comando en §11). Una tabla principal y una figura; como mucho, una tabla pequeña más (la de solvers de POP).
- Subsecciones solo si ayudan a leer: POP tiene cuatro en §2; VRP tiene dos en §3 y tres en §4.
- Etiquetas: `sec:model`, `sec:cws`, `sec:impl`, `sec:res`, `eq:model`, `tab:results`, `fig:...`.

### 3.1 Introduction (2–3 párrafos)

1. El problema en palabras llanas, con un actor concreto y lo que se decide (*"An investor has a fixed budget and $N$ assets to spread it over."*), o su relación con un problema conocido (*"The Vehicle Routing Problem (VRP) generalises the Travelling Salesman Problem (TSP)."*).
2. La variante tratada y por qué es difícil (NP-hard, de ahí las heurísticas) o fácil (QP convexo). Cita del trabajo fundacional y de la referencia del curso. **Frase de alcance** explícita: *"we deal only with the basic version, without short selling"*.
3. Las instancias de prueba: origen, familias, la convención de nombres explicada con un ejemplo (*"An instance named `A-n32-k5` has $n=32$ nodes (the depot and 31 customers) and a best-known solution with $k=5$ vehicles"*), rango de tamaños, qué contiene cada fichero y de dónde salen los valores de referencia (*"taken from Table~1 of~\cite{juan2011}"*).
4. Hoja de ruta en una frase: *"The rest of the paper (i)~states the model, (ii)~explains why we chose OSQP, and (iii)~reports times and accuracy on the five instances."* o *"Below we state the model, describe the CWS and our implementation, and compare the results with that table."*

### 3.2 Modelo

- Notación en prosa, sin tabla, con unidades y un ejemplo numérico (*"All statistics are weekly: $\mu_i=0.0109$ means $1.09\,\%$ per week."*).
- **Una** ecuación en display con el modelo completo: `\min ... \qquad\text{s.t.}\qquad ...`, numerada y con `\label`.
- Lectura en lenguaje llano del objetivo y de cada restricción (*"Read literally: the objective is ..."*), y las restricciones implícitas que no hace falta escribir (*"The upper bound $w_i\le1$ need not be written, since ..."*).
- La propiedad estructural que hace funcionar el método, argumentada en dos líneas y **cuantificada en las instancias**: autovalor mínimo entre $6.1\cdot10^{-6}$ y $2.3\cdot10^{-4}$ (convexidad estricta); desigualdad triangular, luego $s(i,j)\ge0$ (*"Savings are largest for customers that are close to each other and far from the depot."*).

### 3.3 Método

- Punto de partida, pasos y condiciones en `enumerate` con cabeza en negrita y el porqué en una frase: *"\textbf{Different routes.} If $i$ and $j$ were already on the same route, the new edge would close a cycle that does not visit the depot."*
- Variantes en `itemize` con cabeza en negrita (*Parallel CWS* / *Sequential CWS*).
- Justificación de la elección por la **estructura de la tarea**: *"The decisive argument is the structure of the task: a frontier is not one QP but 2000 almost identical QPs ..."*. Si el enunciado pide explicar una elección, tabla breve de alternativas (tabla de solvers de POP).
- Invariantes que justifican el procedimiento: *"an edge that fails the conditions can be discarded for good: routes only grow, so ..."*.
- **El precio**, cuantificado: *"with the default tolerance $10^{-3}$ and no polishing we measured variance errors of up to $3.1\,\%$, $11.2\,\%$ and $6.7\,\%$"*.

### 3.4 Implementation

- Ficheros en `\texttt` (`vrp.py`, `cws.cpp`, `markowitz_osqp.ipynb`), lenguaje y versión (Python~3.14, C++ con Clang a través de Zig, Rust…) y bibliotecas con versión cuando importa (OSQP 1.1.3).
- Estructuras de datos clave y su coste (*"`route_of`, the route index of every node, so that the route of a customer is found in $O(1)$"*); si ayuda, el bucle central en una línea de `\texttt`.
- Parámetros exactos y el motivo de cada valor no por defecto (*"OSQP's default of 4000 is not enough near $R_{\max}$"*). Deben coincidir con el código de `src/`.
- Relación explícita con el paper de referencia: qué se implementa y qué no, y qué tabla o columna es la referencia (*"The reference is thus the column \emph{CWS Sol.} of their Table~1, not \emph{Our Best Sol.}"*).
- Protocolo: rejilla, ejecuciones, semillas, qué se cronometra y cómo (una ejecución o la media de varias), equipo e hilos (*"All runs are single-threaded on an AMD Ryzen 5 5500U laptop under Windows 11."*; usar el equipo y los hilos reales del proyecto).
- Métricas con fórmula: `$\text{gap}=100\,(\text{cost}-\text{ref})/\text{ref}$`.
- Validación con cifras concretas (POP: `port1` con $R_{\min}=0.008$, 125 iteraciones, 1.0 ms, 4 de 31 activos; VRP: el port a C++ coincide bit a bit con NumPy).

### 3.5 Computational results

- Primera frase: qué compara la tabla y la definición de la métrica (*"Table~\ref{tab:results} compares both versions with Table~1 of~\cite{juan2011}, with gap = ..."*) y qué cubren los tiempos.
- Tabla principal: una fila por instancia (o por familia o conjunto si son demasiadas, diciéndolo), las columnas de referencia del paper junto a las nuestras y una fila **Mean** en negrita.
- Una figura que muestre lo que la tabla no puede (fronteras, rutas de una instancia representativa, distribución de gaps…).
- 3–4 párrafos con cabeza en negrita (`\textbf{Accuracy.}`, `\textbf{Quality.}`, `\textbf{Time.}`, `\textbf{Sequential vs.\ parallel.}`, `\textbf{Parallel CWS vs.\ JORS.}`, `\textbf{The failures at the top end.}`). Cada uno: afirmación → cifras (rango, media, recuento: *"27 of the 33 instances"*) → **mecanismo** (por qué pasa) → salvedad.
- Comparación explícita con el paper de referencia: coincidencias, diferencias y su causa (el ejemplo de los empates de ahorros: *"if tied edges are taken in reverse generation order, the same code reproduces the JORS cost in all 33 instances"*). Las discrepancias no se esconden.

### 3.6 Conclusions

- Un solo párrafo de 4–5 frases: qué es el método en una frase, el resultado principal cuantificado, por qué funciona, el punto débil y el siguiente paso enlazado con la asignatura (aleatorización, metaheurísticas). Puede cerrar con una máxima breve: *"solver choice depends on problem geometry, not only on size."*
- Sin resultados nuevos ni viñetas.

## 4. Voz y redacción

- Primera persona del plural (*we formulate*, *we measured*). Presente para hechos y para lo que hace el artículo; pasado para las mediciones.
- Frases declarativas y cortas. Dos puntos para explicar (*"The problem is degenerate: at $R_{\max}$ only one portfolio is feasible"*), punto y coma para encadenar.
- Toda afirmación lleva una cifra o una razón. Se explica el **mecanismo**: *"the reason is geometric rather than numerical"*, *"which is frequent here because the coordinates are integers"*.
- Salvedades honestas y concretas dentro del texto: *"(one Python run against a mean of many C++ runs, so only indicative)"*, *"timings fluctuate between runs (183--275\,s)"*.
- Tono de tutorial: un compañero del máster debe poder reimplementarlo leyendo el artículo. Cada término se define en su primera aparición con `\emph{}` y cada sigla se expande una vez: *Capacitated Vehicle Routing Problem (CVRP)*.
- Terminología fija: elegido un nombre (*parallel CWS*, *savings list*, *dummy solution*), no se cambia; se usan los términos de las transparencias del curso.
- Ortografía británica: *minimise, optimisation, factorisation, randomisation, generalise, travelled, labelled, modelling, behaviour, licence* (sustantivo). Los nombres propios mantienen la suya (*Portfolio Optimization Problem*).
- Prohibido: bombo (*novel, powerful, state-of-the-art, robust* sin medir, *significantly* sin test); relleno (*It is worth noting that*, *In this paper we will*, *In conclusion,*); preguntas retóricas; viñetas donde basta la prosa; contracciones; calcos del español (*realize* por *carry out*, *actual* por *current*, *in base to*).

## 5. Cifras

- **Toda cifra sale de un fichero del proyecto** (`results/*.csv`, figuras, notas de `docs/`, README) o del paper de referencia, y queda anotada con su fuente en la tabla *Cifras y fuentes* de `docs/esquema_articulo.md`. Nada de memoria ni estimaciones.
- Precisión como en los artículos: costes con 2 decimales, gaps con 2 decimales ($4.91\,\%$), tiempos con 2–3 cifras significativas (0.87\,s, 137\,ms), cantidades diminutas en notación científica ($7.8\cdot10^{-6}$).
- Rangos, recuentos y medias en lugar de adjetivos: *"0.4--137\,ms per QP"*, *"27 of the 33 instances"*, *"12 to 58 times less (median 24)"*.
- La misma cifra es idéntica en abstract, texto, tabla y conclusiones; el abstract y las conclusiones solo pueden redondear de forma evidente (*"about $5\,\%$"*).
- Los tiempos siempre dicen qué se mide, en qué equipo, con cuántos hilos y con qué protocolo.

## 6. Tipografía LaTeX

- Espacio fino entre número y unidad: `0.87\,s`, `137\,ms`, `13\,\textmu s`, `$4.91\,\%$`. Miles con espacio fino a partir de 5 cifras (`10\,000`); en tablas, coherente dentro de cada columna (`1\,147.28`).
- Rangos con `--` (`0.4--137\,ms`, `\texttt{port1}--\texttt{port5}`); incisos con ` --- ` (raya con espacios).
- `\texttt{}` para instancias, ficheros, funciones y parámetros; `\emph{}` para términos definidos y nombres de columnas del paper; `\textbf{}` solo para cabezas de párrafo o de ítem y para la fila Mean o la opción elegida.
- Abreviaturas: `e.g.\`, `i.e.\`, `vs.\`, `et al.\` (con `\ ` cuando sigue texto).
- Referencias cruzadas con espacio irrompible: `Table~\ref{}`, `Figure~\ref{}`, `Section~\ref{}`, `\eqref{}`, `~\cite{}`. Nunca un `~` sin nada detrás.
- Solo se numeran las ecuaciones a las que se hace referencia; el resto, `\[ \]` o en línea.

## 7. Tablas y figuras

- `booktabs` (`\toprule`, `\midrule`, `\bottomrule`), sin reglas verticales, `@{}` en ambos extremos, números alineados a la derecha, `\footnotesize` o `\scriptsize` con `\setlength{\tabcolsep}{4pt}` si es ancha.
- Leyenda **encima** de la tabla y **debajo** de la figura, autocontenida: qué se compara, de dónde salen las columnas de referencia, qué significa la negrita. Ejemplo: *"Sequential and parallel CWS. \emph{BKS} (real distances) and \emph{JORS CWS} from Table~1 of~\cite{juan2011}; routes in bold exceed $k$; times of the parallel CWS."*
- Figuras: PNG ya existentes en `results/figures/`, ancho entre `0.45` y `0.85\textwidth`; la leyenda explica colores, marcadores, paneles y ejes.
- Todo flotante se cita en el texto. Posición `[H]`, `[!t]` o `[!htb]`.

## 8. Referencias

- `\begin{thebibliography}{9}` + `\footnotesize\setlength{\itemsep}{0pt}\setlength{\parskip}{0pt}`; de 3 a 5 entradas.
- Formato:
  - Artículo: `J.~E. Beasley. Portfolio optimisation: models and solution approaches. \emph{INFORMS TutORials in OR}, 201--221, 2013.`; con volumen: `\emph{J.\ Oper.\ Res.\ Soc.}, 62(6):1085--1097, 2011.`
  - Transparencias: `A.~A. Juan. \emph{The VRP and the CWS heuristic}. Course slides, Heuristic Algorithms in Transport and Finance, 2026.`
  - Software: `OSQP Developer Team. \emph{OSQP documentation: Python interface}, 2025. \url{...}`
- Siempre se cita: el trabajo fundacional del método, el paper o benchmark del enunciado (de donde salen los valores de referencia) y las transparencias si una cifra o afirmación viene de ellas.
- Cada `\bibitem` se cita y cada `\cite` se resuelve. Los datos bibliográficos se comprueban contra el PDF de `docs/` (primera página) o la diapositiva del enunciado, no de memoria. Las afirmaciones teóricas citan papers, no documentación de software.

## 9. Fuentes y límites

- **Los resultados son fijos.** Ni el escritor ni el revisor lanzan experimentos, tocan `src/` o `data/`, modifican `results/` ni cambian parámetros (CLAUDE.md). Única excepción: el escritor puede extraer a `results/` una figura o tabla que ya existe en el proyecto (salidas guardadas de un notebook, presentación de `docs/`), sin sobrescribir nada (ver `paper-writer`). Si falta una cifra o una figura, se informa al usuario; no se inventa ni se genera.
- Se pueden calcular estadísticos derivados (medias, recuentos, cocientes) a partir de los CSV con un script de un solo uso en `<proyecto>/ignore/paper_loop/tmp/`. En TOP la tubería es solo Rust: un cálculo de comprobación suelto vale, pero no se añade nada al proyecto.
- Orden de autoridad si dos fuentes discrepan: código de `src/` (parámetros) y CSV de `results/` (cifras) > notas de `docs/` > README > presentación. La discrepancia se anota en *Pendiente* del esquema.
- Coherencia con la presentación de clase (`docs/*.pptx`/`.pdf`, `docs/guion_*.md`) y con el README: mismas cifras y mismos mensajes.

## 10. Fallos reales que se colaron en los artículos entregados

Son el tipo de error que el revisor tiene que cazar (casi todos, retoques de última hora sin releer):

- **POP l. 85 frente a l. 93 y l. 141:** la ecuación (1) escribe la restricción de retorno como igualdad (`= R_{\min}`), pero el texto la lee como *"at least"* y el mapeo a OSQP usa $u_0=+\infty$, es decir, $\ge$. Ecuación, lectura y código deben decir lo mismo.
- **POP l. 56:** `Markowitz~` sin `\cite` (faltaba Markowitz 1952 en la bibliografía).
- **POP l. 63:** *"It is built from weekly prices"*: el antecedente de *It* es OR-Library, pero lo que se construye con precios son las instancias.
- **POP l. 94:** *"which is caused by investing negative quantity any asset"*: agramatical (*"short selling, i.e.\ holding a negative amount of an asset"*).
- **POP l. 131:** *"applies ADMM~\cite{osqpdocs}"*: afirmación teórica citada a la documentación del software (lo correcto: Stellato et al., 2020).
- **POP l. 163:** *"Futhermore, it is used as a python libreary which makes the implementation easier."*: dos erratas, *python* en minúscula, vago e informal, y rompe el hilo antes de la lista.
- **POP l. 238:** *": It reproduces"*: mayúscula tras dos puntos y *It* ambiguo.
- **VRP l. 161:** `Appendix~\ref{app:code}` sin apéndice: el PDF dice «Appendix ??», y el enunciado pedía el código como apéndice.

## 11. Comprobaciones mecánicas (Git Bash, desde la carpeta del proyecto)

Motor de LaTeX: el primero disponible de `tectonic`, `latexmk`, `pdflatex` (`command -v`). **No instalar nada**: si no hay ninguno, la longitud se estima con el recuento de palabras y se dice.

```bash
cd "<P>/docs"
# compilar fuera de docs/ (los auxiliares no ensucian) y copiar el PDF
tectonic --keep-logs -o ../ignore/paper_loop/build "<Problem Name>.tex"
#   o: latexmk -pdf -interaction=nonstopmode -outdir=../ignore/paper_loop/build "<Problem Name>.tex"
cp "../ignore/paper_loop/build/<Problem Name>.pdf" "Javier S Jesus I <Problem Name>.pdf"

PDF="Javier S Jesus I <Problem Name>.pdf"
pdftotext "$PDF" - | tr -cd '\f' | wc -c               # páginas (debe dar 3)
pdftotext -f 3 -l 3 "$PDF" - | wc -w                   # palabras de la pág. 3 (comparar con la 1 para ver lo llena que está)
pdftotext "$PDF" - | grep -n '??'                      # referencias rotas
grep -nE 'undefined|Overfull|multiply defined' ../ignore/paper_loop/build/*.log
stat -c '%Y %n' "<Problem Name>.tex" "$PDF"            # el PDF no puede ser más antiguo que el .tex

# prosa (comparable con POP 1 689 y VRP 1 957)
sed -n '/begin{document}/,/end{document}/p' "<Problem Name>.tex" \
  | sed '/begin{table}/,/end{table}/d; /begin{figure}/,/end{figure}/d; /begin{thebibliography}/,/end{thebibliography}/d' | wc -w
sed -n '/begin{abstract}/,/end{abstract}/p' "<Problem Name>.tex" | wc -w
```
