# Preguntas abiertas — artículo PFSP

Dudas que han salido al escribir y revisar `Permutation Flow-Shop Problem.tex`. Contestad debajo de
cada una (en **Respuesta**) y el bucle escritor–revisor las incorporará.

## 1. Enunciado y entrega del código

No hay enunciado en `PFSP/`; los requisitos se han deducido de la portada de la presentación.
¿Hay enunciado? ¿El código va como apéndice o como fichero aparte? Ahora el artículo tiene un
párrafo *Code availability* que nombra el notebook.

**Respuesta:**

## 2. Figura

No existe `results/figures/`. Las figuras guardadas en los notebooks tienen ejes y títulos en
español (también la versión `_en`), y las gráficas de la presentación son nativas de PowerPoint.
Por eso el artículo lleva dos tablas y ninguna figura, y la prosa queda en 2 369 palabras (guía:
1 700–1 950). ¿Regeneráis una figura en inglés ejecutando el notebook, o seguimos sin figura?

**Respuesta:**

## 3. Referencias

No hay ningún paper en `docs/`. Los datos bibliográficos de Juan et al. 2014 (ITOR), NEH 1983 y
Taillard 1990/1993 están puestos de memoria, igual que «NP-hard» citado a Juan et al. ¿Podéis
añadir los PDF a `docs/` o confirmar las referencias? ¿Hay transparencias del curso que citar?

En la ronda 2 se ha añadido, también de memoria, Garey, Johnson y Sethi (1976) para la NP-dificultad con
tres o más máquinas. Mientras no haya PDF se han quitado del texto el presupuesto 10·n·m ms, el NEH
26 797 de tai117, las atribuciones a Juan et al. (fórmula triangular, búsqueda local) y la comparación
con sus medias. ¿Qué significa «ESP» en el notebook, para poder expandirlo?

**Respuesta:**

## 4. Equipo de ejecución

El artículo dice solo «two worker processes of a shared machine». ¿Qué máquina (CPU, RAM) y cuántos
procesos se usaron?

**Respuesta:** La máquina no era compartida: se quita «shared» del artículo (02-11-2026). CPU y RAM
siguen sin constar.

## 5. Discrepancias entre presentación y CSV

- Ganancia sobre NEH: la presentación dice 0.1–0.8 puntos; los CSV dan 0.10–0.73 (distribuciones
  sesgadas) y 0.06 (uniforme) en tai117.
- tai117: la presentación dice 0.57–0.65; el CSV da un máximo de 0.6446 (→ 0.64).

El artículo usa las cifras de los CSV. ¿De acuerdo?

**Respuesta:**

## 6. ¿tai001 tiene óptimo demostrado? (revisión r1, I4)

El artículo dice que tai001 no tiene óptimo demostrado. El revisor cree que 1 278 es óptimo (lo dice
de memoria, no por un fichero). ¿De dónde salen las cotas que usáis? ¿Atribuimos la frase a esa
compilación o la quitamos?

**Respuesta:**

## 7. Reproducibilidad: falta `data/` y el notebook usa `fork` (revisión r1, M11)

No existe `PFSP/data/` y el notebook usa `fork` (solo Linux), así que la frase «PFSP_RERUN=1 repeats
both experiments» no se cumple hoy. ¿Añadís las instancias de Taillard a `data/`? ¿Quitamos o
matizamos la frase?

**Respuesta:**

## 8. Origen de `results/chequeo_beta_alto.csv` (verificación, I1)

Ningún notebook del repo genera ese CSV; hay una copia idéntica en `results/v2_antes_geoadapt/`. El
artículo dice que no lo genera ningún notebook y lo atribuye a «earlier versions», pero eso es una
inferencia. ¿De dónde salió?

**Respuesta:**

---

**Estado del bucle:** APROBADO en la ronda 5 (0 B, 0 I, 5 menores opcionales en
`ignore/paper_loop/revision_r5.md`). Lo que más se nota frente a VRP es la falta de comparación con
las cifras publicadas por Juan et al. (pregunta 3). Si añadís una figura (pregunta 2), habrá que
recortar unas 400 palabras. Cuando respondáis, relanzad `/paper-loop PFSP 1`.
