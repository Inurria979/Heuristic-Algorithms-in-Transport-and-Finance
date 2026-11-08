# simTOP — explicación del paper y especificación para implementarlo

> Nota interna (v4, 08-11-2026). Escrita por `lector-papers` a partir del paper y de la
> conversación con el usuario. Convención: **[paper p. X]** = lo dice el paper (página impresa de
> las actas; la página del PDF es la impresa − 1905); **[deducción]** = lo deduzco yo;
> **[usuario]** = lo ha decidido el usuario; **[TOP]** = se toma del código ya hecho en `TOP/`.
> Estado: **listo para implementar** (v4). Lo que es propuesta y no decisión está marcado en §11.

---

## 1. Fuentes

| Documento | Páginas relevantes |
|---|---|
| `docs/SIMULATION-BASED OPTIMIZATION IN TRANSPORTATION AND LOGISTICS.pdf` — Juan, Reyes-Rubiano, de la Torre, Panadero, Faulin y Latorre (2019), *Simulation-based optimization in transportation and logistics: comparing sample average approximation with simheuristics*, Proc. Winter Simulation Conference, pp. 1906–1917 | §3 problema (pp. 1910–1911), §4 SAA (pp. 1911–1912), §5 simheurística (p. 1912), §6 experimentos y generación de instancias (pp. 1912–1913), Fig. 2 (p. 1913), §7 resultados y Fig. 3 (p. 1914), Fig. 4 (p. 1915) |
| `data/top-data/` — 387 instancias de Chao et al. (1996), idénticas a `TOP/data/top-data/` | — |
| `TOP/src/rust/src/heuristic.rs`, `grasp.rs`, `local_search.rs` — PJS, aleatorización sesgada geométrica y búsqueda local 2-opt + inserción ya implementadas (Rust) para el TOP determinista | referencia de implementación, no fuente del paper |
| `TOP/data/bks_chao.csv` — mejor solución conocida (BKS) determinista de las 353 instancias factibles | necesaria para el gap de la Fig. 3 |

**Enunciado de la práctica:** no hay diapositiva de *Homework* en `docs/`. Alcance fijado por el
usuario (08-11-2026) [usuario]:
- **Implementar la simheurística** del §5 del paper: heurística PJS con aleatorización sesgada (BR)
  + búsqueda local + simulación Monte Carlo rápida (élite) e intensiva (final).
- Evaluarla en los **tres niveles de varianza** y **solo en las instancias pequeñas (< 35 nodos:
  conjuntos p1, p2 y p3)**, que son las que el paper pudo comparar con el SAA.
- **El SAA con CPLEX queda fuera** (solver comercial; además en el paper solo resolvió instancias
  pequeñas).
- La PJS determinista ya está hecha en `TOP/` y es el punto de partida.

---

## 2. Generación de las instancias estocásticas  *(primera sección por decisión del usuario)*

**El paper no crea instancias nuevas desde cero**: toma las de Chao y les añade aleatoriedad en los
tiempos de viaje [paper p. 1912]. Una «instancia estocástica» es el par *(fichero de Chao, nivel de
varianza c)*. «Generarla» significa saber muestrear el tiempo de cada arco. Los tiempos aleatorios se
**generan al vuelo con semilla fija**; no se escriben escenarios en ficheros ni se toca `data/`
[usuario].

### 2.1 Instancias base [paper p. 1912]

- Conjuntos de Chao p1, p2, p3 (pequeños, «menos de 35 nodos») y p4 a p7 (medianos y grandes,
  «de 60 a más de 100 nodos»). **Para esta práctica solo p1–p3** [usuario].
- Formato de los ficheros de `data/top-data/` (separador `;`):

  ```
  n;32          ← número total de nodos, depósitos incluidos
  m;2           ← número de vehículos
  tmax;2.5      ← tiempo máximo por ruta
  10.500;14.400;0     ← nodo 0: depósito de origen (x;y;recompensa)
  18.000;15.900;10    ← clientes 1..n-2
  ...
  11.200;14.100;0     ← nodo n-1: depósito de destino
  ```
  Nombre: `p<conjunto>.<m>.<letra>.txt` (p. ej. `p1.2.a` = conjunto 1, 2 vehículos, instancia *a*).
  Ojo: el `n` del fichero cuenta los depósitos; en la notación del paper hay $n$ clientes y $n+2$
  nodos [paper p. 1910].
- Tamaños: p1 → 32 nodos, p2 → 21, p3 → 33. Ficheros: 54 + 33 + 60 = 147. De ellos, 6 de p1 no tienen
  ningún cliente alcanzable dentro de `tmax` (no figuran en `bks_chao.csv`), así que quedan
  **141 instancias factibles** (48 + 33 + 60).
- Tiempo determinista de cada arco: distancia euclídea sin redondear,
  $t_{ij}=\sqrt{(x_i-x_j)^2+(y_i-y_j)^2}$ [deducción: el paper no lo dice; es la convención de Chao y
  la de `TOP/src/rust/src/model.rs`].

### 2.2 Tiempo de viaje aleatorio [paper p. 1913]

Para cada arco $(i,j)$:

$$T_{ij}\sim \text{LogNormal}(\mu_{ij},\sigma_{ij}),\qquad E[T_{ij}]=t_{ij},\qquad \text{Var}[T_{ij}]=c\cdot t_{ij},$$

con $T_{ij}=T_{ji}$ [paper p. 1910] y $c>0$ el parámetro de incertidumbre. Tres niveles:

| Nivel | $c$ |
|---|---|
| bajo (L) | 0.05 |
| medio (M) | 0.25 |
| alto (H) | 0.75 |

Parámetros de la lognormal (ecuación 5, p. 1913), donde $\mu,\sigma$ son la media y la desviación
típica del **logaritmo** de $T$:

