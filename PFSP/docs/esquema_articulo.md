# Esquema del artículo (PDF, 3 páginas) — PFSP

> Guion del artículo `docs/Permutation Flow-Shop Problem.tex` → `docs/Javier S Jesus I Permutation Flow-Shop Problem.pdf`.
> Fuentes: notebook `src/PFSP_RandNEH_ILS_ESP_en.ipynb` (código; sus salidas guardadas son copia de la ejecución de la
> versión española `PFSP_RandNEH_ILS_ESP.ipynb`, con el mismo código), CSV de `results/` y la presentación
> `docs/PFSP_RandNEH_adaptive_beta.pdf`. No se ha ejecutado nada del proyecto: solo se han leído salidas guardadas y CSV.
> Los estadísticos derivados (rangos, medias entre instancias, recuentos) salen de los scripts de un solo uso
> `ignore/paper_loop/tmp/stats.py` y `stats2.py`, que solo leen los CSV.

**Título:** *NEH, Biased Randomisation and ILS-ESP for the Permutation Flow-Shop Problem: an Adaptive β on Five Taillard Instances*

---

## Requisito del enunciado → Dónde se cubre

**No hay enunciado en el proyecto** (ni imagen en `docs/`, ni diapositiva *Homework* en el único PDF, que es nuestra
presentación, ni README). Se infiere de la portada de la presentación: *"RandNEH and ILS-ESP of Juan et al. (2014),
built in Python with Numba and tested on five Taillard instances, from 20 to 500 jobs"*.

| Requisito (inferido) | Dónde se cubre |
|---|---|
| Artículo de 3 páginas en inglés | Todo el documento |
| Describir el PFSP | §1 (palabras llanas) y §2 (modelo, ec. 1) |
| Describir NEH, RandNEH (aleatorización sesgada) e ILS-ESP de Juan et al. (2014) | §3 |
| Implementarlos | §4 (notebook, Numba, aceleración de Taillard, comprobaciones) |
| Probar en instancias de Taillard | §1 (instancias), §5 (Tablas 1 y 2) |
| Propuesta propia (β adaptativo, Geo-adapt) | §3 (definición), §5 *Adaptive β*, Conclusiones |
| Código | Párrafo *Code availability* (notebook, versiones anteriores); si el enunciado lo pide como apéndice, hay que decidir qué listar |

## Presupuesto por sección (resultado: 3 páginas justas, la 3.ª llena, con unos 29 pt libres)

| § | Contenido | Espacio |
|---|---|---|
| Abstract | 135 palabras | 9 líneas |
| 1 Introduction | problema, Juan et al., preguntas, alcance, instancias | ~0.35 pág. |
| 2 Model | ec. (1), lectura, aceleración de Taillard | ~0.3 pág. |
| 3 NEH, RandNEH, ILS-ESP | NEH, 3 distribuciones, bucle Fig. 6, Geo-adapt, ILS-ESP en 3 pasos | ~0.6 pág. |
| 4 Implementation | notebook, supuestos, validación, protocolo A/B, métrica, test | ~0.45 pág. |
| 5 Results | Tabla 1 (exp. A), Tabla 2 (exp. B), 5 párrafos con cabeza | ~1.1 pág. |
| 6 Conclusions + Code + Refs | | ~0.35 pág. |

Prosa: 2 369 palabras tras la ronda 5 (por encima de 1 700–1 950): no hay figura, sino dos tablas, y el texto cabe en 3 páginas.

## Mensajes clave

