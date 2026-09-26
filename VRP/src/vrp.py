"""Heuristics for the Capacitated Vehicle Routing Problem.

Instances live in ``../data/<name>_input_nodes.txt``: one tab-separated row per node,
``x y demand``, the first row being the depot at the origin. The vehicle capacity is not
in those files and has to be taken from CVRPLIB.
"""

from pathlib import Path

DATA_DIR = Path(__file__).resolve().parent.parent / "data"