$$\mu_{ij}=\ln t_{ij}-\tfrac12\ln\!\Big(1+\frac{c\,t_{ij}}{t_{ij}^2}\Big),\qquad
\sigma_{ij}=\sqrt{\ln\!\Big(1+\frac{c\,t_{ij}}{t_{ij}^2}\Big)}=\sqrt{\ln\!\Big(1+\frac{c}{t_{ij}}\Big)} .$$

(El paper pone el valor absoluto en $\sigma$; es redundante.) Muestreo: $T_{ij}=\exp(\mu_{ij}+\sigma_{ij}Z)$
con $Z\sim N(0,1)$, o `rng.lognormal(mean=mu, sigma=sigma)` en numpy.

**Ejemplo a mano** ($t_{ij}=10$):

| $c$ | Var | $\sigma^2=\ln(1+c/t)$ | $\sigma$ | $\mu$ | coef. de variación $\sqrt{c\,t}/t$ |
|---|---|---|---|---|---|
| 0.05 | 0.5 | 0.004988 | 0.0706 | 2.30010 | 7.1 % |
| 0.25 | 2.5 | 0.024693 | 0.1571 | 2.29024 | 15.8 % |
| 0.75 | 7.5 | 0.072321 | 0.2689 | 2.26643 | 27.4 % |

La varianza es **proporcional a $t$, no a $t^2$**: los arcos cortos son relativamente mucho más
variables (con $t=1$ y $c=0.75$ el coeficiente de variación es 87 %).

### 2.3 Detalles que el paper no fija

| Detalle | Decisión | Estado |
|---|---|---|
| Varianza $c\cdot t_{ij}$ (lineal) y no $c\cdot t_{ij}^2$ | **Al pie de la letra, como el paper**, porque el objetivo es comparar con sus resultados | decidido [usuario] |
| Independencia entre arcos y entre réplicas | **Independientes**: en cada réplica, cada arco se sortea por separado, sin correlación con los demás arcos ni con otras réplicas (el paper no menciona correlación). Dentro de una misma réplica, un arco tiene el mismo valor para todas las candidatas que lo usan (números aleatorios comunes, §6.1) | decidido [usuario] |
| Arcos con $t_{ij}=0$ | En las 387 instancias hay 60 y **todos son el arco origen–destino de p7** (los dos depósitos coinciden); nunca hay dos clientes en el mismo punto. En p1–p3 no aparece. Por robustez: $T_{ij}=0$ si $t_{ij}=0$ y no calcular $\mu,\sigma$ ahí (evita $\ln 0$). No afecta al problema | decidido [usuario] |
| Escenarios en ficheros o al vuelo | **Al vuelo, con semilla fija** | decidido [usuario] |

**Consecuencia de Var = c·t en una ruta entera** [deducción]: con arcos independientes las varianzas
se suman, así que una ruta con tiempo esperado $E_r=\sum t_{ij}$ tiene $\text{Var}_r=c\,E_r$ y desviación
típica $\sqrt{c\,E_r}$. Para una ruta que apura $t_{max}$, la desviación relativa es $\sqrt{c/t_{max}}$:
con $c=0.25$, un 22 % si $t_{max}=5$ y un 5 % si $t_{max}=100$. En p1–p3 $t_{max}$ va de 1.2 a 55
(p1: 1.2–42.5; p2: 3.8–22.5; p3: 3.8–55), así que **el mismo $c$ da incertidumbres muy distintas**
según la instancia. Además, una ruta cuyo $E_r$ es justo $t_{max}$ falla en casi la mitad de las
réplicas.

### 2.4 Qué significa que una ruta «falle» [paper p. 1911]

Una réplica de simulación de una ruta $r=(0,i_1,\dots,i_k,n+1)$ suma los tiempos muestreados de sus
arcos. Si la suma supera $t_{max}$, **se pierde toda la recompensa de esa ruta** en esa réplica; si no,
se cobra entera. La recompensa de la solución en la réplica es la suma sobre sus $m$ rutas.

---

## 3. El problema en dos minutos

Team Orienteering Problem (TOP) con tiempos de viaje aleatorios [paper pp. 1910–1911]. Hay $m$
vehículos iguales que salen del depósito de origen y terminan en el de destino. Cada cliente $i$ da
una recompensa $u_i$ la primera vez que se visita (visitarlo otra vez no da nada). Cada ruta tiene un
tiempo máximo $t_{max}$ (p. ej. la batería de un dron o la jornada de un conductor).

- **Se decide**: qué clientes visita cada vehículo y en qué orden (no hace falta visitarlos todos).
- **Se maximiza**: la recompensa total **esperada**.
- **Regla clave**: el tiempo de cada tramo es aleatorio; si un vehículo llega al destino después de
  $t_{max}$, **pierde toda la recompensa de su ruta**.
- **Por qué es difícil**: ya la versión determinista es NP-difícil (Chao et al. 1996); además, para
  saber cuánto vale una solución hay que estimar una esperanza, lo que obliga a simular.

**Ejemplo mínimo.** Un vehículo, $t_{max}=10$. Ruta A: tiempo esperado 9.8, recompensa 30. Ruta B:
tiempo esperado 8, recompensa 25. En el problema determinista gana A. Con incertidumbre alta, A
llega tarde, digamos, el 45 % de las veces (recompensa esperada ≈ 0.55·30 = 16.5), mientras que B
casi nunca falla (≈ 25). **Con incertidumbre, dejar holgura compensa.**

## 4. La idea del método

El paper compara dos formas de optimizar con simulación: el **SAA**, que sustituye la esperanza por
la media sobre 1 000 escenarios y lo resuelve con CPLEX (fuera de esta práctica), y la
**simheurística**, que es lo que vamos a implementar [paper p. 1912, Fig. 2 p. 1913].

