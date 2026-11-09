# Guion de la presentación de simTOP

Guion para [`simTOP_simheuristic.pptx`](simTOP_simheuristic.pptx), diapositiva a diapositiva. Las
notas del orador del `.pptx` son un resumen de este texto. Duración total: unos 18–19 minutos sin
preguntas.

**Versión 4** (correcciones de las revisiones finales), alineada con el artículo aprobado
([`Stochastic Team Orienteering Problem.tex`](Stochastic%20Team%20Orienteering%20Problem.tex)): mismas
cifras, mismos términos y mismas cautelas. La fuente de cada cifra está en las notas
`ignore/presentacion/notas_v4.md` (nota local; `ignore/` no se sube).

Términos que usamos igual que el artículo: *gap* (respecto a la BKS determinista), *gap determinista*,
*pérdida por incertidumbre* (las dos sobre la BKS, y se suman), *días* (réplicas de la simulación),
*números aleatorios comunes*, *maldición del ganador*.

---

## 1. Portada — *A simheuristic for the team orienteering problem with stochastic travel times*

*(~40 s)*

Buenos días. Presentamos la práctica de simheurísticas: el *Team Orienteering Problem* con tiempos de
viaje aleatorios.

- Hemos implementado en Python la simheurística de Juan et al. (2019): candidatas de la heurística de
  ahorros con aleatorización sesgada y búsqueda local, y después una simulación Monte Carlo rápida y
  otra intensiva.
- La probamos en las 141 instancias factibles de los conjuntos p1–p3 de Chao, con tres niveles de
  incertidumbre.
- El paper también resuelve un SAA con CPLEX. Esa parte no la hacemos, porque necesita un solver
  comercial.

---

## 2. *Team orienteering with random travel times*

*(~1 min 15 s)*

Es el TOP de siempre: m vehículos van del origen al depósito final, cada cliente paga su recompensa
una vez y cada ruta tiene un límite tmax. Lo nuevo es que **el tiempo de cada arco es aleatorio** y que
**una ruta que llega tarde pierde toda su recompensa**. Por eso se maximiza la **recompensa
esperada**, que hay que estimar simulando.

El ejemplo: un vehículo y tmax = 10. La ruta A tarda 9,8 de media y da 30; la B tarda 8 y da 25. En
determinista gana A. Con incertidumbre media, c = 0,25, y el modelo de la diapositiva 3, A llega tarde
el 42 % de los días y vale unos 17,5; B llega tarde solo el 9 % y vale unos 22,8. Son cifras simuladas
con 400 000 días. **Con incertidumbre, la holgura compensa.** (Con c = 0,75 gana también B: 17,5 frente a
20,4.)

---

## 3. *Chao's instances with lognormal travel times*

*(~1 min 15 s)*

Tomamos las instancias de Chao y hacemos aleatorio el tiempo de cada arco, como en el paper:

- una **lognormal con media la distancia euclídea y varianza c por la distancia**;
- **tres niveles:** c = 0,05, 0,25 y 0,75 (L, M y H). Para un arco de longitud 10, el coeficiente de
  variación es del 7, el 16 y el 27 %;
- **arcos independientes:** es un supuesto nuestro, porque el paper no lo dice.

Usamos p1, p2 y p3, los únicos conjuntos de menos de 35 nodos. Son 141 instancias factibles, con tmax
entre 3,8 y 55. Con tres niveles salen 423 casos, y con tres α, **1 269 ejecuciones**. La referencia es
la mejor solución conocida determinista, la BKS, de Dang et al. (2013).

Los escenarios se generan al vuelo con una semilla fija: no se guardan y `data/` no cambia.

---

## 4. *Same c, very different uncertainty*

*(~1 min)*

Dos propiedades del modelo que explican los resultados.

