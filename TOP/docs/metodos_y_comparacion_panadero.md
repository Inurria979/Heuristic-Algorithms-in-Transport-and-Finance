# TOP: nuestros métodos y su comparación con Panadero et al. (2021)

Documento de síntesis. Explica los tres métodos que hemos construido, de menos a más
elaborado, y compara cada uno con Panadero et al. (2021):

1. **PJS determinista con rejilla de α.**
2. **GRASP-PJS** (multiarranque con aleatorización sesgada), con distintos presupuestos:
   10 iteraciones, 100 iteraciones y 1 s por ejecución.
3. **GRASP-PJS + búsqueda local** sobre las soluciones élite.

La comparación tiene dos partes, porque el paper publica dos tipos de resultados:

- **instancia a instancia**, solo para p1 y p2 (tablas 3 y 4): secciones 1 a 4;
- **medias por conjunto** para los siete conjuntos (figura 4): sección 5.

El detalle de cada parte está en otros documentos:

- rejilla de α y GRASP: [`informe_alpha.md`](informe_alpha.md);
- diseño de la búsqueda local: [`busqueda_local_diseno.md`](busqueda_local_diseno.md);
- resultados de la búsqueda local: [`informe_busqueda_local.md`](informe_busqueda_local.md);
- comparaciones con Panadero instancia a instancia: [`comparacion_panadero.md`](comparacion_panadero.md)
  (1 s con el mejor α), [`comparacion_panadero_rejilla.md`](comparacion_panadero_rejilla.md)
  (1 s con la rejilla) y [`comparacion_panadero_rejilla_100it.md`](comparacion_panadero_rejilla_100it.md)
  (100 iteraciones con la rejilla);
- comparación por conjuntos: [`comparacion_bks_conjuntos.md`](comparacion_bks_conjuntos.md).

---

## 0. Cómo se compara en p1 y p2

- **Instancias:** las 81 de las tablas 3 y 4 de Panadero, extraídas en
  `src/data/panadero2021_p1_p2.csv`. En la tabla 4, p2.4.a aparece por errata como
  "p2.4.1".
- **BKS:** la mejor solución conocida de la literatura (columna [a] de sus tablas). Suma
  10 685 en las 81 instancias.
- **Panadero, 1 hilo:** una ejecución de su multiarranque durante 1 s.
- **Panadero, 128 hilos:** la mejor de 128 ejecuciones de 1 s en paralelo (columna *OBS*).
- **Medida principal:** la **distancia media al BKS por instancia**,
  100 · (BKS − nuestra) / BKS promediada sobre las 81 instancias. Es la medida de la fila
  *Average* de las tablas del paper. También se da la distancia sobre la recompensa total,
  que pesa más las instancias grandes.
- **Mejor / igual / peor:** cuántas de las 81 instancias gana, empata o pierde cada método
  frente a Panadero con 128 hilos y frente a Panadero con 1 hilo.

| Referencia | Recompensa total | Distancia sobre el total | **Distancia media por instancia** | BKS alcanzado |
|---|---|---|---|---|
| BKS | 10 685 | — | — | 81 |
| Panadero, 1 hilo | 10 440 | 2.29 % | **1.73 %** | 52 |
| Panadero, 128 hilos | 10 615 | 0.66 % | **0.44 %** | 69 |

---

## 1. PJS determinista con rejilla de α

### Método

La heurística de ahorros de Panadero (sección 4 del paper):

1. **Solución dummy:** una ruta origen → i → destino por cliente; se descartan los clientes
   que ni solos caben en `tmax`.
2. **Ahorro enriquecido:** s'ᵢⱼ = α·sᵢⱼ + (1 − α)·(uᵢ + uⱼ), con
   sᵢⱼ = t(i, destino) + t(origen, j) − t(i, j). Los arcos se consideran en los dos
   sentidos, porque con origen y destino distintos sᵢⱼ ≠ sⱼᵢ.