La simheurística separa dos trabajos:
1. **Generar buenas soluciones rápido, pensando en determinista.** Se usa la heurística de ahorros
   PJS: empieza con una ruta por cliente y las va fusionando por orden de «preferencia» (mezcla de
   tiempo ahorrado y recompensa), usando los tiempos **esperados** y sin pasar de $t_{max}$. La
   **aleatorización sesgada (BR)** hace que, en vez de coger siempre la mejor fusión, se coja casi
   siempre una de las primeras de la lista, pero no siempre la primera. Así cada ejecución da una
   solución distinta y buena. Después, una **búsqueda local** (2-opt) desenreda las rutas, que se
   acortan y dejan sitio para meter más clientes.
2. **Juzgarlas con simulación.** Las soluciones más prometedoras en determinista (en nuestra versión,
   las 7 mejores) se simulan 200 veces sorteando los tiempos (simulación **rápida**) para estimar su
   recompensa esperada; las rutas que llegan tarde no cobran, así que las soluciones que apuran
   demasiado $t_{max}$ bajan. Las mejores (3) se simulan 1 000 veces (simulación **intensiva**) para
   tener una estimación precisa y elegir la ganadora.

Papel de cada fuente de aleatoriedad: la **BR** sirve para explorar (diversidad de soluciones); la
**simulación** sirve para evaluar (estimar la esperanza). Son dos generadores aleatorios distintos.

## 5. Modelo formal

| Símbolo | Significado |
|---|---|
| $N=\{0,1,\dots,n+1\}$ | nodos: 0 origen, $1..n$ clientes, $n+1$ destino |
| $m$ | número de vehículos (rutas) |
| $u_i\ge 0$ | recompensa del cliente $i$; $u_0=u_{n+1}=0$ |
| $t_{ij}$ | tiempo esperado del arco $(i,j)$ = distancia euclídea |
| $T_{ij}$ | tiempo aleatorio del arco, LogNormal con media $t_{ij}$ y varianza $c\,t_{ij}$ |
| $t_{max}$ | tiempo máximo por ruta |
| $c$ | nivel de incertidumbre (0.05 / 0.25 / 0.75) |
| $x^d_{ij}\in\{0,1\}$ | 1 si el vehículo $d$ recorre el arco $(i,j)$ |
| $U_j$ | recompensa realmente cobrada en $j$ (aleatoria): $u_j$ si $j$ está en una ruta que llega a tiempo, 0 si no |

Objetivo [paper p. 1911, ec. 1]: $\max \sum_j E[U_j]$, es decir, maximizar la recompensa total esperada.

Restricciones [paper p. 1911, ecs. 2–3]:
- Continuidad de rutas (lo que entra en un cliente sale de él); cada ruta empieza en 0 y acaba en $n+1$;
  cada cliente se cobra como mucho una vez.
- **Restricción de diseño**: $\sum_{(i,j)} x^d_{ij}\,t_{ij}\le t_{max}$ para cada vehículo $d$ (con tiempos
  **esperados**). En el problema estocástico es «blanda»: el tiempo real puede pasarse, y entonces se
  pierde la recompensa de esa ruta.

**Cómo se programa** [deducción]: una solución es una lista de como mucho $m$ rutas, cada una una
lista de clientes sin repetir entre rutas. Es **factible** si cada ruta tiene tiempo esperado
$\le t_{max}$. Se evalúa de dos formas:
- *Recompensa determinista*: $\sum$ de $u_i$ de los clientes visitados.
- *Recompensa estocástica* (estimada con $R$ réplicas): media sobre réplicas de
  $\sum_{r}\big[\text{tiempo simulado}(r)\le t_{max}\big]\cdot\text{recompensa}(r)$.

## 6. Algoritmo paso a paso

Entradas de una ejecución: instancia, $c\in\{0.05,0.25,0.75\}$, $\alpha\in\{0.25,0.5,0.75\}$,
$\beta=0.3$, 50 iteraciones BR, 7 candidatas a simulación rápida (200 réplicas), 3 a intensiva
(1000 réplicas), semilla (ver F8 y F9).

**F1. `load_instance(path)`** → coordenadas, recompensas, $m$, $t_{max}$; matriz $t_{ij}$ euclídea. (§2.1)

**F2. `lognormal_params(t, c)`** → matrices $\mu,\sigma$ según §2.2 (sin calcular donde $t_{ij}=0$).

**F3. `pjs_construct(inst, alpha, selector, rng)`** — PJS [paper p. 1912; detalle exacto de TOP,
`heuristic.rs`]:
1. *Solución inicial*: una ruta $0\to i\to n+1$ por cada cliente $i$ con $t_{0i}+t_{i,n+1}\le t_{max}$
   (los demás no se pueden atender nunca y se descartan).
2. *Lista de preferencias*: para cada par ordenado de clientes alcanzables $(i,j)$, $i\ne j$
   (**las dos direcciones**, porque origen y destino son nodos distintos y $s_{ij}\ne s_{ji}$):
   $s_{ij}=t_{i,n+1}+t_{0,j}-t_{ij}$ (tiempo ahorrado al unir la ruta que acaba en $i$ con la que empieza
   en $j$) y preferencia $s'_{ij}=\alpha\,s_{ij}+(1-\alpha)(u_i+u_j)$. Ordenar de mayor a menor $s'$.
3. *Fusiones*: mientras queden arcos en la lista, el `selector` elige uno y se quita de la lista (se use
   o no). El arco $(i,j)$ fusiona la ruta A y la ruta B si: A y B son rutas distintas, $i$ es el
   **último** cliente de A, $j$ es el **primero** de B y el tiempo esperado de A+B (recalculado tramo a
   tramo) es $\le t_{max}$. La ruta nueva es A seguida de B.
