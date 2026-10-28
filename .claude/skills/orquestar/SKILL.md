---
name: orquestar
description: Orquesta un trabajo de varias fases sobre una o varias prácticas del máster (código, resultados, README y artículo) repartiéndolo entre los agentes code-writer, code-reviewer, paper-writer, paper-reviewer y Explore. Úsalo cuando se pida orquestar, o terminar o rehacer una práctica de principio a fin, o coordinar código y artículo a la vez. No para un arreglo puntual de código ni para solo el artículo (eso es /paper-loop).
argument-hint: <CARPETA[,CARPETA…]> <objetivo> [rondas máx.]
---

# Orquestador

Argumentos: `$ARGUMENTS`. Lo normal es una o varias carpetas de subproyecto (`GRASP`, `PFSP`, `TOP`…) seguidas del objetivo en lenguaje natural, p. ej. `/orquestar GRASP termina el código y escribe el artículo`. Un número suelto es el máximo de rondas por bucle (por defecto 3). Si falta el objetivo, pregunta cuál.

En esta sesión **tú eres el orquestador**. Lee `.claude/agents/orquestador.md` y sigue su protocolo entero (entender → planificar y esperar el visto bueno → ejecutar los bucles de código y de artículo → cierre), con `$ARGUMENTS` como objetivo. Ese fichero es la única fuente de las reglas; no las dupliques aquí.
