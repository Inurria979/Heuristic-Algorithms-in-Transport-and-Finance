# Contexto para redactar el paper: Markowitz con OSQP sobre OR-Library port1–port5

> Documento de contexto que recoge todas las explicaciones, decisiones, experimentos y cifras de la
> sesión de trabajo (18–19 de septiembre de 2026). Está pensado para usarse como contexto al
> escribir el artículo tutorial de 3 páginas (en inglés). El guion por secciones está en
> `docs/esquema_articulo.md`; el código entregable es `markowitz_osqp.ipynb`.
>
> Todas las cifras de este documento se han medido en esta sesión. Cuando una cifra viene de un
> experimento distinto del notebook final, se indica.

---

## 0. El enunciado (`image.png`)

Actividad 1 (Lab / Homework):

- Preparar un **artículo tutorial de 3 páginas en PDF** que presente la **versión básica del problema de optimización de carteras**.
- Explicar su **modelo de programación cuadrática**.
- Resolver las instancias de prueba **port1 a port5** con un solver como **Gurobi, CPLEX, OSQP, HiGHS o SCIP**, **explicando la elección**.
- Aportar **tiempos de cómputo** y una **representación gráfica** de la frontera eficiente sin restricciones obtenida y de la generada con las soluciones de **portf1 a portf5** (los ficheros `portef1`–`portef5`).
- Incluir el **código Python/Julia en un fichero aparte**.

Referencias del enunciado:
- Beasley, J. E. (2013). *Portfolio optimisation: models and solution approaches*. INFORMS TutORials, pp. 201–221.
- OR-Library, página de carteras: https://people.brunel.ac.uk/~mastjjb/jeb/orlib/portinfo.html

El modelo que muestra la imagen del enunciado:

$$
\min_{w_1,\dots,w_N}\ \sum_{i=1}^{N}\sum_{j=1}^{N}\sigma_{ij}\,w_i w_j
\quad\text{s.a.}\quad \sum_{i=1}^{N}\mu_i w_i \ge R_{\min},\qquad \sum_{i=1}^{N} w_i = 1,\qquad w_i \ge 0,\ i=1,\dots,N.
$$

La imagen escribe las variables como $x_i$; en el notebook y en este documento se usa **$w_i$ (pesos)**, que es también la notación de Beasley.

La imagen incluye además un esquema riesgo-retorno con la frontera eficiente (puntos azules, «efficient portfolio») y carteras ineficientes (puntos rojos, «inefficient portfolio»). Marca un punto A en la frontera y otros B y C que ilustran la dominancia: para el mismo riesgo, A da más retorno; para el mismo retorno, B tiene menos riesgo que C.

---

## 1. Qué es un QP

Un **QP** (*quadratic program*, problema de programación cuadrática) es un problema de optimización con **función objetivo cuadrática** y **restricciones lineales**:

$$
\min_w\ \tfrac12\, w^\top P w + q^\top w \qquad \text{s.a.}\qquad l \le A w \le u .
$$

- **Objetivo cuadrático:** contiene productos entre variables ($w_i w_j$), no solo términos lineales como en programación lineal (LP).
- **Restricciones lineales:** cada una es una combinación lineal de las variables, acotada por arriba y/o por abajo.
- **QP convexo:** si $P$ es semidefinida positiva. Entonces todo mínimo local es global y existen algoritmos eficientes. Si $P$ es definida positiva, el óptimo además es **único**.

En Markowitz el objetivo es la varianza de la cartera (cuadrático) y las restricciones son lineales, así que cada problema es un QP convexo. **«Un QP por cada $R_{\min}$»**: al cambiar $R_{\min}$ cambia el problema y hay que resolverlo de nuevo. La frontera eficiente es el conjunto de soluciones para todos los $R_{\min}$.

---

## 2. El modelo de Markowitz (versión básica)

### 2.1 Notación

| Símbolo | Significado |
|---|---|
| $N$ | número de activos (acciones) |
| $\mu_i$ | retorno medio (esperado) del activo $i$, por periodo (aquí **semanal**) |
| $s_i$ | desviación típica del retorno del activo $i$ |
| $\rho_{ij}$ | correlación entre los retornos de $i$ y $j$ ($-1 \le \rho_{ij} \le 1$) |
| $\sigma_{ij} = \rho_{ij}\,s_i\,s_j$ | covarianza entre $i$ y $j$; $\sigma_{ii} = s_i^2$ |
| $\Sigma = [\sigma_{ij}]$ | matriz de covarianzas ($N\times N$, simétrica) |
| $w_i$ | proporción del presupuesto invertida en el activo $i$ (peso), variable de decisión |
| $R_{\min}$ | retorno mínimo exigido a la cartera |

Nota de notación: se usa $\Sigma$ para la matriz de covarianzas y $\sum$ para los sumatorios.

### 2.2 Formulación

$$
\min_{w}\ \sum_{i=1}^{N}\sum_{j=1}^{N}\sigma_{ij}\,w_i w_j \;=\; w^\top\Sigma w
$$
sujeto a
1. **Retorno mínimo:** $\sum_i \mu_i w_i \ge R_{\min}$. La cartera debe rendir al menos $R_{\min}$.
2. **Presupuesto:** $\sum_i w_i = 1$. Todo el dinero se invierte, sin guardar nada.
3. **Sin ventas en corto:** $w_i \ge 0$. No se permiten posiciones negativas (Beasley las excluye porque las carteras se mantienen demasiado tiempo para que el *shorting* sea práctico).

La cota $w_i \le 1$ no hace falta escribirla: se deduce de $w_i \ge 0$ y $\sum_i w_i = 1$.

En el marco de Markowitz, **riesgo = varianza del retorno de la cartera**. El objetivo minimiza el riesgo para un retorno dado.

### 2.3 Relación con Beasley (2013)

Beasley escribe el modelo como sus ecuaciones (1)–(4):
- (1) minimizar $\sum_i\sum_j w_i w_j \rho_{ij} s_i s_j$;
- (2) $\sum_i w_i \mu_i = R$, **con igualdad**;
- (3) $\sum_i w_i = 1$;
- (4) $0 \le w_i \le 1$.

La ecuación (5) es la versión con covarianzas, $\sum_i\sum_j w_i w_j \sigma_{ij}$. Beasley subraya que, aunque los problemas no lineales suelen ser difíciles, aquí el objetivo es cuadrático y $\Sigma$ es semidefinida positiva, por lo que existen algoritmos eficientes y «en la práctica hay poca dificultad para calcular el óptimo».

**$\ge$ frente a $=$:** para $R_{\min} \in [R_{GMV}, R_{\max}]$ la restricción de retorno está activa en el óptimo, así que ambas versiones dan la misma cartera. Por debajo de $R_{GMV}$, la versión con $\ge$ devuelve la cartera GMV, mientras que la versión con $=$ fuerza un retorno menor con más varianza (una cartera ineficiente). El código usa $\ge$ ($l_0 = R_{\min}$, $u_0 = +\infty$).