4. *Selección*: ordenar las rutas por recompensa decreciente (empate: menor tiempo) y quedarse con las
   $m$ primeras.

**F4. Selector greedy**: siempre el primer arco que queda (PJS determinista).
**F5. Selector BR geométrico** [paper p. 1912 cita Grasas et al. 2017; detalle de TOP, `grasp.rs`]:
posición $k=\lfloor \ln U/\ln(1-\beta)\rfloor \bmod L$, con $U\sim\text{Unif}(0,1]$ y $L$ el número de
arcos que quedan; se toma el $k$-ésimo arco contando desde el principio de la lista ($k=0$ es el
primero). $P(k)\approx\beta(1-\beta)^k$; con $\beta=0.3$ el primero se elige un 30 % de las veces.

**F6. `local_search(inst, sol)`** — búsqueda local [paper p. 1912 menciona 2-opt; detalle exacto de
TOP, `local_search.rs`; usar esta misma, sin operador de destrucción aparte: decisión del usuario].
Repetir:
1. *2-opt* en cada ruta, primera mejora: invertir el tramo $v_i..v_j$ si eso acorta la ruta; repetir hasta
   que ninguna inversión acorte. No cambia la recompensa.
2. *Inserción*: entre los clientes no visitados que caben en alguna ruta (tiempo esperado
   $\le t_{max}$), insertar el de **mayor recompensa** (empate: menor tiempo añadido) en su posición
   más barata.

   hasta que no se pueda insertar ningún cliente. Ningún movimiento empeora la solución determinista.

**F7a. `sample_scenarios(mu, sigma, t, R, rng)`** → array `T` de forma $(R, N, N)$: el tiempo de
cada arco en cada uno de los $R$ «días» simulados. Para cada réplica $r$ y cada par $i<j$ con
$t_{ij}>0$: $Z\sim N(0,1)$ independiente, $T[r,i,j]=T[r,j,i]=\exp(\mu_{ij}+\sigma_{ij}Z)$ (simétrico,
paper p. 1910); $T[r,i,j]=0$ si $t_{ij}=0$. (En p1–p3, $N\le 33$: $1000\times33\times33$ reales ≈ 9 MB.)

**F7b. `simulate(inst, sol, T)`** → para cada réplica $r$: tiempo de cada ruta = suma de
$T[r,\cdot,\cdot]$ sobre sus arcos (incluidos $0\to$ primer cliente y último cliente $\to n+1$);
recompensa de la réplica = suma de las recompensas de las rutas con tiempo $\le t_{max}$ (§2.4).
Devuelve media, desviación típica de la recompensa por réplica y % de rutas que fallan. Al recibir los
escenarios ya sorteados, **varias soluciones se pueden evaluar sobre los mismos días** (ver F8 y §6.1).

**F8. `simheuristic(inst, c, alpha, seed)`** — una ejecución [paper p. 1912 y Fig. 2; números y
filtros: decisión del usuario]. Objetivo de coste: **unos pocos segundos por ejecución** [usuario],
por eso no se simulan todas las soluciones.
1. **Generadores.** De `seed` salen tres generadores independientes: `rng_br` (aleatorización sesgada),
   `rng_fast` (simulación rápida) y `rng_int` (simulación intensiva).
2. **Fase determinista.** Candidatas:
   - la **PJS greedy** (F3 con F4) mejorada con `local_search` [usuario: «una extra»];
   - **50** soluciones `local_search(pjs_construct(inst, alpha, BR(0.3), rng_br))` [usuario].

   Total: 51. Todas respetan $t_{max}$ con tiempos esperados.
3. **Quitar duplicados** [usuario]. Dos soluciones son la misma si tienen las mismas rutas (misma
   secuencia de clientes en cada una), sin importar el orden de las rutas. Forma canónica:
   `frozenset(tuple(ruta) for ruta in sol)`. El sentido de la ruta sí cuenta, porque origen y destino son
   nodos distintos.
4. **Filtro determinista.** Ordenar las distintas por recompensa determinista decreciente (empate:
   menor tiempo esperado total; si sigue el empate, la que se generó antes) y quedarse con las **7**
   primeras; si hay menos de 7 distintas, con todas las que haya [usuario].
5. **Simulación rápida, con números aleatorios comunes** [usuario]:
   `T_fast = sample_scenarios(..., R=200, rng_fast)` **una sola vez**, y las 7 candidatas se evalúan con
   `simulate(inst, sol, T_fast)` sobre esos **mismos** 200 días. La media ya penaliza las soluciones que
   apuran demasiado $t_{max}$ (las rutas que llegan tarde aportan 0), así que la mejor en determinista
   puede quedar descartada aquí.
6. **Simulación intensiva, con sorteos nuevos** [usuario]: las **3** de mayor media rápida (empate:
   mayor recompensa determinista). `T_int = sample_scenarios(..., R=1000, rng_int)`, que son días
   **nuevos**, independientes de `T_fast`; las 3 se evalúan sobre esos mismos 1000 días (comunes entre
   las 3, no reutilizados de la fase rápida; §6.1).
7. **Resultado.** La de mayor media intensiva. Se guarda: recompensa determinista, media rápida, media
   y desviación intensivas, % de rutas que fallan, nº de candidatas distintas, si la ganadora es la
   greedy, rutas, tiempo de cómputo.

**F9. Matriz $\alpha\times c$** [usuario]: para cada instancia, F8 se ejecuta con
$\alpha\in\{0.25,0.5,0.75\}$ y $c\in\{0.05,0.25,0.75\}$ → **9 resultados por instancia**, todos se
guardan. Además, para cada (instancia, $c$), la **mejor de los 3 $\alpha$** (mayor media intensiva):
3 «mejores» por instancia, uno por nivel de $c$. **No se elige un mejor entre niveles de $c$**: cada $c$
es un problema distinto (más incertidumbre siempre da menos recompensa esperada).

