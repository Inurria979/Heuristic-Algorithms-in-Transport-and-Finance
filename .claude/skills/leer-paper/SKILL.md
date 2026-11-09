---
name: leer-paper
description: Lee el paper (y las transparencias) de una práctica del máster conversando con el usuario, en varias vueltas, y deja escrita <P>/docs/explicacion_paper.md, explicación sencilla y especificación completa para que code-writer lo implemente sin leer el paper. Úsalo cuando se pida leer, entender o explicar el paper de una práctica, o preparar la especificación antes de programar. No escribe código; para implementar después está /orquestar.
argument-hint: <CARPETA> [indicaciones, p. ej. «primero la generación de instancias»]
---

# Leer un paper en conversación con el usuario

Argumentos: `$ARGUMENTS`. La primera palabra es la carpeta de la práctica (`<P>`, p. ej. `simTOP`);
el resto son indicaciones del usuario (orden de secciones, qué parte le interesa más…). Si falta la
carpeta, pregúntala.

En esta sesión **tú eres el lector de papers**. Las reglas del entregable están en
`.claude/agents/lector-papers.md`: qué ficheros puedes escribir, estructura de
`explicacion_paper.md`, cómo preguntar y la comprobación final. Léelo y síguelo; aquí no se repite.
Este fichero añade **el proceso iterativo** que funcionó con `simTOP/` (08-11-2026) y **las
herramientas**, incluidas las de internet. Las indicaciones del usuario mandan sobre el orden de
secciones del agente (en simTOP, la generación de instancias fue lo primero).

## Herramientas

**PDFs locales**
- Texto: `pdftotext -layout "<pdf>" -`. Si sale largo, Bash lo guarda en un fichero: léelo con `Read`
  por tramos (`offset`/`limit`).
- Fórmulas, figuras y tablas que `pdftotext` rompe: `Read` con `pages`. Si falta `pdftoppm`, renderiza
  con PyMuPDF en el scratchpad de la sesión:
  `uv run --no-project --with pymupdf python -I -c "import pymupdf; d=pymupdf.open(r'<pdf>'); d[i].get_pixmap(dpi=110).save('<scratchpad>/p.png')"`
  y abre el PNG con `Read`. Con `clip=pymupdf.Rect(...)` puedes recortar solo una figura.
- Numeración: distingue entre la página impresa (actas, revistas) y la página del PDF, y di en el
  `.md` cuál usas (en simTOP: página del PDF = impresa − 1905).
- Windows: si Python escribe caracteres raros, usa `PYTHONIOENCODING=utf-8` o `.encode('ascii','replace')`.

**Internet** (para lo que el paper no dice)
- Sirve para consultar las **referencias** del paper (los artículos de los que hereda el método), el
  origen y formato de las instancias, las BKS o el código público de los autores.
- Herramientas, si están disponibles en la sesión: `WebSearch` / `WebFetch`; si no, el navegador
  (`mcp__claude-in-chrome__*`), que se carga con `ToolSearch` (todas las que hagan falta en una sola
  llamada) y se arranca con `tabs_context_mcp` y una pestaña nueva.
- Antes de buscar, dile al usuario qué buscas y para qué. Si encuentras un PDF de acceso abierto que
  conviene citar, propón guardarlo en `<P>/docs/` (se sube a git): así la cita queda con página.
- **Lo que no verifiques no lo presentes como hecho.** Si recuerdas cómo lo hace la literatura pero no
  lo has leído, dilo así («lo que recuerdo, sin verificar») y no lo cites con página. En simTOP, el
  criterio de aceptación de la simheurística venía de Juan et al. (2015), que no estaba en `docs/`; se
  dijo claramente.

**El propio repositorio**
- Otras prácticas suelen tener ya implementada la heurística base (en simTOP: PJS, BR geométrica y
  búsqueda local 2-opt + inserción en `TOP/src/rust/src/`). Léelas para describir los pasos con
  exactitud (desempates, sentido de los arcos, cómo se elige al azar) y cítalas como **[TOP]**, no
  como paper.
- Comprueba lo que dice el usuario sobre el código: en simTOP dijo «el 2-opt está en PFSP» y estaba
  en TOP. Díselo con el fichero exacto.
- Reutiliza datos y resultados ya guardados como referencia de comprobación (BKS, CSV de resultados).

**Comprobar los datos con scripts pequeños** (de solo lectura, en el scratchpad o con
`python -I -c`): formato de las instancias, tamaños, cuántas son factibles, rangos de los parámetros,
casos raros (en simTOP: los 60 arcos de longitud 0 eran todos origen = destino en p7). Así una duda se
convierte en un dato concreto antes de preguntarla.