> **Coherencia pendiente:** la celda de introducción del notebook escribe la restricción con «$=$» (cambio hecho a mano por el usuario), mientras que el código implementa «$\ge$». Son equivalentes en los niveles de retorno usados, pero conviene que el artículo y el notebook usen la misma forma o expliquen la equivalencia.

### 2.4 Convexidad y optimalidad

- $\Sigma$ es una matriz de covarianzas, luego es semidefinida positiva. En las 5 instancias es incluso **definida positiva** (menor autovalor $> 0$, tabla de la sección 3.3). Por tanto cada QP es estrictamente convexo, con **óptimo global único**.
- Las **condiciones KKT** son necesarias y suficientes:
  $$
  2\Sigma w - \lambda\mu - \nu\mathbf 1 - s = 0,\quad \lambda \ge 0,\ s \ge 0,\quad \lambda(\mu^\top w - R_{\min}) = 0,\quad s_i w_i = 0 .
  $$
  - $\lambda$ es el multiplicador de la restricción de retorno: el **precio sombra** del retorno, igual a la pendiente $dV/dR$ de la frontera.
  - $\nu$ es el multiplicador del presupuesto.
  - $s_i$ son los multiplicadores de $w_i \ge 0$: valen 0 para los activos en cartera y son positivos para los activos con $w_i = 0$.

### 2.5 La frontera eficiente sin restricciones (UEF)

- **Extremo inferior: cartera de mínima varianza global (GMV).** Se obtiene quitando la restricción de retorno (en OSQP, $R_{\min} = -\infty$). Su retorno es $R_{GMV}$.
- **Extremo superior: $R_{\max} = \max_i \mu_i$.** Es la única cartera con ese retorno: el 100 % en el activo de mayor retorno medio. En el fichero de referencia, la varianza de ese punto coincide con $s_j^2$ de ese activo (comprobado en las 5 instancias).
- La función $V^*(R)$ (varianza mínima para cada retorno) es **convexa, no decreciente en $[R_{GMV}, R_{\max}]$ y cuadrática a trozos**. Cada trozo corresponde a un conjunto fijo de activos en cartera; los cambios de trozo son los puntos donde entra o sale un activo.
- Es el conjunto de **carteras Pareto-óptimas (no dominadas)**: ninguna otra cartera da más retorno con el mismo riesgo, ni menos riesgo con el mismo retorno.
- **Carteras ineficientes:** los activos individuales, y cualquier combinación que no esté en la frontera, quedan por debajo y a la derecha de ella. Es la idea de la figura del enunciado.
- **Diversificación:** en port1, la GMV reparte la inversión entre 10 activos. El número de activos en cartera sube a 12 y va bajando por tramos hasta 1 en $R_{\max}$ (figura de composición del script extendido). Beasley cita un caso del FTSE 100 cuya GMV tiene 30 de 100 activos.

### 2.6 Método de trazado: $\varepsilon$-restricción frente a suma ponderada

- **$\varepsilon$-restricción (el del enunciado, el usado):** se fija $R_{\min}$ y se minimiza la varianza, repitiendo para muchos $R_{\min}$. Controla exactamente qué retornos se obtienen, lo que permite comparar punto a punto con portef.
- **Suma ponderada (alternativa):** $\min\ w^\top\Sigma w - t\,\mu^\top w$ con $\sum w_i = 1,\ w \ge 0$, variando $t \ge 0$ (con $t = 0$ se obtiene la GMV; con $t$ grande, la cartera de máximo retorno). Chang et al. (2000) usan la variante $\min\ \lambda\cdot\text{riesgo} - (1-\lambda)\cdot\text{retorno}$ con $\lambda \in [0,1]$. Da la misma frontera, pero no controla los retornos obtenidos.

### 2.7 Extensión (fuera de alcance, pero enlaza con la asignatura)

Si se añade una **restricción de cardinalidad** (invertir en exactamente $K$ activos, con variables binarias $z_i$ y $w_i \le z_i$), el problema pasa a ser una **MIQP**, difícil de resolver (Beasley §3). La frontera con cardinalidad (CCEF) puede ser discontinua. Esto motiva las heurísticas de Chang et al. (2000): algoritmos genéticos, búsqueda tabú y recocido simulado, que son el tema de la asignatura de Heurísticas.

---

## 3. Las instancias (OR-Library port1–port5)

### 3.1 Origen

- Proceden de Chang, Meade, Beasley y Sharaiha (2000), *Heuristics for cardinality constrained portfolio optimisation*, y están publicadas en **OR-Library** (Beasley, 1990).
- Son acciones reales de **5 índices bursátiles**, con **precios semanales de marzo de 1992 a septiembre de 1997**.
- Con esos precios, los autores calcularon para cada acción su retorno medio semanal $\mu_i$ y su desviación típica $s_i$, y para cada par su correlación $\rho_{ij}$. Los ficheros no contienen las series de precios, solo estos estadísticos.
- Una **instancia** es un conjunto de datos concreto para el mismo modelo: cambian los activos (y por tanto $\mu$ y $\Sigma$), pero el problema de optimización es idéntico.

### 3.2 Tabla de instancias

| Instancia | Índice | Mercado | $N$ | $\mu_i$ (mín … máx) | $s_i$ (mín … máx) | Activo de mayor $\mu$ | Líneas de correlación |
|---|---|---|---|---|---|---|---|
| port1 | Hang Seng | Hong Kong | 31 | 0.000141 … 0.010865 | 0.0358 … 0.0691 | 5 | 496 |
| port2 | DAX 100 | Alemania | 85 | −0.004002 … 0.009794 | 0.0212 … 0.0680 | 38 | 3 655 |
| port3 | FTSE 100 | Reino Unido | 89 | −0.001126 … 0.008209 | 0.0236 … 0.0531 | 18 | 4 005 |
| port4 | S&P 100 | EE. UU. | 98 | −0.001980 … 0.009195 | 0.0210 … 0.0638 | 82 | 4 851 |
| port5 | Nikkei 225 | Japón | 225 | −0.008489 … 0.003971 | 0.0258 … 0.0745 | 214 | 25 425 |

Los valores son semanales: $\mu = 0.0109$ significa un 1.09 % de retorno medio por semana. Algunas acciones tienen $\mu$ negativo, es decir, perdieron valor en el periodo.

### 3.3 Propiedades numéricas de $\Sigma$

| Instancia | $\lambda_{\min}(\Sigma)$ | $\lambda_{\max}(\Sigma)$ | Condicionamiento $\lambda_{\max}/\lambda_{\min}$ |
|---|---|---|---|
| port1 | 2.265e-4 | 3.678e-2 | 1.6e2 |
| port2 | 8.183e-5 | 2.433e-2 | 3.0e2 |
| port3 | 5.907e-5 | 2.823e-2 | 4.8e2 |
| port4 | 8.086e-5 | 2.231e-2 | 2.8e2 |
| port5 | 6.054e-6 | 2.263e-1 | 3.7e4 |

- Las 5 matrices son **definidas positivas**, lo que garantiza un óptimo único.
- port5 es con diferencia la peor condicionada (unas 100 veces más que las demás), además de la más grande: es numéricamente la más difícil.
- Las correlaciones vienen redondeadas a 6 decimales, pero aun así ninguna matriz tiene autovalores negativos.