**Semillas** [usuario: una sola semilla por caso; el esquema concreto es propuesta]: una semilla por
(instancia, $c$, $\alpha$), derivada de forma determinista de una semilla base fija (p. ej.
`numpy.random.SeedSequence([base, idx_instancia, idx_c, idx_alpha])` y `.spawn(3)` para los tres
generadores). `idx_instancia` es la posición de la instancia en la lista de nombres de fichero ordenada
alfabéticamente; `idx_c` e `idx_alpha`, la posición en las listas (0.05, 0.25, 0.75) y
(0.25, 0.5, 0.75). Así los resultados no dependen del orden de ejecución ni del paralelismo. No se
repiten ejecuciones con otras semillas: el algoritmo ya es aleatorio. La semilla base se documenta en
el README.

### 6.1 Por qué números aleatorios comunes en la rápida y nuevos en la intensiva

*(Esto hay que justificarlo en el artículo y la presentación [usuario].)*

**Comunes en la simulación rápida: comparar en igualdad de condiciones.** La simulación rápida sirve
para **ordenar** 7 candidatas con solo 200 réplicas. Si cada candidata tuviera sus propios 200
sorteos, a una le podrían tocar por azar «días con poco tráfico» y parecer mejor sin serlo. Con los
mismos 200 días para todas (*common random numbers*, CRN), la diferencia entre dos candidatas solo
puede venir de sus rutas, no de la suerte del sorteo. En términos estadísticos:
$\text{Var}(\bar R_A-\bar R_B)=\text{Var}(\bar R_A)+\text{Var}(\bar R_B)-2\,\text{Cov}(\bar R_A,\bar R_B)$, y con
días comunes la covarianza es positiva (las rutas comparten muchos arcos y, sobre todo, «días buenos y
malos»), así que la varianza de la diferencia baja y el orden es más fiable con las mismas 200 réplicas.
Es una técnica clásica de reducción de varianza; la Fig. 2 del paper (p. 1913) menciona
«variance-reduction methods» y el §4 (p. 1912) cita la reducción de varianza para el muestreo.

**Nuevos en la simulación intensiva: no premiar la suerte.** Las 3 finalistas se eligieron
precisamente porque les fue bien en `T_fast`. Si se volvieran a medir con esos mismos días, su media
heredaría ese golpe de suerte y la recompensa esperada saldría **inflada** (sesgo de selección, la
«maldición del ganador»). Con 1000 días nuevos, la estimación de cada finalista es **insesgada**.

**Comunes no es lo mismo que reutilizados.** Hay que distinguir dos cosas:
- *Comunes*: las candidatas que se comparan **en la misma fase** usan los mismos días. Es bueno, porque
  hace la comparación justa.
- *Reutilizados*: los días que sirvieron para **elegir** se vuelven a usar para **medir** a las elegidas.
  Es malo, porque infla la medida.

En la intensiva, las 3 finalistas comparten los 1000 días nuevos: son días comunes entre ellas, pero
no reutilizados de la fase rápida. **Los 1000 días no incluyen los 200 de la rápida**: se sortean desde
cero con otro generador (`rng_int`). Si los incluyeran, un 20 % de la medida final serían días que ya
favorecieron a las finalistas y volvería parte de la inflación. Es el mismo principio que en la rápida, aplicado a la final.

Queda un sesgo residual pequeño: la ganadora se elige como la mayor de 3 medias intensivas, y esa misma
media es la que se publica. El sesgo existe tanto si los días son comunes como si no; con días comunes
incluso se elige mejor, porque el orden entre las 3 es más fiable. Con 1000 réplicas y solo 3
candidatas (casi siempre muy parecidas) es despreciable. Se menciona en el artículo y no se corrige:
corregirlo exigiría una tercera simulación independiente de la ganadora.

**Ejemplo pequeño.** Candidatas A y B con recompensa esperada real 30 y 29. Con sorteos separados,
el error de cada media con 200 réplicas puede ser de ±1, y B «gana» con bastante frecuencia. Con días
comunes, cuando hay atasco lo hay para las dos, los errores se mueven juntos y la diferencia
(≈ 1) se ve mucho más clara.

Nota menor [deducción]: elegir «el mejor de 3 $\alpha$» con la misma media intensiva que luego se
publica introduce un sesgo de selección pequeño (1000 réplicas por estimación); se comenta en el
artículo y no se corrige.

## 7. Parámetros

| parámetro | valor | fuente | comentario |
|---|---|---|---|
| $c$ (nivel de varianza) | 0.05 / 0.25 / 0.75 | paper p. 1913 | niveles L / M / H |
| réplicas simulación rápida | 200 | paper p. 1912 | |
| réplicas simulación intensiva | 1 000 | paper p. 1912 | |
| $\alpha$ (peso del ahorro en la preferencia) | 0.25, 0.5 y 0.75 (las tres se prueban) | decisión del usuario | el paper no lo da. Se prueban tres porque $s_{ij}$ (distancia) y $u_i+u_j$ (recompensa) tienen escalas distintas y 0.5 no pesa «igual» |
| distribución BR | geométrica | decisión del usuario (paper p. 1912 cita Grasas et al. 2017) | |
| $\beta$ | 0.3 | decisión del usuario (igual que TOP / Panadero et al. 2021) | |
| iteraciones (construcción BR + búsqueda local) | 50 por ejecución | decisión del usuario | instancias pequeñas; el paper para por tiempo (Fig. 2: «Loop while time»), nosotros por iteraciones para que sea reproducible |
| soluciones a simulación rápida | 7 (las de mayor recompensa determinista) | decisión del usuario | |
| soluciones a simulación intensiva | 3 (las de mayor media rápida) | decisión del usuario | |
| tiempo objetivo por ejecución | unos pocos segundos | decisión del usuario | por eso no se simula todo |
| candidatas por ejecución | 1 greedy + 50 BR, todas con búsqueda local; sin duplicados | decisión del usuario | |
| ejecuciones / semillas por caso (instancia, $c$, $\alpha$) | 1 | decisión del usuario | el paper no lo dice |
| números aleatorios en la simulación | comunes entre candidatas en la rápida; nuevos (e independientes de la rápida) en la intensiva | decisión del usuario | justificación en §6.1 |
| escenarios SAA, límite SAA | 1 000; 100 000 s | paper pp. 1912, 1914 | solo informativo: el SAA no se implementa |

