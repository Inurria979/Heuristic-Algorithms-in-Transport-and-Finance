# Comparación con Panadero et al. (2021), conjuntos p1 y p2: 100 iteraciones, rejilla de α

Informe generado por `pjs` (`TOP/src/rust`); se regenera con

```
cd TOP
ignore/target/release/pjs.exe panadero --alpha grid --iters 100 --runs 10 --geometric 0.3 --softmax 1 --seed 1 --best-csv results/panadero_1s_grid.csv \
    --report docs/comparacion_panadero_rejilla_100it.md src/data/top-data src/data/panadero2021_p1_p2.csv results/best_solution.csv results
```

## Qué se compara

Solo las 81 instancias de las tablas 3 y 4 de Panadero et al. (2021), extraídas en `src/data/panadero2021_p1_p2.csv`.

- **BKS:** la mejor solución conocida de la literatura (columna [a] de las tablas).
- **Panadero, 1 hilo / 128 hilos:** su algoritmo (ahorros con aleatorización sesgada, multiarranque), en el que cada hilo itera durante 1 s; con 128 hilos se queda con el mejor (*OBS*).
- **Rejilla:** nuestro experimento principal ([`informe_busqueda_local.md`](informe_busqueda_local.md)): 21 α, 10 ejecuciones de 10 iteraciones por α y aleatorización, élite y búsqueda local.
- **100 iteraciones, rejilla de α:** para cada instancia, **cada uno de los 21 α de la rejilla**, con **10 ejecuciones de GRASP geometric (β = 0.3) y 10 softmax (τ = 1) por α, cada una de 100 iteraciones** (construcciones aleatorias). Con un número fijo de iteraciones el experimento es reproducible. Cada ejecución usa la misma semilla que en la rejilla, así que empieza igual que la de 10 iteraciones y sigue iterando. Después, élite con las mismas reglas que la rejilla (las deterministas de las 3 mejores α y, para cada α y aleatorización, las 3 mejores ejecuciones; con empates y sin duplicados) y búsqueda local (2-opt + inserción). Tardó 29 s. La instancia p2.4.a se añadió después con `--resume`, sin repetir las demás; el tiempo suma las dos ejecuciones.
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
| `panadero_1s_grid.csv`: sin búsqueda local | 10575 | 1.03 % | 0.71 % |
| `panadero_1s_grid.csv`: final (con búsqueda local) | 10665 | 0.19 % | 0.15 % |
| 100 iteraciones, rejilla de α: una ejecución geometric con el mejor α (media de 10), sin búsqueda local | 10234.5 | 4.22 % | 3.28 % |
| 100 iteraciones, rejilla de α: una ejecución softmax con el mejor α (media de 10), sin búsqueda local | 10187.5 | 4.66 % | 3.55 % |
| 100 iteraciones, rejilla de α: mejor de las 420 ejecuciones, sin búsqueda local | 10470 | 2.01 % | 1.45 % |
| **100 iteraciones, rejilla de α: final (con búsqueda local)** | 10645 | 0.37 % | 0.29 % |

*Una ejecución (media)* es lo que da, de media, una sola ejecución de 100 iteraciones con el mejor α (con 1 s es lo comparable con Panadero con 1 hilo). La distancia media por instancia es la medida de la fila *Average* de las tablas del paper; la del total da más peso a las instancias con más recompensa.

- **100 iteraciones, rejilla de α final frente a Panadero con 128 hilos:** mejor en 12, igual en 62 y peor en 7 instancias.
- **100 iteraciones, rejilla de α sin búsqueda local frente a Panadero con 128 hilos:** mejor en 3, igual en 57 y peor en 21 instancias.
- **Rejilla final frente a Panadero con 128 hilos:** mejor en 10, igual en 65 y peor en 6 instancias.
- **100 iteraciones, rejilla de α final frente a la rejilla final:** mejor en 2, igual en 78 y peor en 1 instancias.
- **100 iteraciones, rejilla de α final frente a `panadero_1s_grid.csv` final:** mejor en 0, igual en 77 y peor en 4 instancias.
- **BKS alcanzado:** 100 iteraciones, rejilla de α final en 74, rejilla final en 73, Panadero con 128 hilos en 69 de 81. Ninguna solución nuestra lo supera.
- **La búsqueda local mejora** la mejor solución de 100 iteraciones, rejilla de α en 19 instancias; en 14 de ellas es lo que hace alcanzar o superar a Panadero con 128 hilos.

