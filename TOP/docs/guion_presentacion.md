# Guion de la presentación del TOP

Guion para [`TOP_savings_grasp_local_search.pptx`](TOP_savings_grasp_local_search.pptx), diapositiva a
diapositiva. Las notas del orador del `.pptx` son un resumen de este texto. Duración total: unos
17–20 minutos sin preguntas.

Las cifras salen de [`informe_busqueda_local.md`](informe_busqueda_local.md),
[`metodos_y_comparacion_panadero.md`](metodos_y_comparacion_panadero.md) y los
`comparacion_panadero*.md`.

---

## 1. Portada — *Savings, GRASP and local search for the team orienteering problem*

*(~30 s)*

Buenos días. Vamos a presentar nuestra práctica sobre el *Team Orienteering Problem*, el TOP.

- **Qué hemos hecho:** partimos de la heurística de ahorros de Panadero et al. (2021), que llamamos PJS. La convertimos en un GRASP con aleatorización sesgada y le añadimos una búsqueda local.
- **Con qué:** todo está programado en Rust y lo hemos probado en las 387 instancias clásicas de Chao et al. (1996).
- **Cómo lo contamos:** primero el problema, después el método en tres capas y al final los resultados, incluida la comparación con el propio paper.

---

## 2. *The team orienteering problem*

*(~1 min)*

El TOP es un problema de rutas con tres ingredientes:

1. **Recompensa:** cada cliente tiene una recompensa.
2. **Vehículos:** tenemos m vehículos que salen de un origen y tienen que llegar a un depósito final. En los conjuntos p1 a p6 el origen y el destino son puntos distintos; en p7 coinciden.
3. **Tiempo:** cada ruta tiene un límite de tiempo, tmax, así que no da tiempo a visitar a todos.

Por eso el objetivo no es cubrir clientes ni minimizar distancia, sino **maximizar la recompensa recogida**. En el dibujo, los clientes grises se quedan fuera porque no caben en ninguna ruta.

Usamos las 387 instancias de Chao, en 7 conjuntos, con entre 21 y 102 nodos y de 2 a 4 vehículos. En 34 de ellas ningún cliente cabe ni siquiera solo dentro del tiempo, así que los resultados se dan sobre **353 instancias**.

---

## 3. *The PJS savings heuristic*

*(~1 min 30 s)*

La heurística de Panadero adapta los ahorros de Clarke y Wright al TOP. Tiene cuatro pasos:

1. **Solución dummy:** una ruta origen → cliente → destino por cada cliente. Los clientes que ni solos caben se descartan.
2. **Lista de ahorros:** se ordenan todos los arcos por el ahorro enriquecido, de mayor a menor.
3. **Fusión voraz:** se recorre la lista y se unen dos rutas si la ruta resultante cabe en tmax.
4. **Selección:** de las rutas que quedan, nos quedamos con las m de más recompensa.

La clave es la fórmula. El ahorro enriquecido mezcla dos cosas:

- s_ij, el tiempo que se ahorra al unir las rutas;
- u_i + u_j, la recompensa que recoge la ruta unida.

El parámetro **α** pondera una frente a otra: con α = 1 solo cuenta la distancia y con α = 0 solo la recompensa.

Un detalle de implementación: Panadero dice que el ahorro no depende del sentido del arco, pero con origen y destino distintos s_ij y s_ji no coinciden. Por eso consideramos los arcos en los dos sentidos, igual que Li et al. (2022).

Y el problema que nos encontramos: **el paper no da ningún valor de α**.

---

## 4. *The paper's rule for α does not hold here*

*(~1 min 30 s)*

Lo único que dice el paper es una regla cualitativa: α cerca de cero cuando las recompensas son heterogéneas y cerca de uno cuando son homogéneas. Lo hemos comprobado con nuestros resultados.

**Cómo lo medimos.**

- **Heterogeneidad:** el coeficiente de variación, el CV, de las recompensas.
- **Mejor α:** el que da más recompensa en la heurística determinista.
- **Un matiz importante:** en Chao todas las instancias de un conjunto comparten las mismas recompensas; solo cambian m y tmax. Así que en realidad solo hay 7 valores de CV.

