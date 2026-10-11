# Presentation script: Does biased randomization make GRASP better?

Speaker script for `GRASP_TSP_biased_randomization.pptx` (11 slides, about 10–12 minutes).
Text in *italics* is a stage direction, not something to say.

---

## Slide 1: Title (≈ 20 s)

Good morning. In this lab activity we asked a simple question: does *biased randomization*
make GRASP better? We tested it on the travelling salesman problem, using the Clarke-Wright
savings heuristic to build the tours, four different biased selection rules, and the
standard TSPLIB benchmarks.

---

## Slide 2: The travelling salesman problem (≈ 45 s)

The problem is the classic one: visit every city exactly once, return to the start, and
make the total tour as short as possible.

It is NP-hard. The number of possible tours grows factorially with the number of cities,
so for anything beyond a few dozen cities we cannot enumerate them. The practical question
becomes: how close can a fast heuristic get to the optimum?

To measure that, we used seven TSPLIB instances, from 51 to 280 cities, whose optimal tours
are known. Our metric is the **gap**: how much longer our tour is than the optimum, in
percent. A gap of 2% means our tour is 2% longer than the best possible one.

---

## Slide 3: GRASP is one loop (≈ 1 min 15 s)

GRASP stands for *Greedy Randomized Adaptive Search Procedure*. Despite the long name, it
is just one loop with three steps.

*Point to the three boxes.*

First, **construct**: build a tour step by step, choosing among the best candidates, but
with some randomness. Second, **improve**: run a local search, here 2-opt, until no swap of
two edges makes the tour shorter. Third, **keep**: if this tour is the best so far, store
it. Then repeat; in our experiments, 150 times per run.

Each word in the name matches one idea:

- **Greedy**: candidates are ranked by a score, in our case the saving of joining two cities.
- **Randomized**: we don't always take the top candidate, so every iteration builds a
  different tour. That diversity is what makes repeating worthwhile.
- **Adaptive**: after every choice, the list of valid candidates is updated.

The key message of this slide: **only the choice rule in step 1 changes** between the
methods we compare. The construction, the 2-opt and everything else stay identical.

---

## Slide 4: Clarke-Wright savings rank the candidates (≈ 1 min)

To rank the candidates, we use the savings idea from Clarke and Wright, 1964.

*Point to the diagram.*

We pick a hub, *h*, the city closest to the centre of the instance. Imagine a silly
starting solution where we go from the hub to each city and straight back. Now take two
cities, *i* and *j*. If instead of returning to the hub between them we travel directly
from *i* to *j*, we save d(h,i) plus d(h,j) minus d(i,j). That is the saving.

A large saving means the pair is attractive to join.

An important detail: the saving only depends on the hub, *i* and *j*, never on what we
have built so far. So we compute the savings of all pairs **once per instance**, sort them
from largest to smallest, and that sorted list is our candidate list for every iteration.

---

## Slide 5: Building one tour: filter, choose, merge (≈ 1 min 15 s)

Building one tour takes three steps, repeated until it is finished.

**Step 1, filter.** We keep only the pairs where both cities are still the ends of their
route, and they belong to different routes. Joining a city in the middle of a route would
create a branch, and joining two cities of the same route would close a cycle too early.
Once a pair becomes infeasible it never becomes feasible again, so we simply drop it.

**Step 2, choose.** We pick one pair from the ranked list. This is the only place where
the methods differ: the classic GRASP uses a restricted candidate list; biased
randomization uses a probability that decays with the rank.

**Step 3, merge.** We join the two routes through the chosen pair and go back to step 1.

*Point to the worked example.*

With four cities: join A–B, then B–C, then C–D. After n minus 2 merges a single path is
left, A–B–C–D, and we close it through the hub. That is the tour.

Notice that the candidate list shrinks after every merge. That is the "adaptive" part of
GRASP.

---

## Slide 6: Two ways to pick the next pair (≈ 1 min 15 s)

Here is the core of the comparison.

*Left chart.* The **classic GRASP** uses a restricted candidate list, the RCL. We keep the
pairs whose saving is within a fraction α of the best one, and pick one of them with
equal probability. So the shape is a block: uniform inside the list, zero outside. In this
example there are four pairs, each with 25%.

*Right chart.* **Biased randomization** has no cut-off. Every feasible pair is ranked, and
rank *k* is picked with a probability that decays as *k* grows. Here, a geometric rule with
β = 0.4: 40% for the best pair, 24% for the second, about 14% for the third, and so on. The
best pairs are strongly favoured, but every pair keeps some chance. That long tail gives
diversity without throwing away the greedy information.

Same ranked list, same local search: only the probability rule changes.

---

## Slide 7: Four ways to bias the choice (≈ 1 min)

We tested four biased rules.

- **Geometric**: the probability falls exponentially with the rank, controlled by β. This
  is the usual choice in the biased-randomization literature.
- **Triangular**: the probability falls linearly over the top fraction *f* of the list, and
  is zero after that.
- **Zipf**: a power law, one over (k + 1) to the power *e*. It has a heavier tail than the
  geometric rule.
- **Softmax**: different from the other three. It does not look at the rank but at the
  **value** of the saving, normalised, with a temperature *T*. So it sees how much better
  one saving is than another, not just which one comes first.

