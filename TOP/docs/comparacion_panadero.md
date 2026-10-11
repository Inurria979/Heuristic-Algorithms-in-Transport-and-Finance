# Comparación con Panadero et al. (2021): conjuntos p1 y p2

Informe generado por `pjs` (`TOP/src/rust`); se regenera con

```
cd TOP
ignore/target/release/pjs.exe panadero --time 1 --runs 10 --geometric 0.3 --softmax 1 --seed 1 --report docs/comparacion_panadero.md \
    src/data/top-data src/data/panadero2021_p1_p2.csv results/best_solution.csv results
```

## Qué se compara

Las 80 instancias de las tablas 3 y 4 de Panadero et al. (2021), extraídas en `src/data/panadero2021_p1_p2.csv`.

- **BKS:** la mejor solución conocida de la literatura (columna [a] de las tablas).
- **Panadero, 1 hilo / 128 hilos:** su algoritmo (ahorros con aleatorización sesgada, multiarranque), en el que cada hilo itera durante 1 s; con 128 hilos se queda con el mejor (*OBS*).
- **Rejilla:** nuestro experimento principal ([`informe_busqueda_local.md`](informe_busqueda_local.md)): 21 α, 10 ejecuciones de 10 iteraciones por α y aleatorización, élite y búsqueda local.
- **1 s (como Panadero):** para cada instancia, el mejor α determinista de la rejilla (el menor si empatan) y **10 ejecuciones de GRASP geometric (β = 0.3) y 10 softmax (τ = 1), cada una iterando durante 1 s de reloj**. Después, élite con las mismas reglas que la rejilla (la determinista de ese α y las 3 mejores ejecuciones de cada aleatorización, con empates, sin duplicados) y búsqueda local (2-opt + inserción). Las 1600 ejecuciones se reparten entre 12 hilos en un equipo de 6 núcleos físicos (Ryzen 5 5500U), así que cada ejecución hace menos iteraciones que si estuviera sola. Por usar tiempo de reloj, este experimento **no es reproducible exactamente**: el número de iteraciones depende de la carga de la máquina. Tardó 134 s.
- **Origen:** de qué élite sale la solución final; si una ejecución de GRASP repite la determinista, cuenta como determinista.
- **¿La búsqueda local mejora?** Sí, si la final supera a la mejor sin búsqueda local de la instancia. *Antes* e *Insertados* se refieren a la élite de la que sale la final.

## Resumen

| | Recompensa total | Distancia al BKS (sobre el total) | Distancia media al BKS por instancia (la del paper) |
|---|---|---|---|
| BKS | 10675 | — | — |
| Panadero, 1 hilo | 10430 | 2.30 % | 1.75 % |
| Panadero, 128 hilos | 10605 | 0.66 % | 0.45 % |
| Rejilla: sin búsqueda local | 10385 | 2.72 % | 2.02 % |
| Rejilla: final (con búsqueda local) | 10625 | 0.47 % | 0.35 % |
| 1 s: una ejecución geometric (media de 10), sin búsqueda local | 10329.5 | 3.24 % | 2.48 % |
| 1 s: una ejecución softmax (media de 10), sin búsqueda local | 10331.5 | 3.22 % | 2.35 % |
| 1 s: mejor de las 20 ejecuciones, sin búsqueda local | 10460 | 2.01 % | 1.50 % |
| **1 s: final (con búsqueda local)** | 10575 | 0.94 % | 0.72 % |

*Una ejecución (media)* es lo comparable con Panadero con 1 hilo: lo que da, de media, una sola ejecución de 1 s. La distancia media por instancia es la medida de la fila *Average* de las tablas del paper; la del total da más peso a las instancias con más recompensa.