- **Las varianzas se suman.** La desviación relativa de una ruta es la raíz de c entre su tiempo
  esperado, así que el mismo c pesa más en rutas cortas. Con c = 0,25 y una ruta que llena tmax, la
  desviación es del 25,6 % con el tmax más pequeño de p1–p3 (3,8) y del 6,7 % con el más grande (55).
  La gráfica es la misma fórmula, no un resultado.
- **Una ruta cuyo tiempo esperado es tmax llega tarde casi la mitad de los días.** En una ruta de
  prueba lo hizo en el 47,18 % de 20 000 réplicas. Es algo menos de la mitad porque la mediana de la
  lognormal está por debajo de su media.

**La holgura vale recompensa.**

---

## 5. *The simheuristic: one run per (instance, c, α)*

*(~1 min 30 s)*

Una ejecución tiene dos fases.

**Fase determinista, con tiempos esperados:**

1. **51 candidatas:** la PJS greedy y 50 construcciones con aleatorización sesgada.
2. **Búsqueda local** a todas.
3. **Quitamos los duplicados.**
4. **Pasan las 7** de más recompensa determinista, o todas si hay menos.

**Fase estocástica:**

5. **Simulación rápida:** 200 días, los mismos para las 7.
6. **Pasan las 3** de mayor media.
7. **Simulación intensiva:** 1000 días nuevos, compartidos por las 3.
8. **Gana la de mayor media intensiva,** que estima su recompensa esperada.

La aleatorización sesgada explora y la simulación evalúa. Cada ejecución tiene una semilla, y de ella
salen tres generadores independientes.

---

## 6. *How the candidates are built*

*(~1 min 15 s)*

- **PJS,** la heurística de ahorros de Panadero et al. (2021). Empieza con una ruta por cliente y las
  fusiona por orden de preferencia: el tiempo ahorrado y la recompensa, ponderados por α. Considera los
  arcos en los dos sentidos, porque los depósitos son distintos. Solo fusiona si el tiempo esperado cabe
  en tmax.
- **Aleatorización sesgada geométrica,** con β = 0,3: el primer arco de la lista se toma el 30 % de las
  veces.
- **Búsqueda local:** 2-opt en cada ruta y, después, inserción del cliente de mayor recompensa que quepa,
  en su posición más barata. Se repite hasta que no cabe ninguno.

**Todo usa tiempos esperados, así que las fusiones y las inserciones llenan tmax.**

---

## 7. *Common days to compare, new days to measure*

*(~1 min 30 s)*

**Días comunes para comparar.** La simulación rápida tiene que **ordenar** 7 candidatas con solo 200
réplicas. Con números aleatorios comunes, la diferencia entre dos candidatas viene de sus rutas y no de
la suerte del sorteo. La varianza de la diferencia de medias es la suma de varianzas menos dos veces la
covarianza, y con días comunes la covarianza es positiva.

**Días nuevos para medir.** La intensiva tiene que **medir**. Las finalistas se eligieron porque les fue
bien en los días rápidos. Si se midieran otra vez con esos días, su media heredaría esa suerte: es la
**maldición del ganador**. Por eso los 1000 días se sortean con otro generador y no incluyen los 200.

**Comunes no es lo mismo que reutilizados:** las candidatas de una misma fase comparten días, pero los
días que eligieron una solución nunca la miden.

Queda un sesgo pequeño, porque publicamos la mayor de tres medias. Con 1000 réplicas y finalistas
parecidas es despreciable. Quitarlo exigiría una tercera simulación de la ganadora.

---

## 8. *Two parameters from the paper, the rest ours*

*(~45 s)*

El paper describe la simheurística en unos pocos párrafos cortos y sin valores. Solo salen de él los niveles de c y las
réplicas, 200 y 1000. Todo lo demás es nuestro, y la tercera columna da el motivo de cada decisión. Al
pie, un matiz: la candidata greedy asegura que la mejor candidata determinista no es peor que PJS con
búsqueda local, pero no la solución final, que la elige la simulación.

---

## 9. *α hardly matters; the uncertainty level does*

*(~1 min)*

