# Búsqueda local sobre las soluciones élite de la rejilla

Informe generado por `pjs` (`TOP/src/rust`); se regenera con

```
cd TOP
ignore/target/release/pjs.exe grid --reps 10 --iters 10 --geometric 0.3 --softmax 1 --seed 1 \
    --png-dir results/figures/routes --report docs/informe_alpha.md \
    --ls-png-dir results/figures/routes_ls --ls-report docs/informe_busqueda_local.md src/data/top-data results
```

## Método

Las decisiones y el pseudocódigo están en [`busqueda_local_diseno.md`](busqueda_local_diseno.md). En resumen:

- **Rejilla** (la de [`informe_alpha.md`](informe_alpha.md)): por instancia y por cada uno de los 21 α, una construcción determinista y 10 ejecuciones de GRASP de 10 iteraciones con cada aleatorización (geométrica β = 0.3 y softmax τ = 1).
- **Élite por instancia:** las soluciones deterministas de las α con las 3 mejores recompensas y, para cada aleatorización y cada α, las mejores soluciones de las 3 mejores ejecuciones. Solo cuenta la recompensa y los empates en la tercera entran todos.
- **Duplicados:** dos élite son iguales si visitan el mismo conjunto de clientes; se conserva la primera en el orden determinista → geométrica → softmax, α creciente, ejecución creciente, y la repetida no se sustituye. Se eligieron 110496 élite en las 353 instancias con ruta factible y quedaron **18757** tras quitar duplicados (53.1 por instancia).
- **Búsqueda local** sobre cada élite, en bucle: 2-opt dentro de cada ruta (primera mejora) y después insertar, entre los clientes no visitados que caben en alguna ruta, el de mayor recompensa (en empate, el que menos tiempo añade) en su posición más barata; se repite mientras se inserte alguno. Tarda 0.4 s en paralelo con rayon y cada solución se comprueba de forma independiente.

## Resultados globales

Recompensa total de las 353 instancias con alguna ruta factible:

| | Búsqueda local | Recompensa total | Frente al determinista sin búsqueda local | Instancias que mejora |
|---|---|---|---|---|
| Determinista, mejor α de cada instancia | no | 157868 | — | — |
| Mejor de GRASP (geométrica o softmax), mejor ejecución | no | 170469 | +7.98 % | 251 |
| Mejor élite determinista | sí | 181103 | +14.72 % | 247 |
| **Mejor élite de cualquier origen (solución final)** | sí | **189854** | **+20.26 %** | 262 |

- **GRASP frente a la determinista.** Sin búsqueda local, GRASP sube la recompensa total un **7.98 %** y mejora 251 instancias. Con búsqueda local en las dos, añadir las élite de GRASP a las deterministas la sube un **4.83 %** (de 181103 a 189854) y mejora 203 instancias.
- **Búsqueda local frente a la misma solución sin ella.** Sobre las deterministas sube la recompensa un **14.72 %** (247 instancias mejoran). Sobre la mejor solución disponible sin búsqueda local (la de GRASP), la solución final sube un **11.37 %** (217 instancias mejoran).

## Efecto de la búsqueda local en cada élite

Cada élite comparada consigo misma antes y después de la búsqueda local.

| Origen | Élite | Mejoradas | Ganancia media | Ganancia media de las mejoradas | Ganancia máxima | Clientes insertados (media) | Movimientos 2-opt (media) |
|---|---|---|---|---|---|---|---|
| Determinista | 694 | 545 (78.5 %) | +12.83 % | +16.34 % | +65.15 % | 5.56 | 3.37 |
| geometric | 8995 | 8375 (93.1 %) | +17.02 % | +18.28 % | +128.86 % | 7.40 | 4.24 |
| softmax | 9068 | 8645 (95.3 %) | +19.69 % | +20.65 % | +149.20 % | 7.71 | 4.94 |
| **Todas** | 18757 | 17565 (93.6 %) | +18.16 % | +19.39 % | +149.20 % | 7.48 | 4.54 |

## De dónde sale la mejor solución

- **Origen de la solución final** (la primera élite con la mayor recompensa tras la búsqueda local, en el orden determinista → geométrica → softmax): determinista en 150, geometric en 131 y softmax en 72 instancias.
- **¿Bastaba con la mejor antes de la búsqueda local?** En **163 de 353** instancias la solución final sale de una élite que antes no era la mejor: aplicar la búsqueda local solo a la mejor solución habría dado menos. En esas instancias, la diferencia suma 8187 de recompensa.

## Por conjunto

| Conjunto | Instancias | Determinista | GRASP | GRASP frente a determinista | Determinista + búsqueda local | Final | Búsqueda local (final frente a GRASP) | Final frente a determinista |
|---|---|---|---|---|---|---|---|---|
| p1 | 48 | 5520 | 5780 | +4.71 % | 5820 | 6000 | +3.81 % | +8.70 % |
| p2 | 33 | 4540 | 4615 | +1.65 % | 4585 | 4635 | +0.43 % | +2.09 % |
| p3 | 60 | 22740 | 23930 | +5.23 % | 23960 | 24650 | +3.01 % | +8.40 % |
| p4 | 56 | 32614 | 35252 | +8.09 % | 42595 | 45252 | +28.37 % | +38.75 % |
| p5 | 74 | 51120 | 56440 | +10.41 % | 56100 | 58075 | +2.90 % | +13.61 % |
| p6 | 24 | 15930 | 16998 | +6.70 % | 18066 | 18708 | +10.06 % | +17.44 % |
| p7 | 58 | 25404 | 27454 | +8.07 % | 29977 | 32534 | +18.50 % | +28.07 % |

## Por instancia

*Determinista* y *GRASP*: la mejor recompensa de la rejilla sin búsqueda local. *Final*: la mejor élite tras la búsqueda local, con su origen, su α, su ejecución (solo GRASP), su recompensa antes de la búsqueda local y los clientes que insertó. *Élite*: elegidas / que quedan tras quitar duplicados.