**Qué sale.** En el gráfico los conjuntos están ordenados de menos a más heterogéneo. Si la regla se cumpliera, las barras bajarían de izquierda a derecha, y no lo hacen:

- p4, el más heterogéneo, tiene un α óptimo de 0.70;
- p5, de los más homogéneos, tiene 0.38.

La correlación entre CV y α óptimo, en las 285 instancias en las que α importa, es de **+0.19**: débil y con el signo contrario al que predice el paper. Y α = 0 es justo peor en el conjunto más heterogéneo: en p4 solo llega al 68 % de la mejor recompensa.

También probamos reglas para fijar α a partir del CV y modelos con rasgos de la instancia, y ninguna superó a un α fijo. La recompensa salta de forma irregular con α. **Así que la solución práctica es probar los 21 valores, de 0 a 1 en pasos de 0.05, y quedarse con el mejor de cada instancia.**

---

## 5. *GRASP: the same heuristic, slightly random*

*(~1 min 30 s)*

El siguiente paso es convertir PJS en un GRASP. En lugar de tomar siempre el arco de mayor ahorro, se elige uno al azar, pero con sesgo hacia los primeros de la lista. Se repite la construcción muchas veces y se guarda la mejor.

Dos vocablos:

- **iteración:** una construcción aleatorizada completa;
- **ejecución:** el multiarranque, que parte de la solución determinista y repite iteraciones quedándose con la mejor. Algorítmicamente equivale a un hilo del Algoritmo 1 de Panadero: siembra su generador una vez y sus iteraciones van en serie.

Probamos dos formas de aleatorizar:

