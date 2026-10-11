# Búsqueda local para GRASP-PJS en el TOP

Notas de diseño para la fase de búsqueda local de GRASP, el paso pendiente de
[`scratchpad.md`](scratchpad.md) (sección 8). Recoge cada movimiento candidato: qué hace,
cómo se evalúa, cuánto cuesta, qué ventajas e inconvenientes tiene y cómo encaja con el
código actual (`TOP/src/rust`). Todavía no hay nada implementado.

---

## 0. Punto de partida

### Lo que dicen los papers

- **Ninguno de los dos papers tiene búsqueda local.** Panadero et al. (2021) y Li et al.
  (2022) usan la heurística de ahorros con aleatorización sesgada dentro de un
  multiarranque, y se quedan con la mejor solución. Es lo que tenemos ahora.
- **Por tanto, la búsqueda local es aportación nuestra.** Los movimientos de este documento
  son los habituales en la literatura del TOP y del VRP. La referencia clásica para el TOP
  es la *Guided Local Search* de Vansteenwegen et al. (ref. 45 de Panadero), con los
  movimientos *2-opt*, *swap*, *move*, *insert* y *replace*.

### Representación en nuestro código

- **Nodos:** el origen es el nodo `0` y el destino `inst.sink()`, el nodo `n − 1`. Los
  clientes son `inst.customers()`.
- **Rutas:** `Route.nodes` guarda **solo los clientes**. La ruta completa es
  `0 → nodes[0] → … → nodes[k−1] → sink`, y `inst.route_time(&nodes)` le añade los dos
  extremos.
- **Soluciones:** `Solution` tiene como mucho `m` rutas, con `reward` y `time` totales.
  `Solution::check()` valida la factibilidad de forma independiente.
- **Distancias:** son **euclídeas**, así que se cumplen dos cosas:
  - son simétricas, t(i,j) = t(j,i), y por eso invertir un tramo no cambia su tiempo
    interno;
  - cumplen la desigualdad triangular, así que insertar un cliente nunca acorta una ruta
    (Δ ≥ 0) y quitarlo nunca la alarga.
- **Tamaño de las instancias** (nodos totales; `m` entre 2 y 4):

  | Conjunto | p1 | p2 | p3 | p4 | p5 | p6 | p7 |
  |---|---|---|---|---|---|---|---|
  | Nodos | 32 | 21 | 33 | 100 | 66 | 64 | 102 |

  Son instancias pequeñas, así que se pueden permitir vecindarios cuadráticos e incluso
  cúbicos.

### Notación

| Símbolo | Significado |
|---|---|
| t(i,j) | Tiempo (distancia) entre i y j |
| uᵢ | Recompensa del cliente i |
| T_r | Tiempo de la ruta r |
| h_r = tmax − T_r | Holgura de la ruta r |
| V, U | Clientes visitados y no visitados (solo los alcanzables) |
| (p, i, q) | i y sus vecinos en la ruta: p es el anterior (o 0) y q el siguiente (o sink) |

---

## 1. Qué significa "mejorar" en el TOP

### El objetivo es la recompensa

En el VRP, la búsqueda local acorta rutas porque el objetivo es la distancia. En el TOP
**el objetivo es la recompensa total**, y el tiempo solo es un recurso limitado por `tmax`.
Por eso los movimientos se dividen en dos familias:

| Familia | Movimientos | Efecto en la recompensa | Efecto en el tiempo |
|---|---|---|---|
| **Selección** (qué clientes se visitan) | inserción, reemplazo, destruir y reconstruir | Sube | Suele subir |
| **Enrutamiento** (en qué orden y en qué ruta) | 2-opt, or-opt, relocate, swap, 2-opt* | No cambia | Baja o se redistribuye |

Los movimientos de enrutamiento **no mejoran el objetivo por sí solos**. Su papel es liberar
tiempo para que después quepa una inserción o un reemplazo.

### Criterio de aceptación lexicográfico