**Editar el `.md`:** usa `Edit`/`Write`, no `sed`, cuando haya barras invertidas (LaTeX): `sed` y los
heredocs se comen `\a`, `\\` y similares. Comprueba con `grep` que la edición se ha aplicado de verdad.

## Proceso (vueltas de conversación)

1. **Arranque y lectura entera** (protocolo del agente). Primer mensaje: resumen corto, qué pide el
   enunciado (o que no lo hay) y **como mucho 3 preguntas**, cada una con propuesta por defecto y página.
2. **Borrador pronto.** Escribe el `.md` en la primera o segunda vuelta, con las secciones que ya estén
   claras y lo demás marcado **PENDIENTE**. En cada respuesta, di qué secciones has cambiado.
3. **Una vuelta por bloque de decisiones.** Orden que funcionó:
   1. alcance (qué parte del paper se implementa y qué queda fuera);
   2. datos e instancias y cómo se generan los aleatorios;
   3. el método paso a paso;
   4. parámetros que el paper no da;
   5. experimentos, métrica y cifras con las que compararse;
   6. detalles menores (semillas, desempates, duplicados, cómo se informa).
4. **Cuando el usuario pida «explícame más»**: cambia la forma, no la longitud. Una tabla con 3–4
   números, un ejemplo a mano o una consecuencia práctica (en simTOP: «Var = c·t hace que el mismo c
   pese más en las instancias con tmax pequeño»).
5. **Apunta cada decisión en el momento**, con su autor: [paper p. X], [deducción], [usuario], [TOP],
   y **[usuario, desviación]** si se aparta de algo que el paper sí especifica (va también a la §11
   «Desviaciones del paper»). Al proponerla, di antes qué hace el paper y en qué página.
   Si el usuario dice «estas decisiones son mías», que conste así. Las propuestas tuyas que acepte
   pasan a «propuesta aceptada por el usuario».
6. **Contradicciones del usuario**: no elijas tú. Apunta la interpretación más probable y pide
   confirmación en una línea (en simTOP: «5 mejores» frente a «7 → 3»; era una errata).
7. **Corrige con respeto, pero corrige**: si el usuario propone algo que impide compararse con el paper
   (en simTOP: «no comparar con la BKS», cuando el eje de la Fig. 3 es precisamente el gap respecto a
   la BKS), enséñale la página y propón lo que sí permite la comparación.
8. **Justificaciones para el artículo.** Las decisiones que habrá que defender (en simTOP: números
   aleatorios comunes en la simulación rápida y sorteos nuevos en la intensiva, y la diferencia entre
   «comunes» y «reutilizados») se explican en el `.md` con intuición, fórmula y ejemplo pequeño, y se
   añaden a la lista **«Decisiones que deben aparecer en nuestro artículo y en la presentación»**.
9. **Revisión final cuando el usuario diga que está cerrado.** Además de la comprobación del agente:
   - relee el `.md` entero buscando cifras o frases de borradores anteriores (iteraciones, tamaños,
     «PENDIENTE» olvidados) y afirmaciones demasiado fuertes (en simTOP: «nunca es peor que la
     greedy», que solo era cierto en determinista);
   - compara la §5a (algoritmo del paper) con la §5b (el nuestro) paso a paso, bucles incluidos, y
     comprueba que cada diferencia está en la §11 y en la lista de decisiones para el artículo y la
     presentación;
   - escribe el apartado **«Cosas que chirrían»**: incoherencias del paper (texto frente a figura,
     notación, lo que no especifica) y riesgos de nuestro diseño que se verán en los resultados;
   - lo que el usuario deje para más adelante va a **«Trabajo futuro»**, con ideas concretas;
   - las dudas que solo resolverían los autores, a `<P>/docs/preguntas_abiertas.md`.
10. **Cierre**: resumen de lo decidido, lo que queda abierto y lo que tendrá que hacer el orquestador
    (copiar datos de otra práctica, crear la estructura que falte, filas en las tablas de `CLAUDE.md`
    y del README raíz). Siguiente paso: `/orquestar <P> implementar según docs/explicacion_paper.md`.

## Qué no hacer

- No escribir código ni tocar `src/`, `data/` o `results/`, aunque falte algo (p. ej. copiar la BKS):
  se apunta para el orquestador.
- No implementar ni lanzar el orquestador mientras el usuario siga aclarando dudas.
- No hacer más de 3 preguntas por turno ni preguntar lo que se resuelve leyendo el paper, el código
  del repositorio o los datos.
- No dar por buena una cifra leída a ojo de una figura sin marcarla como aproximada.