### 3.4 Formato de `portK.txt`

```
31                    ← N
.001309 .043208       ← activo 1: μ₁, s₁
.004177 .040258       ← activo 2
...                   ← N líneas en total
 1 1 1.000000         ← i j ρ_ij (índices desde 1, i ≤ j, incluye la diagonal)
 1 2 .562289
...                   ← N(N+1)/2 líneas
```

`read_port` lee todos los números, separa $\mu$ y $s$, rellena la matriz de correlaciones de forma simétrica y construye $\Sigma = \rho \odot (s\,s^\top)$, es decir, $\sigma_{ij} = \rho_{ij} s_i s_j$.

### 3.5 La frontera de referencia: `portefK.txt` (la «solución oficial»)

- Está en `data/portef1.txt` … `data/portef5.txt`, y venía en la misma descarga de OR-Library. El enunciado se refiere a ella como «portf1 to portf5».
- La calcularon Chang et al. (2000). El README de OR-Library la describe como *the unconstrained efficient frontiers for each of these data sets*.
- **Formato:** 2000 líneas con dos números cada una, `retorno_medio varianza`, ordenadas **de mayor a menor retorno**. La primera línea es $R_{\max}$ y la última, la GMV. `read_portef` las reordena de menor a mayor retorno.

  ```
  .0108650000  .0047755010     ← port1, R_max: todo en el activo 5 (varianza = s_5²)
  .0108609579  .0047677406
  ...
  .0027843363  .0006422572     ← GMV
  ```
- **Los niveles de retorno están casi equiespaciados** entre $R_{GMV}$ y $R_{\max}$: el paso es ~4.04e-6 en port1, 3.85e-6 en port2, 2.92e-6 en port3, 3.63e-6 en port4 y 1.95e-6 en port5. No es un *linspace* exacto: se desvía hasta 1.1e-7 (port1) y 4.8e-7 (port5), hasta un 25 % de un paso. Por eso **resolvemos exactamente en los retornos del fichero** en lugar de construir nuestra propia rejilla e interpolar; la interpolación introducía un error comparable a lo que se quiere medir.
- **Precisión:** 10 decimales. Como $V \sim 10^{-4}$–$10^{-3}$, eso supone un error relativo de redondeo de ~$10^{-5}$ %.
- **No contiene los pesos $w$**, solo $(R, V)$. La comparación se hace sobre la varianza (el valor óptimo). Como el óptimo es único ($\Sigma$ definida positiva), si la varianza coincide la cartera es la misma.

Rangos de cada frontera de referencia:

| Instancia | $R_{GMV}$ | $R_{\max}$ | $V_{GMV}$ | $V(R_{\max})$ |
|---|---|---|---|---|
| port1 | 0.0027843363 | 0.0108650000 | 0.0006422572 | 0.0047755010 |
| port2 | 0.0021019640 | 0.0097940000 | 0.0001368553 | 0.0028352430 |
| port3 | 0.0023653252 | 0.0082090000 | 0.0001984935 | 0.0015166351 |
| port4 | 0.0019368822 | 0.0091950000 | 0.0001214131 | 0.0029387241 |
| port5 | 0.0000708236 | 0.0039710000 | 0.0003046407 | 0.0016485224 |

**Comprobación de la GMV:** nuestra GMV (resuelta con $R_{\min} = -\infty$) coincide con el primer punto de portef hasta ~1e-11 en varianza, dentro del redondeo del fichero. En retorno difiere entre 1e-8 y 4e-8: cerca de la GMV la frontera es plana, así que pequeñas diferencias de varianza se traducen en diferencias mayores de retorno. Con ambas formulaciones (directa y Cholesky) se obtiene exactamente la misma GMV.

---

## 4. El solver OSQP

### 4.1 Qué es

OSQP (*Operator Splitting Quadratic Program*) es un solver de QP convexos de **código abierto (licencia Apache 2.0)**, desarrollado en Oxford, Stanford y Princeton (Stellato, Banjac, Goulart, Bemporad y Boyd, 2020). Está basado en **ADMM**, un método de primer orden.

- La biblioteca está escrita en **C**; el paquete de Python (`pip install osqp`) es una envoltura sobre ella. La versión usada es **1.1.3**.
- También se usa desde C++ (con la API de C directamente o con OSQP-Eigen), Rust (crate `osqp`, que envuelve la de C), Julia, MATLAB y R.
- Puede **generar código C** autónomo para sistemas embebidos (`solver.codegen`).

### 4.2 Forma estándar que resuelve

$$
\min_w\ \tfrac12\,w^\top P w + q^\top w \qquad\text{s.a.}\qquad l \le A w \le u
$$

- $w \in \mathbb R^n$ son las variables. La documentación de OSQP las llama $x$; en el notebook se usa $w$ (pesos).
- $P \in \mathbb R^{n\times n}$ es simétrica y semidefinida positiva. OSQP solo lee su **triángulo superior**, en formato disperso CSC.
- $q \in \mathbb R^n$; $A \in \mathbb R^{m\times n}$, donde **cada fila es una restricción**; $l, u \in \mathbb R^m$ son las cotas y admiten $\pm\infty$.
- Solo hay un tipo de restricción, el «sándwich» $l_k \le \sum_i a_{ki} w_i \le u_k$:
  - **igualdad:** $l_k = u_k$;
  - **desigualdad de un solo lado:** la otra cota es $\pm\infty$;
  - **cota sobre una variable:** una fila de la matriz identidad.

### 4.3 Traducción de Markowitz a la forma OSQP (detallada)

**Objetivo.** La doble suma es, en forma matricial, $w^\top\Sigma w$. OSQP pone un ½ delante de la parte cuadrática (convención habitual, que simplifica las derivadas), así que:

$$
\sum_i\sum_j \sigma_{ij}w_iw_j = \tfrac12\sum_i\sum_j (2\sigma_{ij})\,w_iw_j \;\Rightarrow\; P_{ij} = 2\sigma_{ij},\ \text{es decir } P = 2\Sigma,\qquad q = 0 .
$$

- El factor 2 compensa el ½. Con $P = \Sigma$ se obtendría **la misma cartera**, porque escalar el objetivo no cambia el mínimo, pero `obj_val` valdría la mitad de la varianza.
- $q = 0$ porque el objetivo no tiene términos lineales.
- Con $P = 2\Sigma$ y $q = 0$, `res.info.obj_val` es exactamente la varianza $w^\top\Sigma w$ (comprobado: 1.545024e-3 en el ejemplo de la sección 4.6).

**Restricciones.** Cada una aporta una fila de $A$ y un par de cotas:

| Markowitz | Forma «sándwich» | Fila de $A$ | Cotas |
|---|---|---|---|
| $\sum_i \mu_i w_i \ge R_{\min}$ | $R_{\min} \le \sum_i \mu_i w_i \le +\infty$ | fila 0: $(\mu_1,\dots,\mu_N)$ | $l_0 = R_{\min}$, $u_0 = +\infty$ |
| $\sum_i w_i = 1$ | $1 \le \sum_i 1\cdot w_i \le 1$ | fila 1: $(1,\dots,1)$ | $l_1 = u_1 = 1$ (igualdad) |
| $w_i \ge 0,\ i = 1..N$ | $0 \le w_i \le +\infty$ | filas 2…N+1: matriz identidad $I$ | $l = 0$, $u = +\infty$ |