Una solución S' es mejor que S si:

1. tiene más recompensa: R(S') > R(S) + ε; o
2. tiene la misma recompensa y menos tiempo: |R(S') − R(S)| ≤ ε y T(S') < T(S) − ε.

Detalles:

- Sin el segundo criterio, el 2-opt nunca se aplicaría, porque nunca sube la recompensa.
- Sirve el mismo ε = 1e-9 que ya se usa en la comprobación de `tmax`. Hace falta exigir una
  mejora **estricta mayor que ε**; si no, el redondeo puede hacer que dos movimientos se
  deshagan mutuamente sin fin.
- **Fuera de la búsqueda local nada cambia.** `grasp()` sigue comparando solo por
  recompensa, como ahora. El desempate por tiempo es interno de la búsqueda local.

### Un criterio alternativo: concentrar la holgura

Menos tiempo total no siempre es lo que más ayuda. Si dos rutas tienen 3 unidades de
holgura cada una y el cliente que falta necesita 5, ninguna de las dos lo admite; con 6 en
una ruta y 0 en la otra, sí cabe.

Un criterio secundario que premia **concentrar** la holgura es maximizar Σ h_r². Lo
aprovechan los movimientos entre rutas (sección 6). Es una idea a probar, no un estándar.

---

## 2. 2-opt dentro de una ruta

### Qué hace

Quita dos arcos de una ruta y la reconecta invirtiendo el tramo de en medio:

```
antes:   0 → … → a → [b → … → c] → d → … → sink
después: 0 → … → a → [c → … → b] → d → … → sink
```

Elimina los cruces. En el plano euclídeo, una ruta sin cruces es siempre más corta que la
misma ruta con ellos.

### Cómo se evalúa

Sean i ≤ j posiciones de `nodes`, el tramo a invertir `nodes[i..=j]`, `a` el nodo anterior
a `nodes[i]` (o 0 si i = 0) y `d` el siguiente a `nodes[j]` (o sink si j es el último):

  Δ = t(a, nodes[j]) + t(nodes[i], d) − t(a, nodes[i]) − t(nodes[j], d)

- Es O(1) gracias a la simetría: el tramo invertido cuesta lo mismo en un sentido que en el
  otro.
- Se aplica si Δ < −ε.
- **Origen y destino distintos no estorban.** Los extremos `0` y `sink` no se mueven nunca;
  solo se invierte el interior. En p7, donde coinciden, funciona igual.

### Coste

- O(k²) pares por ruta, con k clientes en la ruta. Con k ≤ 50 son menos de 1 300
  evaluaciones.
- Aplicar un movimiento cuesta O(k): `nodes[i..=j].reverse()`.
- Se repite hasta que no haya mejora. En la práctica bastan unas pocas pasadas.

### Por qué encaja tan bien con PJS

PJS une rutas **concatenándolas**: el final de la ruta que acaba en i con el principio de la
que empieza en j. Nunca reordena lo que ya está dentro, así que una ruta formada por muchas
fusiones suele tener zigzags y cruces. El 2-opt debería encontrar mejoras en casi todas las
soluciones construidas.

### Pros

- Es el movimiento más barato y más sencillo de implementar.
- Es muy conocido y fácil de justificar en un informe.
- Corrige justo el defecto típico de la construcción por fusiones.
- Mantiene las rutas sin cruces, y eso también hace más legibles las figuras.

### Contras

- **No sube la recompensa.** Si no va seguido de una inserción, la solución vale lo mismo.
- Solo actúa dentro de una ruta y no ve mejoras entre rutas.
- Los 2-óptimos son óptimos locales débiles: una ruta 2-óptima puede estar bastante lejos
  del TSP óptimo de sus clientes.

---

## 3. Or-opt dentro de una ruta

### Qué hace

Saca un tramo de 1, 2 o 3 clientes consecutivos y lo vuelve a insertar en otra posición de
la misma ruta, en su sentido original o invertido.

```
antes:   0 → a → [s … e] → b → … → x → y → sink
después: 0 → a → b → … → x → [s … e] → y → sink
```

### Cómo se evalúa

- Ahorro al sacar el tramo de entre p y q: g = t(p,s) + t(e,q) − t(p,q).
- Coste al meterlo entre x e y: c = t(x,s) + t(e,y) − t(x,y), o
  t(x,e) + t(s,y) − t(x,y) si va invertido.
- Δ = c − g, en O(1). Hay que tener cuidado cuando x o y son p o q, porque las posiciones
  se solapan con el tramo extraído.

### Coste

O(3·k²) por ruta, del mismo orden que el 2-opt.

### Pros

- Encuentra mejoras que el 2-opt no ve. El 2-opt invierte; el or-opt traslada.
- Juntos, 2-opt y or-opt dejan las rutas cortas de verdad, y con ello más holgura.

### Contras

- Hay más casos especiales (solapamientos, tramos en los extremos) y es fácil equivocarse.
- Tampoco sube la recompensa.
- Aporta menos que el 2-opt. Es un complemento opcional, no imprescindible.

---

## 4. Insertar clientes no visitados

### Qué hace

Mete un cliente k que no está en ninguna ruta en la posición más barata de alguna ruta en
la que quepa:

```
antes:   … → a → b → …          (k sin visitar)
después: … → a → k → b → …
```

### Cómo se evalúa

- Coste de meter k entre a y b: Δ(k,a,b) = t(a,k) + t(k,b) − t(a,b), con Δ ≥ 0 por la
  desigualdad triangular.
- Es factible si T_r + Δ ≤ tmax + ε.
- La recompensa sube en u_k. Es el único movimiento que **siempre** sube el objetivo sin
  quitar nada a cambio.

### Por qué hay clientes para insertar

La construcción de PJS deja muchos clientes no visitados en dos situaciones:

1. **Al quedarse con las m mejores rutas.** Todos los clientes de las rutas descartadas
   quedan libres, y muchos pueden caber en la holgura de las rutas seleccionadas.
2. **Por la lista de ahorros.** Una fusión que no cabía en un momento podría haber cabido
   en otro orden.

Además, si la solución tiene **menos de m rutas**, se puede abrir una ruta nueva
0 → k → sink para cualquier k alcanzable. Pasa en instancias con `tmax` muy pequeño.

### En qué orden insertar

Se pueden insertar varios clientes, y el orden importa: cada inserción gasta holgura que
otra podría necesitar. Hay tres opciones:

| Regla | Qué elige | Comentario |
|---|---|---|
| **Máxima recompensa** | El k de mayor u_k que quepa | Es simple, pero puede gastar mucha holgura en un cliente alejado |
| **Máxima eficiencia** | El k con mayor u_k / Δ_k (Δ_k: su mejor coste de inserción) | Suele dar más recompensa total. Si Δ_k = 0 (k está en línea con a y b), se toma como infinito |
| **Primera que quepa** | El primer k que se encuentra | Es la más barata, pero la peor en calidad |

Esquema con la regla de eficiencia (mejor mejora):

```
repetir:
    para cada k en U:
        Δ_k, posición_k = mejor inserción factible de k (en cualquier ruta)
    si no hay ningún k factible: parar
    elegir el k con mayor u_k / Δ_k e insertarlo
    actualizar U y los tiempos de la ruta afectada
```

Optimización posible: después de insertar en la ruta r, solo cambian las inserciones en r.
Se puede guardar la mejor inserción de cada k en cada ruta y recalcular solo la columna de
r.

### Coste

- Una pasada es O(|U|·N), con N el total de posiciones: unas 100·100 = 10⁴ evaluaciones en
  las instancias grandes.
- Hay como mucho |U| inserciones, así que el peor caso es O(|U|²·N), sin la caché. Es
  asumible con n ≤ 102.

### Pros

- **Es el movimiento que más recompensa aporta por unidad de esfuerzo.**
- Es sencillo y barato.
- Recupera los clientes que la selección de las m rutas tira.
- Se complementa de forma natural con el 2-opt: el 2-opt libera tiempo y la inserción lo
  gasta.

### Contras

- **Solo funciona si hay holgura.** En instancias con `tmax` ajustado (p. ej. las de letra
  alta, como p4.4.t) las rutas están llenas y no cabe nada.
- Es miope: insertar el cliente de mayor eficiencia puede impedir dos inserciones que
  juntas valían más.
- No saca nunca a nadie, así que no corrige malas decisiones de la construcción. Para eso
  está el reemplazo.

---

## 5. Reemplazo: un visitado por un no visitado

### Qué hace

Cambia un cliente visitado de poco valor por uno no visitado de más valor, si el resultado
sigue cabiendo en `tmax`:

```
antes:   … → p → i → q → …      (k sin visitar, u_k > u_i)
después: … → p → q → … → x → k → y → …
```

k puede ir en la misma ruta que tenía i o en otra.

### Cómo se evalúa

1. **Quitar i** de la ruta r, entre p y q: la ruta pasa a durar
   T_r − g_i, con g_i = t(p,i) + t(i,q) − t(p,q) ≥ 0.
2. **Insertar k** en la mejor posición de cualquier ruta r':
   - si r' ≠ r, la condición es T_{r'} + Δ(k) ≤ tmax;
   - si r' = r, la condición es T_r − g_i + Δ(k) ≤ tmax, y las posiciones se calculan sobre
     la ruta **sin** i. El arco nuevo (p,q) también es candidato.
