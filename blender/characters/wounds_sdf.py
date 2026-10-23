"""Zombie-Verletzungen als Distanzfeld: Fleisch wird weggenommen (Etikett 'flesh'),
darunter liegen Knochen und Zähne. Wird nach der Kleidung angewendet, damit Risse auch
durch den Stoff gehen. Jede Wunde wird in info["wounds"] eingetragen (Mitte, Radius) –
der Back-Shader verteilt dort Blut."""

import math

import numpy as np

from sculpt.noise import perlin
from sculpt.sdf import Box, Ellipsoid, Func, RoundCone, Sphere, euler, norm, v3

from .head_sdf import _H


def _ragged(shape, amp, freq, seed):
    """Wundrand ausfransen: Rauschen auf den Abstand."""
    return Func(lambda P: shape(P) + amp * perlin(P, freq, seed), shape.lo - amp, shape.hi + amp)


def torn_cheek(sc, sk, info, sg=-1.0):
    h = _H(sk)
    P, r = h.p, h.r
    c = P(0.04, 0.071, -0.097, sg)
    sc.sub(_ragged(Ellipsoid(c, r(0.0135, 0.02, 0.0105), euler(0, 0, -sg * 20)), 0.0025, 260.0, 3.0), k=0.002, tag="flesh")
    # Backenzähne und Zahnfleisch im Riss
    for i, (x, y) in enumerate(((0.029, 0.072), (0.033, 0.066), (0.0365, 0.059))):
        for row, z in ((0, -0.0985), (1, -0.1035)):
            R = euler(0, 0, -sg * (40 + i * 10))
            sc.add(Box(P(x, y, z, sg), r(0.0034, 0.0042, 0.0022 if row == 0 else 0.002), r(0.001), R), k=0.0, tag="teeth")
    sc.add(RoundCone(P(0.028, 0.074, -0.0948, sg), P(0.037, 0.057, -0.0948, sg), r(0.0028), r(0.0028)), k=0.002, tag="flesh")
    sc.add(RoundCone(P(0.028, 0.074, -0.107, sg), P(0.037, 0.057, -0.107, sg), r(0.0028), r(0.0028)), k=0.002, tag="flesh")
    info["wounds"].append((c, 0.03))


def exposed_ribs(sc, sk, info, sg=1.0):
    cz = sk["chest_z"]
    c = v3((sg * 0.13, 0.025, cz - 0.1))
    sc.sub(_ragged(Ellipsoid(c, (0.045, 0.07, 0.06), euler(0, sg * 15, sg * 20)), 0.006, 70.0, 5.0), k=0.006, tag="flesh")
    # drei Rippen, die im Loch zu sehen sind (verlaufen nach vorn abfallend)
    for i in range(3):
        z0 = cz - 0.06 - i * 0.032
        pts = []
        for th in np.linspace(-0.55, 0.75, 7):
            rx, ry = 0.128, 0.088
            pts.append(v3((sg * rx * math.cos(th), ry * math.sin(th) + 0.004, z0 - 0.03 * th)))
        for a, b in zip(pts, pts[1:]):
            sc.add(RoundCone(a, b, 0.0058, 0.0058), k=0.002, tag="bone")
    info["wounds"].append((c, 0.08))


def bite(sc, sk, info, side="L", t=0.45):
    d = sk["side"][side]
    E, W = d["E"], d["W"]
    p = E + (W - E) * t
    # Oberseite des Unterarms (Handrücken-Seite)
    out = -d["hn"]
    c = p + out * 0.026
    sc.sub(_ragged(Ellipsoid(c, (0.022, 0.016, 0.03), np.stack([d["hw"], out, norm(W - E)], axis=1)), 0.003, 180.0, 7.0), k=0.003, tag="flesh")
    info["wounds"].append((c, 0.04))


def neck_wound(sc, sk, info, sg=1.0):
    nz = sk["neck_base"]
    c = v3((sg * 0.046, 0.012, nz + 0.055))
    sc.sub(_ragged(Ellipsoid(c, (0.016, 0.024, 0.03), euler(20, 0, sg * 30)), 0.0025, 80.0, 9.0), k=0.004, tag="flesh")
    info["wounds"].append((c, 0.05))


def scalp_tear(sc, sk, info, sg=1.0):
    h = _H(sk)
    c = h.p(0.035, -0.02, 0.098, sg)
    sc.sub(_ragged(Ellipsoid(c, h.r(0.03, 0.022, 0.012), euler(-20, sg * 30, 0)), 0.003, 120.0, 11.0), k=0.003, tag="bone")
    info["wounds"].append((c, 0.035))


def gut_tear(sc, sk, info):
    wz = sk["waist_z"]
    c = v3((-0.04, 0.09, wz + 0.04))
    sc.sub(_ragged(Ellipsoid(c, (0.04, 0.03, 0.035)), 0.005, 90.0, 13.0), k=0.004, tag="flesh")
    info["wounds"].append((c, 0.06))


def build(sc, sk, info):
    if sk["kind"] == "female":
        torn_cheek(sc, sk, info, sg=1.0)
        gut_tear(sc, sk, info)
        bite(sc, sk, info, side="R", t=0.35)
        scalp_tear(sc, sk, info, sg=-1.0)
    else:
        torn_cheek(sc, sk, info, sg=-1.0)
        exposed_ribs(sc, sk, info, sg=1.0)
        bite(sc, sk, info, side="L")
        neck_wound(sc, sk, info, sg=1.0)
        scalp_tear(sc, sk, info, sg=1.0)
    _ = Sphere
