"""Haare als Distanzfeld: Haarkappe über der Kopfhaut (wie Kleidung als Aufblähung) und
lange Strähnen, die über den Rücken und neben dem Gesicht herabhängen. Etikett 'hair'.
Die feinen Strähnen entstehen im Back-Shader als Relief."""

import numpy as np

from sculpt.noise import fbm, perlin
from sculpt.sdf import Ellipsoid, Func, Loft, RoundCone, smoothstep, v3

from .head_sdf import _H


def long_hair(sc, sk, seed=31.0):
    h = _H(sk)
    hc, hs = sk["hc"], sk["hs"]

    def region(P, ctx):
        q = (P - hc) / hs
        x, y, z = q[:, 0], q[:, 1], q[:, 2]
        # Haaransatz: vorn hoch (Stirn frei), an den Seiten über den Ohren, hinten tief
        line = 0.055 - 0.03 * smoothstep(0.06, -0.04, y) - 0.05 * smoothstep(0.01, -0.06, y)
        line = line + 0.006 * perlin(P, 60.0, seed)
        face = (y > 0.04) & (np.abs(x) < 0.06) & (z < 0.06)
        reg = (line - z) * hs
        return np.where(face, np.maximum(reg, 0.01), reg)

    def surface(P, d, ctx):
        strands = fbm(P, (55.0, 55.0, 8.0), 3, seed=seed + 1)
        return d - (0.009 + 0.004 * strands) * hs

    sc.garment(region, surface, hc + v3((-0.14, -0.16, -0.08)) * hs, hc + v3((0.14, 0.14, 0.13)) * hs, "hair", hem=0.004)
    # Haarmasse am Hinterkopf und lange Strähnen bis über die Schultern
    def messy(shape, amp, freq, s):
        return Func(lambda P: shape(P) + amp * fbm(P, (freq, freq, freq * 0.25), 3, seed=s), shape.lo - amp, shape.hi + amp)

    back = Loft(
        h.p(0, -0.055, 0.03),
        h.p(0, -0.075, -0.25),
        [(0.0, 0.078, 0.05, 0, 0), (0.35, 0.085, 0.042, 0, -0.005), (0.7, 0.09, 0.03, 0, -0.01), (1.0, 0.075, 0.018, 0, -0.012)],
        up=(0, 1, 0),
    )
    sc.add(messy(back, 0.006, 40.0, seed + 2), k=0.02, tag="hair")
    for sg in (-1.0, 1.0):
        side = RoundCone(h.p(0.068, 0.0, 0.0, sg), h.p(0.072, 0.02, -0.16, sg), 0.022 * hs, 0.012 * hs)
        sc.add(messy(side, 0.005, 50.0, seed + 3 + sg), k=0.015, tag="hair")
    # ein paar lose Strähnen
    rng = np.random.default_rng(int(seed))
    for _ in range(6):
        sg = rng.choice([-1.0, 1.0])
        x0 = rng.uniform(0.03, 0.07)
        a = h.p(x0, rng.uniform(-0.06, 0.02), rng.uniform(0.0, 0.05), sg)
        b = a + v3((sg * rng.uniform(0.0, 0.02), rng.uniform(-0.03, 0.01), -rng.uniform(0.12, 0.22))) * hs
        sc.add(RoundCone(a, b, 0.008 * hs, 0.003 * hs), k=0.006, tag="hair")
    _ = Ellipsoid