3. Se acepta si u_k > u_i + ε (sube la recompensa), o si u_k = u_i y el tiempo baja.

### Variantes

| Variante | Movimiento | Para qué sirve |
|---|---|---|
| 1 ↔ 1 | Quitar i, meter k | La básica |
| 1 → 2 | Quitar i, meter k₁ y k₂ | Cuando un cliente lejano ocupa el sitio de dos cercanos |
| 2 → 1 | Quitar i₁ e i₂, meter k | Cuando hay un cliente muy valioso pero caro de alcanzar |
| Quitar e insertar | Quitar i y después hacer inserciones (sección 4) mientras quepan | Generaliza las anteriores de forma barata |

La última es una forma práctica de implementarlo: para cada i visitado, quitarlo, lanzar la
inserción y aceptar si la recompensa total ha subido. Si no, deshacer.

### Coste

- **Es el movimiento más caro del núcleo.** El 1 ↔ 1 completo es O(|V|·|U|·N): con
  50 × 50 × 100 sale del orden de 2,5·10⁵ evaluaciones por pasada.
- Formas de abaratarlo:
  - probar solo los k con u_k > u_i, porque si no, no hay mejora;
  - limitar los k a los más cercanos a i o a las rutas (lista de candidatos);
  - usar primera mejora;
  - guardar la mejor inserción de cada k por ruta, como en la sección 4.

