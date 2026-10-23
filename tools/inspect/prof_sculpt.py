"""Profil: Zeit je Operation der Zombie-Skulptur auf einer Oberflächen-Punktwolke."""
import os, sys, time
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
import numpy as np
from sculpt.noise import perlin
from characters import zombie_sculpt
P = np.random.rand(1_000_000, 3)
t = time.time(); perlin(P, 10.0, 1.0); print("PROF perlin 1M", round(time.time() - t, 3))
sk, sc, info = zombie_sculpt.sculpt("male")
lo, hi = sc.bounds()
# Punkte nahe der Oberfläche: zufällig im Körpervolumen, nur |d| < 2 cm behalten
Q = lo + np.random.rand(3_000_000, 3) * (hi - lo)
d = sc.eval_chunked(Q)
Q = Q[np.abs(d) < 0.02][:400000]
print("PROF Punkte", len(Q))
import collections
times = collections.Counter()
orig = list(sc.ops)
for i, op in enumerate(orig):
    sc.ops = orig[: i + 1]
    t = time.time(); sc.eval(Q); dt = time.time() - t
    times[(i, op.kind, type(op.shape).__name__)] = dt
prev = 0.0
rows = []
for (i, k, n), t in sorted(times.items()):
    rows.append((t - prev, i, k, n)); prev = t
for r in sorted(rows, reverse=True)[:15]:
    print("PROF", round(r[0], 3), r[1:])
print("PROF gesamt", round(prev, 2), "ops", len(orig))