3. **Fusión voraz:** se recorre la lista de mayor a menor ahorro y se une la ruta que acaba
   en i con la que empieza en j si el resultado cabe en `tmax`.
4. **Selección:** se quedan las m rutas con más recompensa.

**El paper no da ningún valor de α**: solo dice que va cerca de 0 con recompensas
heterogéneas y cerca de 1 con recompensas homogéneas. Probamos reglas para fijarlo a partir
de la instancia (coeficiente de variación, regresión, árboles, kNN) y ninguna superó a un α
fijo: la recompensa salta de forma irregular con α. Por eso **ejecutamos los 21 valores
α = 0, 0.05, …, 1 y nos quedamos con el mejor de cada instancia**. Son 21 construcciones
por instancia, una fracción de milisegundo.

### Resultado frente a Panadero

| | Recompensa total | Distancia sobre el total | Distancia media | BKS | Frente a P-128 (mejor/igual/peor) | Frente a P-1 |
|---|---|---|---|---|---|---|
| Determinista, mejor α | 10 060 | 5.85 % | **4.82 %** | 44 | 0 / 45 / 36 | 0 / 49 / 32 |

**Lectura.** La heurística determinista sola queda lejos: a casi un 5 % del BKS, casi tres
veces más que Panadero con 1 hilo. No gana a Panadero en ninguna instancia. Es lo esperable:
una sola construcción voraz, aunque se elija el mejor de 21 α, no compite con un
multiarranque.

---

## 2. GRASP-PJS: multiarranque con aleatorización sesgada

### Método

Es el Algoritmo 1 de Panadero (y de Li et al., 2022):

- **Iteración:** una construcción PJS completa en la que el siguiente arco no es siempre el
  de mayor ahorro, sino uno elegido al azar con sesgo hacia los mejores.
- **Ejecución:** parte de la solución determinista y repite iteraciones en serie,
  quedándose con la mejor, hasta agotar el presupuesto (un número de iteraciones o un
  tiempo). Algorítmicamente equivale a un hilo de Panadero; en el código, cada ejecución es
  una tarea que rayon reparte entre los 12 hilos.