### Pros

- **Sirve justo cuando la inserción ya no puede**: con las rutas llenas sigue cambiando
  clientes malos por buenos.
- Corrige decisiones de la construcción, porque saca clientes que PJS metió por su ahorro
  de distancia aunque valían poco.
- En la literatura del TOP es, después de la inserción, el movimiento que más aporta.

### Contras

- Es caro. Sin limitar candidatos puede dominar el tiempo de toda la búsqueda local.
- La evaluación en la misma ruta tiene casos delicados (posiciones adyacentes a i).
- Con pocas recompensas distintas, el 1 ↔ 1 rara vez mejora, porque muchos pares empatan en
  u. Pasa en p1 (3 valores distintos entre los clientes) y en p5 (4). Ahí hay que ir al
  2 → 1 y al 1 → 2, que son aún más caros.

---

## 6. Movimientos entre rutas

No cambian qué clientes se visitan, solo en qué ruta van. Su objetivo es **redistribuir la
holgura** para habilitar inserciones y reemplazos.

### 6.1 Relocate: mover un cliente a otra ruta

```
ruta A:  … → p → i → q → …        →   … → p → q → …
ruta B:  … → x → y → …            →   … → x → i → y → …
```

- Δ_A = −g_i (A se acorta) y Δ_B = Δ(i,x,y) (B se alarga). Factible si
  T_B + Δ_B ≤ tmax.
