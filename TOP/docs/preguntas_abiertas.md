# Preguntas abiertas — artículo TOP

Dudas que han salido al escribir y revisar `Team Orienteering Problem.tex`. Contestad debajo de
cada una (en **Respuesta**) y el bucle escritor–revisor las incorporará.

## 1. Python frente a Rust

El enunciado pide «your own Python program» y el código es Rust. El artículo lo reconoce y lo
justifica en §4 (Python solo como comprobación independiente). ¿Vale así o hay que plantearlo de
otra forma? El revisor avisa: si el profesor no acepta Rust, esto pasa a bloqueante.

**Respuesta:**

## 2. Referencia de Chao et al. (1996)

Se cita como la da Panadero (ref. 39: «A fast and effective heuristic for the orienteering
problem», EJOR 88:475–489), que es el artículo del OP. El de las instancias del TOP no está en
ningún PDF de `docs/`. ¿Añadís el PDF o la referencia correcta? El revisor (de memoria) cree que es *The team orienteering problem*, EJOR 88(3):464–474, 1996.

**Respuesta:**

## 3. Tiempo de la rejilla

El artículo pone 127 s + 0.4 s (de `informe_alpha.md` e `informe_busqueda_local.md`); la
presentación dice unos 140 s. ¿Qué cifra vale?

**Respuesta:**

## 4. Discrepancias entre informes y scratchpad

El artículo sigue los informes generados por `pjs`:

- Élite: 110 496 / 18 757 (informe) frente a 125 490 / 18 791 (scratchpad).
- GRASP: 170 469 (informe) frente a 169 571 (scratchpad).

¿Son correctas las del informe?

**Respuesta:**

## 5. Transparencias del curso

En `docs/` no hay transparencias del curso sobre el TOP (solo `homework.png`), así que no se citan.
¿Hay alguna que deba citarse?

**Respuesta:**

---

**Estado del bucle:** APROBADO en la ronda 3 (0 B, 0 I, 3 menores en `ignore/paper_loop/revision_r3.md`).
La pregunta 1 es la crítica: si el profesor no acepta Rust, pasa a bloqueante. Cuando respondáis,
relanzad `/paper-loop TOP 1`.
