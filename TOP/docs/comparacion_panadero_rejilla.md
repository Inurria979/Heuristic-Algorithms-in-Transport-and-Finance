# Comparación con Panadero et al. (2021), conjuntos p1 y p2: 1 s, rejilla de α

Informe generado por `pjs` (`TOP/src/rust`); se regenera con

```
cd TOP
ignore/target/release/pjs.exe panadero --alpha grid --time 1 --runs 10 --geometric 0.3 --softmax 1 --seed 1 --best-csv results/panadero_1s.csv \
    --report docs/comparacion_panadero_rejilla.md src/data/top-data src/data/panadero2021_p1_p2.csv results/best_solution.csv results
```

## Qué se compara

Solo las 81 instancias de las tablas 3 y 4 de Panadero et al. (2021), extraídas en `src/data/panadero2021_p1_p2.csv`.

- **BKS:** la mejor solución conocida de la literatura (columna [a] de las tablas).
- **Panadero, 1 hilo / 128 hilos:** su algoritmo (ahorros con aleatorización sesgada, multiarranque), en el que cada hilo itera durante 1 s; con 128 hilos se queda con el mejor (*OBS*).
- **Rejilla:** nuestro experimento principal ([`informe_busqueda_local.md`](informe_busqueda_local.md)): 21 α, 10 ejecuciones de 10 iteraciones por α y aleatorización, élite y búsqueda local.
- **1 s, rejilla de α:** para cada instancia, **cada uno de los 21 α de la rejilla**, con **10 ejecuciones de GRASP geometric (β = 0.3) y 10 softmax (τ = 1) por α, cada una iterando durante 1 s de reloj**. Las 34020 ejecuciones se reparten entre 12 hilos en un equipo de 6 núcleos físicos (Ryzen 5 5500U), así que cada ejecución hace menos iteraciones que si estuviera sola. Por usar tiempo de reloj, este experimento **no es reproducible exactamente**: el número de iteraciones depende de la carga de la máquina. Cada ejecución usa la misma semilla que en la rejilla, así que empieza igual que la de 10 iteraciones y sigue iterando. Después, élite con las mismas reglas que la rejilla (las deterministas de las 3 mejores α y, para cada α y aleatorización, las 3 mejores ejecuciones; con empates y sin duplicados) y búsqueda local (2-opt + inserción). Tardó 2836 s. La instancia p2.4.a se añadió después con `--resume`, sin repetir las demás; el tiempo suma las dos ejecuciones.
- **Origen:** de qué élite sale la solución final; si una ejecución de GRASP repite una determinista, cuenta como determinista.
- **¿La búsqueda local mejora?** Sí, si la final supera a la mejor sin búsqueda local de la instancia. *Antes* e *Insertados* se refieren a la élite de la que sale la final.

## Resumen

| | Recompensa total | Distancia al BKS (sobre el total) | Distancia media al BKS por instancia (la del paper) |
|---|---|---|---|
| BKS | 10685 | — | — |
| Panadero, 1 hilo | 10440 | 2.29 % | 1.73 % |
| Panadero, 128 hilos | 10615 | 0.66 % | 0.44 % |
| Rejilla: sin búsqueda local | 10395 | 2.71 % | 1.99 % |
| Rejilla: final (con búsqueda local) | 10635 | 0.47 % | 0.34 % |
| `panadero_1s.csv`: sin búsqueda local | 10470 | 2.01 % | 1.49 % |
| `panadero_1s.csv`: final (con búsqueda local) | 10585 | 0.94 % | 0.71 % |
| 1 s, rejilla de α: una ejecución geometric con el mejor α (media de 10), sin búsqueda local | 10344 | 3.19 % | 2.43 % |
| 1 s, rejilla de α: una ejecución softmax con el mejor α (media de 10), sin búsqueda local | 10346 | 3.17 % | 2.29 % |
| 1 s, rejilla de α: mejor de las 420 ejecuciones, sin búsqueda local | 10575 | 1.03 % | 0.71 % |
| **1 s, rejilla de α: final (con búsqueda local)** | 10665 | 0.19 % | 0.15 % |

*Una ejecución (media)* es lo que da, de media, una sola ejecución de 1 s con el mejor α (con 1 s es lo comparable con Panadero con 1 hilo). La distancia media por instancia es la medida de la fila *Average* de las tablas del paper; la del total da más peso a las instancias con más recompensa.

