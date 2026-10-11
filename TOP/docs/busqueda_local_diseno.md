# Búsqueda local sobre soluciones élite: decisiones y pseudocódigo

Este documento fija el diseño **acordado** de la búsqueda local de GRASP-PJS, tal como se
implementa en `pjs grid`. [`busqueda_local.md`](busqueda_local.md) repasa todas las opciones
posibles; aquí solo está lo que se ha decidido y por qué.

---

## 1. Resumen

1. **La rejilla no cambia.** Por cada instancia y cada uno de los 21 α hay:
   - 1 construcción determinista;
   - 10 ejecuciones de GRASP geométrico, cada una de 10 iteraciones;
   - 10 ejecuciones de GRASP softmax, cada una de 10 iteraciones.
2. **Novedad:** cada tarea guarda su **solución**, además de la recompensa.
3. **Al terminar la rejilla**, se elige por instancia un conjunto de **soluciones élite**.
4. A cada élite se le aplica la **búsqueda local**: 2-opt e inserciones en bucle.
5. Se reporta:
   - la mejor solución de cada instancia;
   - cuánto mejora GRASP a la determinista;
   - cuánto mejora la búsqueda local a cada solución, frente a la misma sin ella.

**La búsqueda local va al final, nunca dentro de las iteraciones de GRASP.** Los resultados
de la rejilla (`alpha_grid.csv`, `grasp_runs.csv`, `best_alpha.csv`, `informe_alpha.md`) no
cambian.

---

## 2. Decisiones

### 2.1 Qué soluciones son élite

Para cada instancia hay tres grupos:

| Grupo | Candidatas | Élite |
|---|---|---|
| **Determinista** | Las 21 soluciones deterministas, una por α | Las de las α con las 3 mejores recompensas |
| **GRASP geométrico** | Para cada α, las 10 ejecuciones; de cada una, su mejor solución | Para cada α, las 3 ejecuciones con más recompensa |
| **GRASP softmax** | Lo mismo | Lo mismo |

Detalles:

- **De cada ejecución de GRASP cuenta solo su mejor solución**, la que devuelve `grasp()`.
  Las iteraciones intermedias no se guardan.
- **El único criterio es la recompensa.** El tiempo de las rutas no desempata.
- **Los empates entran todos.** El umbral es la tercera mejor recompensa, contando las
  repeticiones, y entra toda candidata que lo alcance. Así, puede haber más de 3 élite:

  | Recompensas de las candidatas | Umbral | Élite |
  |---|---|---|
  | 100, 95, 90, 80 | 90 | 100, 95, 90 (3) |
  | 100, 100, 100, 100, 90 | 100 | las cuatro de 100 |
  | 100, 95, 90, 90, 80 | 90 | 100, 95, 90, 90 (4) |
  | 100, 90 (solo dos candidatas) | — | las dos |

- **Como mucho por instancia,** sin contar empates: 3 + 21 × 3 + 21 × 3 = 129 élite.
- **En la práctica, los empates son muy frecuentes.** Muchas ejecuciones de GRASP no
  mejoran la determinista y devuelven la misma solución. En la primera ejecución completa
  se eligieron 125 490 élite en las 387 instancias (unas 324 por instancia), y tras quitar
  duplicados quedaron 18 791 (unas 49).

### 2.2 Duplicados

- **Dos soluciones son iguales si visitan el mismo conjunto de clientes.** No importa en qué
  rutas ni en qué orden.
- **La repetida se descarta y no se sustituye.** No se busca la siguiente mejor para
  completar el grupo, así que un grupo puede quedarse con menos de 3.
- **Se compara contra todas las élite de la instancia**, sean del mismo grupo o de otro.
- **Orden de recorrido:** se recorren las élite en este orden y se conserva la primera de
  cada conjunto de clientes:
  1. deterministas, por α creciente;
  2. geométricas, por α creciente y, dentro de cada α, por número de ejecución;
  3. softmax, en el mismo orden.
- **Consecuencia:** si una ejecución de GRASP no mejora la determinista de su α, devuelve
  esa misma solución. Si la determinista es élite, la de GRASP se descarta y la solución
  cuenta como determinista.
- **La comparación se hace antes de la búsqueda local.** Dos élite distintas pueden acabar
  en la misma solución después de ella, y las dos se conservan.