- Coste: O(k_A · k_B) por par de rutas. Con m ≤ 4 hay como mucho 12 pares ordenados.

### 6.2 Swap: intercambiar un cliente de A con uno de B

```
ruta A:  … → p → i → q → …        →   … → p → j → q → …
ruta B:  … → x → j → y → …        →   … → x → i → y → …
```

- Δ_A = t(p,j) + t(j,q) − t(p,i) − t(i,q), y Δ_B de forma análoga.
- Factible si las dos rutas resultantes caben en `tmax`.
- Coste: O(k_A · k_B) por par.

### 6.3 2-opt*: intercambiar los finales de dos rutas

```
ruta A:  0 → a₁ … aᵢ | aᵢ₊₁ … → sink
ruta B:  0 → b₁ … bⱼ | bⱼ₊₁ … → sink

A':      0 → a₁ … aᵢ → bⱼ₊₁ … → sink
B':      0 → b₁ … bⱼ → aᵢ₊₁ … → sink
```

- Es válido siempre en nuestro problema, porque todas las rutas comparten origen y
  destino. Cambia como mucho 2 arcos y se evalúa en O(1) por pareja de cortes.
- Los tiempos de los tramos se calculan con sumas parciales (prefijos de tiempo de cada
  ruta).

### Con qué criterio aceptarlos

Aquí está la dificultad. Como la recompensa no cambia, hace falta un criterio auxiliar:

| Criterio | Pros | Contras |
|---|---|---|
| **Menos tiempo total** (lexicográfico, sección 1) | Simple y estándar; nunca cicla | No siempre habilita inserciones, porque a veces conviene un movimiento que no acorta |
| **Más Σ h_r²** (concentrar holgura) | Apunta justo a lo que necesitan las inserciones | No es estándar, hay que justificarlo, y puede alargar el tiempo total |
| **Solo si permite una inserción inmediata** | Solo aplica movimientos útiles | Cada evaluación lleva una inserción dentro y es mucho más caro |

### Pros

- Escapan de óptimos locales en los que 2-opt, inserción y reemplazo se atascan.
- Aprovechan la holgura repartida entre rutas.
- Diversifican la estructura de las soluciones.

### Contras

- **No suben la recompensa directamente.** Su beneficio es indirecto y difícil de medir.
- Elegir el criterio de aceptación no es obvio, y un mal criterio puede ciclar o no servir
  de nada.
- Son los más laboriosos de implementar y de probar.
- Con m pequeño (2 a 4 rutas) y 10 iteraciones por ejecución, probablemente aporten poco
  en comparación con su coste.

---

## 7. Destruir y reconstruir (perturbación)

### Qué hace

Quita al azar un porcentaje de los clientes visitados (entre el 10 y el 30 %, por ejemplo)
y los vuelve a insertar con la inserción de la sección 4, junto con el resto de no
visitados. Si la solución resultante, tras la búsqueda local, es mejor, se acepta.

### Por qué es distinto

**No es búsqueda local, sino una perturbación.** Repetido en bucle, convierte el método en
un **ILS** (*Iterated Local Search*) o, con varios operadores de destrucción, en un
**LNS/ALNS**. Precisamente el grupo de Juan y Panadero publica mucho con
*biased-randomized ILS* (refs. 23 y 57 de Panadero).

### Pros

- Es muy potente para salir de óptimos locales profundos.
- Reutiliza la inserción, así que añade poco código nuevo.

### Contras

