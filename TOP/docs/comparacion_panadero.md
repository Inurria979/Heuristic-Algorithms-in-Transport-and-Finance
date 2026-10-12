# Comparación con Panadero et al. (2021), conjuntos p1 y p2: 1 s, mejor α

Informe generado por `pjs` (`TOP/src/rust`); se regenera con

```
cd TOP
ignore/target/release/pjs.exe panadero --alpha best --time 1 --runs 10 --geometric 0.3 --softmax 1 --seed 1 \
    --report docs/comparacion_panadero.md src/data/top-data src/data/panadero2021_p1_p2.csv results/best_solution.csv results
```

## Qué se compara

Solo las 81 instancias de las tablas 3 y 4 de Panadero et al. (2021), extraídas en `src/data/panadero2021_p1_p2.csv`.

- **BKS:** la mejor solución conocida de la literatura (columna [a] de las tablas).
- **Panadero, 1 hilo / 128 hilos:** su algoritmo (ahorros con aleatorización sesgada, multiarranque), en el que cada hilo itera durante 1 s; con 128 hilos se queda con el mejor (*OBS*).
- **Rejilla:** nuestro experimento principal ([`informe_busqueda_local.md`](informe_busqueda_local.md)): 21 α, 10 ejecuciones de 10 iteraciones por α y aleatorización, élite y búsqueda local.
- **1 s, mejor α:** para cada instancia, el mejor α determinista de la rejilla (el menor si empatan), con **10 ejecuciones de GRASP geometric (β = 0.3) y 10 softmax (τ = 1) por α, cada una iterando durante 1 s de reloj**. Las 1620 ejecuciones se reparten entre 12 hilos en un equipo de 6 núcleos físicos (Ryzen 5 5500U), así que cada ejecución hace menos iteraciones que si estuviera sola. Por usar tiempo de reloj, este experimento **no es reproducible exactamente**: el número de iteraciones depende de la carga de la máquina. Después, élite con las mismas reglas que la rejilla (la determinista de ese α y las 3 mejores ejecuciones de cada aleatorización; con empates y sin duplicados) y búsqueda local (2-opt + inserción). Tardó 136 s. La instancia p2.4.a se añadió después con `--resume`, sin repetir las demás; el tiempo suma las dos ejecuciones.
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
| 1 s, mejor α: una ejecución geometric con el mejor α (media de 10), sin búsqueda local | 10339.5 | 3.23 % | 2.45 % |
| 1 s, mejor α: una ejecución softmax con el mejor α (media de 10), sin búsqueda local | 10341.5 | 3.21 % | 2.33 % |
| 1 s, mejor α: mejor de las 20 ejecuciones, sin búsqueda local | 10470 | 2.01 % | 1.49 % |
| **1 s, mejor α: final (con búsqueda local)** | 10585 | 0.94 % | 0.71 % |

*Una ejecución (media)* es lo que da, de media, una sola ejecución de 1 s con el mejor α (con 1 s es lo comparable con Panadero con 1 hilo). La distancia media por instancia es la medida de la fila *Average* de las tablas del paper; la del total da más peso a las instancias con más recompensa.

- **1 s, mejor α final frente a Panadero con 128 hilos:** mejor en 7, igual en 62 y peor en 12 instancias.
- **1 s, mejor α sin búsqueda local frente a Panadero con 128 hilos:** mejor en 2, igual en 60 y peor en 19 instancias.
- **Rejilla final frente a Panadero con 128 hilos:** mejor en 10, igual en 65 y peor en 6 instancias.
- **1 s, mejor α final frente a la rejilla final:** mejor en 2, igual en 68 y peor en 11 instancias.
- **BKS alcanzado:** 1 s, mejor α final en 63, rejilla final en 73, Panadero con 128 hilos en 69 de 81. Ninguna solución nuestra lo supera.
- **La búsqueda local mejora** la mejor solución de 1 s, mejor α en 16 instancias; en 7 de ellas es lo que hace alcanzar o superar a Panadero con 128 hilos.
- **Iteraciones en 1 s** (media por ejecución, con los 12 hilos ocupados): 39324 en p1 y 60244 en p2, frente a las 10 de la rejilla.