- **1 s, rejilla de α final frente a Panadero con 128 hilos:** mejor en 12, igual en 65 y peor en 4 instancias.
- **1 s, rejilla de α sin búsqueda local frente a Panadero con 128 hilos:** mejor en 4, igual en 67 y peor en 10 instancias.
- **Rejilla final frente a Panadero con 128 hilos:** mejor en 10, igual en 65 y peor en 6 instancias.
- **1 s, rejilla de α final frente a la rejilla final:** mejor en 5, igual en 76 y peor en 0 instancias.
- **1 s, rejilla de α final frente a `panadero_1s.csv` final:** mejor en 14, igual en 67 y peor en 0 instancias.
- **BKS alcanzado:** 1 s, rejilla de α final en 77, rejilla final en 73, Panadero con 128 hilos en 69 de 81. Ninguna solución nuestra lo supera.
- **La búsqueda local mejora** la mejor solución de 1 s, rejilla de α en 13 instancias; en 6 de ellas es lo que hace alcanzar o superar a Panadero con 128 hilos.
- **Iteraciones en 1 s** (media por ejecución, con los 12 hilos ocupados): 40563 en p1 y 63600 en p2, frente a las 10 de la rejilla.

## Según el origen de la solución final de 1 s, rejilla de α

| Origen | Instancias | La búsqueda local mejora | Mejor que Panadero 128 | Igual | Peor |
|---|---|---|---|---|---|
| determinista | 54 | 0 | 2 | 51 | 1 |
| geometric | 20 | 7 | 7 | 10 | 3 |
| softmax | 7 | 6 | 3 | 4 | 0 |

## Por instancia

*α* es el de la élite de la que sale la solución final.