1. El RandNEH de la Fig. 6 (bucle hasta la primera construcción que mejora a NEH) mejora a NEH en media en las 5 instancias con cualquier distribución; la ganancia es modesta (0.10–0.73 puntos con sesgo) porque se para en la primera mejora.
2. La geométrica mantiene el mismo sesgo para cualquier r; la triangular (media ≈ r/3) y la uniforme no, por eso necesitan muchas más construcciones.
3. Meseta β = 0.3–0.7: misma calidad con 1.2–7.4 construcciones (n ≥ 50).
4. Geo-adapt iguala la calidad sin elegir β, pero cuesta como un β fijo bajo, cerca de Geo(0.05)–(0.1); la rampa es lenta para n = 20 (63 % de éxito).
5. En ILS-ESP el arranque apenas importa: < 0.3 puntos entre arranques, como el ruido entre semillas; la primera búsqueda local borra la ventaja.
6. tai081 (aleatorizados mejor, Geo-adapt p = 0.017) y tai117 (NEH mejor) son indicios, no pruebas: 20 tests no triviales sin corregir, y ninguno sobrevive a Bonferroni.

## Cifras y fuentes

| Cifra en el artículo | Valor | Fuente |
|---|---|---|
| Instancias y tamaños | tai001 20×5 … tai117 500×20 | notebook celda 3 (salida) |
| BKS | 1 278 / 3 846 / 6 171 / 11 158 / 26 389 | notebook celda 1 (`BKS`) |
| Óptimos demostrados | tai101, tai117 | notebook celda 1 (`PROVEN_OPTIMAL`), celda 2 |
| NEH acelerado = ingenuo | 1 286 (tai001), 4 082 (tai051) | notebook celda 9 (salida) |
| Posición media Tri r=20/100 | 6.17 / 32.87 | notebook celda 11 (salida) |
| Posición media Geo(0.1), Geo(0.3) r=100 | 8.97 / 2.32 | notebook celda 11 (salida) |
| Tabla 1 (gap, Con., ms tai117) | todas las celdas | `results/expA_randneh_resumen.csv`: `gap_medio`, `construcciones_medias`, `ms_por_ejecucion` |
| Columna Mean Tabla 1 | 3.64 / 3.39 / 3.37 / 3.36 / 3.32 / 3.32 / 3.29 / 3.27 / 3.48 / 3.35 | media de `gap_medio` por `dist` (stats.py) |
| % ejecuciones que baten a NEH | 100 % n≥50 (Tri, Geo); Unif 67/80/97; tai001 33–97 %, Geo(0.05) 40 | `expA_randneh_resumen.csv`, `pct_mejora_NEH` |
| Ganancia sobre NEH | 0.10–0.73 (sesgadas); 0.06 Unif tai117 | stats.py: NEH − `gap_medio` (mín. Tri tai117 0.096; máx. Geo(0.7) tai051 0.735) |
| Meseta 0.3–0.7: gaps dentro de 0.15 | máx. tai051 5.547−5.401 = 0.146 | `expA_randneh_resumen.csv` |
| Meseta: 1.2–7.4 construcciones | n ≥ 50 | `expA_randneh_resumen.csv` |
| Tri 14.3–14.5 constr., 143 ms; Geo(0.7) 53 ms; Unif 149.9 constr., 2 429 ms | | `expA_randneh_resumen.csv` |
| Geo-adapt 11.3 vs 5.6–6.9 (tai101) | 11.33; 6.90 (Geo(0.5)); 5.60 (Geo(0.7)) | `expA_randneh_resumen.csv`; desde r5 también en el abstract, en lugar de 77 vs 53 ms, que sigue solo en la Tabla 1 |
| Tiempos: 15–17 ms por construcción en tai117, con el NEH inicial incluido | 14.82–16.93 en las 9 filas aleatorizadas; NEH suelto 14.62 | `expA_randneh_resumen.csv`, `ms_por_ejecucion / (1 + construcciones_medias)`; notebook celda 7 (`rand_neh_paper` recalcula NEH) (tmp/r5_check.py) |
| Tiempos: Geo(0.3) en tai117 en dos ejecuciones guardadas, 1.8 y 3.2 construcciones; 39 y 69 ms | 1.80 / 39.48 (v2); 3.20 / 69.02 (final) | `results/v2_antes_geoadapt/expA_randneh_resumen.csv`, fila `Geo(0.3) (paper)`, y `expA_randneh_resumen.csv`; 30 ejecuciones en ambas y mismo `rand_neh_paper`; cambia la semilla, porque depende del índice en `CONFIGS_A` (tmp/r5_check.py) |
| Experimento A en un proceso | | notebook celda 13 (`run_A`, sin `Pool`; el `Pool` es solo del experimento B, celda 23) |
| β de éxito: mediana 0.05 (tai051, tai117), 0.175 tai101 (0.05–0.50) | | `results/expA_geoadapt_beta.csv` (stats.py) |
| 134 de 139 éxitos con β ≤ 0.35; máx. 0.60 | | `expA_geoadapt_beta.csv` (stats.py) |
| tai001: 37 % sin éxito, 63 % éxito; rampa hasta 0.35 | 11 de 30 NaN | `expA_geoadapt_beta.csv`; fórmula de la rampa con t ≤ 20 |
| Tabla 2 (media ± std, p) | todas las celdas | `results/expB_ilsesp_resumen.csv` (`gap_final_medio`, `gap_final_std`), `results/expB_tests_vs_NEH.csv` (`p_valor`) |
| Columna Mean Tabla 2 | 1.20 / 1.13 / 1.17 / 1.19 / 1.16 / 1.16 | stats2.py (media de `gap_final_medio` por `init`) |
| Iteraciones por instancia | 1 439–1 489, 604–642, 322–338, 231–262; tai001 4 732–5 037 | `expB_ilsesp_resumen.csv`, `iters` |
| Presupuestos | 0.2 / 2 / 4 / 8 / 20 s | `expB_ilsesp_raw.csv`, `budget_s`; `TIME_FACTOR = 2.0` (celda 1) |
| Diferencia entre arranques < 0.3; 0.07 tai101, 0.30 tai081 | 0.065 / 0.298 | `expB_ilsesp_resumen.csv` (stats.py) |
| Desviación entre semillas 0.09–0.32 | 0.088–0.317 | `expB_ilsesp_resumen.csv`, `gap_final_std` (sin tai001) |
| tai001 gap 0 en 60 de 60 | | `expB_ilsesp_raw.csv` (stats2.py) |
| tai051 tras LS: 3.94 / 4.21 / 3.92; inicio 6.14 / 5.73 / 5.81 | | `expB_ilsesp_resumen.csv`, `gap_tras_LS`, `gap_init` |
| tai081: aleatorizados por debajo de NEH 0.09–0.30 | | `expB_tests_vs_NEH.csv`, `dif_gap_medio_vs_NEH` |
| tai117: NEH 0.54 vs 0.57–0.64, p 0.063–0.28 | | `expB_ilsesp_resumen.csv`, `expB_tests_vs_NEH.csv` |
| ~~Cambio de semillas 0.71 → 0.63 en tai117~~ | retirado en r4 (I2, decisión de los autores) | 0.63 no está en ningún CSV |
| 300 ejecuciones en 17.2 min, 2 procesos | | notebook celdas 22–23 (salidas), `N_PROCS` |
| Bloques de unos 20 ms; en tai117, una iteración de unos 80 ms | 76.3–86.6 ms | notebook celda 20 (`ils_esp`, `chunk ≥ 1`); `budget_s` / media de `iters` en `expB_ilsesp_raw.csv` (tmp/r4_check.py) |
| 3 de 300 ejecuciones mejoran en el último bloque | tai101 NEH s2029, tai101 Tri s2127, tai051 Geo(0.1) s2231 | último punto de `results/expB_curvas.npz` (t = presupuesto) frente a `gap_final` de `expB_ilsesp_raw.csv` (tmp/r4_check.py) |
| Comprobaciones de validez sobre una muestra: 1 ejecución por distribución e instancia, 5 de Geo-adapt | | notebook celda 9 |
| Semilla, test Mann-Whitney bilateral | | notebook celdas 1, 29 |
| Chequeo previo β 0.3–0.9 | | `results/chequeo_beta_alto.csv` (idéntico en `v2_antes_geoadapt/`); ningún notebook lo escribe; notebook celda 32 |
| El experimento A no tiene test de significación | | notebook: el único `mannwhitneyu` está en la celda 29, experimento B |
| Geo-adapt cuesta cerca de Geo(0.05)–Geo(0.1) | constr. 14.63 / 3.13 / 7.20 / 11.33 / 3.53 frente a Geo(0.05) 16.07 / 2.77 / 12.10 / 13.93 / 3.50 y Geo(0.1) 13.80 / 2.53 / 8.27 / 10.63 / 2.73 | `expA_randneh_resumen.csv`, `construcciones_medias` |
| Meseta 0.3–0.7 entre las más baratas en construcciones (la más barata en 4 de 5 instancias; en tai117, Geo(0.2) 2.2 frente a 2.4 de Geo(0.7)) | | `expA_randneh_resumen.csv`, `construcciones_medias` (tmp/r4_check.py) |
| Salidas guardadas del `_en` = ejecución española | | cadenas en español en las salidas de `_en`, celdas 3, 20, 22 y 31; código igual (`verificacion_codigo.md`) |
| ~~Presupuesto del paper 10·n·m ms; promedia 10 instancias por tamaño~~ | retirado en r2 | sin fuente comprobable (I3) |
| Una construcción sola peor que NEH en media en 32 de 35 pares (200 por caso) | | `results/v1_una_construccion/expA_randneh_resumen.csv` (tmp/v1.py) |
| Geo(0.05): posición media 7.81 (r=20) y 18.45 (r=100) | | notebook celda 11 (salida) |
| tai081: déficit de NEH 0.20–0.33 antes de LS, 0.35–0.57 después (4.81 vs 4.24–4.45) | 0.3549–0.5704 | `expB_ilsesp_resumen.csv`, `gap_init`, `gap_tras_LS` |
| tai117 tras LS: NEH 0.94 vs 0.91–1.07 | | `expB_ilsesp_resumen.csv`, `gap_tras_LS` |
| 20 tests no triviales (5 de tai001 con p = 1) | | `expB_tests_vs_NEH.csv` |
| Mejor de 10 semillas desde Tri | 0.00 / 0.65 / 1.72 / 1.42 / 0.45; 1.43 en tai101 al agotarse el presupuesto | `results/resumen_global.csv`, «ILS-ESP desde Tri, mejor» (tai101 = 1.42499 → 1.42; la presentación pone 1.43); 1.43395 en `expB_curvas.npz`, seed 2127 (tmp/r4_check.py) |
| Python 3.13 | | metadatos del kernel de los dos notebooks |
| Abstract: 11.3 frente a 5.6–6.9 construcciones en 200 trabajos | 11.33 / 6.90 (Geo(0.5)) / 5.60 (Geo(0.7)) | `expA_randneh_resumen.csv`, `construcciones_medias`, tai101 |
| Conclusiones: Geo(0.1)–Geo(0.7) con menos construcciones que Tri y Unif en las 5 instancias; gap medio 3.27–3.36 frente a 3.39 (Tri) y 3.48 (Unif) | | `expA_randneh_resumen.csv` |