- Cambia de metaheurística. El enunciado habla de GRASP, y GRASP + ILS es otra cosa.
- Añade parámetros (porcentaje, criterio de aceptación, número de perturbaciones) que
  habría que calibrar y justificar.
- Para la práctica, mejor dejarlo como trabajo futuro o como extensión opcional.

---

## 8. Cómo combinar los movimientos: VND

### El esquema

El *Variable Neighborhood Descent* recorre los vecindarios en orden y vuelve al primero
cada vez que alguno mejora:

```
fn local_search(sol):
    vecindarios = [2-opt, inserción, reemplazo, (or-opt), (entre rutas)]
    i = 0
    mientras i < len(vecindarios):
        si vecindarios[i] mejora sol:
            i = 0          // volver al primero
        si no:
            i += 1
    devolver sol
```

### Por qué este orden

1. **2-opt** va primero: es el más barato y libera tiempo.
2. **Inserción** gasta ese tiempo en recompensa.
3. **Reemplazo** entra cuando ya no cabe nada. Es más caro, así que solo se ejecuta cuando
   los anteriores no encuentran nada.
4. **Or-opt y entre rutas** (si se implementan) al final, como último recurso.

Con este orden, la mayoría de las vueltas se resuelven con los movimientos baratos.

### Termina siempre

Cada mejora aceptada sube la recompensa o, con la misma recompensa, baja el tiempo, en
ambos casos más que ε. La recompensa está acotada y los tiempos posibles son un conjunto
finito, así que no puede haber ciclos.

---

## 9. Primera mejora o mejor mejora

| | Primera mejora (*first improvement*) | Mejor mejora (*best improvement*) |
|---|---|---|
| Qué hace | Aplica el primer movimiento que mejora y sigue | Evalúa todo el vecindario y aplica el mejor |
| Coste de un paso | Bajo: a menudo se para pronto | Alto: siempre evalúa todo |
| Número de pasos | Más pasos pequeños | Menos pasos, más grandes |
| Calidad final | Similar en promedio; depende del orden de exploración | Similar; algo más estable |
| Determinismo | Depende del orden de recorrido | No depende del orden, salvo empates |

Propuesta por movimiento:

- **2-opt:** primera mejora. Es lo habitual y converge rápido.
- **Inserción:** **mejor mejora** por eficiencia (u_k/Δ_k). Aquí el orden importa mucho,
  porque cada inserción gasta holgura.
- **Reemplazo:** primera mejora, para no evaluar todo el vecindario cúbico en cada paso.

En GRASP, la diversidad ya la aporta la construcción aleatoria, así que la primera mejora
no pierde nada importante y ahorra tiempo.

---

## 10. Cuándo aplicar la búsqueda local dentro de GRASP

Así está ahora `grasp()` (`grasp.rs`):

```
best = construir(determinista)
repetir iteraciones:
    sol = construir(aleatorio)
    si sol.reward > best.reward: best = sol
```

| Opción | Esquema | Pros | Contras |
|---|---|---|---|
| **A. En cada iteración** | `sol = LS(construir())` | Es el GRASP clásico (Feo y Resende). Una construcción mediocre puede ser la mejor después de la búsqueda local | Es lo más caro: multiplica el coste de cada iteración |
| **B. Solo a la mejor final** | `best = LS(best)` al terminar | Casi gratis | Desperdicia el potencial: la mejor antes de la búsqueda local no tiene por qué ser la mejor después |
| **C. Solo a las prometedoras** | `si sol.reward ≥ (1−δ)·best.reward: sol = LS(sol)` | Es un buen compromiso entre coste y calidad | Añade un parámetro δ que hay que justificar |

Detalles en cualquiera de las tres:

- **La solución determinista inicial también debería pasar por la búsqueda local.** Si no,
  se comparan construcciones mejoradas con una sin mejorar.
- `GraspResult` debería guardar también la recompensa de la mejor solución **antes** de la
  búsqueda local. Así se puede medir cuánto aporta cada fase.

---

## 11. Coste esperado