Hay $n = N$ variables y $m = N + 2$ restricciones; en port5, 225 variables y 227 restricciones.

**Ejemplo con $N = 3$:**

$$
\underbrace{\begin{bmatrix}R_{\min}\\1\\0\\0\\0\end{bmatrix}}_{l}
\le
\underbrace{\begin{bmatrix}\mu_1&\mu_2&\mu_3\\ 1&1&1\\ 1&0&0\\ 0&1&0\\ 0&0&1\end{bmatrix}}_{A}
\begin{bmatrix}w_1\\w_2\\w_3\end{bmatrix}
\le
\underbrace{\begin{bmatrix}+\infty\\1\\+\infty\\+\infty\\+\infty\end{bmatrix}}_{u}
$$

Si se multiplica fila por fila, se recuperan exactamente las restricciones de Markowitz:
- fila 0: $R_{\min} \le \mu_1w_1 + \mu_2w_2 + \mu_3w_3$;
- fila 1: $1 \le w_1 + w_2 + w_3 \le 1$;
- filas 2–4: $0 \le w_1$, $0 \le w_2$, $0 \le w_3$.

**En código** (función `build_qp` del notebook):

```python
def build_qp(mu, Sigma, R_min):
    """P, q, A, l, u of the Markowitz model for OSQP."""
    n = len(mu)
    P = sp.csc_matrix(np.triu(2 * Sigma))  # OSQP only reads the upper triangle
    q = np.zeros(n)
    A = sp.vstack([mu, np.ones(n), sp.eye(n)], format="csc")
    l = np.r_[R_min, 1.0, np.zeros(n)]
    u = np.r_[np.inf, 1.0, np.full(n, np.inf)]
    return P, q, A, l, u
```

Para trazar la frontera, **solo cambia $l_0$**.

### 4.4 Cómo resuelve: ADMM

**Idea general.** ADMM (*Alternating Direction Method of Multipliers*) resuelve un problema difícil partiéndolo en dos subproblemas fáciles y alternando entre ellos. Un multiplicador acumula la discrepancia entre ambos hasta que coinciden. La referencia clásica es Boyd et al. (2011).

**En OSQP.** Se introduce una copia $z = Aw$, de modo que el problema se separa en:
- **pieza 1:** minimizar el objetivo, sin mirar las cotas;
- **pieza 2:** $l \le z \le u$.

Se mantienen tres vectores: $w$ (la cartera), $z$ (los valores de las restricciones) e $y$ (los multiplicadores de Lagrange, uno por fila de $A$).

**Una iteración** (algoritmo 1 del paper de OSQP; parámetros por defecto $\rho = 0.1$, $\sigma = 10^{-6}$, $\alpha = 1.6$):

1. **Sistema lineal.** Resolver
   $$
   \begin{bmatrix}P+\sigma I & A^\top\\ A & -\rho^{-1}I\end{bmatrix}\begin{bmatrix}\tilde w\\ \nu\end{bmatrix}=\begin{bmatrix}\sigma w^k - q\\ z^k-\rho^{-1}y^k\end{bmatrix},\qquad \tilde z = z^k+\rho^{-1}(\nu-y^k).
   $$
   La matriz (la **matriz KKT**) no depende de $l$, $u$ ni de la iteración. `setup` la factoriza una vez, y cada iteración solo hace dos sustituciones triangulares, que son baratas.
2. **Relajación:** $w^{k+1} = \alpha\tilde w + (1-\alpha)w^k$.
3. **Proyección:** $z^{k+1} = \Pi_{[l,u]}\big(\alpha\tilde z + (1-\alpha)z^k + \rho^{-1}y^k\big)$, que recorta cada componente al intervalo $[l_k, u_k]$. Aquí se imponen las restricciones: un peso negativo se lleva a 0, el retorno se sube a $R_{\min}$, etc.
4. **Duales:** $y^{k+1} = y^k + \rho\big(\alpha\tilde z + (1-\alpha)z^k - z^{k+1}\big)$. Acumula cuánto se ha tenido que recortar y empuja a la siguiente iteración a respetar las restricciones.

**Criterio de parada.** Se para cuando ambos residuos son menores que $\varepsilon_{abs} + \varepsilon_{rel}\cdot$(escala del problema):
- **residuo primal** $\|Aw - z\|_\infty$: cuánto se violan las restricciones;
- **residuo dual** $\|Pw + q + A^\top y\|_\infty$: cuánto le falta a $w$ para ser óptima.

En la versión 1.x se exige además que la **brecha de dualidad** sea pequeña (`check_dualgap`, activado por defecto).

**Mecanismos adicionales:**
- **Escalado de Ruiz** (`scaling = 10` iteraciones): antes de empezar, reescala filas y columnas de $P$ y $A$ para equilibrarlas. Es relevante aquí porque $\mu$ y $\Sigma$ son del orden de $10^{-3}$.
- **$\rho$ adaptativo:** si el residuo primal y el dual están desequilibrados, OSQP cambia $\rho$ y refactoriza la matriz KKT. En nuestras ejecuciones ocurrió solo 1–3 veces por instancia (2000 QPs).
- ***Solution polishing*:** al terminar ADMM, usa $y$ para adivinar qué restricciones están activas ($y_k < 0$: cota inferior activa; $y_k > 0$: superior; $y_k = 0$: inactiva). Con ellas resuelve un único sistema lineal de igualdades, con una pequeña regularización y 3 pasos de refinamiento iterativo. Si acierta con el conjunto activo, pasa de la precisión de ADMM (~$10^{-6}$) a la de máquina. Si falla (`status_polish = -1`), se queda con la solución de ADMM. Solo se ejecuta si ADMM terminó con estado `solved`.
- ***Warm start*:** el siguiente `solve` arranca desde los $(w, y)$ de la solución anterior.

**Carácter de primer orden:**
- **Ventaja:** iteraciones muy baratas.
- **Inconveniente:** hacen falta muchas. Converge rápido hasta una precisión moderada y lento hasta una alta. Su velocidad depende mucho del condicionamiento y de la geometría del problema.
- Los métodos de **punto interior** (Gurobi, CPLEX, Clarabel) y de **conjunto activo** (HiGHS) hacen pocas iteraciones, más caras, y son más robustos en problemas degenerados.

### 4.5 Las tres operaciones de la API y por qué hay un bucle

1. **`setup(P, q, A, l, u, **settings)`:** escala el problema, construye la matriz KKT y la **factoriza**. Es lo más caro, y se hace una sola vez por instancia (unos milisegundos: 4–9 ms medidos).
2. **`update(l=..., u=..., q=...)`:** cambia vectores **sin refactorizar**, porque $l$ y $u$ no entran en la matriz KKT, solo en la proyección del paso 3.
3. **`solve()`:** ejecuta las iteraciones de ADMM (más el *polishing*) desde el último punto (*warm start*).