## 8. Datos e instancias

- Instancias: las 141 factibles de p1, p2 y p3 en `data/top-data/` (§2.1), con 3 niveles de $c$ y
  3 valores de $\alpha$ → 141 × 3 × 3 = **1 269 ejecuciones** (una semilla por caso).
- BKS determinista: copiar `TOP/data/bks_chao.csv` (columnas `instance,bks`) a `simTOP/data/` sin
  modificarlo, para el gap de la Fig. 3 [usuario]. Lo hace el orquestador / `code-writer`.

## 9. Experimentos a reproducir

Métrica decidida [usuario]: gap respecto a la BKS determinista, como en la Fig. 3.

El paper no da tablas, solo dos diagramas de caja:
- **Fig. 3 (p. 1914)**: «Gap (in %) w.r.t. best-known deterministic solution» de SAA y simheurística
  en los niveles L/M/H. Fórmula no dada; propuesta [deducción]: $\text{gap}=100\,(\text{BKS}-\bar R)/\text{BKS}$,
  con $\bar R$ la recompensa esperada (media intensiva) de la solución final. **Valores de referencia
  de la simheurística, leídos a ojo** (aproximados, ±1–2 puntos):

  | nivel | mediana | caja (Q1–Q3) | bigotes | atípicos |
  |---|---|---|---|---|
  | SimHeu-L | ≈ 7 | ≈ 5–10 | ≈ 1–13 | ≈ 26 |
  | SimHeu-M | ≈ 17 | ≈ 9–20 | ≈ 6–29 | — |
  | SimHeu-H | ≈ 24 | ≈ 19–28 | ≈ 12–31 | — |

  Y del SAA (solo para comentar): SAA-L ≈ 15, SAA-M ≈ 26, SAA-H ≈ 37 de mediana. El paper no dice
  sobre qué instancias está hecha la figura; como el SAA solo resolvió las de ≤ 35 nodos (p. 1914), lo
  razonable es que sean p1–p3 [deducción].
- **Fig. 4 (p. 1915)**: tiempos de cómputo. Simheurística: de pocos segundos a unos 400 s (máquina Xeon
  3.7 GHz, Java). Solo comparable cualitativamente.

**Lo que producimos** [usuario, salvo nombres de fichero, que son propuesta]:
1. `results/runs.csv`: una fila por (instancia, $c$, $\alpha$), 1 269 filas. Columnas mínimas:
   `instance, set, m, tmax, c, alpha, seed, det_reward, fast_mean, exp_reward, exp_std, fail_rate,
   n_distinct, winner_is_greedy, bks, gap_pct, time_s, routes`, con `exp_reward` = media intensiva y
   `gap_pct` = $100\,(\text{BKS}-\text{exp\_reward})/\text{BKS}$.
2. **Matriz $\alpha\times c$**: tabla 3 × 3 con el gap medio y mediano de cada combinación (los 9
   resultados), y diagramas de caja del gap por ($c$, $\alpha$).
3. **Mejor global**: para cada (instancia, $c$), el mejor $\alpha$ (`results/best.csv`). Diagrama de caja
   del gap por nivel L/M/H **comparado con las medianas y cajas de SimHeu-L/M/H de la Fig. 3**.
4. Tiempo por ejecución (resumen por conjunto y nivel), para comentar frente a la Fig. 4.
5. Por la decisión de §11 sobre la varianza: el gap también **por conjunto (p1, p2, p3) o por $t_{max}$**.
6. Descomposición del gap (§11, artículo punto 8): gap determinista de la ganadora,
   $100\,(\text{BKS}-\text{det\_reward})/\text{BKS}$, y pérdida por incertidumbre,
   $100\,(1-\text{exp\_reward}/\text{det\_reward})$. Se calculan desde `runs.csv`.

## 10. Guía de implementación

Lenguaje y entorno: los decide el orquestador según `CLAUDE.md` (por defecto Python con uv) [usuario].
La PJS, la BR geométrica y la búsqueda local están en Rust en `TOP/src/rust/src/` (`heuristic.rs`,
`grasp.rs`, `local_search.rs`) y sirven de referencia línea a línea.

Orden sugerido:
1. Lectura de instancias y matriz de tiempos (F1).
2. Generador de tiempos (F2 + muestreo).
3. PJS determinista (F3+F4) y búsqueda local (F6).
4. Selector BR (F5), escenarios y simulación (F7a, F7b) y bucle simheurístico (F8).
5. Si una ejecución tarda más de unos pocos segundos, optimizar la simulación (vectorizar sobre réplicas
   con numpy o usar Numba) **sin cambiar** ningún parámetro.
6. Experimento (F9): 141 instancias × 3 niveles de $c$ × 3 valores de $\alpha$ = 1 269 ejecuciones;
   salidas de §9 en `results/`.

**Comprobaciones:**
- Lognormal: con $10^6$ muestras de un arco con $t=10$, media muestral ≈ 10 y varianza ≈ $10c$
  (tabla de §2.2).
- PJS determinista con $\alpha=0.5$: debe dar **la misma recompensa** que `TOP` con $\alpha=0.5$
  en las 141 instancias: columna `reward` de las filas con `alpha` = 0.50 de `TOP/results/alpha_grid.csv`
  (la PJS determinista sin búsqueda local).