## Según el origen de la solución final de 100 iteraciones, rejilla de α

| Origen | Instancias | La búsqueda local mejora | Mejor que Panadero 128 | Igual | Peor |
|---|---|---|---|---|---|
| determinista | 56 | 2 | 2 | 51 | 3 |
| geometric | 21 | 13 | 7 | 10 | 4 |
| softmax | 4 | 4 | 3 | 1 | 0 |

## Por instancia

*α* es el de la élite de la que sale la solución final.

| Instancia | BKS | Panadero 1 hilo | Panadero 128 hilos | Rejilla final | `panadero_1s_grid.csv` final | 100 iteraciones, rejilla de α sin búsqueda local | **100 iteraciones, rejilla de α final** | Origen | α | Ejecución | Antes | Insertados | ¿La búsqueda local mejora? | Frente a Panadero 128 | Frente a BKS | Iteraciones por ejecución |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| p1.2.b | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 100 |
| p1.2.c | 20 | 20 | 20 | 20 | 20 | 20 | **20** | determinista | 0.00 | — | 20 | 0 | no | igual | igual | 100 |
| p1.2.d | 30 | 30 | 30 | 30 | 30 | 30 | **30** | determinista | 0.00 | — | 30 | 0 | no | igual | igual | 100 |
| p1.2.e | 45 | 45 | 45 | 45 | 45 | 45 | **45** | determinista | 0.00 | — | 40 | 1 | no | igual | igual | 100 |
| p1.2.f | 80 | 80 | 80 | 80 | 80 | 80 | **80** | determinista | 0.90 | — | 75 | 1 | no | igual | igual | 100 |
| p1.2.g | 90 | 85 | 90 | 90 | 90 | 85 | **90** | geometric | 0.00 | 1 | 75 | 2 | sí (+5) | igual | igual | 100 |
| p1.2.h | 110 | 105 | 110 | 110 | 110 | 110 | **110** | geometric | 0.05 | 2 | 100 | 2 | no | igual | igual | 100 |
| p1.2.i | 135 | 130 | 135 | 130 | 130 | 130 | **130** | determinista | 0.05 | — | 120 | 1 | no | peor (-5) | -5 | 100 |
| p1.2.j | 155 | 150 | 155 | 155 | 155 | 145 | **155** | geometric | 0.05 | 3 | 125 | 4 | sí (+10) | igual | igual | 100 |
| p1.2.k | 175 | 170 | 175 | 175 | 175 | 170 | **175** | determinista | 0.05 | — | 160 | 3 | sí (+5) | igual | igual | 100 |
| p1.2.l | 195 | 185 | 195 | 190 | 190 | 190 | **190** | geometric | 0.00 | 5 | 150 | 5 | no | peor (-5) | -5 | 100 |
| p1.2.m | 215 | 200 | 215 | 205 | 210 | 205 | **205** | geometric | 0.00 | 2 | 150 | 7 | no | peor (-10) | -10 | 100 |
| p1.2.n | 235 | 230 | 235 | 235 | 235 | 235 | **235** | geometric | 0.10 | 2 | 230 | 1 | no | igual | igual | 100 |
| p1.2.o | 240 | 235 | 235 | 240 | 240 | 240 | **240** | determinista | 0.55 | — | 235 | 1 | no | mejor (+5) | igual | 100 |
| p1.2.p | 250 | 240 | 245 | 250 | 250 | 240 | **250** | geometric | 0.15 | 7 | 235 | 3 | sí (+10) | mejor (+5) | igual | 100 |
| p1.2.q | 265 | 250 | 255 | 265 | 265 | 255 | **265** | softmax | 0.05 | 0 | 225 | 5 | sí (+10) | mejor (+10) | igual | 100 |
| p1.2.r | 280 | 275 | 275 | 280 | 280 | 275 | **280** | geometric | 0.80 | 0 | 275 | 1 | sí (+5) | mejor (+5) | igual | 100 |
| p1.3.c | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 100 |
| p1.3.d | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 100 |
| p1.3.e | 30 | 30 | 30 | 30 | 30 | 30 | **30** | determinista | 0.00 | — | 30 | 0 | no | igual | igual | 100 |
| p1.3.f | 40 | 40 | 40 | 40 | 40 | 40 | **40** | determinista | 0.00 | — | 40 | 0 | no | igual | igual | 100 |
| p1.3.g | 50 | 50 | 50 | 50 | 50 | 50 | **50** | determinista | 0.00 | — | 50 | 0 | no | igual | igual | 100 |
| p1.3.h | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 100 |
| p1.3.i | 105 | 95 | 105 | 100 | 105 | 100 | **100** | determinista | 0.05 | — | 95 | 1 | no | peor (-5) | -5 | 100 |
| p1.3.j | 115 | 110 | 115 | 115 | 115 | 105 | **115** | softmax | 0.45 | 6 | 105 | 2 | sí (+10) | igual | igual | 100 |
| p1.3.k | 135 | 125 | 130 | 135 | 135 | 130 | **135** | geometric | 0.05 | 3 | 125 | 2 | sí (+5) | mejor (+5) | igual | 100 |
| p1.3.l | 155 | 155 | 155 | 150 | 150 | 145 | **150** | geometric | 0.10 | 9 | 130 | 3 | sí (+5) | peor (-5) | -5 | 100 |
| p1.3.m | 175 | 160 | 170 | 175 | 175 | 165 | **175** | geometric | 0.05 | 4 | 155 | 3 | sí (+10) | mejor (+5) | igual | 100 |
| p1.3.n | 190 | 185 | 190 | 190 | 190 | 190 | **190** | determinista | 0.05 | — | 175 | 2 | no | igual | igual | 100 |
| p1.3.o | 205 | 205 | 205 | 205 | 205 | 200 | **205** | determinista | 0.05 | — | 190 | 3 | sí (+5) | igual | igual | 100 |
| p1.3.p | 220 | 215 | 215 | 220 | 220 | 205 | **220** | geometric | 0.05 | 1 | 175 | 6 | sí (+15) | mejor (+5) | igual | 100 |
| p1.3.q | 230 | 225 | 230 | 225 | 230 | 225 | **225** | determinista | 0.85 | — | 210 | 2 | no | peor (-5) | -5 | 100 |
| p1.3.r | 250 | 240 | 240 | 240 | 250 | 230 | **250** | softmax | 0.35 | 8 | 220 | 5 | sí (+20) | mejor (+10) | igual | 100 |
| p1.4.d | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 100 |
| p1.4.e | 15 | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 100 |
| p1.4.f | 25 | 25 | 25 | 25 | 25 | 25 | **25** | determinista | 0.00 | — | 25 | 0 | no | igual | igual | 100 |
| p1.4.g | 35 | 35 | 35 | 35 | 35 | 35 | **35** | determinista | 0.00 | — | 35 | 0 | no | igual | igual | 100 |
| p1.4.h | 45 | 45 | 45 | 45 | 45 | 45 | **45** | determinista | 0.00 | — | 45 | 0 | no | igual | igual | 100 |
| p1.4.i | 60 | 60 | 60 | 60 | 60 | 60 | **60** | determinista | 0.05 | — | 60 | 0 | no | igual | igual | 100 |
| p1.4.j | 75 | 75 | 75 | 75 | 75 | 75 | **75** | determinista | 0.00 | — | 70 | 1 | no | igual | igual | 100 |
| p1.4.k | 100 | 100 | 100 | 100 | 100 | 100 | **100** | geometric | 0.00 | 1 | 95 | 1 | no | igual | igual | 100 |
| p1.4.l | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.05 | — | 120 | 0 | no | igual | igual | 100 |
| p1.4.m | 130 | 120 | 125 | 130 | 130 | 130 | **130** | geometric | 0.05 | 0 | 120 | 2 | no | mejor (+5) | igual | 100 |
| p1.4.n | 155 | 145 | 150 | 155 | 155 | 145 | **155** | geometric | 0.80 | 1 | 135 | 3 | sí (+10) | mejor (+5) | igual | 100 |
| p1.4.o | 165 | 160 | 165 | 165 | 165 | 160 | **165** | geometric | 0.00 | 1 | 140 | 3 | sí (+5) | igual | igual | 100 |
| p1.4.p | 175 | 175 | 175 | 175 | 175 | 175 | **175** | determinista | 0.05 | — | 160 | 2 | no | igual | igual | 100 |
| p1.4.q | 190 | 180 | 185 | 185 | 190 | 175 | **190** | softmax | 0.10 | 6 | 160 | 6 | sí (+15) | mejor (+5) | igual | 100 |
| p1.4.r | 210 | 185 | 210 | 210 | 210 | 190 | **205** | geometric | 0.80 | 3 | 180 | 3 | sí (+15) | peor (-5) | -5 | 100 |
| p2.2.a | 90 | 90 | 90 | 90 | 90 | 90 | **90** | geometric | 0.00 | 0 | 90 | 0 | no | igual | igual | 100 |
| p2.2.b | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 100 |
| p2.2.c | 140 | 130 | 140 | 140 | 140 | 140 | **140** | geometric | 0.80 | 2 | 130 | 1 | no | igual | igual | 100 |
| p2.2.d | 160 | 155 | 160 | 160 | 160 | 155 | **160** | geometric | 0.05 | 1 | 135 | 2 | sí (+5) | igual | igual | 100 |
| p2.2.e | 190 | 190 | 190 | 190 | 190 | 190 | **190** | determinista | 0.05 | — | 190 | 0 | no | igual | igual | 100 |
| p2.2.f | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.05 | — | 200 | 0 | no | igual | igual | 100 |
| p2.2.g | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 100 |
| p2.2.h | 230 | 230 | 230 | 230 | 230 | 230 | **230** | determinista | 0.45 | — | 215 | 1 | no | igual | igual | 100 |
| p2.2.i | 230 | 230 | 230 | 230 | 230 | 230 | **230** | determinista | 0.05 | — | 230 | 0 | no | igual | igual | 100 |
| p2.2.j | 260 | 260 | 260 | 260 | 260 | 260 | **260** | determinista | 0.85 | — | 225 | 2 | no | igual | igual | 100 |
| p2.2.k | 275 | 265 | 275 | 275 | 275 | 265 | **275** | geometric | 0.40 | 0 | 255 | 2 | sí (+10) | igual | igual | 100 |
| p2.3.a | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 100 |
| p2.3.b | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 100 |
| p2.3.c | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 100 |
| p2.3.d | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 100 |
| p2.3.e | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 100 |
| p2.3.f | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 100 |
| p2.3.g | 145 | 140 | 140 | 145 | 145 | 145 | **145** | determinista | 0.60 | — | 130 | 1 | no | mejor (+5) | igual | 100 |
| p2.3.h | 165 | 165 | 165 | 165 | 165 | 165 | **165** | determinista | 0.05 | — | 165 | 0 | no | igual | igual | 100 |
| p2.3.i | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.60 | — | 200 | 0 | no | igual | igual | 100 |
| p2.3.j | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 100 |
| p2.3.k | 200 | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 100 |
| p2.4.a | 10 | 10 | 10 | 10 | 10 | 10 | **10** | determinista | 0.00 | — | 10 | 0 | no | igual | igual | 100 |
| p2.4.b | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 100 |
| p2.4.c | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 100 |
| p2.4.d | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 100 |
| p2.4.e | 70 | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 100 |
| p2.4.f | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 100 |
| p2.4.g | 105 | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 100 |
| p2.4.h | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 100 |
| p2.4.i | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 100 |
| p2.4.j | 120 | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 100 |
| p2.4.k | 180 | 180 | 180 | 180 | 180 | 180 | **180** | determinista | 0.80 | — | 180 | 0 | no | igual | igual | 100 |

Ganancia total de 100 iteraciones, rejilla de α final sobre la rejilla final: +0.09 %.