Each rule fixes the *shape* of the distribution, but each one has a parameter that decides
how greedy or how random it is. At one extreme, we always take the top pair and every
iteration builds the same tour. At the other, the choice is nearly uniform and the tours
are poor. The best value lies in between and depends on the problem, so it has to be
tuned. That leads to the experimental design.

---

## Slide 8: A fair comparison (≈ 1 min)

We wanted any difference to come only from the choice rule, so we controlled three things.

*Left column.* **Held constant**: the same Clarke-Wright construction and hub, the same
2-opt with first improvement, 150 iterations per run, and the same random seeds for every
rule.

*Middle column.* **Tuned fairly**: every rule, including the classic RCL, got the same
tuning effort, a grid of six to eight values. We tuned on three separate instances,
kroB100, eil101 and ch150, and **never on the test set**, so the parameters are not fitted
to the instances we report. The chosen values are on the right: α = 0.05, β = 0.4, and so
on.

*Right column.* **Tested**: seven instances, 20 independent runs per instance and rule.
To decide whether differences are real, we used the Wilcoxon rank-sum test against the
classic RCL, with Holm correction because we make many comparisons, and a Friedman test
across all instances.

---

## Slide 9: Quality and time (≈ 1 min 15 s)

*Left chart.* This is the average gap over the seven instances.

The first bar is the deterministic baseline: Clarke-Wright without randomness, followed by
2-opt. It reaches 5.07%. Just adding randomization and repeating, the classic GRASP drops
to 2.22%. So the first result is that **GRASP works**: the diversity from random
construction, combined with local search, more than halves the error.

Among the biased rules, the **geometric** one is the best, at 1.70%. That is about 23% less
error than the classic RCL. Triangular, Zipf and softmax land close to the RCL, between
2.1 and 2.25%.

*Right chart.* Time per run on the largest instance, a280. The geometric and triangular
rules cost the same as the RCL, around five and a half seconds, because they sample a
position directly. Zipf and softmax take almost twice as long, because they compute a
weight for every candidate at every step.

So the geometric rule improves quality **at no extra cost**.

---

## Slide 10: Where each rule wins, instance by instance (≈ 1 min 15 s)

Averages can hide a lot, so here are the instances one by one. An up arrow means
significantly better than the classic RCL; a down arrow, significantly worse.

- **Geometric**: significantly better on berlin52, where the gap falls from 3.7% to 0.9%,
  and on a280, the largest instance. Equal elsewhere, and **never significantly worse**.
- **Triangular**: better on the two smallest instances, eil51 and berlin52, but worse on
  a280. A mixed result.
- **Zipf and softmax**: no clear gain. Softmax is never different from the RCL. That makes
  sense: the top savings have very similar values, so softmax spreads the probability
  almost evenly over the head of the list, which is basically what an RCL does.

One caution. The Friedman test across the seven instances gives p = 0.76, so it does not
detect a global difference between the rules. The evidence for the geometric rule is
**moderate**: two significant wins, none against. It is not conclusive.

---

## Slide 11: Takeaway (≈ 1 min)

Our reading: with savings, the geometric rule gave the lowest gap, but the evidence is
moderate.

*Left column.* Things worth questioning:

- Only two significant wins, and a non-significant Friedman test.
- The parameters were tuned on three instances with few runs; more tuning data could
  change the ranking.
- 2-opt is a basic local search. A stronger one, like Or-opt or neighbour lists, could
  shrink the differences between rules.

*Right column.* Next steps:

- Larger instances and more runs. The advantage looked larger on a280, so instances above
  200 cities are where we would look first.
- An equal-CPU-time comparison, since Zipf and softmax are about 1.7 times slower.
- Path relinking or an elite set, to exploit the diversity that the bias creates.

Thank you. We are happy to take questions.

---

## Possible questions

**Why recompute nothing? Isn't the TSP different after each merge?**
The saving of joining *i* and *j* only depends on *h*, *i* and *j*. While both are ends of
different paths, both are still connected to the hub, so joining them saves exactly the
same amount. Only feasibility changes, and that is what the filter handles.

**Why does the geometric rule beat the RCL?**
The RCL cut-off is defined on saving *values*, so it depends on their scale. With small
integer distances, as in eil51, many savings tie and very few pairs enter the list; runs
end up identical. The geometric rule works on *ranks*, so its diversity does not depend on
that scale.

**Why β = 0.4, and what does it mean?**
It is the probability of picking the best pair: 40% the first, 24% the second, and so on.
It was the value with the lowest mean gap in the tuning grid, chosen on instances not used
for testing.

**How does 2-opt work?**
It removes two non-adjacent edges and reconnects the tour the other way, which means
reversing the segment between them. The change in length is computed in constant time.
We apply the first improving move found and repeat until no move improves. In the plane,
any two crossing edges can always be improved this way, so the final tour has no crossings.

**Why was the 2-opt compiled with numba?**
It is two nested loops with simple arithmetic, run thousands of times. First improvement
is sequential, so it cannot be vectorised with numpy. Numba compiles the loop to machine
code, which makes it fast enough for the whole experiment.
