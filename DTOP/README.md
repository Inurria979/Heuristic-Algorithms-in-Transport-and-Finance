# Dynamic Team Orienteering Problem

Lab activity on the Dynamic Team Orienteering Problem (DTOP) using **Transformers and
reinforcement learning**.

In the Team Orienteering Problem a fleet of `m` vehicles starts at an origin and must reach
a destination within a time budget `T_max`, visiting a subset of the customers; each
customer `i` yields a reward `p_i` when visited and can be served by at most one vehicle.
The goal is to maximise the total reward collected.

```
max  Σ_i p_i y_i
s.t. y_i ≤ 1                        each customer visited at most once
     duration(route_v) ≤ T_max      for every vehicle v
     every route goes from the origin to the destination
```

In the **dynamic** variant the data are not all known up front: rewards, travel times or
the set of available customers change while the routes are being executed, so the policy
has to re-decide on the fly instead of solving one static instance offline. The approach
studied here encodes the state with a Transformer and trains the routing policy with
reinforcement learning.

> **State: finished.** This project contains **no code**: the activity consisted only of a
> presentation in class on the reference papers below.

## Files

| Path | Contents |
|---|---|
| `Guerrero 2025 - Using Transformers and RL for the TOP Under Dynamic Conditions.pdf` | Reference paper |
| `Transformers + RL for the Dynamic Team Orienteering Problem.pdf` | Slides of the class presentation |

`pyproject.toml`, `uv.lock` and `.python-version` are left over from the initial setup of the
repository; they are not needed, since there is nothing to run.
