# Esquema del artículo tutorial (PDF, 3 páginas)

> Esquema para redactar el PDF. El enunciado está en inglés, así que los títulos propuestos van en inglés.
> Las cifras salen del notebook `markowitz_osqp.ipynb`: la tabla de la sección 4 y la figura `figures/frontiers.pdf`.
> Los tiempos son de un portátil (AMD Ryzen 5 5500U, Windows 11, un hilo) y varían entre ejecuciones: en port5 he visto entre 183 y 275 s.

**Título propuesto:** *Tracing the Markowitz Efficient Frontier with OSQP: a Short Tutorial on the OR-Library Portfolio Benchmarks*

**Formato sugerido:** LaTeX `article`, 10–11 pt, márgenes de 2 cm, una columna, figura principal a ancho completo.
Presupuesto aproximado: 1 página para las secciones 1–3, 1 página para las secciones 4–5 (texto y tabla) y 1 página para la figura, las conclusiones y las referencias.

---

## Correspondencia con el enunciado

| Requisito del enunciado | Dónde se cubre |
|---|---|
| Artículo tutorial de 3 páginas en PDF | Todo el documento (presupuesto por sección abajo) |
| Presentar la versión básica del problema | §1 y §2 |
| Explicar el modelo de programación cuadrática | §2 (modelo, convexidad, frontera) |
| Resolver port1–port5 con un solver (explicar la elección) | §3 (elección de OSQP) y §4 (implementación) |
| Tiempos de cómputo | §5, Tabla 1 |
| Gráfico de la frontera propia frente a portef1–portef5 | §5, Figura 1 (`figures/frontiers.pdf`) |
| Código Python en un fichero aparte | `markowitz_osqp.ipynb` (se cita en *Code availability*) |

---

## Abstract (≈ 80 palabras)

- Qué es: la cartera de Markowitz como programa cuadrático convexo.
- Qué hacemos: formularla y resolverla con OSQP (ADMM, código abierto) en los 2000 niveles de retorno de cada una de las 5 instancias de OR-Library.
- Resultado: las fronteras coinciden con las de referencia hasta el redondeo de los ficheros (~10⁻⁵ % en varianza).
- Limitación: cerca del retorno máximo, ADMM converge muy despacio y 63 de los 10 000 puntos quedan sin resolver con precisión.

## 1. Introduction (≈ 0.3 pág.)

- Problema: repartir un presupuesto entre *N* activos equilibrando riesgo y rentabilidad. Markowitz (1952) mide el riesgo con la varianza del retorno de la cartera [4].
- El tutorial de Beasley (2013) [1] presenta el modelo básico y sus extensiones (cardinalidad, *index tracking*...). Aquí solo tratamos el modelo básico, sin ventas en corto.
- Instancias de prueba de OR-Library [2], usadas por Chang et al. (2000) [3]: precios semanales de marzo de 1992 a septiembre de 1997.
  - port1 = Hang Seng (31), port2 = DAX 100 (85), port3 = FTSE 100 (89), port4 = S&P 100 (98) y port5 = Nikkei 225 (225).
  - Los ficheros portef1–portef5 contienen 2000 puntos (retorno medio, varianza) de la frontera eficiente sin restricciones (UEF).
- Objetivo del artículo: (i) formular el modelo, (ii) justificar el solver, (iii) reproducir las cinco fronteras y medir los tiempos.

## 2. The quadratic programming model (≈ 0.7 pág.)

**Notación** (tabla corta o lista): *N*; μᵢ = retorno medio; sᵢ = desviación típica; ρᵢⱼ = correlación; σᵢⱼ = ρᵢⱼ sᵢ sⱼ = covarianza; xᵢ = proporción invertida en el activo *i*; R_min = retorno mínimo exigido.

**Modelo** (el de la imagen del enunciado; es la ec. (1)–(5) de Beasley con “≥” en lugar de “=”):