### 2.3 La búsqueda local

Bucle de dos movimientos, repetido mientras alguno cambie la solución:

1. **2-opt dentro de cada ruta**, hasta que ninguna se pueda acortar.
2. **Una inserción.** Entre los clientes no visitados que caben en alguna ruta:
   - se elige el de **mayor recompensa**;
   - si empatan, el que **menos tiempo añade**;
   - se mete en su posición más barata, en **cualquier ruta**, aunque el 2-opt no la haya
     acortado.
3. Si se ha insertado algún cliente, se vuelve al paso 1. Si no, se termina.

**Por qué en bucle.** Insertar un cliente alarga la ruta y puede crear cruces. Al repetir el
2-opt, el tiempo que libera se aprovecha para la siguiente inserción.

**Por qué no hay criterio de aceptación.** Ningún movimiento empeora la solución:

- el 2-opt solo se aplica si acorta la ruta, y no cambia la recompensa;
- la inserción siempre sube la recompensa y respeta `tmax`.

**Nunca hace falta abrir una ruta nueva.** La selección de PJS se queda con las m mejores
rutas. Si la construcción acaba con menos de m rutas, es porque ya contenían a todos los
clientes alcanzables, así que no queda ninguno por insertar. En el resto de casos ya hay m
rutas y las inserciones van en ellas.

### 2.4 Detalles numéricos

- **2-opt.** Sea Δ la variación del tiempo de la ruta.
  - El movimiento se aplica si Δ < −10⁻⁹, para que el redondeo no provoque un ciclo de
    mejoras ficticias.
  - Después se recalcula el tiempo de la ruta tramo a tramo, con `route_time`, como en el
    resto del código, para que `check()` lo dé por bueno.
- **Inserción.**
  - Primero se filtra con el coste incremental Δ = t(a,k) + t(k,b) − t(a,b).
  - El candidato elegido se comprueba con el tiempo exacto de la ruta resultante.
  - Si por redondeo no cabe, se descarta esa posición y se busca el siguiente mejor. Es el
    mismo esquema de filtro más cálculo exacto que usa `try_merge`.
- **Empates al insertar.** Primero cuenta la recompensa, después el menor Δ y, por último,
  el menor número de cliente, la menor ruta y la menor posición. Así el resultado es
  reproducible.
- **2-opt con primera mejora.** Se recorren los pares (i, j) en orden y se aplica el primero
  que acorta. Es lo habitual, y la ruta resultante no depende de los hilos.

### 2.5 Ejecución y salidas

- **Todo dentro de `pjs grid`.** La búsqueda local se lanza al terminar la rejilla, sobre
  todas las élite de todas las instancias, en una lista paralela con rayon.
- **Memoria.** Las 170 667 soluciones de la rejilla se guardan en memoria hasta elegir las
  élite. Son unas decenas de MB. Se descartó volver a calcular las élite a partir de sus
  semillas, porque añadía tiempo y complicaba el código.
- **Comprobación:** `Solution::check()` sobre cada solución antes y después de la búsqueda
  local.

**Salidas nuevas:**

| Fichero | Contenido |
|---|---|
| `results/local_search.csv` | Una fila por élite (ya sin duplicados): instancia, grupo, α, ejecución, recompensa y tiempo antes y después, movimientos 2-opt y clientes insertados |
| `results/best_solution.csv` | Una fila por instancia: la mejor determinista, la mejor de GRASP (sin búsqueda local), la mejor tras la búsqueda local y de qué élite sale, y cuántas élite había antes y después de quitar duplicados |
| `results/figures/routes_ls/<instancia>.png` | Las rutas de la mejor solución final de cada instancia (opción `--ls-png-dir`) |
| `docs/informe_busqueda_local.md` | El informe (opción `--ls-report`) |

---

## 3. Pseudocódigo

### 3.1 Flujo completo