- Búsqueda local: 2-opt deshace un cruce en una ruta de 4 clientes dibujada a mano; ninguna solución
  tras la búsqueda local tiene menos recompensa ni pasa de $t_{max}$ (tiempos esperados).
- Simulación: con $c\to 0$ (p. ej. $10^{-9}$) la media simulada coincide con la recompensa
  determinista; con $c$ mayor la media nunca la supera.
- Una ruta cuyo tiempo esperado es $t_{max}$ debe fallar en torno al 45–50 % de las réplicas.
- Monotonía esperada: gap(L) < gap(M) < gap(H) en media.
- Números aleatorios comunes: dos soluciones idénticas evaluadas con el mismo `T_fast` dan exactamente
  la misma media; una solución cuyas rutas son un subconjunto de las de otra nunca tiene más media con
  el mismo `T_fast`.
- `T_int` es distinto de `T_fast` (generadores distintos) y simétrico: `T[r,i,j] == T[r,j,i]`.
- Reproducibilidad: dos ejecuciones con la misma semilla dan el mismo CSV.
- Greedy extra: la recompensa determinista de la mejor candidata nunca es menor que la de PJS greedy +
  búsqueda local.
- Tras quitar duplicados no quedan dos candidatas con la misma forma canónica.

## 11. Decisiones y dudas

| Tema | Decisión | Quién |
|---|---|---|
| Orden del documento: la generación de instancias va primero | Sí | usuario |
| Alcance: simheurística (PJS + BR + búsqueda local + Monte Carlo); el SAA queda fuera | Sí | usuario |
| Instancias: solo p1–p3 (< 35 nodos), con los 3 niveles de $c$ | Sí | usuario |
| Escenarios generados al vuelo con semilla fija, sin ficheros | Sí | usuario |
| Var = c·t literal; arcos independientes; T = 0 si t = 0 | Sí | usuario |
| $\alpha\in\{0.25, 0.5, 0.75\}$, las tres se prueban | Sí | usuario |
| BR geométrica, $\beta=0.3$ | Sí | usuario |
| Parada: 50 iteraciones por ejecución, no por tiempo (instancias pequeñas) | Sí | usuario |
| A simulación rápida solo las 7 de mayor recompensa determinista (no se simula todo: cada ejecución debe durar pocos segundos) | Sí | usuario |
| A simulación intensiva las 3 de mayor media rápida; gana la de mayor media intensiva | Sí | usuario |
| Una solución que apura $t_{max}$ queda penalizada en la simulación (rutas tardías = 0) y puede quedar descartada | Sí | usuario |
| Métrica: gap respecto a la BKS determinista; copiar `bks_chao.csv` de TOP | Sí | usuario |
| Búsqueda local = la de TOP (bucle 2-opt + inserción mientras se libere tiempo), sin destrucción aparte | Sí | usuario |
| Una semilla por caso; no se repiten ejecuciones | Sí | usuario |
| La PJS greedy (+ búsqueda local) entra como candidata extra junto a las 50 BR | Sí | usuario |
| Números aleatorios comunes en la simulación rápida; sorteos nuevos en la intensiva (§6.1) | Sí | usuario |
| Quitar soluciones repetidas; si quedan menos de 7, se simulan las que haya | Sí | usuario |
| 9 resultados por instancia (matriz $\alpha\times c$) y el mejor $\alpha$ para cada $c$ | Sí | usuario |
| Las 3 finalistas comparten los 1000 días nuevos de la intensiva (comunes entre ellas, no reutilizados de la rápida; sesgo residual despreciable, §6.1) | Sí | propuesta aceptada por el usuario |
| Esquema de semillas con `SeedSequence`; desempates de F8 | Sí | propuesta aceptada por el usuario |
| Esquema 7 → 3 (el «5» de un mensaje anterior fue una errata) | Sí | usuario |
| Sin margen de seguridad sobre $t_{max}$ (fiel al paper); los fallos cerca de $t_{max}$ se muestran en los resultados; el margen queda como trabajo futuro | Sí | usuario |
| Lenguaje y entorno | Los decide el orquestador según `CLAUDE.md` | usuario |
| Distancia euclídea sin redondear | Propuesta | deducción |

**Abierto:** nada que bloquee la implementación. Las dudas que solo resolverían los autores están en
`preguntas_abiertas.md`.

### Cosas que chirrían (revisión final)

**Del paper** (para leerlo con cuidado y, si procede, comentarlo en el artículo):
1. **El texto de la Fig. 3 no cuadra con la figura** (p. 1914): dice que la simheurística da
   «solutions with a lower expected time and a lower variability», pero el eje mide el gap de la
   recompensa respecto a la BKS. Nos guiamos por el eje.
2. **No dice qué instancias entran en cada caja** ni da la fórmula del gap; tampoco hay tablas por
   instancia. La comparación con el paper solo puede ser de medianas y cajas leídas a ojo.
3. **Notación de las ecuaciones descuidada** (p. 1911): la ec. 1 suma $E[U_j]$ sobre arcos y vehículos,
   cuando $U_j$ es por cliente, y la ec. 3 se presenta como restricción con tiempos esperados que luego
   es «blanda». No nos afecta: implementamos la definición en palabras (§2.4).
4. **La simheurística se describe en un solo párrafo** (p. 1912), sin parámetros. Todos nuestros valores
   son decisiones propias (lista de arriba, punto 4).
5. **Var = c·t no es invariante a la escala** (§2.3): si las coordenadas se midieran en otras unidades,
   el mismo $c$ daría otra incertidumbre. Es un efecto del modelo del paper, no un fallo nuestro.
6. **Los tiempos de la Fig. 4 no son comparables** (Java en un Xeon a 3.7 GHz, parada por tiempo, frente
   a nuestra parada por iteraciones).

