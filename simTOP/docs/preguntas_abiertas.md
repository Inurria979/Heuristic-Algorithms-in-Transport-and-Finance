# simTOP — preguntas que solo pueden responder los autores del paper

Juan et al. (2019), WSC. Ninguna bloquea la implementación: en `explicacion_paper.md` §11 está la
decisión que hemos tomado para cada una.

1. **Fig. 3 (p. 1914):** ¿sobre qué instancias se calcula? Suponemos p1–p3, las únicas que resolvió el SAA.
2. **Fig. 3:** fórmula del gap. Suponemos $100\,(\text{BKS}-\bar R)/\text{BKS}$, con $\bar R$ la media de
   la simulación intensiva.
3. **§5 (p. 1912):** fórmula y peso de la «preferencia» (combinación lineal de ahorro y recompensa).
   Usamos $\alpha s_{ij}+(1-\alpha)(u_i+u_j)$ con $\alpha\in\{0.25,0.5,0.75\}$.
4. **§5:** criterio para actualizar la solución base y la mejor, criterio de parada, tamaño de la élite
   y en qué consiste la destrucción-reconstrucción.
5. **§6 (p. 1913):** ¿Var $=c\cdot t_{ij}$ es intencionado (frente a $c\cdot t_{ij}^2$)? ¿Arcos
   independientes?
