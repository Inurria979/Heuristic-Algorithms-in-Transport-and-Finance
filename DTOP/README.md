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

> **State: not started.** This folder only holds the reference paper. The uv environment is
> already in place, so the code can be started right away.

## Files

| Path | Contents |
|---|---|
| `Guerrero 2025 - Using Transformers and RL for the TOP Under Dynamic Conditions.pdf` | Reference paper |
| `Transformers + RL for the Dynamic Team Orienteering Problem.pdf` | Reference document for the activity |

## Usage

Requires [uv](https://docs.astral.sh/uv/) only; it fetches Python 3.14 itself.

```bash
uv sync    # numpy, matplotlib (+ ipykernel)
```

The environment is intentionally minimal. Add the deep-learning stack when the
implementation starts, for example:

```bash
uv add torch gymnasium
```

If the wheels for the chosen framework do not yet cover Python 3.14, lower the pin in
`.python-version` and the `requires-python` field of `pyproject.toml` for this project
only — every folder in the repository resolves independently, so it will not affect the
others.