| Instancia | m | tmax | Determinista | GRASP | Determinista + búsqueda local | Final | Origen | α | Ejecución | Antes | Insertados | Élite | Rutas |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| p1.2.b | 2 | 5 | 15 | 15 | 15 | **15** | deterministic | 0.00 |  | 15 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.2.b.png) |
| p1.2.c | 2 | 7.5 | 20 | 20 | 20 | **20** | deterministic | 0.00 |  | 20 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.2.c.png) |
| p1.2.d | 2 | 10 | 30 | 30 | 30 | **30** | deterministic | 0.00 |  | 30 | 0 | 426 / 1 | [png](../results/figures/routes_ls/p1.2.d.png) |
| p1.2.e | 2 | 12.5 | 40 | 45 | 45 | **45** | deterministic | 0.00 |  | 40 | 1 | 332 / 4 | [png](../results/figures/routes_ls/p1.2.e.png) |
| p1.2.f | 2 | 15 | 75 | 80 | 80 | **80** | deterministic | 0.90 |  | 75 | 1 | 272 / 7 | [png](../results/figures/routes_ls/p1.2.f.png) |
| p1.2.g | 2 | 17.5 | 75 | 85 | 80 | **90** | geometric | 0.90 | 1 | 75 | 2 | 264 / 51 | [png](../results/figures/routes_ls/p1.2.g.png) |
| p1.2.h | 2 | 20 | 90 | 110 | 105 | **110** | geometric | 0.05 | 1 | 95 | 3 | 244 / 71 | [png](../results/figures/routes_ls/p1.2.h.png) |
| p1.2.i | 2 | 23 | 120 | 130 | 130 | **130** | deterministic | 0.05 |  | 120 | 1 | 366 / 24 | [png](../results/figures/routes_ls/p1.2.i.png) |
| p1.2.j | 2 | 25 | 130 | 140 | 135 | **155** | geometric | 0.05 | 2 | 120 | 5 | 325 / 75 | [png](../results/figures/routes_ls/p1.2.j.png) |
| p1.2.k | 2 | 27.5 | 165 | 165 | 175 | **175** | deterministic | 0.05 |  | 160 | 3 | 347 / 33 | [png](../results/figures/routes_ls/p1.2.k.png) |
| p1.2.l | 2 | 30 | 180 | 185 | 185 | **190** | geometric | 0.00 | 9 | 145 | 6 | 336 / 38 | [png](../results/figures/routes_ls/p1.2.l.png) |
| p1.2.m | 2 | 32.5 | 200 | 200 | 200 | **205** | geometric | 0.00 | 5 | 145 | 8 | 331 / 28 | [png](../results/figures/routes_ls/p1.2.m.png) |
| p1.2.n | 2 | 35 | 230 | 235 | 230 | **235** | geometric | 0.10 | 3 | 235 | 0 | 341 / 32 | [png](../results/figures/routes_ls/p1.2.n.png) |
| p1.2.o | 2 | 36.5 | 235 | 240 | 240 | **240** | deterministic | 0.55 |  | 235 | 1 | 316 / 32 | [png](../results/figures/routes_ls/p1.2.o.png) |
| p1.2.p | 2 | 37.5 | 240 | 240 | 245 | **250** | geometric | 0.10 | 7 | 235 | 3 | 340 / 38 | [png](../results/figures/routes_ls/p1.2.p.png) |
| p1.2.q | 2 | 40 | 245 | 250 | 260 | **265** | geometric | 0.05 | 1 | 225 | 5 | 328 / 41 | [png](../results/figures/routes_ls/p1.2.q.png) |
| p1.2.r | 2 | 42.5 | 270 | 275 | 275 | **280** | geometric | 0.15 | 0 | 265 | 3 | 292 / 48 | [png](../results/figures/routes_ls/p1.2.r.png) |
| p1.3.c | 3 | 5 | 15 | 15 | 15 | **15** | deterministic | 0.00 |  | 15 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.3.c.png) |
| p1.3.d | 3 | 6.7 | 15 | 15 | 15 | **15** | deterministic | 0.00 |  | 15 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.3.d.png) |
| p1.3.e | 3 | 8.3 | 30 | 30 | 30 | **30** | deterministic | 0.00 |  | 30 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.3.e.png) |
| p1.3.f | 3 | 10 | 40 | 40 | 40 | **40** | deterministic | 0.00 |  | 40 | 0 | 428 / 1 | [png](../results/figures/routes_ls/p1.3.f.png) |
| p1.3.g | 3 | 11.7 | 50 | 50 | 50 | **50** | deterministic | 0.00 |  | 50 | 0 | 441 / 2 | [png](../results/figures/routes_ls/p1.3.g.png) |
| p1.3.h | 3 | 13.3 | 70 | 70 | 70 | **70** | deterministic | 0.00 |  | 70 | 0 | 424 / 6 | [png](../results/figures/routes_ls/p1.3.h.png) |
| p1.3.i | 3 | 15.3 | 95 | 100 | 100 | **100** | deterministic | 0.05 |  | 95 | 1 | 399 / 2 | [png](../results/figures/routes_ls/p1.3.i.png) |
| p1.3.j | 3 | 16.7 | 95 | 100 | 100 | **115** | softmax | 0.00 | 7 | 95 | 3 | 401 / 39 | [png](../results/figures/routes_ls/p1.3.j.png) |
| p1.3.k | 3 | 18.3 | 120 | 130 | 125 | **135** | geometric | 0.05 | 9 | 125 | 2 | 390 / 50 | [png](../results/figures/routes_ls/p1.3.k.png) |
| p1.3.l | 3 | 20 | 125 | 140 | 145 | **150** | geometric | 0.25 | 8 | 125 | 4 | 249 / 97 | [png](../results/figures/routes_ls/p1.3.l.png) |
| p1.3.m | 3 | 21.7 | 135 | 160 | 170 | **175** | geometric | 0.05 | 4 | 155 | 3 | 245 / 123 | [png](../results/figures/routes_ls/p1.3.m.png) |
| p1.3.n | 3 | 23.3 | 175 | 185 | 190 | **190** | deterministic | 0.05 |  | 175 | 2 | 319 / 39 | [png](../results/figures/routes_ls/p1.3.n.png) |
| p1.3.o | 3 | 24.3 | 195 | 200 | 205 | **205** | deterministic | 0.05 |  | 190 | 3 | 344 / 28 | [png](../results/figures/routes_ls/p1.3.o.png) |
| p1.3.p | 3 | 25 | 190 | 200 | 195 | **220** | geometric | 0.05 | 1 | 175 | 6 | 349 / 57 | [png](../results/figures/routes_ls/p1.3.p.png) |
| p1.3.q | 3 | 26.7 | 215 | 220 | 225 | **225** | deterministic | 0.85 |  | 210 | 2 | 363 / 19 | [png](../results/figures/routes_ls/p1.3.q.png) |
| p1.3.r | 3 | 28.3 | 225 | 230 | 240 | **240** | deterministic | 0.05 |  | 215 | 4 | 364 / 36 | [png](../results/figures/routes_ls/p1.3.r.png) |
| p1.4.d | 4 | 5 | 15 | 15 | 15 | **15** | deterministic | 0.00 |  | 15 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.4.d.png) |
| p1.4.e | 4 | 6.2 | 15 | 15 | 15 | **15** | deterministic | 0.00 |  | 15 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.4.e.png) |
| p1.4.f | 4 | 7.5 | 25 | 25 | 25 | **25** | deterministic | 0.00 |  | 25 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.4.f.png) |
| p1.4.g | 4 | 8.8 | 35 | 35 | 35 | **35** | deterministic | 0.00 |  | 35 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.4.g.png) |
| p1.4.h | 4 | 10 | 45 | 45 | 45 | **45** | deterministic | 0.00 |  | 45 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p1.4.h.png) |
| p1.4.i | 4 | 11.5 | 60 | 60 | 60 | **60** | deterministic | 0.05 |  | 60 | 0 | 433 / 3 | [png](../results/figures/routes_ls/p1.4.i.png) |
| p1.4.j | 4 | 12.5 | 70 | 75 | 75 | **75** | deterministic | 0.00 |  | 70 | 1 | 344 / 9 | [png](../results/figures/routes_ls/p1.4.j.png) |
| p1.4.k | 4 | 13.8 | 95 | 100 | 95 | **100** | geometric | 0.00 | 1 | 95 | 1 | 287 / 5 | [png](../results/figures/routes_ls/p1.4.k.png) |
| p1.4.l | 4 | 15 | 120 | 120 | 120 | **120** | deterministic | 0.05 |  | 120 | 0 | 434 / 3 | [png](../results/figures/routes_ls/p1.4.l.png) |
| p1.4.m | 4 | 16.2 | 115 | 125 | 125 | **130** | geometric | 0.10 | 0 | 120 | 2 | 298 / 37 | [png](../results/figures/routes_ls/p1.4.m.png) |
| p1.4.n | 4 | 17.5 | 135 | 145 | 145 | **155** | geometric | 0.80 | 1 | 135 | 3 | 324 / 44 | [png](../results/figures/routes_ls/p1.4.n.png) |
| p1.4.o | 4 | 18.2 | 150 | 155 | 160 | **165** | geometric | 0.15 | 8 | 155 | 2 | 403 / 33 | [png](../results/figures/routes_ls/p1.4.o.png) |
| p1.4.p | 4 | 18.8 | 165 | 170 | 175 | **175** | deterministic | 0.05 |  | 160 | 2 | 413 / 23 | [png](../results/figures/routes_ls/p1.4.p.png) |
| p1.4.q | 4 | 20 | 160 | 175 | 175 | **185** | softmax | 0.00 | 3 | 155 | 5 | 309 / 65 | [png](../results/figures/routes_ls/p1.4.q.png) |
| p1.4.r | 4 | 21.2 | 165 | 190 | 190 | **210** | softmax | 0.95 | 0 | 165 | 5 | 283 / 110 | [png](../results/figures/routes_ls/p1.4.r.png) |
| p2.2.a | 2 | 7.5 | 80 | 90 | 80 | **90** | geometric | 0.15 | 9 | 90 | 0 | 333 / 3 | [png](../results/figures/routes_ls/p2.2.a.png) |
| p2.2.b | 2 | 10 | 120 | 120 | 120 | **120** | deterministic | 0.00 |  | 120 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.2.b.png) |
| p2.2.c | 2 | 11.5 | 110 | 140 | 125 | **140** | geometric | 0.95 | 7 | 130 | 1 | 269 / 14 | [png](../results/figures/routes_ls/p2.2.c.png) |
| p2.2.d | 2 | 12.5 | 145 | 155 | 155 | **160** | geometric | 0.05 | 7 | 135 | 2 | 229 / 48 | [png](../results/figures/routes_ls/p2.2.d.png) |
| p2.2.e | 2 | 13.5 | 190 | 190 | 190 | **190** | deterministic | 0.05 |  | 190 | 0 | 429 / 8 | [png](../results/figures/routes_ls/p2.2.e.png) |
| p2.2.f | 2 | 15 | 200 | 200 | 200 | **200** | deterministic | 0.05 |  | 200 | 0 | 436 / 2 | [png](../results/figures/routes_ls/p2.2.f.png) |
| p2.2.g | 2 | 16 | 200 | 200 | 200 | **200** | deterministic | 0.00 |  | 200 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.2.g.png) |
| p2.2.h | 2 | 17.5 | 230 | 230 | 230 | **230** | deterministic | 0.45 |  | 215 | 1 | 286 / 26 | [png](../results/figures/routes_ls/p2.2.h.png) |
| p2.2.i | 2 | 19 | 230 | 230 | 230 | **230** | deterministic | 0.05 |  | 230 | 0 | 429 / 1 | [png](../results/figures/routes_ls/p2.2.i.png) |
| p2.2.j | 2 | 20 | 260 | 260 | 260 | **260** | deterministic | 0.85 |  | 225 | 2 | 232 / 17 | [png](../results/figures/routes_ls/p2.2.j.png) |
| p2.2.k | 2 | 22.5 | 245 | 265 | 255 | **275** | geometric | 0.40 | 4 | 235 | 3 | 273 / 22 | [png](../results/figures/routes_ls/p2.2.k.png) |
| p2.3.a | 3 | 5 | 70 | 70 | 70 | **70** | deterministic | 0.00 |  | 70 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.a.png) |
| p2.3.b | 3 | 6.7 | 70 | 70 | 70 | **70** | deterministic | 0.00 |  | 70 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.b.png) |
| p2.3.c | 3 | 7.7 | 105 | 105 | 105 | **105** | deterministic | 0.00 |  | 105 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.c.png) |
| p2.3.d | 3 | 8.3 | 105 | 105 | 105 | **105** | deterministic | 0.00 |  | 105 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.d.png) |
| p2.3.e | 3 | 9 | 120 | 120 | 120 | **120** | deterministic | 0.00 |  | 120 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.e.png) |
| p2.3.f | 3 | 10 | 120 | 120 | 120 | **120** | deterministic | 0.00 |  | 120 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.f.png) |
| p2.3.g | 3 | 10.7 | 135 | 140 | 145 | **145** | deterministic | 0.60 |  | 130 | 1 | 404 / 7 | [png](../results/figures/routes_ls/p2.3.g.png) |
| p2.3.h | 3 | 11.7 | 165 | 165 | 165 | **165** | deterministic | 0.05 |  | 165 | 0 | 407 / 4 | [png](../results/figures/routes_ls/p2.3.h.png) |
| p2.3.i | 3 | 12.7 | 200 | 200 | 200 | **200** | deterministic | 0.60 |  | 200 | 0 | 320 / 3 | [png](../results/figures/routes_ls/p2.3.i.png) |
| p2.3.j | 3 | 13.3 | 200 | 200 | 200 | **200** | deterministic | 0.00 |  | 200 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.j.png) |
| p2.3.k | 3 | 15 | 200 | 200 | 200 | **200** | deterministic | 0.00 |  | 200 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.3.k.png) |
| p2.4.a | 4 | 3.8 | 10 | 10 | 10 | **10** | deterministic | 0.00 |  | 10 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.a.png) |
| p2.4.b | 4 | 5 | 70 | 70 | 70 | **70** | deterministic | 0.00 |  | 70 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.b.png) |
| p2.4.c | 4 | 5.8 | 70 | 70 | 70 | **70** | deterministic | 0.00 |  | 70 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.c.png) |
| p2.4.d | 4 | 6.2 | 70 | 70 | 70 | **70** | deterministic | 0.00 |  | 70 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.d.png) |
| p2.4.e | 4 | 6.8 | 70 | 70 | 70 | **70** | deterministic | 0.00 |  | 70 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.e.png) |
| p2.4.f | 4 | 7.5 | 105 | 105 | 105 | **105** | deterministic | 0.00 |  | 105 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.f.png) |
| p2.4.g | 4 | 8 | 105 | 105 | 105 | **105** | deterministic | 0.00 |  | 105 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.g.png) |
| p2.4.h | 4 | 8.8 | 120 | 120 | 120 | **120** | deterministic | 0.00 |  | 120 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.h.png) |
| p2.4.i | 4 | 9.5 | 120 | 120 | 120 | **120** | deterministic | 0.00 |  | 120 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.i.png) |
| p2.4.j | 4 | 10 | 120 | 120 | 120 | **120** | deterministic | 0.00 |  | 120 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p2.4.j.png) |
| p2.4.k | 4 | 11.2 | 180 | 180 | 180 | **180** | deterministic | 0.80 |  | 180 | 0 | 367 / 3 | [png](../results/figures/routes_ls/p2.4.k.png) |
| p3.2.a | 2 | 7.5 | 90 | 90 | 90 | **90** | deterministic | 0.05 |  | 90 | 0 | 435 / 1 | [png](../results/figures/routes_ls/p3.2.a.png) |
| p3.2.b | 2 | 10 | 150 | 150 | 150 | **150** | deterministic | 0.00 |  | 150 | 0 | 433 / 3 | [png](../results/figures/routes_ls/p3.2.b.png) |
| p3.2.c | 2 | 12.5 | 170 | 180 | 180 | **180** | deterministic | 0.85 |  | 170 | 1 | 344 / 5 | [png](../results/figures/routes_ls/p3.2.c.png) |
| p3.2.d | 2 | 15 | 190 | 220 | 210 | **220** | geometric | 0.90 | 3 | 220 | 0 | 294 / 22 | [png](../results/figures/routes_ls/p3.2.d.png) |
| p3.2.e | 2 | 17.5 | 240 | 250 | 260 | **260** | deterministic | 0.05 |  | 220 | 3 | 314 / 14 | [png](../results/figures/routes_ls/p3.2.e.png) |
| p3.2.f | 2 | 20 | 290 | 290 | 300 | **300** | deterministic | 0.95 |  | 290 | 1 | 290 / 35 | [png](../results/figures/routes_ls/p3.2.f.png) |
| p3.2.g | 2 | 22.5 | 320 | 350 | 340 | **360** | geometric | 0.05 | 0 | 290 | 4 | 268 / 40 | [png](../results/figures/routes_ls/p3.2.g.png) |
| p3.2.h | 2 | 25 | 320 | 360 | 370 | **390** | geometric | 0.45 | 0 | 320 | 4 | 322 / 73 | [png](../results/figures/routes_ls/p3.2.h.png) |
| p3.2.i | 2 | 27.5 | 440 | 440 | 440 | **440** | deterministic | 0.05 |  | 390 | 3 | 360 / 35 | [png](../results/figures/routes_ls/p3.2.i.png) |
| p3.2.j | 2 | 30 | 440 | 490 | 500 | **510** | geometric | 0.70 | 2 | 440 | 4 | 240 / 90 | [png](../results/figures/routes_ls/p3.2.j.png) |
| p3.2.k | 2 | 32.5 | 450 | 540 | 510 | **550** | geometric | 0.40 | 3 | 460 | 6 | 279 / 86 | [png](../results/figures/routes_ls/p3.2.k.png) |
| p3.2.l | 2 | 35 | 490 | 500 | 510 | **560** | geometric | 0.50 | 5 | 450 | 4 | 285 / 43 | [png](../results/figures/routes_ls/p3.2.l.png) |
| p3.2.m | 2 | 37.5 | 500 | 600 | 570 | **620** | geometric | 0.60 | 2 | 490 | 6 | 222 / 77 | [png](../results/figures/routes_ls/p3.2.m.png) |
| p3.2.n | 2 | 40 | 600 | 600 | 620 | **620** | deterministic | 0.95 |  | 600 | 1 | 297 / 59 | [png](../results/figures/routes_ls/p3.2.n.png) |
| p3.2.o | 2 | 42.5 | 600 | 630 | 630 | **670** | softmax | 0.35 | 0 | 520 | 6 | 274 / 55 | [png](../results/figures/routes_ls/p3.2.o.png) |
| p3.2.p | 2 | 45 | 650 | 660 | 710 | **710** | deterministic | 0.45 |  | 620 | 5 | 335 / 63 | [png](../results/figures/routes_ls/p3.2.p.png) |
| p3.2.q | 2 | 47.5 | 680 | 760 | 740 | **760** | geometric | 0.80 | 1 | 690 | 4 | 249 / 91 | [png](../results/figures/routes_ls/p3.2.q.png) |
| p3.2.r | 2 | 50 | 700 | 780 | 780 | **790** | softmax | 0.95 | 7 | 750 | 3 | 231 / 98 | [png](../results/figures/routes_ls/p3.2.r.png) |
| p3.2.s | 2 | 52.5 | 780 | 800 | 800 | **800** | deterministic | 0.90 |  | 690 | 5 | 267 / 67 | [png](../results/figures/routes_ls/p3.2.s.png) |
| p3.2.t | 2 | 55 | 800 | 800 | 800 | **800** | deterministic | 0.90 |  | 800 | 0 | 271 / 77 | [png](../results/figures/routes_ls/p3.2.t.png) |
| p3.3.a | 3 | 5 | 30 | 30 | 30 | **30** | deterministic | 0.00 |  | 30 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.3.a.png) |
| p3.3.b | 3 | 6.7 | 90 | 90 | 90 | **90** | deterministic | 0.00 |  | 90 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.3.b.png) |
| p3.3.c | 3 | 8.3 | 120 | 120 | 120 | **120** | deterministic | 0.00 |  | 120 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.3.c.png) |
| p3.3.d | 3 | 10 | 170 | 170 | 170 | **170** | deterministic | 0.00 |  | 170 | 0 | 429 / 2 | [png](../results/figures/routes_ls/p3.3.d.png) |
| p3.3.e | 3 | 11.7 | 200 | 200 | 200 | **200** | deterministic | 0.85 |  | 200 | 0 | 387 / 5 | [png](../results/figures/routes_ls/p3.3.e.png) |
| p3.3.f | 3 | 13.3 | 210 | 220 | 210 | **230** | geometric | 0.00 | 4 | 200 | 2 | 353 / 27 | [png](../results/figures/routes_ls/p3.3.f.png) |
| p3.3.g | 3 | 15 | 240 | 270 | 260 | **270** | geometric | 0.90 | 1 | 270 | 0 | 283 / 19 | [png](../results/figures/routes_ls/p3.3.g.png) |
| p3.3.h | 3 | 16.7 | 290 | 300 | 300 | **300** | deterministic | 0.05 |  | 280 | 2 | 255 / 13 | [png](../results/figures/routes_ls/p3.3.h.png) |
| p3.3.i | 3 | 18.3 | 300 | 310 | 310 | **330** | geometric | 0.00 | 2 | 290 | 3 | 307 / 27 | [png](../results/figures/routes_ls/p3.3.i.png) |
| p3.3.j | 3 | 20 | 380 | 380 | 380 | **380** | deterministic | 0.85 |  | 350 | 3 | 274 / 62 | [png](../results/figures/routes_ls/p3.3.j.png) |
| p3.3.k | 3 | 21.7 | 390 | 400 | 440 | **440** | deterministic | 1.00 |  | 370 | 4 | 232 / 44 | [png](../results/figures/routes_ls/p3.3.k.png) |
| p3.3.l | 3 | 23.3 | 420 | 450 | 460 | **470** | geometric | 0.80 | 3 | 420 | 4 | 331 / 34 | [png](../results/figures/routes_ls/p3.3.l.png) |
| p3.3.m | 3 | 25 | 420 | 470 | 460 | **520** | geometric | 0.95 | 6 | 470 | 3 | 322 / 87 | [png](../results/figures/routes_ls/p3.3.m.png) |
| p3.3.n | 3 | 26.7 | 550 | 550 | 570 | **570** | deterministic | 0.05 |  | 490 | 5 | 278 / 35 | [png](../results/figures/routes_ls/p3.3.n.png) |
| p3.3.o | 3 | 28.3 | 580 | 580 | 580 | **580** | deterministic | 0.05 |  | 550 | 2 | 358 / 44 | [png](../results/figures/routes_ls/p3.3.o.png) |
| p3.3.p | 3 | 30 | 590 | 610 | 600 | **640** | geometric | 0.45 | 7 | 550 | 4 | 224 / 60 | [png](../results/figures/routes_ls/p3.3.p.png) |
| p3.3.q | 3 | 31.7 | 580 | 660 | 640 | **670** | geometric | 0.95 | 0 | 660 | 1 | 272 / 63 | [png](../results/figures/routes_ls/p3.3.q.png) |
| p3.3.r | 3 | 33.3 | 690 | 710 | 710 | **710** | deterministic | 1.00 |  | 690 | 1 | 297 / 68 | [png](../results/figures/routes_ls/p3.3.r.png) |
| p3.3.s | 3 | 35 | 600 | 640 | 680 | **690** | geometric | 0.15 | 4 | 600 | 5 | 289 / 56 | [png](../results/figures/routes_ls/p3.3.s.png) |
| p3.3.t | 3 | 36.7 | 660 | 740 | 670 | **740** | geometric | 0.95 | 0 | 740 | 0 | 280 / 81 | [png](../results/figures/routes_ls/p3.3.t.png) |
| p3.4.a | 4 | 3.8 | 20 | 20 | 20 | **20** | deterministic | 0.00 |  | 20 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.4.a.png) |
| p3.4.b | 4 | 5 | 30 | 30 | 30 | **30** | deterministic | 0.00 |  | 30 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.4.b.png) |
| p3.4.c | 4 | 6.2 | 90 | 90 | 90 | **90** | deterministic | 0.00 |  | 90 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.4.c.png) |
| p3.4.d | 4 | 7.5 | 100 | 100 | 100 | **100** | deterministic | 0.00 |  | 100 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.4.d.png) |
| p3.4.e | 4 | 8.8 | 140 | 140 | 140 | **140** | deterministic | 0.00 |  | 140 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p3.4.e.png) |
| p3.4.f | 4 | 10 | 190 | 190 | 190 | **190** | deterministic | 0.00 |  | 190 | 0 | 435 / 2 | [png](../results/figures/routes_ls/p3.4.f.png) |
| p3.4.g | 4 | 11.2 | 220 | 220 | 220 | **220** | deterministic | 0.85 |  | 220 | 0 | 392 / 5 | [png](../results/figures/routes_ls/p3.4.g.png) |
| p3.4.h | 4 | 12.5 | 230 | 240 | 230 | **240** | geometric | 0.10 | 6 | 240 | 0 | 353 / 6 | [png](../results/figures/routes_ls/p3.4.h.png) |
| p3.4.i | 4 | 13.8 | 260 | 260 | 260 | **260** | deterministic | 1.00 |  | 260 | 0 | 277 / 43 | [png](../results/figures/routes_ls/p3.4.i.png) |
| p3.4.j | 4 | 15 | 290 | 310 | 300 | **310** | geometric | 0.90 | 3 | 310 | 0 | 262 / 22 | [png](../results/figures/routes_ls/p3.4.j.png) |
| p3.4.k | 4 | 16.2 | 350 | 350 | 350 | **350** | deterministic | 0.05 |  | 320 | 2 | 285 / 18 | [png](../results/figures/routes_ls/p3.4.k.png) |
| p3.4.l | 4 | 17.5 | 340 | 370 | 380 | **380** | deterministic | 0.05 |  | 320 | 4 | 287 / 43 | [png](../results/figures/routes_ls/p3.4.l.png) |
| p3.4.m | 4 | 18.8 | 360 | 380 | 370 | **390** | softmax | 0.85 | 2 | 360 | 3 | 313 / 31 | [png](../results/figures/routes_ls/p3.4.m.png) |
| p3.4.n | 4 | 20 | 430 | 430 | 430 | **440** | geometric | 0.35 | 8 | 400 | 3 | 280 / 45 | [png](../results/figures/routes_ls/p3.4.n.png) |
| p3.4.o | 4 | 21.2 | 470 | 480 | 490 | **500** | softmax | 0.05 | 8 | 420 | 5 | 285 / 31 | [png](../results/figures/routes_ls/p3.4.o.png) |
| p3.4.p | 4 | 22.5 | 510 | 520 | 540 | **560** | geometric | 0.30 | 2 | 480 | 5 | 255 / 72 | [png](../results/figures/routes_ls/p3.4.p.png) |
| p3.4.q | 4 | 23.8 | 510 | 520 | 540 | **560** | softmax | 0.85 | 5 | 500 | 5 | 297 / 58 | [png](../results/figures/routes_ls/p3.4.q.png) |
| p3.4.r | 4 | 25 | 500 | 550 | 550 | **600** | softmax | 0.00 | 0 | 480 | 7 | 293 / 61 | [png](../results/figures/routes_ls/p3.4.r.png) |
| p3.4.s | 4 | 26.2 | 620 | 670 | 670 | **670** | deterministic | 0.05 |  | 590 | 5 | 300 / 41 | [png](../results/figures/routes_ls/p3.4.s.png) |
| p3.4.t | 4 | 27.5 | 670 | 670 | 670 | **670** | deterministic | 0.05 |  | 620 | 3 | 299 / 35 | [png](../results/figures/routes_ls/p3.4.t.png) |
| p4.2.a | 2 | 25 | 167 | 196 | 168 | **206** | softmax | 0.75 | 6 | 161 | 3 | 204 / 36 | [png](../results/figures/routes_ls/p4.2.a.png) |
| p4.2.b | 2 | 30 | 250 | 285 | 274 | **316** | softmax | 0.55 | 9 | 274 | 4 | 221 / 67 | [png](../results/figures/routes_ls/p4.2.b.png) |
| p4.2.c | 2 | 35 | 328 | 359 | 368 | **444** | softmax | 0.95 | 4 | 283 | 11 | 214 / 78 | [png](../results/figures/routes_ls/p4.2.c.png) |
| p4.2.d | 2 | 40 | 374 | 389 | 435 | **491** | softmax | 0.95 | 5 | 361 | 10 | 288 / 66 | [png](../results/figures/routes_ls/p4.2.d.png) |
| p4.2.e | 2 | 45 | 415 | 453 | 520 | **571** | softmax | 0.45 | 0 | 397 | 13 | 267 / 76 | [png](../results/figures/routes_ls/p4.2.e.png) |
| p4.2.f | 2 | 50 | 455 | 494 | 546 | **628** | geometric | 0.00 | 8 | 327 | 20 | 261 / 80 | [png](../results/figures/routes_ls/p4.2.f.png) |
| p4.2.g | 2 | 55 | 514 | 533 | 586 | **691** | softmax | 0.70 | 7 | 438 | 17 | 263 / 85 | [png](../results/figures/routes_ls/p4.2.g.png) |
| p4.2.h | 2 | 60 | 582 | 582 | 722 | **760** | softmax | 0.70 | 4 | 495 | 18 | 253 / 84 | [png](../results/figures/routes_ls/p4.2.h.png) |
| p4.2.i | 2 | 65 | 557 | 613 | 711 | **809** | geometric | 0.45 | 2 | 571 | 18 | 281 / 73 | [png](../results/figures/routes_ls/p4.2.i.png) |
| p4.2.j | 2 | 70 | 605 | 682 | 821 | **870** | softmax | 0.85 | 6 | 585 | 16 | 243 / 82 | [png](../results/figures/routes_ls/p4.2.j.png) |
| p4.2.k | 2 | 75 | 629 | 685 | 858 | **921** | softmax | 0.55 | 3 | 590 | 22 | 212 / 96 | [png](../results/figures/routes_ls/p4.2.k.png) |
| p4.2.l | 2 | 80 | 629 | 721 | 922 | **1004** | geometric | 0.65 | 4 | 615 | 25 | 240 / 102 | [png](../results/figures/routes_ls/p4.2.l.png) |
| p4.2.m | 2 | 85 | 738 | 773 | 1038 | **1042** | geometric | 0.95 | 4 | 699 | 17 | 256 / 95 | [png](../results/figures/routes_ls/p4.2.m.png) |
| p4.2.n | 2 | 90 | 738 | 820 | 1093 | **1115** | geometric | 1.00 | 4 | 786 | 18 | 231 / 97 | [png](../results/figures/routes_ls/p4.2.n.png) |
| p4.2.o | 2 | 95 | 738 | 820 | 1102 | **1142** | geometric | 0.70 | 3 | 694 | 29 | 265 / 85 | [png](../results/figures/routes_ls/p4.2.o.png) |
| p4.2.p | 2 | 100 | 738 | 794 | 1194 | **1194** | deterministic | 0.50 |  | 723 | 35 | 248 / 91 | [png](../results/figures/routes_ls/p4.2.p.png) |
| p4.2.q | 2 | 105 | 783 | 821 | 1190 | **1224** | softmax | 0.60 | 3 | 676 | 44 | 281 / 93 | [png](../results/figures/routes_ls/p4.2.q.png) |
| p4.2.r | 2 | 110 | 815 | 867 | 1212 | **1254** | geometric | 0.75 | 1 | 759 | 37 | 280 / 92 | [png](../results/figures/routes_ls/p4.2.r.png) |
| p4.2.s | 2 | 115 | 815 | 867 | 1233 | **1291** | geometric | 0.45 | 1 | 781 | 49 | 232 / 85 | [png](../results/figures/routes_ls/p4.2.s.png) |
| p4.2.t | 2 | 120 | 815 | 863 | 1268 | **1293** | geometric | 0.45 | 2 | 781 | 48 | 204 / 94 | [png](../results/figures/routes_ls/p4.2.t.png) |
| p4.3.b | 3 | 20 | 38 | 38 | 38 | **38** | deterministic | 0.00 |  | 38 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p4.3.b.png) |
| p4.3.c | 3 | 23.3 | 169 | 181 | 193 | **193** | deterministic | 0.60 |  | 169 | 3 | 240 / 22 | [png](../results/figures/routes_ls/p4.3.c.png) |
| p4.3.d | 3 | 26.7 | 278 | 299 | 293 | **327** | softmax | 0.55 | 4 | 263 | 6 | 233 / 52 | [png](../results/figures/routes_ls/p4.3.d.png) |
| p4.3.e | 3 | 30 | 367 | 399 | 412 | **447** | softmax | 0.25 | 3 | 338 | 11 | 206 / 78 | [png](../results/figures/routes_ls/p4.3.e.png) |
| p4.3.f | 3 | 33.3 | 404 | 440 | 465 | **506** | softmax | 0.85 | 3 | 367 | 8 | 199 / 100 | [png](../results/figures/routes_ls/p4.3.f.png) |
| p4.3.g | 3 | 36.7 | 460 | 514 | 552 | **619** | softmax | 0.70 | 8 | 428 | 13 | 237 / 84 | [png](../results/figures/routes_ls/p4.3.g.png) |
| p4.3.h | 3 | 40 | 490 | 523 | 647 | **692** | softmax | 0.25 | 4 | 400 | 20 | 249 / 86 | [png](../results/figures/routes_ls/p4.3.h.png) |
| p4.3.i | 3 | 43.3 | 540 | 581 | 677 | **725** | geometric | 1.00 | 4 | 536 | 10 | 259 / 88 | [png](../results/figures/routes_ls/p4.3.i.png) |
| p4.3.j | 3 | 46.7 | 584 | 645 | 763 | **768** | geometric | 0.35 | 4 | 550 | 17 | 272 / 77 | [png](../results/figures/routes_ls/p4.3.j.png) |
| p4.3.k | 3 | 50 | 614 | 689 | 722 | **874** | geometric | 0.45 | 7 | 625 | 19 | 264 / 85 | [png](../results/figures/routes_ls/p4.3.k.png) |
| p4.3.l | 3 | 53.3 | 663 | 727 | 885 | **900** | geometric | 0.45 | 5 | 717 | 14 | 273 / 80 | [png](../results/figures/routes_ls/p4.3.l.png) |
| p4.3.m | 3 | 56.7 | 694 | 752 | 912 | **940** | geometric | 0.70 | 1 | 693 | 18 | 303 / 80 | [png](../results/figures/routes_ls/p4.3.m.png) |
| p4.3.n | 3 | 60 | 778 | 778 | 976 | **1038** | geometric | 0.85 | 7 | 742 | 20 | 260 / 84 | [png](../results/figures/routes_ls/p4.3.n.png) |
| p4.3.o | 3 | 63.3 | 772 | 838 | 1056 | **1057** | geometric | 0.30 | 5 | 741 | 24 | 287 / 84 | [png](../results/figures/routes_ls/p4.3.o.png) |
| p4.3.p | 3 | 66.7 | 785 | 858 | 1123 | **1134** | geometric | 0.65 | 0 | 798 | 27 | 256 / 84 | [png](../results/figures/routes_ls/p4.3.p.png) |
| p4.3.q | 3 | 70 | 821 | 899 | 1086 | **1170** | geometric | 0.75 | 0 | 820 | 28 | 282 / 78 | [png](../results/figures/routes_ls/p4.3.q.png) |
| p4.3.r | 3 | 73.3 | 830 | 913 | 1128 | **1229** | geometric | 0.70 | 7 | 791 | 34 | 246 / 99 | [png](../results/figures/routes_ls/p4.3.r.png) |
| p4.3.s | 3 | 76.7 | 813 | 925 | 1157 | **1252** | geometric | 0.65 | 0 | 827 | 35 | 236 / 100 | [png](../results/figures/routes_ls/p4.3.s.png) |
| p4.3.t | 3 | 80 | 827 | 937 | 1250 | **1277** | softmax | 0.65 | 0 | 782 | 44 | 239 / 101 | [png](../results/figures/routes_ls/p4.3.t.png) |
| p4.4.d | 4 | 20 | 38 | 38 | 38 | **38** | deterministic | 0.00 |  | 38 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p4.4.d.png) |
| p4.4.e | 4 | 22.5 | 172 | 183 | 183 | **183** | deterministic | 1.00 |  | 172 | 1 | 242 / 9 | [png](../results/figures/routes_ls/p4.4.e.png) |
| p4.4.f | 4 | 25 | 282 | 291 | 283 | **308** | geometric | 1.00 | 0 | 264 | 6 | 239 / 40 | [png](../results/figures/routes_ls/p4.4.f.png) |
| p4.4.g | 4 | 27.5 | 377 | 409 | 420 | **460** | softmax | 0.90 | 4 | 332 | 10 | 229 / 62 | [png](../results/figures/routes_ls/p4.4.g.png) |
| p4.4.h | 4 | 30 | 482 | 495 | 506 | **544** | geometric | 0.75 | 0 | 431 | 10 | 246 / 69 | [png](../results/figures/routes_ls/p4.4.h.png) |
| p4.4.i | 4 | 32.5 | 472 | 552 | 545 | **627** | geometric | 0.85 | 1 | 531 | 7 | 194 / 100 | [png](../results/figures/routes_ls/p4.4.i.png) |
| p4.4.j | 4 | 35 | 530 | 588 | 633 | **665** | geometric | 0.90 | 6 | 546 | 10 | 259 / 79 | [png](../results/figures/routes_ls/p4.4.j.png) |
| p4.4.k | 4 | 37.5 | 560 | 626 | 708 | **778** | softmax | 0.85 | 4 | 489 | 25 | 251 / 74 | [png](../results/figures/routes_ls/p4.4.k.png) |
| p4.4.l | 4 | 40 | 604 | 639 | 783 | **789** | geometric | 0.55 | 0 | 609 | 17 | 245 / 88 | [png](../results/figures/routes_ls/p4.4.l.png) |
| p4.4.m | 4 | 42.5 | 681 | 694 | 838 | **841** | geometric | 0.60 | 1 | 658 | 19 | 277 / 88 | [png](../results/figures/routes_ls/p4.4.m.png) |
| p4.4.n | 4 | 45 | 715 | 743 | 879 | **879** | deterministic | 0.55 |  | 715 | 18 | 316 / 62 | [png](../results/figures/routes_ls/p4.4.n.png) |
| p4.4.o | 4 | 47.5 | 753 | 821 | 915 | **992** | geometric | 0.65 | 9 | 764 | 21 | 290 / 77 | [png](../results/figures/routes_ls/p4.4.o.png) |
| p4.4.p | 4 | 50 | 778 | 859 | 881 | **1057** | geometric | 0.65 | 8 | 812 | 22 | 275 / 83 | [png](../results/figures/routes_ls/p4.4.p.png) |
| p4.4.q | 4 | 52.5 | 857 | 878 | 1016 | **1086** | softmax | 0.80 | 0 | 754 | 27 | 272 / 81 | [png](../results/figures/routes_ls/p4.4.q.png) |
| p4.4.r | 4 | 55 | 861 | 956 | 1073 | **1133** | softmax | 0.70 | 6 | 770 | 27 | 293 / 69 | [png](../results/figures/routes_ls/p4.4.r.png) |
| p4.4.s | 4 | 57.5 | 911 | 956 | 1126 | **1196** | geometric | 0.35 | 8 | 919 | 27 | 283 / 83 | [png](../results/figures/routes_ls/p4.4.s.png) |
| p4.4.t | 4 | 60 | 929 | 971 | 1172 | **1224** | softmax | 0.65 | 0 | 849 | 38 | 274 / 82 | [png](../results/figures/routes_ls/p4.4.t.png) |
| p5.2.b | 2 | 5 | 20 | 20 | 20 | **20** | deterministic | 0.00 |  | 20 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p5.2.b.png) |
| p5.2.c | 2 | 7.5 | 40 | 50 | 50 | **50** | deterministic | 0.05 |  | 40 | 2 | 378 / 32 | [png](../results/figures/routes_ls/p5.2.c.png) |
| p5.2.d | 2 | 10 | 70 | 80 | 80 | **80** | deterministic | 0.05 |  | 70 | 2 | 353 / 80 | [png](../results/figures/routes_ls/p5.2.d.png) |
| p5.2.e | 2 | 12.5 | 160 | 170 | 180 | **180** | deterministic | 0.05 |  | 160 | 4 | 413 / 11 | [png](../results/figures/routes_ls/p5.2.e.png) |
| p5.2.f | 2 | 15 | 165 | 205 | 240 | **240** | deterministic | 1.00 |  | 150 | 6 | 273 / 110 | [png](../results/figures/routes_ls/p5.2.f.png) |
| p5.2.g | 2 | 17.5 | 245 | 290 | 310 | **315** | geometric | 0.05 | 0 | 230 | 7 | 286 / 132 | [png](../results/figures/routes_ls/p5.2.g.png) |
| p5.2.h | 2 | 20 | 350 | 370 | 350 | **395** | softmax | 0.70 | 8 | 370 | 3 | 332 / 106 | [png](../results/figures/routes_ls/p5.2.h.png) |
| p5.2.i | 2 | 22.5 | 380 | 445 | 410 | **460** | geometric | 0.25 | 7 | 430 | 4 | 210 / 118 | [png](../results/figures/routes_ls/p5.2.i.png) |
| p5.2.j | 2 | 25 | 580 | 580 | 580 | **580** | deterministic | 0.05 |  | 580 | 0 | 426 / 11 | [png](../results/figures/routes_ls/p5.2.j.png) |
| p5.2.k | 2 | 27.5 | 650 | 660 | 670 | **670** | deterministic | 0.05 |  | 650 | 4 | 374 / 46 | [png](../results/figures/routes_ls/p5.2.k.png) |
| p5.2.l | 2 | 30 | 650 | 740 | 740 | **770** | softmax | 0.65 | 3 | 720 | 4 | 260 / 103 | [png](../results/figures/routes_ls/p5.2.l.png) |
| p5.2.m | 2 | 32.5 | 650 | 840 | 820 | **860** | geometric | 0.60 | 2 | 745 | 7 | 245 / 98 | [png](../results/figures/routes_ls/p5.2.m.png) |
| p5.2.n | 2 | 35 | 810 | 840 | 860 | **890** | geometric | 0.85 | 7 | 825 | 3 | 372 / 34 | [png](../results/figures/routes_ls/p5.2.n.png) |
| p5.2.o | 2 | 37.5 | 840 | 990 | 910 | **995** | geometric | 0.45 | 2 | 990 | 1 | 313 / 62 | [png](../results/figures/routes_ls/p5.2.o.png) |
| p5.2.p | 2 | 40 | 1140 | 1150 | 1150 | **1150** | deterministic | 0.05 |  | 1140 | 2 | 367 / 13 | [png](../results/figures/routes_ls/p5.2.p.png) |
| p5.2.q | 2 | 42.5 | 1140 | 1190 | 1190 | **1190** | deterministic | 0.05 |  | 1140 | 2 | 322 / 50 | [png](../results/figures/routes_ls/p5.2.q.png) |
| p5.2.r | 2 | 45 | 1140 | 1235 | 1250 | **1255** | geometric | 0.20 | 5 | 1195 | 4 | 314 / 64 | [png](../results/figures/routes_ls/p5.2.r.png) |
| p5.2.s | 2 | 47.5 | 1140 | 1320 | 1305 | **1320** | geometric | 0.05 | 5 | 1320 | 0 | 308 / 65 | [png](../results/figures/routes_ls/p5.2.s.png) |
| p5.2.t | 2 | 50 | 1140 | 1370 | 1350 | **1380** | geometric | 1.00 | 3 | 1295 | 5 | 321 / 67 | [png](../results/figures/routes_ls/p5.2.t.png) |
| p5.2.u | 2 | 52.5 | 1240 | 1400 | 1320 | **1430** | geometric | 0.10 | 5 | 1370 | 4 | 271 / 81 | [png](../results/figures/routes_ls/p5.2.u.png) |
| p5.2.v | 2 | 55 | 1210 | 1445 | 1430 | **1475** | geometric | 0.05 | 8 | 1440 | 3 | 279 / 78 | [png](../results/figures/routes_ls/p5.2.v.png) |
| p5.2.w | 2 | 57.5 | 1200 | 1510 | 1470 | **1540** | softmax | 0.55 | 8 | 1200 | 14 | 266 / 82 | [png](../results/figures/routes_ls/p5.2.w.png) |
| p5.2.x | 2 | 60 | 1190 | 1555 | 1530 | **1585** | softmax | 0.95 | 2 | 1395 | 10 | 236 / 92 | [png](../results/figures/routes_ls/p5.2.x.png) |
| p5.2.y | 2 | 62.5 | 1190 | 1610 | 1580 | **1635** | geometric | 0.05 | 4 | 1585 | 4 | 254 / 86 | [png](../results/figures/routes_ls/p5.2.y.png) |
| p5.2.z | 2 | 65 | 1550 | 1630 | 1645 | **1680** | geometric | 0.10 | 7 | 1615 | 5 | 250 / 98 | [png](../results/figures/routes_ls/p5.2.z.png) |
| p5.3.b | 3 | 3.3 | 15 | 15 | 15 | **15** | deterministic | 0.00 |  | 15 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p5.3.b.png) |
| p5.3.c | 3 | 5 | 20 | 20 | 20 | **20** | deterministic | 0.00 |  | 20 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p5.3.c.png) |
| p5.3.d | 3 | 6.7 | 60 | 60 | 60 | **60** | deterministic | 0.00 |  | 60 | 0 | 441 / 2 | [png](../results/figures/routes_ls/p5.3.d.png) |
| p5.3.e | 3 | 8.3 | 95 | 95 | 95 | **95** | deterministic | 0.05 |  | 95 | 0 | 440 / 3 | [png](../results/figures/routes_ls/p5.3.e.png) |
| p5.3.f | 3 | 10 | 105 | 110 | 110 | **110** | deterministic | 0.00 |  | 105 | 1 | 323 / 54 | [png](../results/figures/routes_ls/p5.3.f.png) |
| p5.3.g | 3 | 11.7 | 165 | 175 | 185 | **185** | deterministic | 0.00 |  | 165 | 4 | 361 / 59 | [png](../results/figures/routes_ls/p5.3.g.png) |
| p5.3.h | 3 | 13.3 | 240 | 240 | 240 | **240** | deterministic | 0.00 |  | 240 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p5.3.h.png) |
| p5.3.i | 3 | 15 | 245 | 280 | 280 | **335** | geometric | 0.00 | 7 | 265 | 6 | 281 / 161 | [png](../results/figures/routes_ls/p5.3.i.png) |
| p5.3.j | 3 | 16.7 | 315 | 380 | 330 | **470** | softmax | 0.00 | 8 | 355 | 9 | 234 / 103 | [png](../results/figures/routes_ls/p5.3.j.png) |
| p5.3.k | 3 | 18.3 | 410 | 440 | 470 | **495** | softmax | 0.10 | 6 | 405 | 8 | 231 / 121 | [png](../results/figures/routes_ls/p5.3.k.png) |
| p5.3.l | 3 | 20 | 525 | 535 | 525 | **565** | softmax | 1.00 | 2 | 515 | 6 | 266 / 133 | [png](../results/figures/routes_ls/p5.3.l.png) |
| p5.3.m | 3 | 21.7 | 570 | 620 | 580 | **645** | geometric | 0.00 | 6 | 545 | 6 | 286 / 87 | [png](../results/figures/routes_ls/p5.3.m.png) |
| p5.3.n | 3 | 23.3 | 645 | 690 | 645 | **745** | softmax | 0.55 | 1 | 665 | 6 | 260 / 89 | [png](../results/figures/routes_ls/p5.3.n.png) |
| p5.3.o | 3 | 25 | 845 | 870 | 860 | **870** | geometric | 0.05 | 0 | 870 | 0 | 387 / 57 | [png](../results/figures/routes_ls/p5.3.o.png) |
| p5.3.p | 3 | 26.7 | 975 | 990 | 990 | **990** | deterministic | 0.05 |  | 975 | 3 | 387 / 29 | [png](../results/figures/routes_ls/p5.3.p.png) |
| p5.3.q | 3 | 28.3 | 975 | 1045 | 1025 | **1065** | geometric | 0.10 | 3 | 1045 | 4 | 326 / 68 | [png](../results/figures/routes_ls/p5.3.q.png) |
| p5.3.r | 3 | 30 | 975 | 1075 | 1090 | **1100** | softmax | 1.00 | 0 | 930 | 6 | 311 / 72 | [png](../results/figures/routes_ls/p5.3.r.png) |
| p5.3.s | 3 | 31.7 | 975 | 1110 | 1165 | **1180** | geometric | 1.00 | 6 | 1080 | 6 | 236 / 96 | [png](../results/figures/routes_ls/p5.3.s.png) |
| p5.3.t | 3 | 33.3 | 975 | 1210 | 1240 | **1255** | geometric | 0.75 | 6 | 1070 | 9 | 236 / 103 | [png](../results/figures/routes_ls/p5.3.t.png) |
| p5.3.u | 3 | 35 | 1215 | 1245 | 1240 | **1305** | geometric | 0.00 | 9 | 890 | 17 | 369 / 25 | [png](../results/figures/routes_ls/p5.3.u.png) |
| p5.3.v | 3 | 36.7 | 1260 | 1385 | 1290 | **1390** | geometric | 0.05 | 4 | 1280 | 6 | 323 / 36 | [png](../results/figures/routes_ls/p5.3.v.png) |
| p5.3.w | 3 | 38.3 | 1260 | 1435 | 1315 | **1435** | geometric | 0.15 | 4 | 1420 | 1 | 347 / 28 | [png](../results/figures/routes_ls/p5.3.w.png) |
| p5.3.x | 3 | 40 | 1410 | 1500 | 1475 | **1505** | geometric | 0.05 | 4 | 1495 | 2 | 351 / 39 | [png](../results/figures/routes_ls/p5.3.x.png) |
| p5.3.y | 3 | 41.7 | 1410 | 1530 | 1500 | **1530** | geometric | 0.05 | 3 | 1530 | 0 | 313 / 39 | [png](../results/figures/routes_ls/p5.3.y.png) |
| p5.3.z | 3 | 43.3 | 1410 | 1580 | 1550 | **1620** | geometric | 0.35 | 4 | 1580 | 4 | 330 / 47 | [png](../results/figures/routes_ls/p5.3.z.png) |
| p5.4.c | 4 | 3.8 | 20 | 20 | 20 | **20** | deterministic | 0.00 |  | 20 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p5.4.c.png) |
| p5.4.d | 4 | 5 | 20 | 20 | 20 | **20** | deterministic | 0.00 |  | 20 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p5.4.d.png) |
| p5.4.e | 4 | 6.2 | 20 | 20 | 20 | **20** | deterministic | 0.00 |  | 20 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p5.4.e.png) |
| p5.4.f | 4 | 7.5 | 80 | 80 | 80 | **80** | deterministic | 0.00 |  | 80 | 0 | 441 / 2 | [png](../results/figures/routes_ls/p5.4.f.png) |
| p5.4.g | 4 | 8.8 | 135 | 140 | 140 | **140** | deterministic | 0.00 |  | 135 | 1 | 314 / 16 | [png](../results/figures/routes_ls/p5.4.g.png) |
| p5.4.h | 4 | 10 | 140 | 140 | 140 | **140** | deterministic | 0.00 |  | 140 | 0 | 441 / 2 | [png](../results/figures/routes_ls/p5.4.h.png) |
| p5.4.i | 4 | 11.2 | 210 | 220 | 240 | **240** | deterministic | 0.00 |  | 210 | 4 | 272 / 95 | [png](../results/figures/routes_ls/p5.4.i.png) |
| p5.4.j | 4 | 12.5 | 320 | 320 | 340 | **340** | deterministic | 0.05 |  | 320 | 4 | 429 / 8 | [png](../results/figures/routes_ls/p5.4.j.png) |
| p5.4.k | 4 | 13.8 | 320 | 340 | 340 | **340** | deterministic | 0.05 |  | 320 | 4 | 332 / 84 | [png](../results/figures/routes_ls/p5.4.k.png) |
| p5.4.l | 4 | 15 | 320 | 370 | 400 | **430** | geometric | 0.20 | 6 | 305 | 9 | 270 / 157 | [png](../results/figures/routes_ls/p5.4.l.png) |
| p5.4.m | 4 | 16.2 | 480 | 540 | 550 | **550** | deterministic | 0.05 |  | 480 | 6 | 326 / 34 | [png](../results/figures/routes_ls/p5.4.m.png) |
| p5.4.n | 4 | 17.5 | 485 | 560 | 620 | **620** | deterministic | 0.05 |  | 440 | 12 | 240 / 135 | [png](../results/figures/routes_ls/p5.4.n.png) |
| p5.4.o | 4 | 18.8 | 570 | 590 | 590 | **650** | geometric | 0.00 | 0 | 580 | 6 | 231 / 141 | [png](../results/figures/routes_ls/p5.4.o.png) |
| p5.4.p | 4 | 20 | 695 | 700 | 695 | **735** | softmax | 0.85 | 5 | 690 | 5 | 252 / 115 | [png](../results/figures/routes_ls/p5.4.p.png) |
| p5.4.q | 4 | 21.2 | 690 | 730 | 820 | **835** | geometric | 0.00 | 0 | 710 | 9 | 195 / 128 | [png](../results/figures/routes_ls/p5.4.q.png) |
| p5.4.r | 4 | 22.5 | 760 | 855 | 820 | **895** | geometric | 0.00 | 8 | 740 | 9 | 236 / 92 | [png](../results/figures/routes_ls/p5.4.r.png) |
| p5.4.s | 4 | 23.8 | 960 | 1020 | 1020 | **1020** | deterministic | 1.00 |  | 960 | 4 | 338 / 38 | [png](../results/figures/routes_ls/p5.4.s.png) |
| p5.4.t | 4 | 25 | 1110 | 1160 | 1140 | **1160** | geometric | 0.10 | 0 | 1160 | 0 | 344 / 44 | [png](../results/figures/routes_ls/p5.4.t.png) |
| p5.4.u | 4 | 26.2 | 1300 | 1300 | 1300 | **1300** | deterministic | 0.05 |  | 1300 | 0 | 427 / 10 | [png](../results/figures/routes_ls/p5.4.u.png) |
| p5.4.v | 4 | 27.5 | 1300 | 1320 | 1320 | **1320** | deterministic | 0.05 |  | 1300 | 4 | 382 / 21 | [png](../results/figures/routes_ls/p5.4.v.png) |
| p5.4.w | 4 | 28.8 | 1300 | 1355 | 1320 | **1370** | geometric | 0.10 | 7 | 1330 | 4 | 328 / 80 | [png](../results/figures/routes_ls/p5.4.w.png) |
| p5.4.x | 4 | 30 | 1300 | 1380 | 1420 | **1430** | softmax | 1.00 | 0 | 1240 | 12 | 318 / 78 | [png](../results/figures/routes_ls/p5.4.x.png) |
| p5.4.y | 4 | 31.2 | 1300 | 1440 | 1490 | **1490** | deterministic | 0.05 |  | 1300 | 12 | 292 / 62 | [png](../results/figures/routes_ls/p5.4.y.png) |
| p5.4.z | 4 | 32.5 | 1300 | 1490 | 1520 | **1560** | geometric | 0.65 | 9 | 1470 | 4 | 268 / 80 | [png](../results/figures/routes_ls/p5.4.z.png) |
| p6.2.d | 2 | 15 | 168 | 192 | 192 | **192** | deterministic | 0.05 |  | 168 | 4 | 336 / 132 | [png](../results/figures/routes_ls/p6.2.d.png) |
| p6.2.e | 2 | 17.5 | 288 | 318 | 336 | **360** | softmax | 0.75 | 6 | 294 | 3 | 278 / 88 | [png](../results/figures/routes_ls/p6.2.e.png) |
| p6.2.f | 2 | 20 | 588 | 588 | 588 | **588** | deterministic | 0.05 |  | 564 | 4 | 429 / 10 | [png](../results/figures/routes_ls/p6.2.f.png) |
| p6.2.g | 2 | 22.5 | 654 | 654 | 660 | **660** | deterministic | 0.00 |  | 654 | 1 | 428 / 12 | [png](../results/figures/routes_ls/p6.2.g.png) |
| p6.2.h | 2 | 25 | 648 | 738 | 780 | **780** | deterministic | 0.05 |  | 648 | 6 | 281 / 108 | [png](../results/figures/routes_ls/p6.2.h.png) |
| p6.2.i | 2 | 27.5 | 762 | 804 | 888 | **888** | deterministic | 0.05 |  | 648 | 10 | 225 / 106 | [png](../results/figures/routes_ls/p6.2.i.png) |
| p6.2.j | 2 | 30 | 840 | 864 | 936 | **936** | deterministic | 0.05 |  | 840 | 4 | 351 / 52 | [png](../results/figures/routes_ls/p6.2.j.png) |
| p6.2.k | 2 | 32.5 | 840 | 900 | 1020 | **1032** | geometric | 0.00 | 4 | 828 | 10 | 312 / 70 | [png](../results/figures/routes_ls/p6.2.k.png) |
| p6.2.l | 2 | 35 | 840 | 894 | 1080 | **1098** | softmax | 0.25 | 8 | 798 | 15 | 266 / 104 | [png](../results/figures/routes_ls/p6.2.l.png) |
| p6.2.m | 2 | 37.5 | 840 | 948 | 1152 | **1152** | deterministic | 0.35 |  | 840 | 14 | 231 / 111 | [png](../results/figures/routes_ls/p6.2.m.png) |
| p6.2.n | 2 | 40 | 840 | 954 | 1152 | **1224** | geometric | 1.00 | 4 | 660 | 24 | 275 / 87 | [png](../results/figures/routes_ls/p6.2.n.png) |
| p6.3.g | 3 | 15 | 252 | 276 | 276 | **276** | deterministic | 0.05 |  | 252 | 4 | 294 / 109 | [png](../results/figures/routes_ls/p6.3.g.png) |
| p6.3.h | 3 | 16.7 | 414 | 414 | 414 | **414** | deterministic | 0.05 |  | 414 | 0 | 412 / 12 | [png](../results/figures/routes_ls/p6.3.h.png) |
| p6.3.i | 3 | 18.3 | 564 | 600 | 642 | **642** | deterministic | 0.80 |  | 540 | 7 | 272 / 95 | [png](../results/figures/routes_ls/p6.3.i.png) |
| p6.3.j | 3 | 20 | 786 | 798 | 810 | **828** | geometric | 0.10 | 9 | 798 | 5 | 383 / 22 | [png](../results/figures/routes_ls/p6.3.j.png) |
| p6.3.k | 3 | 21.7 | 840 | 852 | 876 | **888** | geometric | 0.80 | 1 | 762 | 8 | 338 / 44 | [png](../results/figures/routes_ls/p6.3.k.png) |
| p6.3.l | 3 | 23.3 | 858 | 888 | 924 | **972** | geometric | 0.00 | 7 | 876 | 7 | 358 / 59 | [png](../results/figures/routes_ls/p6.3.l.png) |
| p6.3.m | 3 | 25 | 858 | 960 | 990 | **1080** | geometric | 0.35 | 6 | 918 | 8 | 295 / 106 | [png](../results/figures/routes_ls/p6.3.m.png) |
| p6.3.n | 3 | 26.7 | 888 | 978 | 1062 | **1170** | geometric | 0.45 | 8 | 936 | 11 | 244 / 114 | [png](../results/figures/routes_ls/p6.3.n.png) |
| p6.4.j | 4 | 15 | 336 | 360 | 336 | **360** | geometric | 0.00 | 0 | 354 | 1 | 313 / 62 | [png](../results/figures/routes_ls/p6.4.j.png) |
| p6.4.k | 4 | 16.2 | 390 | 486 | 444 | **528** | softmax | 0.00 | 7 | 480 | 4 | 199 / 100 | [png](../results/figures/routes_ls/p6.4.k.png) |
| p6.4.l | 4 | 17.5 | 540 | 600 | 588 | **660** | geometric | 0.00 | 5 | 576 | 5 | 265 / 90 | [png](../results/figures/routes_ls/p6.4.l.png) |
| p6.4.m | 4 | 18.8 | 876 | 900 | 888 | **912** | geometric | 0.80 | 0 | 840 | 6 | 358 / 25 | [png](../results/figures/routes_ls/p6.4.m.png) |
| p6.4.n | 4 | 20 | 1020 | 1032 | 1032 | **1068** | geometric | 0.55 | 5 | 1032 | 6 | 368 / 28 | [png](../results/figures/routes_ls/p6.4.n.png) |
| p7.2.a | 2 | 10 | 30 | 30 | 30 | **30** | deterministic | 0.00 |  | 30 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p7.2.a.png) |
| p7.2.b | 2 | 20 | 64 | 64 | 64 | **64** | deterministic | 0.00 |  | 64 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p7.2.b.png) |
| p7.2.c | 2 | 30 | 101 | 101 | 101 | **101** | deterministic | 0.00 |  | 101 | 0 | 350 / 6 | [png](../results/figures/routes_ls/p7.2.c.png) |
| p7.2.d | 2 | 40 | 159 | 164 | 159 | **183** | softmax | 0.45 | 9 | 152 | 2 | 287 / 17 | [png](../results/figures/routes_ls/p7.2.d.png) |
| p7.2.e | 2 | 50 | 254 | 265 | 271 | **289** | softmax | 1.00 | 0 | 238 | 3 | 245 / 54 | [png](../results/figures/routes_ls/p7.2.e.png) |
| p7.2.f | 2 | 60 | 285 | 318 | 351 | **370** | geometric | 0.25 | 0 | 290 | 5 | 224 / 87 | [png](../results/figures/routes_ls/p7.2.f.png) |
| p7.2.g | 2 | 70 | 330 | 371 | 410 | **448** | geometric | 0.05 | 3 | 317 | 10 | 216 / 88 | [png](../results/figures/routes_ls/p7.2.g.png) |
| p7.2.h | 2 | 80 | 373 | 427 | 436 | **483** | softmax | 0.85 | 2 | 398 | 7 | 211 / 85 | [png](../results/figures/routes_ls/p7.2.h.png) |
| p7.2.i | 2 | 90 | 411 | 450 | 471 | **531** | softmax | 0.00 | 8 | 331 | 13 | 219 / 91 | [png](../results/figures/routes_ls/p7.2.i.png) |
| p7.2.j | 2 | 100 | 452 | 528 | 529 | **607** | softmax | 0.10 | 3 | 430 | 13 | 196 / 102 | [png](../results/figures/routes_ls/p7.2.j.png) |
| p7.2.k | 2 | 110 | 489 | 558 | 589 | **669** | softmax | 0.20 | 7 | 458 | 15 | 172 / 103 | [png](../results/figures/routes_ls/p7.2.k.png) |
| p7.2.l | 2 | 120 | 532 | 597 | 656 | **731** | softmax | 0.20 | 5 | 476 | 19 | 185 / 106 | [png](../results/figures/routes_ls/p7.2.l.png) |
| p7.2.m | 2 | 130 | 584 | 639 | 728 | **773** | softmax | 0.25 | 5 | 568 | 17 | 152 / 116 | [png](../results/figures/routes_ls/p7.2.m.png) |
| p7.2.n | 2 | 140 | 633 | 665 | 758 | **831** | geometric | 0.45 | 3 | 595 | 16 | 193 / 102 | [png](../results/figures/routes_ls/p7.2.n.png) |
| p7.2.o | 2 | 150 | 666 | 708 | 820 | **901** | softmax | 0.15 | 5 | 553 | 18 | 180 / 111 | [png](../results/figures/routes_ls/p7.2.o.png) |
| p7.2.p | 2 | 160 | 684 | 780 | 837 | **918** | softmax | 0.20 | 2 | 590 | 19 | 175 / 116 | [png](../results/figures/routes_ls/p7.2.p.png) |
| p7.2.q | 2 | 170 | 741 | 816 | 917 | **982** | softmax | 0.30 | 0 | 705 | 16 | 185 / 108 | [png](../results/figures/routes_ls/p7.2.q.png) |
| p7.2.r | 2 | 180 | 775 | 842 | 959 | **1025** | geometric | 0.30 | 2 | 735 | 16 | 183 / 110 | [png](../results/figures/routes_ls/p7.2.r.png) |
| p7.2.s | 2 | 190 | 822 | 867 | 984 | **1083** | softmax | 0.20 | 4 | 714 | 26 | 161 / 115 | [png](../results/figures/routes_ls/p7.2.s.png) |
| p7.2.t | 2 | 200 | 821 | 889 | 1007 | **1111** | geometric | 0.15 | 0 | 695 | 27 | 176 / 113 | [png](../results/figures/routes_ls/p7.2.t.png) |
| p7.3.b | 3 | 13.3 | 46 | 46 | 46 | **46** | deterministic | 0.00 |  | 46 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p7.3.b.png) |
| p7.3.c | 3 | 20 | 79 | 79 | 79 | **79** | deterministic | 0.00 |  | 79 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p7.3.c.png) |
| p7.3.d | 3 | 26.7 | 105 | 117 | 105 | **117** | softmax | 0.00 | 2 | 117 | 0 | 400 / 4 | [png](../results/figures/routes_ls/p7.3.d.png) |
| p7.3.e | 3 | 33.3 | 151 | 170 | 168 | **170** | geometric | 0.00 | 1 | 156 | 1 | 263 / 20 | [png](../results/figures/routes_ls/p7.3.e.png) |
| p7.3.f | 3 | 40 | 211 | 225 | 227 | **247** | geometric | 0.90 | 3 | 217 | 2 | 291 / 23 | [png](../results/figures/routes_ls/p7.3.f.png) |
| p7.3.g | 3 | 46.7 | 312 | 322 | 344 | **344** | deterministic | 0.25 |  | 292 | 5 | 252 / 55 | [png](../results/figures/routes_ls/p7.3.g.png) |
| p7.3.h | 3 | 53.3 | 340 | 370 | 377 | **425** | softmax | 0.70 | 5 | 341 | 7 | 185 / 82 | [png](../results/figures/routes_ls/p7.3.h.png) |
| p7.3.i | 3 | 60 | 400 | 413 | 480 | **480** | deterministic | 0.35 |  | 400 | 8 | 202 / 94 | [png](../results/figures/routes_ls/p7.3.i.png) |
| p7.3.j | 3 | 66.7 | 417 | 460 | 516 | **554** | geometric | 0.10 | 9 | 393 | 12 | 188 / 101 | [png](../results/figures/routes_ls/p7.3.j.png) |
| p7.3.k | 3 | 73.3 | 472 | 495 | 591 | **596** | geometric | 0.05 | 0 | 393 | 18 | 201 / 97 | [png](../results/figures/routes_ls/p7.3.k.png) |
| p7.3.l | 3 | 80 | 532 | 582 | 609 | **655** | geometric | 0.35 | 3 | 525 | 10 | 206 / 101 | [png](../results/figures/routes_ls/p7.3.l.png) |
| p7.3.m | 3 | 86.7 | 584 | 645 | 687 | **741** | softmax | 0.25 | 9 | 523 | 16 | 208 / 98 | [png](../results/figures/routes_ls/p7.3.m.png) |
| p7.3.n | 3 | 93.3 | 574 | 637 | 623 | **789** | softmax | 0.20 | 6 | 540 | 18 | 205 / 104 | [png](../results/figures/routes_ls/p7.3.n.png) |
| p7.3.o | 3 | 100 | 626 | 666 | 708 | **824** | geometric | 0.15 | 2 | 589 | 16 | 178 / 108 | [png](../results/figures/routes_ls/p7.3.o.png) |
| p7.3.p | 3 | 106.7 | 660 | 707 | 862 | **871** | softmax | 0.20 | 1 | 579 | 20 | 166 / 115 | [png](../results/figures/routes_ls/p7.3.p.png) |
| p7.3.q | 3 | 113.3 | 748 | 764 | 832 | **958** | geometric | 0.20 | 7 | 625 | 21 | 181 / 102 | [png](../results/figures/routes_ls/p7.3.q.png) |
| p7.3.r | 3 | 120 | 740 | 787 | 884 | **991** | softmax | 0.25 | 0 | 677 | 22 | 158 / 117 | [png](../results/figures/routes_ls/p7.3.r.png) |
| p7.3.s | 3 | 126.7 | 767 | 825 | 917 | **1024** | softmax | 0.45 | 7 | 743 | 21 | 161 / 120 | [png](../results/figures/routes_ls/p7.3.s.png) |
| p7.3.t | 3 | 133.3 | 775 | 854 | 963 | **1067** | softmax | 0.30 | 9 | 754 | 23 | 153 / 122 | [png](../results/figures/routes_ls/p7.3.t.png) |
| p7.4.b | 4 | 10 | 30 | 30 | 30 | **30** | deterministic | 0.00 |  | 30 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p7.4.b.png) |
| p7.4.c | 4 | 15 | 46 | 46 | 46 | **46** | deterministic | 0.00 |  | 46 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p7.4.c.png) |
| p7.4.d | 4 | 20 | 79 | 79 | 79 | **79** | deterministic | 0.00 |  | 79 | 0 | 441 / 1 | [png](../results/figures/routes_ls/p7.4.d.png) |
| p7.4.e | 4 | 25 | 123 | 123 | 123 | **123** | deterministic | 0.00 |  | 123 | 0 | 380 / 5 | [png](../results/figures/routes_ls/p7.4.e.png) |
| p7.4.f | 4 | 30 | 158 | 164 | 158 | **164** | geometric | 0.55 | 1 | 155 | 2 | 330 / 9 | [png](../results/figures/routes_ls/p7.4.f.png) |
| p7.4.g | 4 | 35 | 198 | 209 | 201 | **209** | geometric | 0.00 | 0 | 197 | 2 | 277 / 14 | [png](../results/figures/routes_ls/p7.4.g.png) |
| p7.4.h | 4 | 40 | 260 | 277 | 274 | **279** | geometric | 0.50 | 2 | 254 | 2 | 279 / 30 | [png](../results/figures/routes_ls/p7.4.h.png) |
| p7.4.i | 4 | 45 | 323 | 346 | 354 | **363** | softmax | 0.80 | 4 | 331 | 3 | 276 / 54 | [png](../results/figures/routes_ls/p7.4.i.png) |
| p7.4.j | 4 | 50 | 397 | 411 | 446 | **451** | geometric | 0.60 | 2 | 406 | 4 | 238 / 68 | [png](../results/figures/routes_ls/p7.4.j.png) |
| p7.4.k | 4 | 55 | 398 | 428 | 467 | **511** | softmax | 0.35 | 0 | 428 | 8 | 191 / 102 | [png](../results/figures/routes_ls/p7.4.k.png) |
| p7.4.l | 4 | 60 | 498 | 511 | 578 | **578** | deterministic | 0.35 |  | 498 | 8 | 195 / 110 | [png](../results/figures/routes_ls/p7.4.l.png) |
| p7.4.m | 4 | 65 | 471 | 567 | 578 | **634** | geometric | 0.25 | 0 | 511 | 10 | 204 / 92 | [png](../results/figures/routes_ls/p7.4.m.png) |
| p7.4.n | 4 | 70 | 520 | 575 | 639 | **682** | geometric | 0.05 | 7 | 517 | 17 | 172 / 107 | [png](../results/figures/routes_ls/p7.4.n.png) |
| p7.4.o | 4 | 75 | 587 | 622 | 738 | **741** | softmax | 0.40 | 1 | 593 | 14 | 217 / 104 | [png](../results/figures/routes_ls/p7.4.o.png) |
| p7.4.p | 4 | 80 | 673 | 733 | 803 | **807** | softmax | 0.40 | 1 | 662 | 13 | 245 / 92 | [png](../results/figures/routes_ls/p7.4.p.png) |
| p7.4.q | 4 | 85 | 685 | 730 | 796 | **843** | softmax | 0.50 | 6 | 709 | 11 | 214 / 93 | [png](../results/figures/routes_ls/p7.4.q.png) |
| p7.4.r | 4 | 90 | 681 | 725 | 836 | **911** | softmax | 0.15 | 8 | 632 | 20 | 177 / 116 | [png](../results/figures/routes_ls/p7.4.r.png) |
| p7.4.s | 4 | 95 | 757 | 799 | 870 | **954** | softmax | 0.25 | 8 | 718 | 17 | 170 / 110 | [png](../results/figures/routes_ls/p7.4.s.png) |
| p7.4.t | 4 | 100 | 775 | 836 | 866 | **1021** | softmax | 0.25 | 8 | 750 | 20 | 178 / 115 | [png](../results/figures/routes_ls/p7.4.t.png) |

Instancias sin ninguna ruta factible (34): p1.2.a, p1.3.a, p1.3.b, p1.4.a, p1.4.b, p1.4.c, p4.3.a, p4.4.a, p4.4.b, p4.4.c, p5.2.a, p5.3.a, p5.4.a, p5.4.b, p6.2.a, p6.2.b, p6.2.c, p6.3.a, p6.3.b, p6.3.c, p6.3.d, p6.3.e, p6.3.f, p6.4.a, p6.4.b, p6.4.c, p6.4.d, p6.4.e, p6.4.f, p6.4.g, p6.4.h, p6.4.i, p7.3.a, p7.4.a.