**De nuestro diseño** (riesgo conocido; no se cambia nada sin que lo decida el usuario):
7. **Todo empuja hacia rutas al límite de $t_{max}$.** La PJS fusiona mientras el tiempo esperado
   quepa; la búsqueda local inserta clientes hasta que no cabe ninguno (rellena la holgura que el 2-opt
   libera), y el filtro determinista se queda con las 7 de **más recompensa**, que suelen ser las más
   apuradas. Ya hemos visto que una ruta con tiempo esperado igual a $t_{max}$ falla casi la mitad de
   las veces (§2.3). La simulación solo elige entre candidatas que pueden ser todas arriesgadas. Es
   fiel al paper (que construye con $E[T]\le t_{max}$, p. 1912) y probablemente explica gaps como los de
   la Fig. 3, pero con $c$ alto puede dar gaps grandes. Si ocurre, es un resultado que hay que explicar
   en el artículo, no un error de código. **Decisión del usuario: se deja tal cual.** Es el
   comportamiento esperado y los resultados deben mostrarlo (por ejemplo, con el % de rutas que fallan
   y la pérdida por incertidumbre por nivel de $c$). El margen de seguridad queda como trabajo futuro
   (ver abajo).
8. Instancias con $t_{max}$ muy pequeño y $c=0.75$: con arcos de longitud ~1, el coeficiente de
   variación de cada arco ronda el 87 %, y casi cualquier ruta puede fallar. Cabe esperar gaps muy
   altos (atípicos) en esas instancias.

### Trabajo futuro (no se implementa ahora) [usuario]

**Margen de seguridad sobre $t_{max}$.** Construir y mejorar las rutas con un límite reducido
$t'_{max}=t_{max}-\delta$ (o $k\cdot t_{max}$ con $k<1$), de modo que las rutas tengan holgura frente a la
variabilidad, y seguir evaluándolas con el $t_{max}$ real en la simulación. La pregunta abierta es **cómo
calcular ese margen de la mejor forma**. Ideas para el apartado de trabajo futuro del artículo y la
presentación:
- Margen fijo probado en una rejilla (p. ej. $k\in\{0.80,0.85,\dots,1\}$), con la simulación eligiendo el
  mejor por instancia y nivel de $c$.
- Margen calculado a partir de la propia varianza: como $\text{Var}_r=c\,E_r$, exigir
  $E_r+z\sqrt{c\,E_r}\le t_{max}$ (probabilidad de fallo de cada ruta acotada, con $z$ el cuantil
  deseado). Esto ajusta el margen a cada ruta y a cada $c$ en lugar de usar uno fijo.
- Margen adaptativo dentro de la búsqueda: ajustar $\delta$ según el % de fallos observado en la
  simulación rápida.

Este punto debe aparecer en el artículo y la presentación como **mejora propuesta**, apoyada en los
resultados (% de rutas fallidas y pérdida por incertidumbre al crecer $c$).

### Decisiones que deben aparecer en nuestro artículo y en la presentación [usuario]

1. **Varianza $c\cdot t_{ij}$ (del paper) y lo que implica.** Una ruta de tiempo esperado $E_r$ tiene
   desviación relativa $\sqrt{c/E_r}$, así que **el mismo $c$ pesa mucho más en las instancias con
   $t_{max}$ pequeño** que en las de $t_{max}$ grande. El paper no lo comenta (p. 1913 solo da la
   fórmula); conviene analizar los resultados por conjunto o por $t_{max}$ y no solo agregados como en
   la Fig. 3.
2. **Arcos independientes**: supuesto nuestro; el paper no lo menciona.
3. (Menor) El arco origen–destino de longitud 0 solo existe en p7: no afecta al problema.
4. $\alpha\in\{0.25,0.5,0.75\}$, 50 iteraciones, 7 a simulación rápida / 3 a intensiva y parada por
   iteraciones en vez de tiempo: valores nuestros, el paper no los da.
5. Búsqueda local de TOP (2-opt + inserción) en lugar de «destrucción-reconstrucción + 2-opt» del paper.
6. **Números aleatorios comunes en la simulación rápida y sorteos nuevos en la intensiva**, con la
   justificación de §6.1: comparar en igualdad de condiciones y evitar el sesgo de selección. La idea
   clave es que *comunes* no es lo mismo que *reutilizados*.
7. La PJS greedy como candidata extra y una sola semilla por caso. Ojo con cómo se dice: la greedy
   garantiza que **la mejor candidata en determinista** no es peor que PJS + búsqueda local, pero **no**
   que la solución final lo sea en recompensa esperada. La final se elige por simulación, y la greedy
   puede quedarse fuera de las 7 si hay 7 soluciones BR con más recompensa determinista.
8. **El gap respecto a la BKS determinista mezcla dos cosas**: lo que pierde la heurística (en
   determinista, el GRASP + búsqueda local de TOP, con el mejor α, queda a 0–0.7 % de la BKS en p1–p3
   según `TOP/README.md`; con nuestras 50 iteraciones y α fijo será algo más) y el
   **precio de la incertidumbre**, que ninguna solución puede evitar (incluso la óptima estocástica
   tiene gap > 0). Conviene separarlos: gap determinista de la ganadora frente a pérdida por
   simulación ($1-\bar R/\text{det\_reward}$). Ambas cosas salen de `runs.csv`.
9. **Rutas al límite de $t_{max}$ (§11, «Cosas que chirrían», punto 7)**: construcción, búsqueda local
   y filtro determinista empujan hacia rutas que apuran $t_{max}$. Es el comportamiento esperado y los
   resultados deben mostrarlo. Como **trabajo futuro** se propone un margen de seguridad y una forma
   óptima de calcularlo (§11, «Trabajo futuro»).