- **la geométrica, la del paper:** se toma el k-ésimo arco, con k geométrica de parámetro β = 0.3. Solo importa la posición en la lista, no cuánto se diferencian los ahorros;
- **el softmax, que es propuesta nuestra:** cada arco con probabilidad proporcional a exp(s'/τ). Aquí sí importa la diferencia: arcos con ahorros parecidos tienen probabilidades parecidas.

Y tres presupuestos por ejecución:

- **10 iteraciones:** es la rejilla principal, 21 α por 20 ejecuciones;
- **100 iteraciones:** con las mismas semillas, así que cada ejecución empieza igual que la de 10 y sigue;
- **1 segundo:** como en el paper, entre 40 000 y 56 000 iteraciones.

Las ejecuciones por iteraciones son reproducibles exactamente; las de 1 s no, porque dependen de la carga de la máquina.

---

## 6. *Local search, applied to the elite at the end*

*(~1 min 30 s)*

La tercera capa es la búsqueda local. Una decisión de diseño: **no la metemos dentro de GRASP**, sino que la aplicamos al final, sobre un conjunto de soluciones élite de cada instancia.

**Qué soluciones son élite:**

- las deterministas de los 3 mejores α;
- para cada α y cada aleatorización, la mejor solución de las 3 mejores ejecuciones;
- si hay empates en la tercera posición, entran todos;
- las repetidas, que visitan el mismo conjunto de clientes, se quitan.

Quedan unas 53 por instancia y, en total, las 18 757 tardan 0.4 s.

**Qué hace la búsqueda local:** un bucle de dos movimientos.

1. **2-opt** dentro de cada ruta: invierte un tramo mientras la ruta se acorte.
2. **Inserción:** mete el cliente no visitado de mayor recompensa que quepa, en su posición más barata de cualquier ruta.

Se repite mientras se inserte alguno. La idea es que el tiempo que libera el 2-opt se aprovecha en la inserción siguiente. Como ningún movimiento puede perder recompensa, no hace falta criterio de aceptación.

¿Por qué tantas élite y no solo la mejor? Porque la mejor antes de la búsqueda local muchas veces no es la mejor después, como veremos en los resultados.

---

## 7. *The whole method, per instance*

*(~1 min 30 s)*

Aquí está todo junto en pseudocódigo.

**A la izquierda, lo que se hace con cada instancia.**

- Para cada uno de los 21 α:
  - la construcción determinista;
  - para cada aleatorización, 10 ejecuciones. Cada ejecución:
    - prepara la lista de ahorros de su α y siembra su generador pseudoaleatorio una sola vez, con una semilla propia;
    - arranca de la solución determinista;
    - hace 10 iteraciones **en serie**. Cada iteración es una construcción PJS completa eligiendo los arcos al azar. El generador sigue avanzando, así que las 10 salen distintas;
    - se queda con la mejor.
- Después se eligen las élite: las deterministas de los 3 mejores α y las 3 mejores ejecuciones por α y aleatorización, sin duplicados.
- Se aplica la búsqueda local a cada una y se devuelve la de más recompensa.

**A la derecha, la búsqueda local.**

- 2-opt en cada ruta.
- Se busca el cliente no visitado con más recompensa que cabe; en caso de empate, el que menos tiempo añade.
- Si no hay ninguno, se termina. Si lo hay, se inserta en su posición más barata y se vuelve a empezar.

**Qué va en paralelo.** La unidad paralela no es la instancia: es cada construcción determinista y cada ejecución.

- Por instancia y α hay 21 tareas: 1 determinista y 20 ejecuciones. En total son 170 667 tareas, en una sola lista que rayon reparte entre los 12 hilos según van quedando libres.
- Un hilo encadena muchas ejecuciones; las iteraciones de una ejecución nunca se reparten entre hilos.
- La búsqueda local también va en paralelo, una tarea por élite.

Las variantes de 100 iteraciones y de 1 s solo cambian el límite del bucle de iteraciones; el esquema es el mismo.

---

## 8. *Why Rust and not Python*

*(~1 min 30 s)*

Hemos programado todo en Rust en lugar de Python, por cuatro razones.

1. **Velocidad de cálculo.** Rust se compila a código nativo y no hay intérprete en el bucle interior. Siendo honestos: no hemos medido una versión en Python. Pero la rejilla principal, unos 1,6 millones de construcciones más la búsqueda local, tarda 140 s en las 387 instancias.
2. **Paralelismo con rayon.** Pasar de un hilo a todos es cambiar `iter` por `par_iter`. Además, el compilador no deja compilar código con carreras de datos, así que paralelizar no introduce errores sutiles. La rejilla entera es una sola lista paralela de 170 667 tareas.
3. **El código lo ha escrito sobre todo una IA.** La gran ventaja de Python es que una persona lo escribe más rápido. Si quien escribe la mayor parte del código es una IA, esa ventaja casi desaparece, y el lenguaje rápido no cuesta nada extra.
4. **El compilador guía a la IA.** Los mensajes de error de Rust son muy descriptivos: señalan la línea exacta, explican el error y a menudo proponen cómo arreglarlo. Eso le da a la IA un ciclo de corrección muy rápido. Los errores de tipos y de propiedad aparecen al compilar, no a mitad de una ejecución de dos minutos.

Python solo lo usamos una vez, como comprobación independiente de la heurística determinista: 38 700 ejecuciones comparadas y cero diferencias.

---

## 9. *Each layer adds reward: +20 % over plain PJS*

*(~1 min 30 s)*

Resultados de la rejilla principal, sobre las 353 instancias factibles. Las barras son la recompensa total.

- **PJS con el mejor α:** 157 868.
- **GRASP, sin búsqueda local:** 170 469, un **8 % más**.
- **Búsqueda local solo sobre las deterministas:** 181 103.
- **Búsqueda local sobre todas las élite, incluidas las de GRASP:** 189 854. Es un **20 % más** que la heurística determinista.

Tres datos a la derecha:

- GRASP solo ya aporta un 8 %;
- la búsqueda local mejora el 93.6 % de las élite, de media un 18 %;
- en **163 de 353 instancias** la solución final sale de una élite que antes de la búsqueda local no era la mejor. Por eso merece la pena aplicarla a muchas soluciones y no solo a la mejor.

Todo esto, para las 387 instancias, tarda 140 s con 12 hilos.

---

## 10. *One instance, before and after: p4.3.k*

*(~1 min)*

Un ejemplo concreto: la instancia p4.3.k, con 3 vehículos, tmax = 50 y 98 clientes.

- **A la izquierda, la heurística determinista con su mejor α (0.40):** recoge 614 de recompensa y visita 32 clientes.
- **A la derecha, la solución final:** 874 de recompensa y 54 clientes, un **42 % más** con los mismos tres vehículos y el mismo límite de tiempo.

Esta solución final sale de una ejecución geométrica con α = 0.45 que antes de la búsqueda local valía 625. La búsqueda local le insertó 19 clientes.

Se ve en el dibujo: las rutas finales recorren zonas densas de clientes, mientras que las deterministas cruzan el mapa con tramos largos casi vacíos.

---

## 11. *The gain grows with the instance*

*(~1 min)*

La ganancia sobre PJS no es uniforme: depende del conjunto.

- **p1 a p3:** son pequeños, de 21 a 33 nodos. Hay pocos clientes que añadir, y la ganancia va del 2 al 9 %.
- **p4 y p7:** tienen unos 100 nodos, rutas largas y muchos clientes fuera. La ganancia es del **39 % en p4** y del **28 % en p7**. En alguna solución la búsqueda local llega a insertar 49 clientes.

Esto es importante para la comparación que viene: Panadero solo publica instancia a instancia **p1 y p2**, que son justo los conjuntos donde hay menos margen. De p3 a p7 da medias por conjunto, que veremos después.

---

## 12. *p1 and p2, instance by instance: ahead of Panadero*

*(~2 min)*

Esta diapositiva es solo de p1 y p2, los dos conjuntos que el paper publica instancia a instancia. Comparamos con sus tablas 3 y 4: las 81 instancias de p1 y p2. (En la tabla 4, p2.4.a aparece por errata como "p2.4.1".) La medida es la del propio paper: la distancia media a la mejor solución conocida, el BKS, promediada por instancia. Cuanto más corta la barra, mejor.

- **PJS determinista:** 4.82 %.
- **GRASP de 10 iteraciones con la rejilla de α:** 1.99 %.
- **Panadero con un hilo:** 1.73 %.
- **Nuestro GRASP de 1 s con la rejilla:** 0.71 %.
- **Panadero con 128 hilos, su mejor resultado:** 0.44 %.

Sin búsqueda local no llegamos a Panadero con 128 hilos. Con búsqueda local:

- **10 iteraciones:** 0.34 %;
- **100 iteraciones:** 0.29 %;
- **1 s con la rejilla:** **0.15 %**.

Todas mejoran a Panadero con 128 hilos.

En número de instancias en las que se alcanza el BKS: Panadero con 128 hilos, 69 de 81; nosotros, 73 con la versión barata y 77 con la de 1 s. Ninguna solución nuestra supera un BKS: igualamos, no batimos la literatura.

---

## 13. *All seven sets: below Panadero's gap in every one*

*(~1 min 30 s)*

Para p3 a p7, el paper no da tablas por instancia: solo la distancia media al BKS de cada conjunto, en su figura 4, con 128 hilos. Para compararnos con eso necesitábamos el BKS de todas las instancias.

- **De dónde lo sacamos:** de los resultados públicos de PSOiA (Dang, Guibadj y Moukrim), que traen el mejor valor conocido de las 353 instancias factibles.
- **Validación:** en las 81 instancias de p1 y p2 coinciden exactamente con los BKS que usa Panadero, que son los de Ke et al. (2016).

Aquí usamos la versión barata: la rejilla de 10 iteraciones con búsqueda local. Las barras verdes son nuestras y las naranjas, de Panadero con 128 hilos.

- **En los siete conjuntos quedamos por debajo de Panadero.** Por ejemplo:
  - p4, el más difícil: 5.87 % frente a 7.97 %;
  - p7: 3.42 % frente a 5.10 %;
  - p3: 0.67 % frente a 1.44 %.
- **En global:** 2.04 % frente a 2.85 %. El 2.85 % es la media de la figura 4 ponderada por el número de instancias; el texto del paper habla de en torno a un 3 %.
- **Alcanzamos el BKS en 189 de 353 instancias:** todas las de p2, pero solo 5 de 56 en p4.

Y un dato que conecta con la factibilidad: **ninguna de las 353 soluciones supera el BKS**. Si la búsqueda local metiera clientes que no caben, lo normal sería ver recompensas por encima de la mejor conocida, sobre todo en p4 y p7, donde más inserta. No pasa en ningún conjunto. Además, un verificador independiente ha recalculado todas las rutas finales desde los ficheros originales: todas caben en tmax, sin clientes repetidos y con como mucho m vehículos.

---

## 14. *Trying many α beats iterating longer with one*

*(~1 min)*

Una conclusión que no esperábamos: **para GRASP, probar muchos α rinde más que iterar mucho con uno solo.**

| Variante | Construcciones por instancia | Distancia media al BKS, sin búsqueda local |
|---|---|---|
| 1 s con el mejor α (20 ejecuciones de ~45 000 iteraciones) | ~900 000 | 1.49 % |
| 100 iteraciones con la rejilla de α (21 α × 20 ejecuciones × 100 iteraciones) | 42 000 | 1.45 % |

Es la misma calidad con unas 20 veces menos trabajo.

La explicación es la de la diapositiva 4: un cambio pequeño en α reordena la lista de ahorros y la fusión voraz construye rutas distintas. Repartir el presupuesto entre α explora más que gastarlo en uno.

Una nota honesta, al pie: una sola ejecución nuestra de 1 s da de media un 2.3–2.4 %, algo peor que el 1.73 % de un hilo de Panadero. La construcción es la misma idea, así que la diferencia estará en detalles que el paper no fija, como el orden en los empates o el tratamiento de los arcos en los dos sentidos.

---

## 15. Conclusiones — *Diversity first, then local search*

*(~1 min 30 s)*

El mensaje principal: **primero diversidad, después búsqueda local.** La rejilla de α y las dos aleatorizaciones generan un buen conjunto de élite, y el 2-opt con inserción las convierte en las rutas finales.

**Lo que hemos encontrado:**

- un 20 % más de recompensa que PJS en las 353 instancias factibles;
- quedamos por debajo de Panadero con 128 hilos en los siete conjuntos: un 2.04 % de distancia media al BKS frente a un 2.85 %. En p1 y p2, con 1 s, un 0.15 % frente a un 0.44 %;
- la regla del paper para α no se cumple en estos conjuntos.

**Lo que conviene matizar:**

- **El cómputo no es igual.** La rejilla de 1 s gasta unos 420 segundos de CPU por instancia, más que los 128 de Panadero. La versión de 10 iteraciones, en cambio, gasta mucho menos y también le gana.
- **Reproducibilidad.** Las ejecuciones de 1 s dependen de la carga de la máquina y no son reproducibles exactamente.
- **Lo que falta:** movimientos entre rutas e intercambios de clientes en la búsqueda local. Y para p3 a p7 el paper solo publica medias por conjunto, así que ahí no podemos compararnos instancia a instancia.

Gracias. ¿Preguntas?

---

## 16. Reserva — *All variants against Panadero et al.*

*(Solo si preguntan.)*

Tabla con todas las variantes sobre las 81 instancias de p1 y p2: distancia media al BKS, instancias en las que se alcanza y comparación con Panadero con 128 hilos (mejor / igual / peor).

- **La mejor:** GRASP de 1 s con la rejilla y búsqueda local, con 0.15 %, BKS en 77 de 81 y mejor que Panadero en 12, igual en 65 y peor en 4. Las 4 son p1.2.i, p1.2.l, p1.2.m y p1.3.l.
- **Una pregunta probable:** "¿por qué con 100 iteraciones sale peor en alguna instancia que con 10?". En p1.4.r, con 10 iteraciones se llega a 210 y con 100 solo a 205. Las élite se eligen por la recompensa *antes* de la búsqueda local. Con 100 iteraciones esa ejecución encuentra otra solución, mejor antes de la búsqueda local pero peor después. La búsqueda local no es monótona respecto a la calidad del punto de partida.