## Según el origen de la solución final de 1 s, mejor α

| Origen | Instancias | La búsqueda local mejora | Mejor que Panadero 128 | Igual | Peor |
|---|---|---|---|---|---|
| determinista | 60 | 3 | 2 | 53 | 5 |
| geometric | 11 | 5 | 2 | 6 | 3 |
| softmax | 10 | 8 | 3 | 3 | 4 |

## Por instancia

*α* es el de la élite de la que sale la solución final.

| Instancia | BKS | Panadero 1 hilo | Panadero 128 hilos | Rejilla final | 1 s, mejor α sin búsqueda local | **1 s, mejor α final** | Origen | α | Ejecución | Antes | Insertados | ¿La búsqueda local mejora? | Frente a Panadero 128 | Frente a BKS | Iteraciones por ejecución |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| p1.2.b | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 193582 |
| p1.2.c | 20 | 20 | 20 | 20 | 20 | **20** | determinista | 0.00 | — | 20 | 0 | no | igual | igual | 137486 |
| p1.2.d | 30 | 30 | 30 | 30 | 30 | **30** | determinista | 0.00 | — | 30 | 0 | no | igual | igual | 62146 |
| p1.2.e | 45 | 45 | 45 | 45 | 45 | **45** | determinista | 0.00 | — | 40 | 1 | no | igual | igual | 16563 |
| p1.2.f | 80 | 80 | 80 | 80 | 80 | **80** | determinista | 0.90 | — | 75 | 1 | no | igual | igual | 11253 |
| p1.2.g | 90 | 85 | 90 | 90 | 85 | **90** | softmax | 0.05 | 4 | 85 | 1 | sí (+5) | igual | igual | 5836 |
| p1.2.h | 110 | 105 | 110 | 110 | 110 | **110** | geometric | 0.80 | 0 | 105 | 1 | no | igual | igual | 4642 |
| p1.2.i | 135 | 130 | 135 | 130 | 130 | **130** | determinista | 0.05 | — | 120 | 1 | no | peor (-5) | -5 | 4229 |
| p1.2.j | 155 | 150 | 155 | 155 | 150 | **155** | softmax | 0.90 | 0 | 145 | 2 | sí (+5) | igual | igual | 3942 |
| p1.2.k | 175 | 170 | 175 | 175 | 170 | **175** | geometric | 0.90 | 0 | 170 | 1 | sí (+5) | igual | igual | 3918 |
| p1.2.l | 195 | 185 | 195 | 190 | 185 | **190** | softmax | 0.85 | 0 | 185 | 1 | sí (+5) | peor (-5) | -5 | 3896 |
| p1.2.m | 215 | 200 | 215 | 205 | 205 | **210** | geometric | 0.55 | 1 | 205 | 1 | sí (+5) | peor (-5) | -5 | 3804 |
| p1.2.n | 235 | 230 | 235 | 235 | 230 | **230** | determinista | 0.55 | — | 230 | 0 | no | peor (-5) | -5 | 4003 |
| p1.2.o | 240 | 235 | 235 | 240 | 235 | **240** | determinista | 0.55 | — | 235 | 1 | sí (+5) | mejor (+5) | igual | 3989 |
| p1.2.p | 250 | 240 | 245 | 250 | 245 | **245** | determinista | 0.55 | — | 240 | 1 | no | igual | -5 | 3993 |
| p1.2.q | 265 | 250 | 255 | 265 | 255 | **260** | determinista | 0.55 | — | 245 | 2 | sí (+5) | mejor (+5) | -5 | 3906 |
| p1.2.r | 280 | 275 | 275 | 280 | 275 | **275** | determinista | 0.55 | — | 270 | 1 | no | igual | -5 | 4109 |
| p1.3.c | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 209476 |
| p1.3.d | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 194458 |
| p1.3.e | 30 | 30 | 30 | 30 | 30 | **30** | determinista | 0.00 | — | 30 | 0 | no | igual | igual | 98159 |
| p1.3.f | 40 | 40 | 40 | 40 | 40 | **40** | determinista | 0.00 | — | 40 | 0 | no | igual | igual | 53321 |
| p1.3.g | 50 | 50 | 50 | 50 | 50 | **50** | determinista | 0.00 | — | 50 | 0 | no | igual | igual | 24290 |
| p1.3.h | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 12248 |
| p1.3.i | 105 | 95 | 105 | 100 | 100 | **100** | determinista | 0.05 | — | 95 | 1 | no | peor (-5) | -5 | 7011 |
| p1.3.j | 115 | 110 | 115 | 115 | 105 | **110** | softmax | 0.05 | 0 | 100 | 2 | sí (+5) | peor (-5) | -5 | 5898 |
| p1.3.k | 135 | 125 | 130 | 135 | 130 | **130** | softmax | 0.90 | 0 | 130 | 0 | no | igual | -5 | 5548 |
| p1.3.l | 155 | 155 | 155 | 150 | 145 | **150** | geometric | 0.80 | 1 | 135 | 2 | sí (+5) | peor (-5) | -5 | 4443 |
| p1.3.m | 175 | 160 | 170 | 175 | 155 | **170** | determinista | 0.00 | — | 135 | 5 | sí (+15) | igual | -5 | 4205 |
| p1.3.n | 190 | 185 | 190 | 190 | 190 | **190** | determinista | 0.05 | — | 175 | 2 | no | igual | igual | 4271 |
| p1.3.o | 205 | 205 | 205 | 205 | 205 | **205** | determinista | 0.90 | — | 195 | 1 | no | igual | igual | 4414 |
| p1.3.p | 220 | 215 | 215 | 220 | 210 | **220** | geometric | 0.90 | 0 | 205 | 2 | sí (+10) | mejor (+5) | igual | 3980 |
| p1.3.q | 230 | 225 | 230 | 225 | 225 | **225** | determinista | 0.90 | — | 215 | 1 | no | peor (-5) | -5 | 3866 |
| p1.3.r | 250 | 240 | 240 | 240 | 240 | **245** | softmax | 0.90 | 4 | 230 | 2 | sí (+5) | mejor (+5) | -5 | 4047 |
| p1.4.d | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 203964 |
| p1.4.e | 15 | 15 | 15 | 15 | 15 | **15** | determinista | 0.00 | — | 15 | 0 | no | igual | igual | 201480 |
| p1.4.f | 25 | 25 | 25 | 25 | 25 | **25** | determinista | 0.00 | — | 25 | 0 | no | igual | igual | 129893 |
| p1.4.g | 35 | 35 | 35 | 35 | 35 | **35** | determinista | 0.00 | — | 35 | 0 | no | igual | igual | 78396 |
| p1.4.h | 45 | 45 | 45 | 45 | 45 | **45** | determinista | 0.00 | — | 45 | 0 | no | igual | igual | 62709 |
| p1.4.i | 60 | 60 | 60 | 60 | 60 | **60** | determinista | 0.05 | — | 60 | 0 | no | igual | igual | 29006 |
| p1.4.j | 75 | 75 | 75 | 75 | 75 | **75** | determinista | 0.00 | — | 70 | 1 | no | igual | igual | 17036 |
| p1.4.k | 100 | 100 | 100 | 100 | 100 | **100** | geometric | 0.05 | 0 | 100 | 0 | no | igual | igual | 11909 |
| p1.4.l | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.05 | — | 120 | 0 | no | igual | igual | 10339 |
| p1.4.m | 130 | 120 | 125 | 130 | 130 | **130** | geometric | 0.90 | 0 | 120 | 2 | no | mejor (+5) | igual | 5656 |
| p1.4.n | 155 | 145 | 150 | 155 | 145 | **155** | softmax | 0.05 | 0 | 145 | 2 | sí (+10) | mejor (+5) | igual | 5644 |
| p1.4.o | 165 | 160 | 165 | 165 | 160 | **160** | determinista | 0.90 | — | 150 | 1 | no | peor (-5) | -5 | 5709 |
| p1.4.p | 175 | 175 | 175 | 175 | 175 | **175** | determinista | 0.90 | — | 165 | 1 | no | igual | igual | 5958 |
| p1.4.q | 190 | 180 | 185 | 185 | 175 | **180** | softmax | 0.90 | 4 | 175 | 1 | sí (+5) | peor (-5) | -10 | 4660 |
| p1.4.r | 210 | 185 | 210 | 210 | 190 | **205** | softmax | 0.70 | 3 | 185 | 3 | sí (+15) | peor (-5) | -5 | 4242 |
| p2.2.a | 90 | 90 | 90 | 90 | 90 | **90** | geometric | 0.00 | 0 | 90 | 0 | no | igual | igual | 60600 |
| p2.2.b | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 53338 |
| p2.2.c | 140 | 130 | 140 | 140 | 130 | **130** | geometric | 0.00 | 3 | 130 | 0 | no | peor (-10) | -10 | 28092 |
| p2.2.d | 160 | 155 | 160 | 160 | 160 | **160** | geometric | 1.00 | 1 | 160 | 0 | no | igual | igual | 24636 |
| p2.2.e | 190 | 190 | 190 | 190 | 190 | **190** | determinista | 0.05 | — | 190 | 0 | no | igual | igual | 24614 |
| p2.2.f | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.05 | — | 200 | 0 | no | igual | igual | 25364 |
| p2.2.g | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 26030 |
| p2.2.h | 230 | 230 | 230 | 230 | 230 | **230** | determinista | 0.95 | — | 230 | 0 | no | igual | igual | 20404 |
| p2.2.i | 230 | 230 | 230 | 230 | 230 | **230** | determinista | 0.05 | — | 230 | 0 | no | igual | igual | 20676 |
| p2.2.j | 260 | 260 | 260 | 260 | 260 | **260** | determinista | 1.00 | — | 260 | 0 | no | igual | igual | 17609 |
| p2.2.k | 275 | 265 | 275 | 275 | 265 | **275** | geometric | 0.45 | 0 | 255 | 2 | sí (+10) | igual | igual | 14011 |
| p2.3.a | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 115000 |
| p2.3.b | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 69319 |
| p2.3.c | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 60396 |
| p2.3.d | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 60962 |
| p2.3.e | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 51041 |
| p2.3.f | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 50337 |
| p2.3.g | 145 | 140 | 140 | 145 | 145 | **145** | softmax | 1.00 | 0 | 145 | 0 | no | mejor (+5) | igual | 33442 |
| p2.3.h | 165 | 165 | 165 | 165 | 165 | **165** | determinista | 0.05 | — | 165 | 0 | no | igual | igual | 29872 |
| p2.3.i | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.60 | — | 200 | 0 | no | igual | igual | 24401 |
| p2.3.j | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 24948 |
| p2.3.k | 200 | 200 | 200 | 200 | 200 | **200** | determinista | 0.00 | — | 200 | 0 | no | igual | igual | 25979 |
| p2.4.a | 10 | 10 | 10 | 10 | 10 | **10** | determinista | 0.00 | — | 10 | 0 | no | igual | igual | 371564 |
| p2.4.b | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 116351 |
| p2.4.c | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 81947 |
| p2.4.d | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 113324 |
| p2.4.e | 70 | 70 | 70 | 70 | 70 | **70** | determinista | 0.00 | — | 70 | 0 | no | igual | igual | 114892 |
| p2.4.f | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 68161 |
| p2.4.g | 105 | 105 | 105 | 105 | 105 | **105** | determinista | 0.00 | — | 105 | 0 | no | igual | igual | 70236 |
| p2.4.h | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 51314 |
| p2.4.i | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 55987 |
| p2.4.j | 120 | 120 | 120 | 120 | 120 | **120** | determinista | 0.00 | — | 120 | 0 | no | igual | igual | 53789 |
| p2.4.k | 180 | 180 | 180 | 180 | 180 | **180** | determinista | 0.80 | — | 180 | 0 | no | igual | igual | 29426 |

Ganancia total de 1 s, mejor α final sobre la rejilla final: -0.47 %.
