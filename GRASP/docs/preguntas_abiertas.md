# Preguntas abiertas — artículo GRASP

Dudas que han salido al escribir y revisar `GRASP for the Travelling Salesman Problem.tex`.
Contestad debajo de cada una (en **Respuesta**) y el bucle escritor–revisor las incorporará.

## 1. Enunciado y entrega del código

No hay enunciado en `GRASP/`. El artículo tiene un párrafo *Code availability* que remite
al notebook, como en POP. ¿Hay enunciado? ¿Hay que incluir el código como apéndice?

**Respuesta:**

## 2. Figuras con rótulos en español

Las figuras de `results/figures/` se han extraído de las salidas guardadas del notebook EN, pero esas
salidas vienen de la ejecución en español («Gap por instancia», «GRASP estándar», «Sesgada…»). La
leyenda de la Figura 1 lo avisa y traduce los términos. Para corregirlo hay que volver a ejecutar la
celda de gráficos. ¿La regeneráis en inglés o se queda así?

**Respuesta:**

## 3. Equipo de cómputo

El notebook no dice en qué máquina corrió (solo Python 3.13.16, probablemente Linux). El artículo dice
solo que los tiempos son medias de 20 ejecuciones en una misma máquina. ¿Qué equipo fue?

**Respuesta:**

## 4. Discrepancia con la presentación: tiempos de Zipf y softmax

La presentación dice «≈1.7 veces más lentos», pero eso solo se cumple en a280. En las 7 instancias el
cociente va de 1.26 a 1.75 (media 1.47), y el artículo da «about 1.3–1.75». ¿De acuerdo?

**Respuesta:**

## 5. Explicación de la triangular en a280

El peor resultado de la triangular en a280 se atribuye, con «possibly», a que admite 78 posiciones en
el primer paso frente a 10 en kroB100. Es una cuenta a partir del código, no algo medido. ¿Lo dejamos
como hipótesis, lo quitamos o tenéis otra explicación?

**Respuesta:**

## 6. Nombre del fichero

Lo eligió el escritor: «GRASP for the Travelling Salesman Problem». ¿Os vale?

**Respuesta:**

## 7. Explicación de la RCL por la «escala» de los ahorros (revisión r1, I1)

El artículo (y la presentación) explican el resultado de la RCL por la «escala» de los ahorros. El
revisor señala que la RCL y softmax se normalizan por el rango, así que escalar las distancias no les
afecta, y que «pocos candidatos pasan el corte» no se ha medido. El escritor cambia la redacción
según la propuesta del revisor (ver `ignore/paper_loop/revision_r1.md`). ¿Estáis de acuerdo?

**Respuesta:**

> Nota (revisión r2): si aceptáis la nueva explicación de la pregunta 7, hay que cambiar también
> `docs/presentation_script.md`, líneas 235–238, que sigue hablando de «scale».

---

**Estado del bucle:** APROBADO en la ronda 4 (0 B, 0 I, 8 menores en `ignore/paper_loop/revision_r4.md`).
Cuando respondáis a estas preguntas, relanzad `/paper-loop GRASP 1` para incorporarlas.