- **α casi no importa.** Con cada α fijo, el gap medio varía menos de 0,6 puntos dentro de cada nivel,
  y el mejor α se reparte 142, 138 y 143 casos.
- **Quedarse con el mejor α** de cada caso baja el gap medio entre 1,3 y 2,2 puntos, en parte por el
  sesgo de selección de elegir el mejor de tres con la media que publicamos.
- **Lo que importa es c.** Con el mejor α, el gap medio es de **14,65, 25,02 y 30,51 %**.

A partir de aquí usamos el mejor α de cada caso, 423 casos. Nunca mezclamos niveles: cada c es un
problema distinto.

---

## 10. *Fig. 3: same ordering, 7–9 points higher*

*(~1 min 30 s)*

Primero, las cautelas. Las cajas del paper las leímos **a ojo** (±1–2 puntos), y el paper no dice ni
qué instancias entran en cada caja (suponemos p1–p3, los únicos conjuntos que resolvió el SAA) ni cómo
calcula el gap.

- **El orden es el mismo,** L < M < H.
- **Nuestras medianas quedan entre 7 y 9 puntos por encima:** 15,69, 26,02 y 31,56 %, frente a unos 7,
  17 y 24 %. En L y M, nuestro primer cuartil está más o menos en su tercero.
- **No viene principalmente de la construcción:** nuestro gap determinista es de solo 2,4–3,2 puntos.

**Una causa plausible, que no podemos comprobar sin su código:** Juan et al. simulan dentro del bucle
de búsqueda, con ejecuciones de hasta unos 400 s, así que la simulación puede
llevar la búsqueda hacia rutas con holgura. Nosotros solo simulamos 7 candidatas elegidas con tiempos
esperados.

---

## 11. *Most of the gap is lost to uncertainty*

*(~1 min 30 s)*

El gap se descompone **sumando**, todo sobre la BKS:

- el **gap determinista** de la ganadora, lo que pierde la heurística: 3,15, 2,96 y 2,41 puntos;
- la **pérdida por incertidumbre**, lo que pierden las rutas que llegan tarde: 11,50, 22,06 y 28,10
  puntos.

Juntas dan el gap: 14,65, 25,02 y 30,51. La pérdida es **en torno al 80 % del gap en L y al 90 % en M
y H**.

La mediana del gap determinista es 0: **en 72, 72 y 82 de los 141 casos** la ganadora alcanza la BKS
en términos deterministas.

Si la pérdida se mide sobre la **propia recompensa determinista de la ganadora** (y no sobre la BKS),
sale 11,86, 22,72 y 28,79 %. Parte de esa pérdida no la evita ninguna solución.

---

## 12. *Likely cause: candidates are built up to tmax*

*(~1 min 15 s)*

La gráfica pone dos cosas juntas: la pérdida sobre la recompensa determinista de la ganadora y la
proporción de rutas que llegan tarde en un día simulado (12,73, 23,57 y 29,45 %). Casi coinciden, como
cabe esperar si una ruta tardía lo pierde todo.

**La explicación probable** es la del diseño: la PJS fusiona mientras cabe el tiempo esperado, la
búsqueda local rellena la holgura y el filtro se queda con las más llenas.

- La mediana del tiempo esperado de las rutas ganadoras es **0,93 tmax** en los tres niveles.
- Un **4, 6 y 8 %** de ellas está a menos de un 1 % de tmax, donde una ruta falla casi la mitad de las
  veces.
- Además, la simulación a menudo tiene poco donde elegir: en el **27,1 %** de las ejecuciones sobreviven
  menos de 7 candidatas distintas, así que el filtro no actúa, y en el **9,0 %** solo una.

**La simulación solo puede elegir entre rutas construidas sin holgura.** Es fiel al paper, que también
planifica con tiempos esperados.

---

## 13. *Larger tmax, larger gap — despite Var = c·t*

*(~45 s)*