## Tablas y figuras

- **Tabla 1**: experimento A, filas = 10 configuraciones, columnas gap y construcciones por instancia, media de gaps y ms en tai117.
- **Tabla 2**: experimento B, filas = 6 arranques, columnas media ± std y p por instancia (sin tai001), media y fila de iteraciones.
- **Figura: ninguna.** No existe `results/figures/`. Las figuras guardadas en las salidas del notebook (convergencia de
  ILS-ESP, histograma de β de Geo-adapt, boxplots) tienen títulos y ejes **en español** también en la versión `_en`
  (las salidas se copiaron de la ejecución española), y las gráficas de la presentación son gráficos nativos de PowerPoint,
  no imágenes. Por eso no se ha extraído ninguna.

## Referencias

Ningún paper está en `docs/`: los datos bibliográficos **no se han podido comprobar contra un PDF**.
- Juan, Lourenço, Mateo, Luo, Castellà (2014), *Int. Trans. Oper. Res.* 21(1):103–126 — citado en el notebook como «Juan et al. (2014)», Fig. 6; datos de memoria.
- Nawaz, Enscore, Ham (1983), *Omega* 11(1):91–95 — presentación diap. 3 («Nawaz, Enscore and Ham (1983)»); resto de memoria.
- Taillard (1990), *EJOR* 47(1):65–74 (aceleración) — de memoria.
- Taillard (1993), *EJOR* 64(2):278–285 (benchmark) — de memoria.
- Garey, Johnson y Sethi (1976), *Math. Oper. Res.* 1(2):117–129 (NP-dificultad con m ≥ 3) — de memoria.