**Por qué un bucle.** OSQP resuelve **un** QP por llamada, y la frontera no es un QP sino 2000, uno por $R_{\min}$. OSQP no tiene una función para resolver muchos lados derechos a la vez, así que alguien tiene que recorrer los $R_{\min}$. El bucle aprovecha la estructura paramétrica:
- `setup` una vez, fuera del bucle;
- dentro, solo `update` + `solve`;
- cada `solve` arranca desde la solución anterior, que está muy cerca de la nueva.

```python
def solve_frontier(mu, Sigma, R_levels):
    """Solve the QP for every R_min in R_levels. Returns variances, statuses, iterations, seconds."""
    P, q, A, l, u = build_qp(mu, Sigma, R_levels[0])
    start = time.perf_counter()
    solver = osqp.OSQP()
    solver.setup(P, q, A, l, u, **SETTINGS)
    variance, status, iterations = [], [], []
    for R in R_levels:
        l[0] = R
        solver.update(l=l)
        res = solver.solve(raise_error=False)
        variance.append(res.x @ Sigma @ res.x)
        status.append(res.info.status)
        iterations.append(res.info.iter)
    seconds = time.perf_counter() - start
    return np.array(variance), np.array(status), np.array(iterations), seconds
```

El tiempo medido incluye `setup` y los 2000 `solve`. `res.x` es el nombre del atributo en OSQP; contiene los pesos $w$.

### 4.6 Qué devuelve OSQP y qué significa en Markowitz

Comprobado con **port1, $R_{\min} = 0.008$:**
- `res.info.status = "solved"`, 125 iteraciones, 0.71 ms.
- `res.x` = pesos $w$: retorno 0.008000, varianza 1.545024e-3, suma de pesos 1.000000. Se mantienen 4 activos: **5 (40.09 %), 9 (16.74 %), 26 (5.66 %) y 29 (37.51 %)**.
- `res.info.obj_val` = 1.545024e-3 = $w^\top\Sigma w$, la varianza.
- **`res.y`, los multiplicadores, tienen interpretación financiera:**
  - **$-y_0 = 0.5847 = dV/dR$:** la pendiente de la frontera en ese punto, es decir, cuánta varianza cuesta cada unidad extra de retorno. Coincide con la derivada numérica por diferencias centradas, 0.5847. Es el $\lambda$ de las KKT.
  - $y_1 = 0.001588$: el precio de la restricción de presupuesto.
  - $y_{2:}$: vale **0 para los activos en cartera** y es **negativo** (entre −0.00343 y −0.00079) para los activos en su cota $w_i = 0$.
  - Convención de signos de OSQP: $Pw + q + A^\top y = 0$, con $y_k < 0$ cuando la cota inferior está activa.
- `res.info.iter`, `run_time`, `solve_time`, `polish_time`, `status_polish` y `rho_updates` son información de diagnóstico.

### 4.7 Estados de OSQP y «puntos sin resolver»

- **`solved`:** los residuos han bajado de `eps`. La solución es fiable.
- **`solved inaccurate`:** solo se ha llegado a una precisión menor. La solución es aproximada.
- **`maximum iterations reached`:** se agotó `max_iter` sin llegar a la precisión pedida. Devuelve la última aproximación.
- **`primal infeasible inaccurate`:** OSQP «cree» que el problema es infactible sin estar seguro. Aquí es un falso diagnóstico, porque los problemas son factibles; ocurrió con `max_iter = 4000` en port5.

Un **punto sin resolver** es un nivel de retorno cuyo estado no es `solved`.
- No significa que el QP no tenga solución: la tiene, y portef la trae. Es una limitación del algoritmo, no del modelo.
- El notebook los cuenta (columna `not solved`), indica en qué rango de $R$ están, **los excluye de la desviación y de la gráfica** (la línea azul de port5 termina antes del último círculo negro) y lo indica en la leyenda.

### 4.8 Por qué OSQP (elección del solver)

| Solver | Algoritmo para QP convexo | Licencia | Valoración para este problema |
|---|---|---|---|
| Gurobi / CPLEX | Barrera (punto interior) y simplex | Comercial (licencia académica gratuita) | Muy robustos y precisos, también para MIQP (cardinalidad); requieren licencia |
| HiGHS | Conjunto activo (*active set*) | MIT | Preciso y ligero; sin MIQP |
| SCIP | Branch-and-bound para MINLP/CIP | Apache 2.0 | Sobredimensionado para un QP convexo; útil para la versión con cardinalidad |
| **OSQP** | **ADMM / *operator splitting*** | **Apache 2.0** | **Elegido** |

**Argumentos a favor:**
- La frontera exige **2000 QPs casi idénticos** en los que solo cambia $R_{\min}$. OSQP factoriza una vez, `update` cambia la cota sin refactorizar y el *warm start* aprovecha la solución anterior, así que cada QP cuesta milisegundos.
- Es de código abierto, sin licencia y se instala con `pip install osqp`.
- El *polishing* permite obtener alta precisión.

**Inconvenientes (a discutir con honestidad en el artículo):**
- Es un método de primer orden. Con la tolerancia por defecto ($10^{-3}$, sin *polishing*) el error en varianza llega a **3.1 % (port1), 11.2 % (port2) y 6.7 % (port3)** (medido). Hay que usar $10^{-6}$ con *polishing*.
- Converge muy despacio en problemas degenerados, como los puntos junto a $R_{\max}$.

---

## 5. Configuración usada y su justificación

```python
SETTINGS = dict(eps_abs=1e-6, eps_rel=1e-6, polishing=True, max_iter=20_000, verbose=False)
```

- **`eps_abs = eps_rel = 1e-6`:** suficiente para que ADMM identifique el conjunto activo y el *polishing* dé una solución exacta. Con $10^{-3}$ los errores son de varios %.
- **`polishing = True`:** lleva la solución a la precisión de máquina cuando acierta con el conjunto activo.
- **`max_iter = 20000`** (por defecto, 4000): con 4000 fallan muchos puntos cerca de $R_{\max}$ (tabla de la sección 7.2). Con 20000, port2 queda completa y solo fallan 3 puntos en port4 y 60 en port5, a cambio de más tiempo.
- **`warm_starting`:** activado por defecto.
- Se mantienen el resto de valores por defecto: $\rho = 0.1$ adaptativo, $\sigma = 10^{-6}$, $\alpha = 1.6$ y escalado de Ruiz con 10 iteraciones.
- **Formulación directa** ($P = 2\Sigma$): es la traducción literal del modelo del enunciado. El usuario eligió esta opción «sencilla» frente a otras más robustas (sección 8).
- **Rejilla:** los 2000 niveles de retorno de `portefK.txt`, para una comparación punto a punto.

---

## 6. Resultados del notebook final

### 6.1 Entorno

- CPU AMD Ryzen 5 5500U (portátil), Windows 11 Home, **un solo hilo**.
- Python 3.14.5, OSQP 1.1.3, NumPy 2.5.3, SciPy 1.18.1, pandas 3.0.6, matplotlib 3.11.2.
- Gestión del entorno con `uv`.