- **Ahora:** la rejilla completa son 170 667 tareas en unos 140 s con 12 hilos. Cada tarea
  es una construcción determinista o una ejecución de 10 iteraciones de GRASP.
- **El cuello de botella actual es la construcción:** generar y ordenar unos n² arcos, del
  orden de 10⁴ en las instancias grandes.
- **La búsqueda local núcleo** (2-opt + inserción + reemplazo con candidatos) sobre n ≤ 102
  debería costar algo del mismo orden que una construcción, quizá varias veces más si el
  reemplazo no se limita. **Hay que medirlo**; es una estimación, no un dato.
- **Si la rejilla con búsqueda local en cada iteración resulta demasiado lenta**, hay tres
  salidas:
  - aplicar la búsqueda local solo a las prometedoras (opción C);
  - restringir la rejilla con búsqueda local a menos α;
  - compartir la lista de ahorros entre las ejecuciones de cada (instancia, α), que es la
    optimización pendiente 6 del scratchpad.

---

## 12. Cómo verificarla

Siguiendo lo que ya se hace con la construcción:

1. **`Solution::check()` después de cada búsqueda local.** Comprueba que haya como mucho m
   rutas, que no se repita ningún cliente y que cada ruta respete `tmax`, con tiempos y
   recompensas recalculados desde cero.
2. **Monotonía:** la recompensa nunca baja, y si no cambia, el tiempo nunca sube.
3. **Idempotencia:** aplicar la búsqueda local dos veces seguidas da la misma solución. Si
   no, la primera no paraba en un óptimo local.
4. **Deltas frente a recálculo:** en los tests, comparar cada Δ calculado en O(1) con
   `route_time` sobre la ruta modificada.
5. **Tests unitarios por movimiento,** con rutas pequeñas construidas a mano:
   - un cruce que el 2-opt debe deshacer;
   - un cliente que cabe y otro que no;
   - un reemplazo obvio.
6. **Reproducibilidad:** con la misma semilla, la salida es idéntica byte a byte. La
   búsqueda local es determinista, así que no debería romper nada.

---

## 13. Resumen y propuesta

| Movimiento | Sube la recompensa | Coste | Dificultad | Aporte esperado | Propuesta |
|---|---|---|---|---|---|
| 2-opt | No (libera tiempo) | Bajo, O(k²) | Baja | Alto como apoyo | **Núcleo** |
| Or-opt | No (libera tiempo) | Bajo, O(k²) | Media | Medio-bajo | Opcional |
| Inserción | **Sí** | Bajo, O(U·N) | Baja | **Muy alto** | **Núcleo** |
| Reemplazo | **Sí** | Alto, O(V·U·N) | Media | Alto con rutas llenas | **Núcleo** (con candidatos) |
| Relocate / swap / 2-opt* | No (redistribuye) | Medio | Media-alta | Bajo-medio | Segunda fase |
| Destruir y reconstruir | Sí (perturbación) | Medio | Media | Alto, pero cambia a ILS | Trabajo futuro |

**Propuesta:**

1. Un módulo nuevo, `local_search.rs`.
2. Un **VND con 2-opt → inserción → reemplazo** y el criterio lexicográfico.
3. Aplicado en **cada iteración de GRASP**, incluida la solución determinista inicial.
4. Medir el coste. Si es excesivo en la rejilla, pasar a la opción C (solo las
   prometedoras).
5. Los movimientos entre rutas y la perturbación, solo si los resultados se estancan.

**Decisiones abiertas:**

- ¿Qué movimientos entran en la primera versión?
- ¿La búsqueda local en cada iteración, solo en la mejor o solo en las prometedoras?
- ¿Eficiencia (u/Δ) o recompensa para ordenar las inserciones?
- ¿Con qué experimento se evalúa?
  - la rejilla completa con y sin búsqueda local, para compararla con
    `informe_alpha.md`;
  - ejecuciones largas con el mejor α, para compararlas con el BKS de p1/p2;
  - o ambas.