$$
\min_{x}\; \sum_{i=1}^{N}\sum_{j=1}^{N}\sigma_{ij}x_i x_j \quad
\text{s.t.}\quad \sum_i \mu_i x_i \ge R_{\min},\quad \sum_i x_i = 1,\quad x_i \ge 0 .
$$

Puntos a explicar:
1. Significado de cada restricción: rentabilidad mínima, presupuesto (todo invertido) y ausencia de ventas en corto. La cota xᵢ ≤ 1 queda implícita.
2. **Convexidad.** Σ es una matriz de covarianzas y por tanto semidefinida positiva; en las 5 instancias es definida positiva (λ_min entre 6·10⁻⁶ y 2·10⁻⁴, primera tabla del notebook).
   - Consecuencias: QP convexa con óptimo global único, y las condiciones KKT son necesarias y suficientes.
   - Opcional, una línea con las KKT: 2Σx − λμ − ν𝟏 − s = 0, λ, s ≥ 0, λ(μᵀx − R) = 0, sᵢxᵢ = 0.
3. **La frontera eficiente.**
   - Extremo inferior: la cartera de mínima varianza global (GMV), que se obtiene quitando la restricción de retorno; su retorno es R_GMV.
   - Extremo superior: R_max = maxᵢ μᵢ, con toda la inversión en el activo de mayor retorno.
   - Para R_min ∈ [R_GMV, R_max] la restricción de retorno es activa, así que “≥” y “=” (Beasley) dan la misma solución.
   - V*(R) es convexa, no decreciente y cuadrática a trozos.
4. Carteras eficientes frente a ineficientes, como en la figura del enunciado: los activos individuales quedan por debajo y a la derecha de la frontera (puntos rojos de la Figura 1).
5. (Una frase) Alternativa de suma ponderada, min xᵀΣx − t·μᵀx, usada en Chang et al. [3]. Aquí usamos la versión con R_min del enunciado. Si se añade cardinalidad, el problema pasa a ser una MIQP (Beasley §3), que queda fuera del alcance.

## 3. Choice of solver (≈ 0.4 pág.)

Tabla comparativa breve:

| Solver | Algoritmo para QP convexa | Licencia | Comentario para este problema |
|---|---|---|---|
| Gurobi / CPLEX | Barrera (punto interior) y simplex | Comercial (licencia académica gratuita) | Muy robustos y precisos; también MIQP. Requieren licencia |
| HiGHS | Active set | MIT | Preciso, ligero; sin MIQP |
| SCIP | Branch-and-bound para MINLP | Apache 2.0 | Sobredimensionado para una QP convexa; útil para la versión con cardinalidad |
| **OSQP** | **ADMM / operator splitting** [5, 6] | **Apache 2.0** | **Elegido** |

Argumentos para OSQP:
- La frontera exige **2000 QPs casi idénticas**: solo cambia el lado derecho R_min.
  - OSQP factoriza la matriz KKT una vez (`setup`).
  - `update(l=…)` cambia la cota sin refactorizar.
  - El *warm start* arranca desde la cartera del punto anterior.
- Iteraciones baratas, instalación trivial (`pip install osqp`) y sin licencia.
- *Solution polishing*: una vez identificado el conjunto activo, devuelve una solución de alta precisión.

Inconvenientes (se retoman en §5):
- Es un método de primer orden. Con la tolerancia por defecto (10⁻³) el error en varianza llega a varios % (medido: hasta ~11 % en port2), así que hay que usar 10⁻⁶ con *polishing*.
- Converge muy despacio en problemas degenerados, como los puntos junto a R_max.

## 4. Implementation (≈ 0.4 pág.)

