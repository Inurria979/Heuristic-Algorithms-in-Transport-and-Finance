# Comparación con el BKS de todas las instancias, por conjunto

Informe generado por `pjs bks` (`TOP/src/rust`); se regenera con

```
cd TOP
ignore/target/release/pjs.exe bks --report docs/comparacion_bks_conjuntos.md src/data/bks_chao.csv results/best_solution.csv results/bks_by_set.csv
```

## Qué se compara

- **BKS:** la mejor solución conocida de cada instancia, columna *BEST* de la hoja de resultados que publican Dang, Guibadj y Moukrim con PSOiA (`src/data/bks_chao.csv`; el original está en `ignore/downloads/solutions-psoia.zip`). En las 80 instancias de las tablas 3 y 4 de Panadero coincide con su BKS (Ke et al., 2016).
- **Nuestros resultados:** la rejilla de [`informe_busqueda_local.md`](informe_busqueda_local.md) (`results/best_solution.csv`): la determinista con el mejor α, la mejor sin búsqueda local (determinista o GRASP de 10 iteraciones) y la final, con búsqueda local sobre las élite.
- **Panadero, 128 hilos:** la distancia media al BKS de cada conjunto que da su figura 4 (1 s por hilo). El paper no publica los valores por instancia de p3 a p7, solo estas medias.
- **Distancia:** 100 · (BKS − nuestra) / BKS, media por instancia, sobre las instancias con BKS > 0.

## Distancia media al BKS por conjunto

| Conjunto | Instancias | Determinista, mejor α | Mejor sin búsqueda local | **Final** | Panadero, 128 hilos (figura 4) | BKS alcanzado (final) |
|---|---|---|---|---|---|---|
| p1 | 48 | 6.88 % | 3.15 % | **0.58 %** | 0.69 % | 40 de 48 |
| p2 | 33 | 1.81 % | 0.31 % | **0.00 %** | 0.11 % | 33 de 33 |
| p3 | 60 | 6.84 % | 2.96 % | **0.67 %** | 1.44 % | 48 de 60 |
| p4 | 56 | 29.25 % | 23.73 % | **5.87 %** | 7.97 % | 5 de 56 |
| p5 | 74 | 11.82 % | 4.49 % | **1.30 %** | 1.46 % | 35 de 74 |
| p6 | 24 | 14.45 % | 8.51 % | **1.17 %** | 1.42 % | 14 de 24 |
| p7 | 58 | 19.75 % | 14.48 % | **3.42 %** | 5.10 % | 14 de 58 |
| **Todas** | 353 | 13.61 % | 8.63 % | **2.04 %** | 2.85 % (media ponderada) | 189 de 353 |

- **Final frente a Panadero con 128 hilos:** menor distancia media en 7 de los 7 conjuntos; sobre todas las instancias, 2.04 % frente a 2.85 %.
- **Por encima del BKS:** 0 instancias con la determinista, 0 sin búsqueda local y 0 con la final. Superar el BKS indicaría una solución no factible o un récord nuevo.
- **Recompensa total:** BKS 195931, final 189854 (3.10 % por debajo).
- **Cómputo:** la rejilla gasta unos 140 s para las 387 instancias con 12 hilos; Panadero, 1 s de reloj por instancia con 128 hilos. Las medias de la figura 4 son de 128 hilos, la mejor configuración del paper.