## Pendiente

1. **Enunciado ausente**: confirmar qué pide (¿código como apéndice o fichero aparte?, ¿comparación con alguna tabla del paper?).
2. **Referencias sin verificar**: añadir a `docs/` el PDF de Juan et al. (2014) (y si es posible los otros) para comprobar autores, volumen y páginas; también la afirmación «NP-hard» citada a [1] y el presupuesto de 10·n·m ms.
3. **Transparencias del curso**: el notebook cita «the PDF» y «slides 43–44»; no están en el proyecto, así que no se citan. El valor NEH 26 797 de tai117 se atribuye a «the course material» sin referencia.
4. **Figura**: decidir si se regenera con etiquetas en inglés (convergencia de ILS-ESP o histograma de β) — implicaría ejecutar el notebook, cosa que no he hecho — o se acepta el artículo con dos tablas.
5. **Equipo**: el notebook no dice en qué máquina se ejecutó (usa `fork` y 2 procesos). Los autores confirman que no era compartida, y el artículo dice ya solo «two worker processes»; CPU y RAM siguen sin constar (pregunta 4); si los autores los dan, van en la frase de §4 sobre el experimento A. Desde r5 los tiempos se presentan como indicativos, con motivo y cifras de dos ejecuciones guardadas, y no como «orden de magnitud». Versión de Python según los metadatos del kernel: 3.13.
6. **Discrepancias con la presentación**: la presentación dice «0.1 to 0.8» puntos de ganancia; los CSV dan 0.10–0.73 (0.06 con la uniforme en tai117). La presentación da 0.57–0.65 en tai117; el CSV da 0.6446, que redondea a 0.64. En las conclusiones del notebook NEH en tai081 aparece como 2.41 (2.4048 → 2.40).
7. **Expansión de «ESP»**: no se expande la sigla porque no se ha podido comprobar en el paper.
8. Prosa de 2 369 palabras tras la ronda 5 (> 1 950) por sustituir la figura por una segunda tabla.
9. Ronda 2: retiradas por prudencia, hasta tener el paper y las transparencias, la atribución del presupuesto 10·n·m a [1], la comprobación NEH = 26 797 en tai117 («course material») y las referencias a [1] para la fórmula triangular y la búsqueda local. Añadido Garey, Johnson y Sethi (1976) para la NP-dificultad con m ≥ 3, también de memoria.
10. **Ronda 4, discrepancias con otras fuentes que este bucle no puede tocar.** La presentación dice aún «shared machine» y que Geo-adapt cuesta «about as much as Geo(0.1)–(0.2)» (`docs/PFSP_RandNEH_adaptive_beta.pdf`). La celda 32 del notebook repite ese coste, «within the noise» y 0.71/0.63. El artículo sigue a los CSV y al código (guía §9). La lista completa está en `ignore/paper_loop/revision_r4.md`, *Pendiente de los autores*.
11. **Chequeo de β**: `results/chequeo_beta_alto.csv` no tiene código que lo genere; los autores deben decir de dónde sale. El artículo lo dice sin atribuirlo a un notebook concreto.
12. **Reproducibilidad**: desde r5, *Code availability* dice solo que los notebooks leen los resultados guardados en lugar de repetir los experimentos, cierto con o sin `data/`. Sin `data/` el notebook no arranca (pregunta 7, sin responder).
13. **Fecha**: el artículo sigue con «October 2026»; hay que cambiarla si se entrega en noviembre.