- **1 s final frente a Panadero con 128 hilos:** mejor en 7, igual en 61 y peor en 12 instancias.
- **1 s sin búsqueda local frente a Panadero con 128 hilos:** mejor en 2, igual en 59 y peor en 19 instancias.
- **Rejilla final frente a Panadero con 128 hilos:** mejor en 10, igual en 64 y peor en 6 instancias.
- **1 s final frente a la rejilla final:** mejor en 2, igual en 67 y peor en 11 instancias.
- **BKS alcanzado:** 1 s final en 62, rejilla final en 72, Panadero con 128 hilos en 68 de 80. Ninguna solución nuestra lo supera.
- **La búsqueda local mejora** la mejor solución de 1 s en 16 instancias; en 7 de ellas es lo que hace alcanzar o superar a Panadero con 128 hilos.
- **Iteraciones en 1 s** (media por ejecución, con los 12 hilos ocupados): 39324 en p1 y 50516 en p2, frente a las 10 de la rejilla.

## Según el origen de la solución final de 1 s

| Origen | Instancias | La búsqueda local mejora | Mejor que Panadero 128 | Igual | Peor |
|---|---|---|---|---|---|
| determinista | 59 | 3 | 2 | 52 | 5 |
| geometric | 11 | 5 | 2 | 6 | 3 |
| softmax | 10 | 8 | 3 | 3 | 4 |

## Por instancia