```
procedimiento GRID_CON_BUSQUEDA_LOCAL(instancias):
    // Fase 1: la rejilla de siempre, guardando las soluciones
    para cada (instancia I, α, tarea) en paralelo:
        si tarea = determinista:
            det[I][α] ← PJS(I, α)
        si no:
            (tipo, r) ← tarea                      // tipo ∈ {geométrica, softmax}, r ∈ 0..9
            grasp[I][α][tipo][r] ← GRASP(I, α, tipo, semilla(I, α, tipo, r))

    // Fase 2: élite y búsqueda local, por instancia
    para cada instancia I en paralelo:
        élite ← ELEGIR_ELITE(det[I], grasp[I])
        para cada e en élite:
            e.después ← BUSQUEDA_LOCAL(I, e.solución)
        mejor[I] ← la élite con mayor recompensa después de la búsqueda local

    escribir CSV, figuras e informe
```

**Explicación:**

- **Fase 1.** Es exactamente la rejilla actual: mismas tareas, mismas semillas, mismas
  soluciones. Lo único nuevo es que las soluciones se guardan.
- **Fase 2.** Cada instancia es independiente, así que se reparten entre los hilos. Al
  terminar, se elige como mejor solución final la élite que más recompensa tiene tras la
  búsqueda local. Si empatan, la primera en el orden de recorrido.

### 3.2 Elegir las élite

```
función MEJORES_CON_EMPATES(candidatas):
    si |candidatas| ≤ 3: devolver candidatas
    umbral ← tercera mayor recompensa de candidatas (contando repetidas)
    devolver { c ∈ candidatas : c.recompensa ≥ umbral }

función ELEGIR_ELITE(det, grasp):
    lista ← []
    añadir a lista MEJORES_CON_EMPATES(det[α] para α en 0..20), por α creciente
    para tipo en (geométrica, softmax):
        para α en 0..20:
            añadir a lista MEJORES_CON_EMPATES(grasp[α][tipo][r] para r en 0..9), por r creciente

    vistos ← conjunto vacío
    élite ← []
    para e en lista:                               // en el orden de arriba
        clave ← conjunto de clientes visitados por e
        si clave ∉ vistos:
            vistos ← vistos ∪ {clave}
            élite.añadir(e)
        // si ya se había visto, e se descarta sin sustituto
    devolver élite
```

**Explicación:**

- **`MEJORES_CON_EMPATES`** ordena las recompensas de mayor a menor y toma la tercera como
  umbral. Al quedarse con todo lo que lo alcanza, entran los empates. Si hay 3 candidatas o
  menos, entran todas.
- **El orden de `lista`** decide qué copia sobrevive cuando hay duplicados. Primero las
  deterministas: una solución que GRASP repite sin mejorar cuenta como determinista.
- **La clave** es el conjunto de clientes. En el código es la lista ordenada de los clientes
  visitados, que identifica el conjunto sin depender del orden de las rutas.

### 3.3 Búsqueda local

```
función BUSQUEDA_LOCAL(I, S):
    repetir:
        para cada ruta R de S:
            DOS_OPT(I, R)
        insertado ← INSERTAR_UNO(I, S)
    hasta que no insertado
    devolver S
```

**Explicación:** el bucle termina porque cada vuelta que sigue ha insertado un cliente, y
como mucho hay tantas vueltas como clientes alcanzables. El 2-opt no cuenta para seguir
iterando. Al principio de cada vuelta las rutas ya se repasan de todos modos, y si la última
inserción no ha creado cruces, el 2-opt no encontrará nada.

### 3.4 2-opt dentro de una ruta

```
procedimiento DOS_OPT(I, R):                       // R = 0 → v₁ → … → v_k → sink
    repetir:
        mejorado ← falso
        para i en 1..k-1:
            para j en i+1..k:
                a ← v_{i-1} (o el origen 0 si i = 1)
                d ← v_{j+1} (o el destino si j = k)
                Δ ← t(a, v_j) + t(v_i, d) − t(a, v_i) − t(v_j, d)
                si Δ < −10⁻⁹:
                    invertir el tramo v_i … v_j
                    mejorado ← verdadero
                    salir de los dos bucles        // primera mejora
    hasta que no mejorado
    R.tiempo ← tiempo de R recalculado tramo a tramo
```

**Explicación:**

- Invertir el tramo `v_i … v_j` cambia solo dos arcos: `(a, v_i)` y `(v_j, d)` pasan a ser
  `(a, v_j)` y `(v_i, d)`. Las distancias son euclídeas y simétricas, así que el interior
  del tramo cuesta lo mismo en un sentido que en otro, y Δ se calcula en O(1).