### 6.2 Tabla principal (ejecución del notebook del 18-09-2026)

| Instancia | Índice | $N$ | QPs | Tiempo (s) | ms/QP | Iter. medias | No resueltos | Máx. \|desv.\| (%) | Media \|desv.\| (%) |
|---|---|---|---|---|---|---|---|---|---|
| port1 | Hang Seng | 31 | 2000 | 0.87 | 0.44 | 76 | 0 | 7.8e-06 | 2.9e-06 |
| port2 | DAX 100 | 85 | 2000 | 25.93 | 12.96 | 797 | 0 | 3.6e-05 | 1.0e-05 |
| port3 | FTSE 100 | 89 | 2000 | 10.08 | 5.04 | 255 | 0 | 3.8e-02 | 3.6e-05 |
| port4 | S&P 100 | 98 | 2000 | 33.90 | 16.95 | 803 | 3 | 1.0e-02 | 1.7e-05 |
| port5 | Nikkei 225 | 225 | 2000 | 274.96 | 137.48 | 974 | 60 | 1.1e-01 | 1.2e-04 |

- Tiempo total de las 5 fronteras (10 000 QPs): ~346 s ≈ 5.8 min.
- **Variabilidad de los tiempos:** el mismo código ha tardado en port5 **183 s, 193 s y 275 s** en ejecuciones distintas del mismo portátil. En el artículo conviene decirlo o dar un rango.

### 6.3 Qué significa la desviación

$$
\text{desv.}_k = 100\cdot\frac{V_{\text{OSQP}}(R_k) - V_{\text{portef}}(R_k)}{V_{\text{portef}}(R_k)}\ \%
$$

- $V_{\text{portef}}(R_k)$: la varianza del fichero de referencia para el retorno $R_k$ (la «solución oficial»).
- $V_{\text{OSQP}}(R_k) = \sum_i\sum_j\sigma_{ij}w_iw_j$, con $w$ la cartera que devuelve OSQP resolviendo con $R_{\min} = R_k$.
- Se calcula solo sobre los puntos con estado `solved`.

**Ejemplo real** (port1, punto 1000):
```
R        = 0.0068266003
V_portef = 0.0010585969      (fichero, 10 decimales)
V_OSQP   = 0.001058596893
desviación = −6.9e-7 %       → diferencia en la décima cifra decimal: redondeo del fichero
```

**Interpretación de las columnas:**
- **Desviaciones de ~$10^{-5}$ %:** son el redondeo de portef (10 decimales); la solución es la misma.
- **Máximos de ~0.01–0.1 %** (port3, port4, port5): son unos pocos puntos `solved` en los que **falló el *polishing*** (`status_polish = -1`), así que la solución se queda en la precisión de ADMM. Violan la restricción de retorno en ~$1.6\text{–}1.9\cdot10^{-8}$ y por eso su varianza sale ligeramente **menor** que la de referencia (desviación negativa). Verificado en port3: puntos 1437 (−8.5e-3 %), 1445 (−1.05e-2 %) y 1985 (−3.8e-2 %).

### 6.4 Puntos no resueltos (con `max_iter = 20000`)

| Instancia | Nº | Rango de $R$ | $R_{\max}$ | Estados |
|---|---|---|---|---|
| port4 | 3 | 0.009090 – 0.009097 | 0.009195 | 3 `solved inaccurate` |
| port5 | 60 | 0.003856 – 0.003971 | 0.003971 | 44 `maximum iterations reached`, 16 `solved inaccurate` |

En total, 63 de 10 000 puntos, todos en el extremo superior de la frontera.

### 6.5 La figura (`figures/frontiers.pdf` / `.png`)

- Rejilla de 2×3 paneles.
- **Paneles 1–5**, uno por instancia, con riesgo (varianza) en el eje X y retorno medio en el eje Y:
  - **puntos rojos:** activos individuales ($s_i^2$, $\mu_i$), que son carteras ineficientes;
  - **círculos negros:** 26 puntos de portef, incluidos los dos extremos;
  - **línea azul:** la frontera de OSQP (solo puntos `solved`); la leyenda indica cuántos puntos no se resolvieron.
- La línea azul pasa por encima de los círculos.
- En port5 se ve un hueco al final: la línea termina antes del último círculo ($R_{\max}$).
- **Panel 6:** desviación (%) frente a la posición en la frontera, $(R - R_{GMV})/(R_{\max} - R_{GMV})$, para las 5 instancias. Es prácticamente cero salvo unos pocos picos negativos (hasta −0.11 % en port5), que son los fallos de *polishing*.

---

## 7. Análisis del rendimiento: cuál es el cuello de botella

### 7.1 Perfil del código del notebook (ejecución independiente)

| Instancia | Total | Iteraciones ADMM | *Polishing* | Python (update + envoltura + varianza) | `build_qp` + `setup` | Tiempo en el último 5 % de la frontera | Iter. medias: primer 95 % / último 5 % |
|---|---|---|---|---|---|---|---|
| port1 | 0.9 s | 0.53 s | 0.18 s | 0.23 s | 7 ms | 10 % | 71 / 175 |
| port2 | 26.5 s | 24.9 s | 1.1 s | 0.41 s | 6 ms | 47 % | 421 / 7 951 |
| port3 | 10.1 s | 8.7 s | 1.1 s | 0.24 s | 5 ms | 30 % | 177 / 1 741 |
| port4 | 31.0 s | 29.3 s | 1.4 s | 0.30 s | 7 ms | 69 % | 210 / 12 076 |
| port5 | 192.9 s | 185.3 s | 6.9 s | 0.57 s | 12 ms | 73 % | 261 / 14 512 |

**Conclusiones:**
- **~96 % del tiempo se gasta en las iteraciones de ADMM dentro de OSQP (código C).** El bucle y los cálculos en Python cuestan menos del 1 %: optimizar el Python, o reescribirlo en C++/Rust, no cambiaría nada apreciable.
- **El tiempo se concentra en el extremo de máximo retorno:** en port5, el último 5 % de los puntos consume el 73 % del tiempo, con 50 veces más iteraciones por QP.
- `setup` (la factorización) es despreciable: milisegundos.

En el script extendido (formulación de Cholesky, sección 8.1), el patrón era el mismo. En port5, los puntos 0–1499 necesitaban ~25 iteraciones por QP (~1.1 s por cada 100 puntos) y los **últimos 100 puntos se llevaban 455.7 s de 507.5 s**, con hasta 89 575 iteraciones por QP contando el re-solve.

### 7.2 Efecto de `max_iter` (formulación directa, `eps = 1e-6` + *polishing*)

| | port1 | port2 | port3 | port4 | port5 |
|---|---|---|---|---|---|
| Tiempo, `max_iter = 4000` | 0.8 s | 14.3 s | 9.8 s | 16.3 s | 87.2 s |
| No resueltos, `max_iter = 4000` | 0 | 121 | 0 | 82 | 125 |
| Tiempo, `max_iter = 20000` | 0.9 s | 25.9 s | 10.1 s | 33.9 s | 275.0 s |
| No resueltos, `max_iter = 20000` | 0 | 0 | 0 | 3 | 60 |