| Instancia | BKS | Panadero 1 hilo | Panadero 128 hilos | Rejilla final | `panadero_1s.csv` final | 1 s, rejilla de α sin búsqueda local | **1 s, rejilla de α final** | Origen | α | Ejecución | Antes | Insertados | ¿La búsqueda local mejora? | Frente a Panadero 128 | Frente a BKS | Iteraciones por ejecución |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| p1.2.b | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 201694 |
| p1.2.c | 20 | 20 | 20 | 20 | 20 | 20 | **20** | determinista | 0.00 | — | 20 | 0 | no | igual | igual | 138236 |
| p1.2.d | 30 | 30 | 30 | 30 | 30 | 30 | **30** | determinista | 0.00 | — | 30 | 0 | no | igual | igual | 66583 |
| p1.2.e | 45 | 45 | 45 | 45 | 45 | 45 | **45** | determinista | 0.00 | — | 40 | 1 | no | igual | igual | 16340 |
| p1.2.f | 80 | 80 | 80 | 80 | 80 | 80 | **80** | determinista | 0.90 | — | 75 | 1 | no | igual | igual | 11345 |
| p1.2.g | 90 | 85 | 90 | 90 | 90 | 90 | **90** | geometric | 0.00 | 1 | 75 | 2 | no | igual | igual | 5790 |
| p1.2.h | 110 | 105 | 110 | 110 | 110 | 110 | **110** | geometric | 0.30 | 0 | 100 | 2 | no | igual | igual | 4674 |
| p1.2.i | 135 | 130 | 135 | 130 | 130 | 130 | **130** | determinista | 0.05 | — | 120 | 1 | no | peor (-5) | -5 | 4264 |
| p1.2.j | 155 | 150 | 155 | 155 | 155 | 155 | **155** | geometric | 0.05 | 3 | 125 | 4 | no | igual | igual | 3998 |
| p1.2.k | 175 | 170 | 175 | 175 | 175 | 175 | **175** | determinista | 0.05 | — | 160 | 3 | no | igual | igual | 4074 |
| p1.2.l | 195 | 185 | 195 | 190 | 190 | 190 | **190** | geometric | 0.00 | 3 | 155 | 5 | no | peor (-5) | -5 | 3968 |
| p1.2.m | 215 | 200 | 215 | 205 | 210 | 205 | **210** | geometric | 0.40 | 4 | 205 | 1 | sí (+5) | peor (-5) | -5 | 4013 |
| p1.2.n | 235 | 230 | 235 | 235 | 230 | 235 | **235** | geometric | 0.05 | 2 | 235 | 0 | no | igual | igual | 3925 |
| p1.2.o | 240 | 235 | 235 | 240 | 240 | 240 | **240** | determinista | 0.55 | — | 235 | 1 | no | mejor (+5) | igual | 4036 |
| p1.2.p | 250 | 240 | 245 | 250 | 245 | 245 | **250** | geometric | 0.15 | 2 | 240 | 2 | sí (+5) | mejor (+5) | igual | 4005 |
| p1.2.q | 265 | 250 | 255 | 265 | 260 | 255 | **265** | softmax | 0.05 | 0 | 225 | 5 | sí (+10) | mejor (+10) | igual | 3887 |
| p1.2.r | 280 | 275 | 275 | 280 | 275 | 280 | **280** | geometric | 0.80 | 0 | 275 | 1 | no | mejor (+5) | igual | 4008 |
| p1.3.c | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 196642 |
| p1.3.d | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 205662 |
| p1.3.e | 30 | 30 | 30 | 30 | 30 | 30 | **30** | determinista | 0.00 | — | 30 | 0 | no | igual | igual | 104809 |
| p1.3.f | 40 | 40 | 40 | 40 | 40 | 40 | **40** | determinista | 0.00 | — | 40 | 0 | no | igual | igual | 63210 |
| p1.3.g | 50 | 50 | 50 | 50 | 50 | 50 | **50** | determinista | 0.00 | — | 50 | 0 | no | igual | igual | 23959 |
| p1.3.h | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 12629 |
| p1.3.i | 105 | 95 | 105 | 100 | 100 | 105 | **105** | softmax | 1.00 | 1 | 105 | 0 | no | igual | igual | 6777 |
| p1.3.j | 115 | 110 | 115 | 115 | 110 | 110 | **115** | softmax | 0.10 | 3 | 105 | 2 | sí (+5) | igual | igual | 5746 |
| p1.3.k | 135 | 125 | 130 | 135 | 130 | 130 | **135** | geometric | 0.05 | 4 | 130 | 1 | sí (+5) | mejor (+5) | igual | 5635 |
| p1.3.l | 155 | 155 | 155 | 150 | 150 | 150 | **150** | geometric | 0.20 | 8 | 135 | 3 | no | peor (-5) | -5 | 4509 |
| p1.3.m | 175 | 160 | 170 | 175 | 170 | 170 | **175** | geometric | 0.05 | 2 | 160 | 2 | sí (+5) | mejor (+5) | igual | 4347 |
| p1.3.n | 190 | 185 | 190 | 190 | 190 | 190 | **190** | determinista | 0.05 | — | 175 | 2 | no | igual | igual | 4397 |
| p1.3.o | 205 | 205 | 205 | 205 | 205 | 205 | **205** | determinista | 0.05 | — | 190 | 3 | no | igual | igual | 4232 |
| p1.3.p | 220 | 215 | 215 | 220 | 220 | 210 | **220** | geometric | 0.05 | 0 | 180 | 5 | sí (+10) | mejor (+5) | igual | 3819 |
| p1.3.q | 230 | 225 | 230 | 225 | 225 | 225 | **230** | softmax | 0.95 | 4 | 225 | 1 | sí (+5) | igual | igual | 3957 |
| p1.3.r | 250 | 240 | 240 | 240 | 245 | 240 | **250** | softmax | 0.00 | 0 | 195 | 7 | sí (+10) | mejor (+10) | igual | 4037 |
| p1.4.d | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 211845 |
| p1.4.e | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 217633 |
| p1.4.f | 25 | 25 | 25 | 25 | 25 | 25 | **25** | determinista | 0.00 | — | 25 | 0 | no | igual | igual | 130509 |
| p1.4.g | 35 | 35 | 35 | 35 | 35 | 35 | **35** | determinista | 0.00 | — | 35 | 0 | no | igual | igual | 81805 |
| p1.4.h | 45 | 45 | 45 | 45 | 45 | 45 | **45** | determinista | 0.00 | — | 45 | 0 | no | igual | igual | 65677 |
| p1.4.i | 60 | 60 | 60 | 60 | 60 | 60 | **60** | determinista | 0.05 | — | 60 | 0 | no | igual | igual | 28992 |
| p1.4.j | 75 | 75 | 75 | 75 | 75 | 75 | **75** | determinista | 0.00 | — | 70 | 1 | no | igual | igual | 16399 |
| p1.4.k | 100 | 100 | 100 | 100 | 100 | 100 | **100** | geometric | 0.00 | 1 | 95 | 1 | no | igual | igual | 12152 |
| p1.4.l | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.05 | — | 120 | 0 | no | igual | igual | 11151 |
| p1.4.m | 130 | 120 | 125 | 130 | 130 | 130 | **130** | geometric | 0.05 | 0 | 120 | 2 | no | mejor (+5) | igual | 5768 |
| p1.4.n | 155 | 145 | 150 | 155 | 155 | 150 | **155** | geometric | 0.90 | 2 | 140 | 2 | sí (+5) | mejor (+5) | igual | 5642 |
| p1.4.o | 165 | 160 | 165 | 165 | 160 | 160 | **165** | geometric | 0.00 | 0 | 145 | 2 | sí (+5) | igual | igual | 5778 |
| p1.4.p | 175 | 175 | 175 | 175 | 175 | 175 | **175** | determinista | 0.05 | — | 160 | 2 | no | igual | igual | 5623 |
| p1.4.q | 190 | 180 | 185 | 185 | 180 | 180 | **190** | softmax | 0.45 | 2 | 170 | 4 | sí (+10) | mejor (+5) | igual | 4564 |
| p1.4.r | 210 | 185 | 210 | 210 | 205 | 200 | **210** | softmax | 1.00 | 8 | 185 | 3 | sí (+10) | igual | igual | 4282 |
| p2.2.a | 90 | 90 | 90 | 90 | 90 | 90 | **90** | geometric | 0.00 | 0 | 90 | 0 | no | igual | igual | 71630 |
| p2.2.b | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 55925 |
| p2.2.c | 140 | 130 | 140 | 140 | 130 | 140 | **140** | geometric | 0.70 | 1 | 130 | 1 | no | igual | igual | 28702 |
| p2.2.d | 160 | 155 | 160 | 160 | 160 | 160 | **160** | geometric | 0.05 | 1 | 135 | 2 | no | igual | igual | 25770 |
| p2.2.e | 190 | 190 | 190 | 190 | 190 | 190 | **190** | determinista | 0.05 | — | 190 | 0 | no | igual | igual | 25092 |
| p2.2.f | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.05 | — | 200 | 0 | no | igual | igual | 24820 |
| p2.2.g | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 24996 |
| p2.2.h | 230 | 230 | 230 | 230 | 230 | 230 | **230** | determinista | 0.45 | — | 215 | 1 | no | igual | igual | 21677 |
| p2.2.i | 230 | 230 | 230 | 230 | 230 | 230 | **230** | determinista | 0.05 | — | 230 | 0 | no | igual | igual | 21973 |
| p2.2.j | 260 | 260 | 260 | 260 | 260 | 260 | **260** | determinista | 0.85 | — | 225 | 2 | no | igual | igual | 19008 |
| p2.2.k | 275 | 265 | 275 | 275 | 275 | 275 | **275** | geometric | 0.05 | 0 | 255 | 2 | no | igual | igual | 14851 |
| p2.3.a | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 119878 |
| p2.3.b | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 118188 |
| p2.3.c | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 72277 |
| p2.3.d | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 72185 |
| p2.3.e | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 55633 |
| p2.3.f | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 56725 |
| p2.3.g | 145 | 140 | 140 | 145 | 145 | 145 | **145** | determinista | 0.60 | — | 130 | 1 | no | mejor (+5) | igual | 38258 |
| p2.3.h | 165 | 165 | 165 | 165 | 165 | 165 | **165** | determinista | 0.05 | — | 165 | 0 | no | igual | igual | 31024 |
| p2.3.i | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.60 | — | 200 | 0 | no | igual | igual | 25407 |
| p2.3.j | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 25573 |
| p2.3.k | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 26063 |
| p2.4.a | 10 | 10 | 10 | 10 | 10 | 10 | **10** | determinista | 0.00 | — | 10 | 0 | no | igual | igual | 300750 |
| p2.4.b | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 117157 |
| p2.4.c | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 121499 |
| p2.4.d | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 121066 |
| p2.4.e | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 118563 |
| p2.4.f | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 72304 |
| p2.4.g | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 73469 |
| p2.4.h | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 56224 |
| p2.4.i | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 55888 |
| p2.4.j | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 56485 |
| p2.4.k | 180 | 180 | 180 | 180 | 180 | 180 | **180** | determinista | 0.80 | — | 180 | 0 | no | igual | igual | 29742 |

Ganancia total de 1 s, rejilla de α final sobre la rejilla final: +0.28 %.