- Python 3.14, NumPy/SciPy, pandas, matplotlib y OSQP 1.1.3, en el notebook `markowitz_osqp.ipynb`, entregado aparte.
- Lectura de portK: Σ = D·ρ·D.
- **Forma estándar de OSQP**: min ½ xᵀPx + qᵀx s.a. l ≤ Ax ≤ u, con P = 2Σ, q = 0, A = [μᵀ; 𝟏ᵀ; I], l = [R_min; 1; 0], u = [∞; 1; ∞].
- **Resolución paramétrica**: `setup` una sola vez por instancia; después, para cada R_k, `l[0] = R_k`, `update(l=l)` y `solve()` (*warm start* automático).
- **Ajustes**: `eps_abs = eps_rel = 1e-6`, `polishing = True`, `max_iter = 20000` (el valor por defecto, 4000, no basta cerca de R_max).
- **Rejilla**: los 2000 niveles de retorno de portefK → comparación punto a punto con la varianza de referencia.
- Métricas:
  - Tiempo de pared por instancia (setup + 2000 QPs, `time.perf_counter`), ms por QP e iteraciones medias.
  - Puntos cuyo estado no es `solved`.
  - Desviación relativa 100·(V_OSQP − V_portef)/V_portef sobre los puntos resueltos.
- Equipo: AMD Ryzen 5 5500U, Windows 11, un solo hilo.

## 5. Computational results (≈ 0.8 pág. incluida la figura)

**Tabla 1** (tabla del notebook, ejecución del 18-09-2026):

| Instance | Index | N | Time (s) | ms/QP | Mean iter. | Not solved | Max \|dev\| (%) | Mean \|dev\| (%) |
|---|---|---|---|---|---|---|---|---|
| port1 | Hang Seng | 31 | 0.87 | 0.44 | 76 | 0 | 7.8e-06 | 2.9e-06 |
| port2 | DAX 100 | 85 | 25.93 | 12.96 | 797 | 0 | 3.6e-05 | 1.0e-05 |
| port3 | FTSE 100 | 89 | 10.08 | 5.04 | 255 | 0 | 3.8e-02 | 3.6e-05 |
| port4 | S&P 100 | 98 | 33.90 | 16.95 | 803 | 3 | 1.0e-02 | 1.7e-05 |
| port5 | Nikkei 225 | 225 | 274.96 | 137.48 | 974 | 60 | 1.1e-01 | 1.2e-04 |

**Figura 1** (`figures/frontiers.pdf`, ancho completo):
- Paneles 1–5: la UEF de OSQP (línea), los puntos de portef (círculos) y los activos individuales (puntos rojos, carteras ineficientes).
- Panel 6: desviación relativa en varianza a lo largo de la frontera.

Mensajes a destacar:
- **Precisión.** En la mayoría de puntos, la coincidencia con portef es de ~10⁻⁵ %. Es el redondeo de los ficheros (10 decimales): los errores no son del modelo.
- **Picos de hasta 0.1 %.** Son unos pocos puntos `solved` en los que falló el *polishing*. La solución se queda en la precisión de ADMM: viola la restricción de retorno en ~10⁻⁸ y por eso sale una varianza ligeramente menor.
- **Tiempos.**
  - De menos de 1 s (port1) a ~4.5 min (port5).
  - El tiempo no depende solo de *N*: port3 (N = 89) necesita 255 iteraciones de media y port2 (N = 85), 797.
  - Casi todo el tiempo se va en el tramo final de la frontera.
- **Puntos no resueltos** (port4: 3; port5: 60, todos junto a R_max). Explicación para el artículo:
  - Cerca de R_max solo se cumple la restricción de retorno invirtiendo casi todo en el activo de mayor μ.
  - El conjunto factible es una esquina muy estrecha del símplex, y en R_max un único punto sin punto estrictamente factible.
  - Ahí ADMM converge muy despacio.
  - Con `max_iter = 4000` (el valor por defecto) fallan 121, 82 y 125 puntos en port2, port4 y port5, algunos incluso marcados como `primal infeasible inaccurate`.
  - Ajustar `rho`, desactivar el escalado o usar la igualdad no lo arregla.
- (Opcional, 2–3 líneas) Remedios medidos, con valores orientativos para port5:
  - Reformulación de Cholesky + re-solve con 10⁻⁹ (`markowitz_osqp.py`): todos los puntos resueltos, ~500 s.
  - Forma ponderada min xᵀΣx − t·μᵀx: todos resueltos, ~36 s, pero es otro modelo y la comparación exige interpolar.
  - Un solver de conjunto activo o de punto interior (HiGHS, Gurobi).