Detalle con `max_iter = 4000`:
- **port2:** 37 `maximum iterations reached` y 84 `solved inaccurate`, con $R$ entre 0.009263 y 0.009794.
- **port4:** 50 `max iter` y 32 `inaccurate`, con $R$ entre 0.008901 y 0.009195.
- **port5:** 11 `max iter`, **46 `primal infeasible inaccurate`** y 68 `inaccurate`, con $R$ entre 0.003729 y 0.003971. En algunos puntos la `x` devuelta era basura, con varianzas ~$10^{25}$ veces la real.

En otra medición con 20000 iteraciones (tres procesos en paralelo): port2 27.0 s (0 fallos, máx. desv. 3.6e-5 %), port4 34.4 s (3 fallos) y port5 183.1 s (60 fallos).

### 7.3 Por qué los puntos junto a $R_{\max}$ son tan difíciles

- Para conseguir un retorno muy cerca de $R_{\max}$ hay que invertir casi todo en el activo de mayor $\mu$: el 214 en port5, el 82 en port4. Por ejemplo, si el segundo mejor activo tiene un $\mu$ bastante menor, cumplir la restricción exige $w_{214} \approx 1$.
- El **conjunto factible se reduce a una esquina muy estrecha del símplex**, y en $R_{\max}$ exactamente es **un único punto** (100 % en ese activo), sin punto estrictamente factible (falla la condición de Slater). El problema es **degenerado**.
- La **pendiente $dV/dR = -y_0$ se dispara**, porque la frontera se vuelve casi vertical en el plano varianza-retorno. El multiplicador de la restricción de retorno es enorme comparado con la escala del problema.
- ADMM avanza con pasos de corrección de tamaño $\rho$. En una región tan estrecha, la proyección (paso 3) recorta casi todo y el paso dual (paso 4) tiene que acumular un multiplicador enorme poco a poco: **miles de iteraciones**. Con proyecciones alternadas sobre conjuntos casi tangentes, la convergencia es muy lenta.
- Afecta a port2, port4 y port5 (con `max_iter = 4000`) y sobre todo a port5, que además es la peor condicionada ($\kappa \approx 3.7\cdot10^4$).

---

## 8. Experimentos realizados y alternativas evaluadas

### 8.1 Reformulación de Cholesky (script extendido, archivado en `ignore/markowitz_osqp.py`)

- **Idea:** $\Sigma = LL^\top$; se introduce $y = L^\top w$, de modo que $w^\top\Sigma w = \|y\|^2$. Las variables pasan a ser $(w, y)$, con $P = \mathrm{diag}(0, 2I)$ y $N$ filas extra $[L^\top\ \ -I]$ con $l = u = 0$. El problema es equivalente pero mejor condicionado para ADMM; es el truco del paper de OSQP para carteras.
- **Resultado:** en la parte media de la frontera necesita 2–3 veces menos iteraciones y tiempo que la formulación directa (con `eps = 1e-7`: port2 17.5 s frente a 52.4 s; port3 9.6 s frente a 20.0 s; port4 31.1 s frente a 67.6 s). **No resuelve el extremo de $R_{\max}$.**
- **Script completo:** Cholesky + `eps = 1e-6` + *polishing* + **re-solve con `eps = 1e-9` y `max_iter = 200000`** en los puntos donde falla el *polishing*:
  - **todos los puntos resueltos**, con máx. desv. ≤ 4.1e-5 % en las 5 instancias;
  - tiempos: port1 0.90 s, port2 10.88 s, port3 6.25 s, port4 14.01 s y **port5 507.5 s (~8.5 min)**;
  - puntos re-resueltos: 1, 0, 4, 1 y 39.
  - Los tiempos de port1–port4 se midieron con otro proceso compitiendo por la CPU.

### 8.2 Ajustes de OSQP probados en el extremo de port5 (ninguno lo arregla)

- **Restricción de retorno como igualdad** ($l_0 = u_0 = R$; OSQP da un $\rho$ 1000 veces mayor a las igualdades): 50 últimos puntos, 20 000 iteraciones cada uno, `solved inaccurate`, desviación hasta 1.5 % (frente a 1.7 % con $\ge$).
- **`rho = 10`:** `solved inaccurate`, desviación hasta 3.1 %.
- **`rho = 1e-3`, `adaptive_rho_interval = 25`, `scaling = 0`, `check_dualgap = False`:** `primal infeasible inaccurate` en los 20 últimos puntos (arrancando en frío).
- **Fila de retorno normalizada** ($(\mu - R_{GMV})/(R_{\max} - R_{GMV})$, equivalente usando el presupuesto): evita el falso diagnóstico de infactibilidad, pero sigue con 53 `max iter` y 10 `inaccurate` en los 150 últimos puntos (desviación hasta 3.6 %).
- **Reescritura como déficit** $(\mu_{\max} - \mu)^\top w \le \mu_{\max} - R$: `primal infeasible inaccurate`.
- **Tolerancia laxa (`1e-4`) dejando el trabajo al *polishing*:** rápido (1.7 s en 150 puntos), pero con 36–40 fallos de *polishing* y desviaciones de hasta el 12 %.
- **Reescalado manual del objetivo y de la fila de retorno:** peor que dejar el escalado de Ruiz de OSQP (más iteraciones en port2).

### 8.3 Formulación ponderada (suma ponderada)

$\min\ w^\top\Sigma w - t\,\mu^\top w$ s.a. $\sum w_i = 1$, $w \ge 0$. Solo cambia $q = -t\mu$ (`update(q=...)`). Se usaron 2000 valores de $t$: $t = 0$ más una rejilla geométrica hasta $1.05\,t_{\max}$, donde $t_{\max} = \max_i 2(\sigma_{jj} - \sigma_{ij})/(\mu_j - \mu_i)$ es el valor a partir del cual la solución es el activo $j$ de mayor $\mu$.
- **Todos los puntos `solved` en las 5 instancias.** Tiempos: port1 0.49 s, port2 2.77 s, port3 2.24 s, port4 4.25 s y **port5 35.7 s**, con 500 iteraciones como máximo por QP.
- Desviación frente a portef **interpolada linealmente** en los retornos obtenidos: máx. 5.6e-5 % (port1), 1.5e-3 % (port2), 1.9e-4 % (port3), 2.7e-3 % (port4) y 2.3e-2 % (port5). Hay que interpolar porque no se controla qué retornos salen.
- **Por qué funciona:** el conjunto factible es siempre el símplex completo y no hay esquinas estrechas.
- **Por qué no se eligió:** no es el modelo con $R_{\min}$ del enunciado.

### 8.4 Otras alternativas (no probadas aquí)

- **Solver de punto interior o de conjunto activo** (HiGHS, Gurobi, CPLEX, Clarabel): no tendría el problema de $R_{\max}$, pero dejaría de ser OSQP.
- **Paralelizar las 5 instancias:** el tiempo total bajaría al de port5, pero los tiempos por instancia de la tabla serían menos fiables por la competencia por la CPU.
- **Paralelizar dentro de una instancia:** pierde parte del *warm start*. No recomendado.

