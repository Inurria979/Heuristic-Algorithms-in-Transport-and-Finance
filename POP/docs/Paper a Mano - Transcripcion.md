# Markowitz portfolio optimization & OSQP
*Transcripción literal de las notas manuscritas — Paper a Mano.pdf (Papers, páginas 24–28)*

---

## Página 24

### Parameters
- μi = expected return of i-th asset
- ρij = correlation between asset i, j   (−1 ≤ ρij ≤ 1)
- si = standard deviation of asset i-th
- R = desired expected return

### Decision variables
wi = Amount of i-asset invested
(wi ≥ 0) do not allow shorting

### Model
    minimize:   Σi^N Σj^N si ρij sj wi wj              (1)
    such that   R = Σi^N wi μi                         (2)
                Σi^N wi = 1                            (3)
                0 ≤ 1 ≤ wi   ∀ i ∈ {1, N}              (4)

### Reading of the model
**minimize:** the sum of the risk times the correlation times amount invested

    risk = si · correlation = ρij · amount invested = wi

**such that:** the return which is: the sum of each investment times its expected return

    expected return = μi

Investing N = 1 and each asset is between 0 and 1.

## Página 25

### Covariance form
Let σij (covariance between assets)

    σij = si sj ρij, then eq (1)
    minimize  Σi^N Σj^N σij wi wj

Since all covariance matrices are positive semidefinite the solution is unique.
In addition, even though this is a non linear problem the SDP σij allows efficient
solver implementation.

*[Esquema: "Matrix SDP" — curva convexa con un único mínimo; "Matrix No-SDP" — curva
ondulada multimodal.]*

The formulation above built an efficient frontier (UEF), a smooth non decreasing curve
which gives the best possible trade-off of risk vs return.

*[Esquema: eje vertical "Return", eje horizontal "risk", curva creciente cóncava.]*

## Página 26

The rest of the paper adds complexity because it suppose scenarios where no more than δi
is invested to any asset, the portfolio is rebalancing, the rebalancing takes money…

### The solver used: OSQP
OSQP is an open-source wrapper for OSQP written in C.

It always resolve the same kind of problem:

    minimize  ½ xᵀ P x + qᵀ x
    such that  l ≤ A ≤ u

where x ∈ ℝⁿ is the optimization variable and P ∈ Sⁿ₊ is a PSD matrix.

Therefore, the first step is translating Markowitz formulation to OSQP.

| Markowitz | OSQP | Value |
|---|---|---|
| min Σi^N Σj^N σij wi wj | ½ xᵀP x + qᵀx | P = 2Σ , q = 0 |
| Σi^N μi wi = Rmin | row 0 of A | l₀ |
| Σi^N wi = 0 | row 1 of A | l₁ |
| wi ≥ 0 | rows 2 – N of A | l = 0 , u = +∞ |

## Página 27

### Some OSQP advantages
- **One factorization for the whole frontier.** In the frontier only l₀ changes so the
  factorization is faster.
- **Warm start.** The previous sol is used for the next problems.
- **Accuracy setting.** It allows the user to define the number of iterations and the
  tolerance. Since the covariances are 1e−3 the tolerance decided is 1e−6.

### Instancies
The instances presented in this edition correspond to OR-Library built by Chang, Meade,
Beasley and Sharaiha (2000). Those instances has the following vals:

| Instance | Index Market | N | μi | si |
|---|---|---|---|---|
| port1 | Hang Sen | ⋮ | ⋮ | ⋮ |
| ⋮ | ⋮ | ⋮ | ⋮ | ⋮ |
| port5 | Nikkei Japan 225 | ⋮ | ⋮ | ⋮ |

## Página 28

Furthermore, each instance has the optimal portfolio at portX.txt which contains the
proportion of assets with highest expected return to each risk.

### The solution proposed
For each…

Table…

*[La página se interrumpe aquí; el resto de la hoja está en blanco en el original.]*

---

## Notas de transcripción

- Se ha respetado el texto original en inglés, incluidas sus abreviaturas y sus
  incorrecciones gramaticales (*suppose*, *resolve*, *has*, *Instancies*…).
- La ecuación (4) aparece manuscrita como `0 ≤ 1 ≤ wi`. Por el contexto es casi seguro que
  se quería escribir `0 ≤ wi ≤ 1`; se transcribe tal cual está en el papel.
- En la tabla de traducción a OSQP la segunda restricción aparece como `Σ wi = 0`, mientras
  que la ecuación (3) de la página 24 dice `Σ wi = 1`. Se mantiene lo escrito; conviene
  revisarlo, porque la restricción de presupuesto debería ser 1.
- La primera línea de la página 27 está cortada en el escaneo: corresponde a la repetición
  de la primera fila de la tabla de la página 26 (la función objetivo).
- Lecturas dudosas por la caligrafía: «where **no more than** δi is invested to any asset»
  (página 26) y «only **l₀** changes» (página 27). Ambas encajan con el contexto.
- «UEF» corresponde a *Unconstrained Efficient Frontier*, la frontera eficiente sin
  restricciones de cardinalidad de Chang, Meade, Beasley y Sharaiha (2000).