- **Dos aleatorizaciones:**
  - **geométrica** (la de Panadero): se toma el k-ésimo arco de los que quedan, con
    k ~ Geom(β), β = 0.3;
  - **softmax** (propuesta nuestra): cada arco con probabilidad proporcional a exp(s'/τ),
    τ = 1.
- **Semillas:** una por ejecución, derivada de (instancia, α, aleatorización, número de
  ejecución). Las ejecuciones de distintos presupuestos con la misma etiqueta **comparten
  semilla**: una ejecución de 100 iteraciones empieza exactamente igual que la de 10 y sigue
  iterando, así que nunca es peor.
- **Dentro de GRASP no hay búsqueda local.** Solo se generan soluciones y se guarda la
  mejor.

### Presupuestos probados

| Variante | α | Ejecuciones por instancia | Presupuesto por ejecución | Construcciones por instancia | Tiempo total (81 inst.) | Reproducible |
|---|---|---|---|---|---|---|
| **10 iteraciones, rejilla** (experimento principal) | los 21 | 21 × (10 geom. + 10 softmax) = 420 | 10 iteraciones | ≈ 4 200 | parte de los 140 s de las 387 instancias | sí |
| **100 iteraciones, rejilla** | los 21 | 420 | 100 iteraciones | ≈ 42 000 | 29 s | sí |
| **1 s, mejor α** (como Panadero) | el mejor determinista | 10 geom. + 10 softmax = 20 | 1 s de reloj (~39 000 it. en p1, ~50 000 en p2) | ≈ 900 000 | 136 s | no |
| **1 s, rejilla** | los 21 | 420 | 1 s de reloj (~41 000 it. en p1, ~56 000 en p2) | ≈ 20 millones | 2 836 s (47 min) | no |

Todas con 12 hilos en un Ryzen 5 5500U (6 núcleos físicos). Con tiempo de reloj, las
iteraciones dependen de la carga de la máquina, y por eso esas variantes no se pueden
reproducir exactamente. p2.4.a se añadió después con `pjs panadero --resume`, que ejecuta
solo las instancias que faltan en los CSV; los tiempos suman las dos ejecuciones.

### Resultado frente a Panadero (sin búsqueda local)

**Una sola ejecución** con el mejor α, media de las 10 ejecuciones. Es lo comparable con
Panadero con 1 hilo:

| Presupuesto de la ejecución | Geométrica: distancia media | Softmax: distancia media |
|---|---|---|
| 10 iteraciones | 4.05 % | 4.35 % |
| 100 iteraciones | 3.28 % | 3.55 % |
| 1 s | 2.43 % | 2.29 % |
| *Panadero, 1 hilo (1 s)* | *1.73 %* | — |

**La mejor de todas las ejecuciones** de la variante:

| Variante | Recompensa total | Distancia sobre el total | Distancia media | BKS | Frente a P-128 | Frente a P-1 |
|---|---|---|---|---|---|---|
| 10 iteraciones, rejilla (420 ejecuciones) | 10 395 | 2.71 % | **1.99 %** | 53 | 1 / 56 / 24 | 8 / 62 / 11 |
| 100 iteraciones, rejilla (420) | 10 470 | 2.01 % | **1.45 %** | 57 | 3 / 57 / 21 | 14 / 60 / 7 |
| 1 s, mejor α (20) | 10 470 | 2.01 % | **1.49 %** | 56 | 2 / 60 / 19 | 11 / 65 / 5 |
| 1 s, rejilla (420) | 10 575 | 1.03 % | **0.71 %** | 65 | 4 / 67 / 10 | 22 / 57 / 2 |

**Lectura.**

- **GRASP mejora mucho a la determinista.** Con solo 10 iteraciones por ejecución, la
  distancia media baja del 4.82 % al 1.99 %.
- **Una ejecución nuestra de 1 s rinde algo menos que una de Panadero** (2.3–2.4 % frente a
  1.73 %). La construcción es la misma idea, así que la diferencia estará en detalles de
  implementación que el paper no fija (orden de la lista en empates, tratamiento de los
  arcos en los dos sentidos, máquina, α elegido).
- **Probar varios α compensa más que iterar más con uno fijo.** La rejilla de 100
  iteraciones (≈ 42 000 construcciones) iguala a 1 s con el mejor α (≈ 900 000
  construcciones), con unas 20 veces menos trabajo.
- **Sin búsqueda local, ninguna variante alcanza a Panadero con 128 hilos.** La mejor,
  1 s con la rejilla, se queda en 0.71 % frente a 0.44 %.

---

## 3. GRASP-PJS + búsqueda local sobre las élite

### Método

La búsqueda local se aplica **al final**, no dentro de las iteraciones de GRASP, y solo a
un conjunto de soluciones élite por instancia:

- **Élite:**
  - las soluciones deterministas de las 3 α con más recompensa;
  - para cada aleatorización y cada α, la mejor solución de las 3 mejores ejecuciones.
- **Criterio:** solo la recompensa; los empates en la tercera posición entran todos.
- **Duplicados:** dos élite son iguales si visitan el mismo conjunto de clientes; se
  conserva la primera en el orden determinista → geométrica → softmax, α y ejecución
  crecientes.
- **Búsqueda local**, en bucle:
  1. **2-opt** dentro de cada ruta, con primera mejora, hasta que ninguna se acorte;
  2. **insertar un cliente:** entre los no visitados que caben en alguna ruta, el de mayor
     recompensa (en empate, el que menos tiempo añade), en su posición más barata;
  3. si se ha insertado alguno, volver a 1.

  El 2-opt libera tiempo que aprovecha la inserción siguiente. Ningún movimiento empeora la
  solución, así que no hace falta criterio de aceptación.
- **Solución final:** la élite con más recompensa tras la búsqueda local.

La búsqueda local es barata: 0.4 s para las ~18 800 élite de las 387 instancias. Las rutas
finales de la rejilla se comprueban además con un verificador independiente
(`verify_routes`): todas caben en `tmax`, sin clientes repetidos y con como mucho m rutas.

### Resultado frente a Panadero

| Variante | Recompensa total | Distancia sobre el total | Distancia media | BKS | Frente a P-128 | Frente a P-1 |
|---|---|---|---|---|---|---|
| Solo élite deterministas + búsqueda local | 10 405 | 2.62 % | **2.13 %** | 53 | 3 / 56 / 22 | 10 / 61 / 10 |
| 10 iteraciones, rejilla + búsqueda local | 10 635 | 0.47 % | **0.34 %** | 73 | 10 / 65 / 6 | 26 / 54 / 1 |
| 100 iteraciones, rejilla + búsqueda local | 10 645 | 0.37 % | **0.29 %** | 74 | 12 / 62 / 7 | 27 / 53 / 1 |
| 1 s, mejor α + búsqueda local | 10 585 | 0.94 % | **0.71 %** | 63 | 7 / 62 / 12 | 21 / 59 / 1 |
| **1 s, rejilla + búsqueda local** | **10 665** | **0.19 %** | **0.15 %** | **77** | **12 / 65 / 4** | 28 / 52 / 1 |

**Lectura.**

- **La búsqueda local es lo que nos pone por delante de Panadero con 128 hilos.** Con la
  rejilla de 10 iteraciones, la distancia media pasa del 1.99 % al 0.34 %, ya mejor que el
  0.44 % del paper.
- **La búsqueda local necesita la diversidad de GRASP.** Aplicada solo a las deterministas
  se queda en 2.13 %. Cuando se aplica también a las élite de GRASP baja a 0.34 %: muchas
  soluciones finales salen de una élite que no era la mejor antes de la búsqueda local.
- **El mejor resultado es 1 s con la rejilla y búsqueda local:** 0.15 %, BKS en 77 de 81
  instancias, y mejor que Panadero con 128 hilos en 12, igual en 65 y peor en 4 (p1.2.i,
  p1.2.l, p1.2.m y p1.3.l). Ninguna solución nuestra supera el BKS.
- **Más presupuesto no garantiza más recompensa después de la búsqueda local.** En p1.4.r,
  con 10 iteraciones se llega a 210 y con 100 iteraciones solo a 205. Las élite se eligen
  por la recompensa *antes* de la búsqueda local, y una élite peor puede acabar mejor tras
  ella.
- **Frente a Panadero con 1 hilo**, todas las variantes con rejilla y búsqueda local pierden
  solo en una instancia.

---

## 4. Resumen de todas las variantes en p1 y p2

Ordenadas por distancia media al BKS en las 81 instancias:

| Método | Distancia media al BKS | BKS alcanzado | Frente a Panadero 128 (mejor/igual/peor) |
|---|---|---|---|
| Determinista, mejor α | 4.82 % | 44 | 0 / 45 / 36 |
| GRASP, una ejecución de 10 it. (geométrica, media) | 4.05 % | — | — |
| GRASP, una ejecución de 1 s (geométrica, media) | 2.43 % | — | — |
| Deterministas + búsqueda local | 2.13 % | 53 | 3 / 56 / 22 |
| GRASP 10 it., rejilla | 1.99 % | 53 | 1 / 56 / 24 |
| *Panadero, 1 hilo* | *1.73 %* | *52* | — |
| GRASP 1 s, mejor α | 1.49 % | 56 | 2 / 60 / 19 |
| GRASP 100 it., rejilla | 1.45 % | 57 | 3 / 57 / 21 |
| GRASP 1 s, rejilla | 0.71 % | 65 | 4 / 67 / 10 |
| GRASP 1 s, mejor α + búsqueda local | 0.71 % | 63 | 7 / 62 / 12 |
| *Panadero, 128 hilos* | *0.44 %* | *69* | — |
| GRASP 10 it., rejilla + búsqueda local | 0.34 % | 73 | 10 / 65 / 6 |
| GRASP 100 it., rejilla + búsqueda local | 0.29 % | 74 | 12 / 62 / 7 |
| **GRASP 1 s, rejilla + búsqueda local** | **0.15 %** | **77** | **12 / 65 / 4** |

---

## 5. Los siete conjuntos: medias por conjunto

Para p3 a p7 el paper no publica valores por instancia, solo la distancia media al BKS de
cada conjunto con 128 hilos (figura 4). Para compararnos:

- **BKS de las 353 instancias factibles:** columna *BEST* de los resultados públicos de
  PSOiA (Dang, Guibadj y Moukrim), en `src/data/bks_chao.csv`. En las 81 instancias de las
  tablas de Panadero coincide exactamente con su BKS (Ke et al., 2016).
- **Nuestros resultados:** la rejilla de 10 iteraciones con búsqueda local, la versión
  barata.

| Conjunto | Instancias | Determinista, mejor α | Mejor sin búsqueda local | **Final** | Panadero, 128 hilos (figura 4) |
|---|---|---|---|---|---|
| p1 | 48 | 6.88 % | 3.15 % | **0.58 %** | 0.69 % |
| p2 | 33 | 1.81 % | 0.31 % | **0.00 %** | 0.11 % |
| p3 | 60 | 6.84 % | 2.96 % | **0.67 %** | 1.44 % |
| p4 | 56 | 29.25 % | 23.73 % | **5.87 %** | 7.97 % |
| p5 | 74 | 11.82 % | 4.49 % | **1.30 %** | 1.46 % |
| p6 | 24 | 14.45 % | 8.51 % | **1.17 %** | 1.42 % |
| p7 | 58 | 19.75 % | 14.48 % | **3.42 %** | 5.10 % |
| **Todas** | 353 | 13.61 % | 8.63 % | **2.04 %** | 2.85 % (media ponderada) |

- **Quedamos por debajo de Panadero con 128 hilos en los siete conjuntos.**
- **BKS alcanzado en 189 de 353 instancias**, y **ninguna solución lo supera**. En p4 y p7,
  donde la búsqueda local inserta más clientes, esto es además una señal de factibilidad.
- Se regenera con `pjs bks` (ver [`comparacion_bks_conjuntos.md`](comparacion_bks_conjuntos.md)).

---

## 6. Conclusiones

1. **El determinista no basta**: un 4.8 % del BKS en p1 y p2, y un 13.6 % en el conjunto
   de las 353 instancias, aun eligiendo el mejor de 21 α.
2. **GRASP es la primera gran mejora** y **la rejilla de α pesa más que el número de
   iteraciones**: diversificar α rinde más que iterar mucho con uno solo.
3. **La búsqueda local sobre las élite es la segunda gran mejora**, y es la que nos lleva
   por delante de Panadero con 128 hilos. Ya con el experimento barato (10 iteraciones)
   queda en 0.34 % frente a 0.44 % en p1 y p2, y por debajo en los siete conjuntos
   (2.04 % frente a 2.85 %).
4. **Coste.** La comparación no es en igualdad de cómputo:
   - la rejilla de 10 iteraciones hace unas 4 200 construcciones por instancia, mucho menos
     que los 128 s de CPU de Panadero, y ya le gana;
   - la de 1 s gasta unos 420 s de CPU por instancia, más que Panadero. Es la mejor, pero
     a cambio de mucho más tiempo.
5. **Limitaciones:**
   - las variantes de 1 s no son reproducibles exactamente;
   - para p3 a p7 solo hay medias por conjunto, así que ahí no podemos compararnos con
     Panadero instancia a instancia.