### 8.5 Decisiones del usuario

- **Formulación:** «modelo con $R_{\min}$, sencillo» (la directa), frente a la robusta (Cholesky + re-solve, ~8.5 min) y la ponderada (~45 s, pero otro modelo).
- **Idioma:** notebook, código y README en inglés; guion del artículo en español.
- **Estilo:** notebook sencillo, sin clases propias y usando OSQP directamente: carga de datos, resolución y gráficas.
- **Notación:** pesos $w_i$ en lugar de $x_i$.
- **`max_iter = 20000`:** decisión tomada durante la sesión y aceptada. Las cifras del artículo dependen de ella.

---

## 9. Mensajes clave para el artículo

1. **El modelo básico de Markowitz es un QP convexo** (estrictamente convexo aquí), «fácil» en el sentido de Beasley: existe un óptimo único y algoritmos eficientes para encontrarlo.
2. **La frontera eficiente se traza resolviendo una familia paramétrica de QPs** en la que solo cambia $R_{\min}$. OSQP explota esa estructura (una factorización, `update` de las cotas y *warm start*), de modo que cada QP cuesta entre 0.4 ms (port1) y ~140 ms (port5).
3. **Precisión:** en los puntos resueltos, la frontera de OSQP coincide con la de OR-Library hasta el redondeo del fichero (~$10^{-5}$ %). Solo hay unos pocos picos de hasta 0.1 % donde falla el *polishing*.
4. **Hay que configurar OSQP:** con los valores por defecto (`eps = 1e-3`, sin *polishing*, 4000 iteraciones) los errores son de varios % y se pierden muchos puntos.
5. **Limitación de un método de primer orden:** en el extremo de máximo retorno el QP es degenerado y ADMM necesita decenas de miles de iteraciones. Resultado: 63 de 10 000 puntos sin resolver y el ~73 % del tiempo de port5 en el último 5 % de la frontera. Es un resultado interesante en sí mismo y explica por qué no hay un solver universalmente mejor: OSQP es excelente para resolver muchos QPs parecidos rápido, pero no es la mejor herramienta para problemas degenerados.
6. **El tiempo no depende solo de $N$:** port3 ($N = 89$) necesita 255 iteraciones medias frente a 797 de port2 ($N = 85$). Influyen el condicionamiento y la geometría del extremo superior de la frontera.
7. **Conexión con la asignatura:** con restricción de cardinalidad el problema se vuelve MIQP y aparecen las heurísticas de Chang et al. (2000).

---

## 10. Otras explicaciones de la sesión

### 10.1 Sobre el renderizado de LaTeX

El chat de Claude Code en VS Code no renderiza LaTeX. Las fórmulas sí se ven en el notebook, en la vista previa de Markdown de VS Code y en el PDF final.

### 10.2 Plan para una versión en C++ (no realizada)

- **No iría más rápido:** el tiempo está dentro de la biblioteca en C de OSQP.
- **No sustituye al entregable:** el enunciado pide Python o Julia.
- **Estructura propuesta:**
  - `cpp/CMakeLists.txt`, con OSQP 1.1.3 vía `FetchContent`;
  - `src/data.cpp` (lectores);
  - `src/qp.cpp` (matrices CSC a mano: P triangular superior con $N(N+1)/2$ elementos; A con 3 elementos por columna, en las filas 0 ($\mu_j$), 1 (1) y 2+j (1));
  - `src/main.cpp`, con `OSQPSettings_new`, `osqp_setup`, `osqp_update_data_vec`, `osqp_solve` y `osqp_cleanup`;
  - salida CSV y un script de Python para las gráficas.
- **Validación:** con la misma versión y los mismos ajustes, iteraciones, estados y varianzas deben coincidir con los de Python.

### 10.3 Estado del repositorio (a tener en cuenta)

- **Entregable de código:** `markowitz_osqp.ipynb`. La figura para el artículo es `figures/frontiers.pdf`.
- **`results/`:** salidas del script extendido antiguo: CSVs por instancia, `summary.md` y `figures/`, con una figura de composición de carteras de port1 (`results/figures/composition_port1.pdf`) útil como figura opcional.
- **La nota final del notebook cita `markowitz_osqp.py`,** que ahora está archivado en `ignore/`: conviene corregir la referencia.
- **`main.py` y la configuración de pytest** apuntan a código y tests que ya no están en la raíz.

---

## 11. Bibliografía (DOIs verificados)

1. Beasley, J. E. (2013). Portfolio optimisation: Models and solution approaches. In H. Topaloglu (Ed.), *Theory Driven by Influential Applications* (pp. 201–221). INFORMS TutORials in Operations Research. https://doi.org/10.1287/educ.2013.0114
2. Beasley, J. E. (1990). OR-Library: Distributing test problems by electronic mail. *Journal of the Operational Research Society*, 41(11), 1069–1072. https://doi.org/10.1057/jors.1990.166 · Datos: https://people.brunel.ac.uk/~mastjjb/jeb/orlib/portinfo.html
3. Chang, T.-J., Meade, N., Beasley, J. E., & Sharaiha, Y. M. (2000). Heuristics for cardinality constrained portfolio optimisation. *Computers & Operations Research*, 27(13), 1271–1302. https://doi.org/10.1016/S0305-0548(99)00074-X
4. Markowitz, H. (1952). Portfolio selection. *The Journal of Finance*, 7(1), 77–91. https://doi.org/10.1111/j.1540-6261.1952.tb01525.x
5. Stellato, B., Banjac, G., Goulart, P., Bemporad, A., & Boyd, S. (2020). OSQP: An operator splitting solver for quadratic programs. *Mathematical Programming Computation*, 12(4), 637–672. https://doi.org/10.1007/s12532-020-00179-2
6. Boyd, S., Parikh, N., Chu, E., Peleato, B., & Eckstein, J. (2011). Distributed optimization and statistical learning via the alternating direction method of multipliers. *Foundations and Trends in Machine Learning*, 3(1), 1–122. https://doi.org/10.1561/2200000016
7. Boyd, S., & Vandenberghe, L. (2004). *Convex Optimization*. Cambridge University Press. (QP y optimización de carteras, §4.4)
8. Cornuéjols, G., Peña, J., & Tütüncü, R. (2018). *Optimization Methods in Finance* (2nd ed.). Cambridge University Press. https://doi.org/10.1017/9781107297340
9. Huangfu, Q., & Hall, J. A. J. (2018). Parallelizing the dual revised simplex method. *Mathematical Programming Computation*, 10(1), 119–142. (Base de HiGHS; su solver de QP es de conjunto activo: https://highs.dev)
10. Bolusani, S., et al. (2024). *The SCIP Optimization Suite 9.0*. arXiv:2402.17702. https://arxiv.org/abs/2402.17702
11. Gurobi Optimization, LLC. *Gurobi Optimizer Reference Manual*. https://www.gurobi.com · IBM. *ILOG CPLEX Optimization Studio*. https://www.ibm.com/products/ilog-cplex-optimization-studio
12. OSQP documentation (settings, warm start, polishing). https://osqp.org/docs/