| Instancia | α | BKS | Panadero 1 hilo | Panadero 128 hilos | Rejilla final | 1 s: media geometric | 1 s sin búsqueda local | **1 s final** | Origen | Ejecución | Antes | Insertados | ¿La búsqueda local mejora? | Frente a Panadero 128 | Frente a BKS | Iteraciones por ejecución |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| p1.2.b | 0.00 | 15 | 15 | 15 | 15 | 15.0 | 15 | **15** | determinista | — | 15 | 0 | no | igual | igual | 193582 |
| p1.2.c | 0.00 | 20 | 20 | 20 | 20 | 20.0 | 20 | **20** | determinista | — | 20 | 0 | no | igual | igual | 137486 |
| p1.2.d | 0.00 | 30 | 30 | 30 | 30 | 30.0 | 30 | **30** | determinista | — | 30 | 0 | no | igual | igual | 62146 |
| p1.2.e | 0.00 | 45 | 45 | 45 | 45 | 45.0 | 45 | **45** | determinista | — | 40 | 1 | no | igual | igual | 16563 |
| p1.2.f | 0.90 | 80 | 80 | 80 | 80 | 80.0 | 80 | **80** | determinista | — | 75 | 1 | no | igual | igual | 11253 |
| p1.2.g | 0.05 | 90 | 85 | 90 | 90 | 85.0 | 85 | **90** | softmax | 4 | 85 | 1 | sí (+5) | igual | igual | 5836 |
| p1.2.h | 0.80 | 110 | 105 | 110 | 110 | 105.0 | 110 | **110** | geometric | 0 | 105 | 1 | no | igual | igual | 4642 |
| p1.2.i | 0.05 | 135 | 130 | 135 | 130 | 130.0 | 130 | **130** | determinista | — | 120 | 1 | no | peor (-5) | -5 | 4229 |
| p1.2.j | 0.90 | 155 | 150 | 155 | 155 | 145.0 | 150 | **155** | softmax | 0 | 145 | 2 | sí (+5) | igual | igual | 3942 |
| p1.2.k | 0.90 | 175 | 170 | 175 | 175 | 170.0 | 170 | **175** | geometric | 0 | 170 | 1 | sí (+5) | igual | igual | 3918 |
| p1.2.l | 0.85 | 195 | 185 | 195 | 190 | 185.0 | 185 | **190** | softmax | 0 | 185 | 1 | sí (+5) | peor (-5) | -5 | 3896 |
| p1.2.m | 0.55 | 215 | 200 | 215 | 205 | 201.0 | 205 | **210** | geometric | 1 | 205 | 1 | sí (+5) | peor (-5) | -5 | 3804 |
| p1.2.n | 0.55 | 235 | 230 | 235 | 235 | 230.0 | 230 | **230** | determinista | — | 230 | 0 | no | peor (-5) | -5 | 4003 |
| p1.2.o | 0.55 | 240 | 235 | 235 | 240 | 235.0 | 235 | **240** | determinista | — | 235 | 1 | sí (+5) | mejor (+5) | igual | 3989 |
| p1.2.p | 0.55 | 250 | 240 | 245 | 250 | 240.5 | 245 | **245** | determinista | — | 240 | 1 | no | igual | -5 | 3993 |
| p1.2.q | 0.55 | 265 | 250 | 255 | 265 | 250.5 | 255 | **260** | determinista | — | 245 | 2 | sí (+5) | mejor (+5) | -5 | 3906 |
| p1.2.r | 0.55 | 280 | 275 | 275 | 280 | 275.0 | 275 | **275** | determinista | — | 270 | 1 | no | igual | -5 | 4109 |
| p1.3.c | 0.00 | 15 | 15 | 15 | 15 | 15.0 | 15 | **15** | determinista | — | 15 | 0 | no | igual | igual | 209476 |
| p1.3.d | 0.00 | 15 | 15 | 15 | 15 | 15.0 | 15 | **15** | determinista | — | 15 | 0 | no | igual | igual | 194458 |
| p1.3.e | 0.00 | 30 | 30 | 30 | 30 | 30.0 | 30 | **30** | determinista | — | 30 | 0 | no | igual | igual | 98159 |
| p1.3.f | 0.00 | 40 | 40 | 40 | 40 | 40.0 | 40 | **40** | determinista | — | 40 | 0 | no | igual | igual | 53321 |
| p1.3.g | 0.00 | 50 | 50 | 50 | 50 | 50.0 | 50 | **50** | determinista | — | 50 | 0 | no | igual | igual | 24290 |
| p1.3.h | 0.00 | 70 | 70 | 70 | 70 | 70.0 | 70 | **70** | determinista | — | 70 | 0 | no | igual | igual | 12248 |
| p1.3.i | 0.05 | 105 | 95 | 105 | 100 | 95.0 | 100 | **100** | determinista | — | 95 | 1 | no | peor (-5) | -5 | 7011 |
| p1.3.j | 0.05 | 115 | 110 | 115 | 115 | 97.0 | 105 | **110** | softmax | 0 | 100 | 2 | sí (+5) | peor (-5) | -5 | 5898 |
| p1.3.k | 0.90 | 135 | 125 | 130 | 135 | 125.0 | 130 | **130** | softmax | 0 | 130 | 0 | no | igual | -5 | 5548 |
| p1.3.l | 0.80 | 155 | 155 | 155 | 150 | 135.0 | 145 | **150** | geometric | 1 | 135 | 2 | sí (+5) | peor (-5) | -5 | 4443 |
| p1.3.m | 0.00 | 175 | 160 | 170 | 175 | 149.5 | 155 | **170** | determinista | — | 135 | 5 | sí (+15) | igual | -5 | 4205 |
| p1.3.n | 0.05 | 190 | 185 | 190 | 190 | 189.0 | 190 | **190** | determinista | — | 175 | 2 | no | igual | igual | 4271 |
| p1.3.o | 0.90 | 205 | 205 | 205 | 205 | 200.0 | 205 | **205** | determinista | — | 195 | 1 | no | igual | igual | 4414 |
| p1.3.p | 0.90 | 220 | 215 | 215 | 220 | 206.0 | 210 | **220** | geometric | 0 | 205 | 2 | sí (+10) | mejor (+5) | igual | 3980 |
| p1.3.q | 0.90 | 230 | 225 | 230 | 225 | 225.0 | 225 | **225** | determinista | — | 215 | 1 | no | peor (-5) | -5 | 3866 |
| p1.3.r | 0.90 | 250 | 240 | 240 | 240 | 237.0 | 240 | **245** | softmax | 4 | 230 | 2 | sí (+5) | mejor (+5) | -5 | 4047 |
| p1.4.d | 0.00 | 15 | 15 | 15 | 15 | 15.0 | 15 | **15** | determinista | — | 15 | 0 | no | igual | igual | 203964 |
| p1.4.e | 0.00 | 15 | 15 | 15 | 15 | 15.0 | 15 | **15** | determinista | — | 15 | 0 | no | igual | igual | 201480 |
| p1.4.f | 0.00 | 25 | 25 | 25 | 25 | 25.0 | 25 | **25** | determinista | — | 25 | 0 | no | igual | igual | 129893 |
| p1.4.g | 0.00 | 35 | 35 | 35 | 35 | 35.0 | 35 | **35** | determinista | — | 35 | 0 | no | igual | igual | 78396 |
| p1.4.h | 0.00 | 45 | 45 | 45 | 45 | 45.0 | 45 | **45** | determinista | — | 45 | 0 | no | igual | igual | 62709 |
| p1.4.i | 0.05 | 60 | 60 | 60 | 60 | 60.0 | 60 | **60** | determinista | — | 60 | 0 | no | igual | igual | 29006 |
| p1.4.j | 0.00 | 75 | 75 | 75 | 75 | 75.0 | 75 | **75** | determinista | — | 70 | 1 | no | igual | igual | 17036 |
| p1.4.k | 0.05 | 100 | 100 | 100 | 100 | 100.0 | 100 | **100** | geometric | 0 | 100 | 0 | no | igual | igual | 11909 |
| p1.4.l | 0.05 | 120 | 120 | 120 | 120 | 120.0 | 120 | **120** | determinista | — | 120 | 0 | no | igual | igual | 10339 |
| p1.4.m | 0.90 | 130 | 120 | 125 | 130 | 119.5 | 130 | **130** | geometric | 0 | 120 | 2 | no | mejor (+5) | igual | 5656 |
| p1.4.n | 0.05 | 155 | 145 | 150 | 155 | 145.0 | 145 | **155** | softmax | 0 | 145 | 2 | sí (+10) | mejor (+5) | igual | 5644 |
| p1.4.o | 0.90 | 165 | 160 | 165 | 165 | 160.0 | 160 | **160** | determinista | — | 150 | 1 | no | peor (-5) | -5 | 5709 |
| p1.4.p | 0.90 | 175 | 175 | 175 | 175 | 172.0 | 175 | **175** | determinista | — | 165 | 1 | no | igual | igual | 5958 |
| p1.4.q | 0.90 | 190 | 180 | 185 | 185 | 163.5 | 175 | **180** | softmax | 4 | 175 | 1 | sí (+5) | peor (-5) | -10 | 4660 |
| p1.4.r | 0.70 | 210 | 185 | 210 | 210 | 175.0 | 190 | **205** | softmax | 3 | 185 | 3 | sí (+15) | peor (-5) | -5 | 4242 |
| p2.2.a | 0.00 | 90 | 90 | 90 | 90 | 90.0 | 90 | **90** | geometric | 0 | 90 | 0 | no | igual | igual | 60600 |
| p2.2.b | 0.00 | 120 | 120 | 120 | 120 | 120.0 | 120 | **120** | determinista | — | 120 | 0 | no | igual | igual | 53338 |
| p2.2.c | 0.00 | 140 | 130 | 140 | 140 | 125.5 | 130 | **130** | geometric | 3 | 130 | 0 | no | peor (-10) | -10 | 28092 |
| p2.2.d | 1.00 | 160 | 155 | 160 | 160 | 158.5 | 160 | **160** | geometric | 1 | 160 | 0 | no | igual | igual | 24636 |
| p2.2.e | 0.05 | 190 | 190 | 190 | 190 | 190.0 | 190 | **190** | determinista | — | 190 | 0 | no | igual | igual | 24614 |
| p2.2.f | 0.05 | 200 | 200 | 200 | 200 | 200.0 | 200 | **200** | determinista | — | 200 | 0 | no | igual | igual | 25364 |
| p2.2.g | 0.00 | 200 | 200 | 200 | 200 | 200.0 | 200 | **200** | determinista | — | 200 | 0 | no | igual | igual | 26030 |
| p2.2.h | 0.95 | 230 | 230 | 230 | 230 | 230.0 | 230 | **230** | determinista | — | 230 | 0 | no | igual | igual | 20404 |
| p2.2.i | 0.05 | 230 | 230 | 230 | 230 | 230.0 | 230 | **230** | determinista | — | 230 | 0 | no | igual | igual | 20676 |
| p2.2.j | 1.00 | 260 | 260 | 260 | 260 | 260.0 | 260 | **260** | determinista | — | 260 | 0 | no | igual | igual | 17609 |
| p2.2.k | 0.45 | 275 | 265 | 275 | 275 | 255.0 | 265 | **275** | geometric | 0 | 255 | 2 | sí (+10) | igual | igual | 14011 |
| p2.3.a | 0.00 | 70 | 70 | 70 | 70 | 70.0 | 70 | **70** | determinista | — | 70 | 0 | no | igual | igual | 115000 |
| p2.3.b | 0.00 | 70 | 70 | 70 | 70 | 70.0 | 70 | **70** | determinista | — | 70 | 0 | no | igual | igual | 69319 |
| p2.3.c | 0.00 | 105 | 105 | 105 | 105 | 105.0 | 105 | **105** | determinista | — | 105 | 0 | no | igual | igual | 60396 |
| p2.3.d | 0.00 | 105 | 105 | 105 | 105 | 105.0 | 105 | **105** | determinista | — | 105 | 0 | no | igual | igual | 60962 |
| p2.3.e | 0.00 | 120 | 120 | 120 | 120 | 120.0 | 120 | **120** | determinista | — | 120 | 0 | no | igual | igual | 51041 |
| p2.3.f | 0.00 | 120 | 120 | 120 | 120 | 120.0 | 120 | **120** | determinista | — | 120 | 0 | no | igual | igual | 50337 |
| p2.3.g | 1.00 | 145 | 140 | 140 | 145 | 140.0 | 145 | **145** | softmax | 0 | 145 | 0 | no | mejor (+5) | igual | 33442 |
| p2.3.h | 0.05 | 165 | 165 | 165 | 165 | 165.0 | 165 | **165** | determinista | — | 165 | 0 | no | igual | igual | 29872 |
| p2.3.i | 0.60 | 200 | 200 | 200 | 200 | 200.0 | 200 | **200** | determinista | — | 200 | 0 | no | igual | igual | 24401 |
| p2.3.j | 0.00 | 200 | 200 | 200 | 200 | 200.0 | 200 | **200** | determinista | — | 200 | 0 | no | igual | igual | 24948 |
| p2.3.k | 0.00 | 200 | 200 | 200 | 200 | 200.0 | 200 | **200** | determinista | — | 200 | 0 | no | igual | igual | 25979 |
| p2.4.b | 0.00 | 70 | 70 | 70 | 70 | 70.0 | 70 | **70** | determinista | — | 70 | 0 | no | igual | igual | 116351 |
| p2.4.c | 0.00 | 70 | 70 | 70 | 70 | 70.0 | 70 | **70** | determinista | — | 70 | 0 | no | igual | igual | 81947 |
| p2.4.d | 0.00 | 70 | 70 | 70 | 70 | 70.0 | 70 | **70** | determinista | — | 70 | 0 | no | igual | igual | 113324 |
| p2.4.e | 0.00 | 70 | 70 | 70 | 70 | 70.0 | 70 | **70** | determinista | — | 70 | 0 | no | igual | igual | 114892 |
| p2.4.f | 0.00 | 105 | 105 | 105 | 105 | 105.0 | 105 | **105** | determinista | — | 105 | 0 | no | igual | igual | 68161 |
| p2.4.g | 0.00 | 105 | 105 | 105 | 105 | 105.0 | 105 | **105** | determinista | — | 105 | 0 | no | igual | igual | 70236 |
| p2.4.h | 0.00 | 120 | 120 | 120 | 120 | 120.0 | 120 | **120** | determinista | — | 120 | 0 | no | igual | igual | 51314 |
| p2.4.i | 0.00 | 120 | 120 | 120 | 120 | 120.0 | 120 | **120** | determinista | — | 120 | 0 | no | igual | igual | 55987 |
| p2.4.j | 0.00 | 120 | 120 | 120 | 120 | 120.0 | 120 | **120** | determinista | — | 120 | 0 | no | igual | igual | 53789 |
| p2.4.k | 0.80 | 180 | 180 | 180 | 180 | 180.0 | 180 | **180** | determinista | — | 180 | 0 | no | igual | igual | 29426 |

Ganancia total de 1 s final sobre la rejilla final: -0.47 %.
