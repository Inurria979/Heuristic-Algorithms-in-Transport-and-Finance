---
name: cuaderno-explicado
description: Cómo escribir o rehacer el notebook principal de una práctica del máster para que sea un cuaderno explicado, al estilo de VRP, GRASP y PFSP. Tiene una introducción del problema y del método, recorre paso a paso las funciones del módulo sobre una instancia de ejemplo, luego presenta el experimento, la comparación con el paper y las conclusiones. Úsalo cuando se pida que el notebook «quede como el resto de trabajos», con explicaciones, o al crear el notebook de una práctica nueva. Lo usa sobre todo code-writer; code-reviewer lo usa como lista de comprobación.
argument-hint: <CARPETA> [notebook]
---

# Cuaderno explicado de una práctica

El notebook principal de cada práctica no es solo un script de análisis: es el documento que el
profesor abre para **entender el método y ver que funciona**. Los modelos son
`VRP/src/vrp.ipynb`, `GRASP/src/GRASP_TSP_savings_biased_EN.ipynb` y
`PFSP/src/PFSP_RandNEH_ILS_ESP_en.ipynb`. Antes de escribir, abre al menos uno de ellos y lee sus
celdas markdown: imita su tono, su densidad de explicación y su orden.

## Reglas firmes

1. **Los resultados no cambian.** Si el notebook escribe ficheros en `results/` (CSV, figuras),
   tienen que salir iguales que antes del cambio. Antes de tocar nada, copia `results/` a
   `<P>/ignore/<loop>/prev/`. Al terminar, compara con un script de solo lectura:
   - CSV: comparación celda a celda con tolerancia `1e-12`. Las columnas de tiempo (`time_s`…)
     quedan fuera.
   - Figuras: el mismo contenido. Mira la imagen y comprueba que los datos dibujados no cambian;
     no hace falta que los bytes del PNG coincidan.

   Si un CSV o una cifra publicada cambia, para y dilo: no es una mejora de estructura.
2. **El algoritmo vive en el módulo.** El notebook importa el módulo de `src/` (`import vrp`,
   `import simtop as st`) y llama a sus funciones; no copia ni reimplementa el algoritmo. Las
   únicas funciones que se definen en el notebook son de dibujo o de presentación.
3. **Experimentos largos.** Si el experimento completo tarda más de ~1 min, el notebook lee sus
   CSV de `results/` (los genera un script, p. ej. `run_experiment.py`) y lo dice en una frase,
   con el comando para regenerarlos. Si tarda menos, puede lanzarlo él mismo.
4. **Idioma y rutas.** Notebook en inglés. Se ejecuta con `src/` como directorio de trabajo:
   lee de `../data/` y escribe en `../results/` y `../results/figures/`.
5. **Se entrega ejecutado.** Se ejecuta de principio a fin con
   `uv run --with nbconvert jupyter nbconvert --to notebook --execute --inplace src/<nb>.ipynb`
   y se guardan sus salidas. No debe quedar ninguna celda con error ni una salida enorme: las
   tablas largas se recortan con `.head()` o se resumen.
6. **Cifras con respaldo.** Toda cifra de una celda markdown, sobre todo en las conclusiones,
   tiene que verse en una salida del propio notebook. Si una cifra viene del paper, se dice de
   dónde (tabla, figura, página) y, si se ha leído a ojo en una figura, se dice también.

## Estructura

1. **Título (`#`)** con el método y el problema.
2. **Introducción** (una o dos celdas markdown):
   - **El problema**: definición formal breve, en LaTeX (`$...$`, `$$...$$`): datos,
     restricciones y objetivo.
   - **El método**: idea en pocas frases y pseudocódigo en un bloque ```` ``` ````.
     - Si hay parámetros, una tabla con sus valores.
     - Si la implementación se aparta del paper, una frase sobre el alcance o las diferencias.
       Es una explicación, no una disculpa.
   - **Qué hace el cuaderno**: con enlace al módulo (`[simtop.py](simtop.py)`).
   - **La referencia del paper**: en un bloque `>`.
3. **Imports y utilidades de dibujo**: una celda de imports y, si hace falta, otra con las
   funciones de dibujo (con su docstring).
4. **Recorrido paso a paso** (`## 1.`, `## 2.`, …), una sección por pieza del método y en el orden
   en que se ejecutan. Ejemplos: lectura de la instancia, matriz de distancias o tiempos, lista de
   ahorros, construcción, aleatorización sesgada, búsqueda local, simulación, selección… En cada
   sección:
   - una celda markdown que dice qué función se usa (`` `st.func(args)` ``), qué recibe y qué
     devuelve, la fórmula si la hay, y el porqué de las decisiones que no son obvias;
   - una celda de código que la aplica a **una instancia de ejemplo pequeña** (la misma en todo
     el recorrido) e imprime algo que se pueda comprobar (tamaño, una tabla corta, una
     propiedad que se cumple, un antes y después);
   - cuando aporte, una figura (rutas, distribución, convergencia).
5. **Experimento completo**:
   - una celda markdown con el diseño: instancias, niveles, semillas, réplicas y cómo
     regenerarlo;
   - las tablas resumen, una por pregunta, cada una precedida de la pregunta que contesta;
   - las figuras guardadas en `../results/figures/`, cada una con una frase de lectura debajo
     («la mediana crece de … a …»).
6. **Comparación con el paper**: las mismas magnitudes que el paper, con la fuente de cada cifra
   de referencia, y las diferencias explicadas sin exagerar las causas.
7. **Conclusiones o *Remarks*** (último `##`): de 4 a 6 viñetas numeradas o en negrita, con las
   cifras clave tal como salen arriba. Si hay límites o trabajo futuro, una viñeta.

## Estilo

- Markdown breve y directo, como en VRP: frases cortas, listas cuando hay más de dos cosas,
  negrita solo para el concepto clave de cada párrafo.
- Celdas de código cortas: una idea por celda. Nada de celdas de 80 líneas.
- Las salidas con `print(f"...")` llevan las unidades y pocas cifras decimales;
  `DataFrame.round(2)`.
- Figuras con título, ejes rotulados y leyenda; `plt.show()` después de `savefig`.
- No repitas en markdown lo que ya dice la salida; explica lo que la salida no dice (por qué
  sale eso).

## Comprobación final (escritor y revisor)

- [ ] El notebook se ejecuta entero sin errores, con salidas guardadas.
- [ ] Los CSV y figuras de `results/` coinciden con la copia previa (salvo tiempos).
- [ ] Hay una introducción con problema, método, pseudocódigo y referencia.
- [ ] Hay un recorrido paso a paso de cada función del módulo sobre una instancia de ejemplo.
- [ ] El experimento tiene tablas y figuras explicadas, y una comparación con el paper con sus
      fuentes.
- [ ] Las conclusiones tienen cifras que aparecen en las salidas.
- [ ] El notebook está en inglés, usa rutas relativas y no reimplementa el algoritmo.