- El origen y el destino no se mueven nunca. Por eso funciona igual con origen y destino
  distintos (p1–p6) que iguales (p7).
- Cuando encuentra una mejora, la aplica y vuelve a empezar desde el principio. Termina
  cuando un recorrido completo no encuentra ninguna.
- La recompensa no cambia y el tiempo solo baja, así que la ruta sigue cabiendo en `tmax`.

### 3.5 Insertar un cliente

```
función INSERTAR_UNO(I, S):
    descartados ← ∅                                // posiciones que fallan por redondeo
    repetir:
        mejor ← ninguno
        para cada cliente k alcanzable y no visitado:
            para cada ruta R de S, para cada hueco (a, b) consecutivo de R:
                si (k, R, hueco) ∈ descartados: continuar
                Δ ← t(a,k) + t(k,b) − t(a,b)
                si R.tiempo + Δ ≤ tmax + 10⁻⁹:
                    candidato ← (u_k, Δ, k, R, hueco)
                    si candidato es mejor que mejor:  // más u_k; si empata, menos Δ;
                        mejor ← candidato          // después, menor k, R y hueco
        si mejor = ninguno: devolver falso
        R' ← R con k insertado en el hueco
        si tiempo exacto de R' ≤ tmax + 10⁻⁹:
            R ← R'; actualizar tiempo, recompensa y totales de S
            devolver verdadero
        descartados ← descartados ∪ {(k, R, hueco)}
```

**Explicación:**

- **Huecos.** Una ruta con k clientes tiene k + 1 huecos: entre el origen y el primero,
  entre cada par de clientes consecutivos y entre el último y el destino.
- **Un cliente cabe** si, en alguno de sus huecos, el tiempo de la ruta más Δ no supera
  `tmax`. Para cada cliente cuenta la posición más barata entre todos sus huecos de todas
  las rutas.
- **Elección.** De todos los que caben, gana el de más recompensa y, en caso de empate, el
  de menor Δ. Se inserta **un solo** cliente por llamada; el siguiente se decide después de
  volver a pasar el 2-opt.
- **Comprobación exacta.** Δ es una estimación por diferencias. Antes de aceptar, se
  recalcula el tiempo de la ruta tramo a tramo. Si por redondeo se pasa de `tmax` (es muy
  raro), esa posición se descarta y se repite la búsqueda.
- **Coste.** Cada llamada evalúa todos los no visitados en todos los huecos: O(U·N), con U
  los no visitados y N los huecos. Con n ≤ 102 son del orden de 10⁴ evaluaciones.

---

## 4. Qué se reporta y cómo se lee

El informe (`docs/informe_busqueda_local.md`) usa las 353 instancias con alguna ruta
factible y responde a tres preguntas:

1. **La mejor solución de cada instancia:** recompensa, número de rutas y clientes, de qué
   élite sale (grupo, α y ejecución) y cuánto le ha aportado la búsqueda local. Las rutas
   están en `results/figures/routes_ls/`.
2. **Cuánto mejora GRASP a la determinista**, en recompensa total y número de instancias
   mejoradas:
   - **sin búsqueda local:** la mejor determinista de cada instancia frente a la mejor de
     GRASP;
   - **con búsqueda local:** la mejor de las élite deterministas frente a la mejor de todas
     las élite, ambas tras la búsqueda local. Es decir, cuánto añade GRASP cuando las dos
     pasan por la búsqueda local.

     No se compara contra "las élite de GRASP" a secas, porque a GRASP le faltarían las
     soluciones que repiten una determinista: se descartaron como duplicados. Por la misma
     razón, la mejor de GRASP sin búsqueda local se lee de la rejilla (`grasp_runs`) y no de
     las élite.
3. **Cuánto mejora la búsqueda local**, comparando cada solución consigo misma antes y
   después:
   - por élite, agrupado por origen (determinista, geométrica, softmax): ganancia media, en
     cuántas mejora y cuántos clientes inserta de media;
   - por instancia: la mejor sin búsqueda local frente a la mejor con ella;
   - si la mejor final viene de la élite que ya era la mejor antes, o de otra. Esto mide si
     compensa aplicar la búsqueda local a varias élite y no solo a la mejor.

Hay además una tabla por conjunto (p1–p7) y otra por instancia.