Por la diapositiva 4 esperaríamos más gap con tmax pequeño. Pasa lo contrario: el gap crece con tmax
(Spearman 0,49, 0,56 y 0,67), y p2, con el tmax más pequeño, tiene el menor gap. Pero el gap
determinista también crece con tmax (0,56, 0,54 y 0,59), más que la pérdida por incertidumbre (0,30,
0,39 y 0,44). Y conjunto y tmax están confundidos, porque p2 tiene solo 21 nodos. **Lo contamos como
una observación, no como una explicación.**

---

## 14. *Fast enough: 1 269 runs in about 25 seconds*

*(~1 min)*

Una ejecución tarda entre 0,05 y 0,23 s (de media 0,07 s en p2 y 0,16–0,17 s en p1 y p3). Las 1 269
ejecuciones tardan unos 25 s en **12 hilos de un AMD Ryzen 5 5500U de 6 núcleos**. Lo pesado son núcleos
de Numba; los números aleatorios salen de NumPy. La figura 4 del paper da de unos segundos a unos 400 s,
en Java, en un Xeon a 3,7 GHz y con un criterio de parada que no especifica: solo cabe compararlo cualitativamente. Como es tan
barato, probamos todos los α en todos los niveles en lugar de ajustar uno.

---

## 15. *Limitations and next step: a safety margin*

*(~1 min 15 s)*

**Limitaciones:**

- no hay SAA, porque necesita un solver comercial;
- la figura 3 está leída a ojo, suponemos que sus cajas son p1–p3 y no sabemos cómo calcula el gap;
- hay dos sesgos de selección pequeños: publicamos la mayor de tres medias finales, y elegimos el mejor de
  tres α con esa misma media;
- las candidatas se construyen con tiempos esperados, como en el paper: sin holgura.

**Siguiente paso: un margen de seguridad.** Construir con tmax − δ y seguir simulando con el tmax real.
δ puede salir de:

1. una **rejilla**;
2. la **varianza de la ruta**: aceptarla solo si E_d + z · raíz(c · E_d) ≤ tmax, un margen por ruta y por
   c;
3. un margen **adaptativo**, ajustado a la tasa de fallos de la simulación rápida.

Lo motiva que, de media, entre el 13 y el 29 % de las rutas ganadoras llegan tarde en un día simulado, y más
cuanto mayor es c.

---

## 16. Conclusiones — *Simulation can only choose among routes built without slack*

*(~1 min)*

La simheurística construye candidatas para un mundo determinista y deja que la simulación elija:
días comunes para ordenar y días nuevos para medir.

- Da el orden L < M < H de Juan et al., con gaps medios de 14,65, 25,02 y 30,51 %. Nuestras medianas
  quedan 7–9 puntos por encima de las suyas, leídas a ojo.
- Más o menos la mitad de las ganadoras (51–58 %, 72, 72 y 82 de 141) alcanza la BKS en determinista, pero la ruta mediana llena 0,93 tmax y, de
  media, entre el 13 y el 29 % de las rutas llegan tarde en un día simulado.
- Una ejecución tarda una fracción de segundo, y α apenas importa.

**La simulación solo puede elegir entre rutas construidas sin holgura.** El siguiente paso es el margen
de seguridad.

Gracias. ¿Preguntas?

---

## 17. Reserva — *Gap by α and level, all 1 269 runs*

*(Solo si preguntan.)*

Cajas del gap por α y nivel con las 1 269 ejecuciones: en cada nivel se solapan casi por completo.

**Preguntas probables:**

- *¿Por qué no simuláis las 51 candidatas?* Para que una ejecución dure segundos. Además, en un 27,1 %
  de las ejecuciones ni siquiera hay 7 distintas.
- *¿Por qué no reutilizar los 200 días en la intensiva?* Inflaría la media de las finalistas (diapositiva
  7).
- *¿Por qué vuestro gap es mayor que el del paper?* La causa plausible está en la diapositiva 10, y no
  podemos comprobarla sin su código.