## 6. Conclusions (≈ 0.2 pág.)

- El modelo básico de Markowitz es una QP convexa, como señala Beasley [1]. Un solver de código abierto de primer orden lo resuelve con precisión en casi toda la frontera si se ajustan la tolerancia y el *polishing*.
- La estructura paramétrica (una factorización + *warm start*) hace que cada QP cueste milisegundos.
- La frontera con OSQP tiene un punto débil: el extremo de máximo retorno, que es degenerado. Ahí un método de primer orden no es la mejor herramienta.
- Extensión natural: la restricción de cardinalidad convierte el problema en una MIQP difícil. Motiva las heurísticas de Chang et al. [3] (algoritmos genéticos, búsqueda tabú, recocido simulado), el tema de la asignatura.

## Code availability (1–2 líneas)

`markowitz_osqp.ipynb` (fichero aparte): carga de datos, resolución con OSQP y gráficas. Ejecución: abrir con el kernel del proyecto y *Run All* (~6 min).

---

## Bibliografía

[1] Beasley, J. E. (2013). Portfolio optimisation: Models and solution approaches. En H. Topaloglu (Ed.), *Theory Driven by Influential Applications* (pp. 201–221). INFORMS TutORials in Operations Research. https://doi.org/10.1287/educ.2013.0114

[2] Beasley, J. E. (1990). OR-Library: Distributing test problems by electronic mail. *Journal of the Operational Research Society*, 41(11), 1069–1072. https://doi.org/10.1057/jors.1990.166 — Datos: https://people.brunel.ac.uk/~mastjjb/jeb/orlib/portinfo.html

[3] Chang, T.-J., Meade, N., Beasley, J. E., & Sharaiha, Y. M. (2000). Heuristics for cardinality constrained portfolio optimisation. *Computers & Operations Research*, 27(13), 1271–1302. https://doi.org/10.1016/S0305-0548(99)00074-X

[4] Markowitz, H. (1952). Portfolio selection. *The Journal of Finance*, 7(1), 77–91. https://doi.org/10.1111/j.1540-6261.1952.tb01525.x

[5] Stellato, B., Banjac, G., Goulart, P., Bemporad, A., & Boyd, S. (2020). OSQP: An operator splitting solver for quadratic programs. *Mathematical Programming Computation*, 12(4), 637–672. https://doi.org/10.1007/s12532-020-00179-2

[6] Boyd, S., Parikh, N., Chu, E., Peleato, B., & Eckstein, J. (2011). Distributed optimization and statistical learning via the alternating direction method of multipliers. *Foundations and Trends in Machine Learning*, 3(1), 1–122. https://doi.org/10.1561/2200000016

[7] Boyd, S., & Vandenberghe, L. (2004). *Convex Optimization*. Cambridge University Press. (QP y optimización de carteras, §4.4)

[8] Cornuéjols, G., Peña, J., & Tütüncü, R. (2018). *Optimization Methods in Finance* (2.ª ed.). Cambridge University Press. https://doi.org/10.1017/9781107297340

[9] Huangfu, Q., & Hall, J. A. J. (2018). Parallelizing the dual revised simplex method. *Mathematical Programming Computation*, 10(1), 119–142. (Referencia de HiGHS; su solver de QP es de tipo *active set*: https://highs.dev)

[10] Bolusani, S., et al. (2024). *The SCIP Optimization Suite 9.0*. arXiv:2402.17702. https://arxiv.org/abs/2402.17702

[11] Gurobi Optimization, LLC. *Gurobi Optimizer Reference Manual*. https://www.gurobi.com — IBM. *ILOG CPLEX Optimization Studio*. https://www.ibm.com/products/ilog-cplex-optimization-studio

[12] OSQP documentation (settings, warm start, polishing). https://osqp.org/docs/
