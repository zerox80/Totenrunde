"""Kleidung als Distanzfeld: die Körperform wird in einer Region aufgebläht (Stoff liegt um
den Körper), mit Faltenwurf, ausgefransten Säumen und Rissen; an Löchern liegt der Stoff
dünn an, dort sieht man die Haut. Stiefel sind eigene Formen.

Jede Figur hat ein Outfit:
  male:   kurzärmeliges Arbeitshemd, Jeans mit Gürtel, Arbeitsstiefel
  female: ärmelloses Top, Stoffhose, Stiefeletten
"""

import numpy as np

from sculpt.noise import fbm, perlin
from sculpt.sdf import BIG, Box, Ellipsoid, Loft, euler, smoothstep, v3


def _arm_coords(P, d):
    """Achsparameter t (0 = Schulter, 1 = Ellbogen) und Abstand zur Oberarmachse."""
    S, E = d["S"], d["E"]
    ax = E - S
    L = float(np.linalg.norm(ax))
    ax = ax / L
    q = P - S
    t = (q @ ax) / L
    rad = np.linalg.norm(q - np.outer(q @ ax, ax), axis=1)
    return t, rad, L


def _holes(P, freq, thr, seed):
    """> 0 im Loch."""
    return fbm(P, freq, octaves=4, seed=seed) - thr


# --------------------------------------------------------------------------------------
# Oberteile
# --------------------------------------------------------------------------------------


def shirt(sc, sk, sleeve=0.55, seed=1.0, tag="shirt", hole_thr=0.62, base=0.007, sleeveless=False):
    s, wz, nz = sk["s"], sk["waist_z"], sk["neck_base"]
    sides = list(sk["side"].values())

    def holes(P):
        h = fbm(P, (7.0, 7.0, 5.0), 3, seed=seed) - hole_thr
        # Risse eher an Flanken und Rücken unten
        return h

    def region(P, ctx):
        x, y, z = P[:, 0], P[:, 1], P[:, 2]
        hem = wz - 0.075 * s + 0.014 * fbm(P, (9, 9, 4), 2, seed=seed + 2) + 0.004 * perlin(P, 90, seed + 3)
        collar = nz + 0.008 - 0.065 * smoothstep(0.0, 0.09, y) * smoothstep(0.06, 0.0, np.abs(x))
        if sleeveless:
            collar = collar - 0.02 * smoothstep(0.05, 0.12, np.abs(x))
        torso = np.maximum.reduce([hem - z, z - collar, np.abs(x) - (0.2 if sleeveless else 0.25) * sk["w"]])
        reg = torso
        if not sleeveless:
            for d in sides:
                t, rad, L = _arm_coords(P, d)
                end = sleeve + 0.06 * perlin(P, 11.0, seed + 5) + 0.012 * perlin(P, 70, seed + 6) / L
                arm = np.maximum.reduce([(t - end) * L, rad - 0.11, (-t - 0.35) * L])
                reg = np.minimum(reg, arm)
        ctx["holes"] = holes(P)
        return np.maximum(reg, ctx["holes"] * 0.06)

    def surface(P, d, ctx):
        x, y, z = P[:, 0], P[:, 1], P[:, 2]
        folds = fbm(P, (26, 26, 7), 3, seed=seed + 7)  # senkrechte Falten (hängender Stoff)
        fine = perlin(P, (60, 60, 30), seed + 8)
        off = base + 0.0045 * folds + 0.0007 * fine
        # Stoff hängt von der Brust gerade nach unten: vorn am Bauch und am Kreuz weiter weg
        belly = smoothstep(0.0, 0.06, y) * smoothstep(sk["chest_z"] - 0.02, sk["chest_z"] - 0.12, z) * smoothstep(wz - 0.1, wz - 0.02, z)
        back = smoothstep(0.0, -0.06, y) * smoothstep(sk["chest_z"] - 0.1, wz, z)
        off = off + 0.012 * belly + 0.006 * back
        # über der Hose aufgebauscht
        off = off + 0.003 * smoothstep(wz - 0.02, wz - 0.07, z)
        # Ärmel weit
        if not sleeveless:
            for dd in sides:
                t, rad, L = _arm_coords(P, dd)
                m = smoothstep(0.05, 0.2, t) * smoothstep(0.16, 0.1, rad)
                off = off + m * (0.008 + 0.004 * perlin(P, (40, 40, 40), seed + 9))
        # an Lochrändern dünn anliegend
        off = off * (0.25 + 0.75 * smoothstep(0.0, 0.05, -ctx["holes"]))
        return d - off

    lo = v3((-0.5, -0.25, wz - 0.16))
    hi = v3((0.5, 0.25, nz + 0.06))
    sc.garment(region, surface, lo, hi, tag, hem=0.0025)


# --------------------------------------------------------------------------------------
# Hosen
# --------------------------------------------------------------------------------------


def pants(sc, sk, seed=4.0, tag="pants", rise=0.0, base=0.009, knee_holes=True):
    s, wz = sk["s"], sk["waist_z"]
    top = wz - 0.035 * s + rise
    bottom = sk["ankle_z"] + 0.07

    def holes(P):
        h = fbm(P, (8, 8, 6), 3, seed=seed) - 0.66
        if knee_holes:
            for d in sk["side"].values():
                K = d["K"]
                dk = np.linalg.norm((P - K - v3((0, 0.04, 0.02))) / v3((0.05, 0.06, 0.06)), axis=1)
                h = np.maximum(h, (1.0 - dk) * 0.25 + 0.08 * perlin(P, 40, seed + 1) - 0.05)
        return h

    def region(P, ctx):
        x, z = P[:, 0], P[:, 2]
        bot = bottom + 0.006 * perlin(P, 60, seed + 2)
        reg = np.maximum.reduce([z - top, bot - z, np.abs(x) - 0.27])
        ctx["holes"] = holes(P)
        return np.maximum(reg, ctx["holes"] * 0.06)

    def surface(P, d, ctx):
        z = P[:, 2]
        bunch = fbm(P, (10, 10, 34), 3, seed=seed + 3)  # waagerechte Stauchfalten
        long = fbm(P, (30, 30, 8), 2, seed=seed + 4)
        knee = smoothstep(sk["knee_z"] - 0.12, sk["knee_z"], z) * smoothstep(sk["knee_z"] + 0.15, sk["knee_z"], z)
        ankle = smoothstep(bottom + 0.2, bottom, z)
        off = base + 0.0035 * long + (0.003 + 0.004 * knee + 0.006 * ankle) * bunch
        # Schritt und Gesäß etwas weiter
        off = off + 0.004 * smoothstep(sk["hip_z"] - 0.2, sk["hip_z"], z)
        off = off * (0.3 + 0.7 * smoothstep(0.0, 0.05, -ctx["holes"]))
        return d - off

    lo = v3((-0.32, -0.25, bottom - 0.05))
    hi = v3((0.32, 0.25, top + 0.05))
    sc.garment(region, surface, lo, hi, tag, hem=0.002)
    return top


def belt(sc, sk, top):
    """Gürtel als schmales Band über dem Hosenbund, Schnalle vorn."""

    def region(P, ctx):
        z = P[:, 2]
        return np.maximum.reduce([z - (top - 0.004), (top - 0.04) - z, np.abs(P[:, 0]) - 0.25])

    def surface(P, d, ctx):
        return d - 0.004

    sc.garment(region, surface, v3((-0.3, -0.2, top - 0.06)), v3((0.3, 0.2, top + 0.02)), "belt", hem=0.0015)
    # Schnalle: vorderste Stelle des Bunds suchen
    zc = top - 0.022
    ys = np.linspace(0.05, 0.25, 400)
    P = np.stack([np.zeros_like(ys), ys, np.full_like(ys, zc)], axis=1)
    dd = sc.eval(P)
    yf = ys[np.argmax(dd > 0)] if np.any(dd > 0) else 0.11
    sc.add(Box((0, yf + 0.002, zc), (0.024, 0.003, 0.018), 0.003), k=0.002, tag="buckle")


# --------------------------------------------------------------------------------------
# Schuhe
# --------------------------------------------------------------------------------------


def boots(sc, sk, shaft=0.14, tag="shoes"):
    for d in sk["side"].values():
        sg = d["sg"]
        A = d["A"]
        ax, ay = A[0], A[1]
        sc.add(
            Loft(A + v3((0, 0.004, shaft)), A + v3((0, 0.0, -0.035)), [(0, 0.047, 0.05, 0, 0), (0.6, 0.044, 0.05, 0, 0.004), (1.0, 0.048, 0.056, 0, 0.0)], up=(0, 1, 0)),
            k=0.01,
            tag=tag,
        )
        # Schaft-Unterteil, Ferse, Rist (abfallend), Schuhkörper, Zehenkappe
        sc.add(Box((ax + sg * 0.004, ay + 0.055, 0.045), (0.032, 0.105, 0.024), 0.016, euler(0, 0, -sg * 3)), k=0.02, tag=tag)
        sc.add(Ellipsoid((ax, ay - 0.03, 0.06), (0.044, 0.05, 0.05)), k=0.025, tag=tag)
        sc.add(Ellipsoid((ax + sg * 0.004, ay + 0.05, 0.07), (0.046, 0.08, 0.044), euler(-18, 0, 0)), k=0.03, tag=tag)
        sc.add(Ellipsoid((ax + sg * 0.009, ay + 0.15, 0.04), (0.047, 0.062, 0.032)), k=0.03, tag=tag)
        # Sohle (Gummi) mit Absatz
        sc.add(Box((ax + sg * 0.006, ay + 0.07, 0.012), (0.044, 0.122, 0.0095), 0.0055, euler(0, 0, -sg * 3)), k=0.004, tag="sole")
        sc.add(Box((ax, ay - 0.04, 0.02), (0.038, 0.04, 0.012), 0.005), k=0.003, tag="sole")


def build(sc, sk, info):
    if sk["kind"] == "female":
        shirt(sc, sk, seed=11.0, tag="shirt", hole_thr=0.6, base=0.004, sleeveless=True)
        pants(sc, sk, seed=14.0, tag="pants", rise=0.01, base=0.007, knee_holes=False)
        boots(sc, sk, shaft=0.1)
    else:
        top = pants(sc, sk)
        belt(sc, sk, top)
        shirt(sc, sk)
        boots(sc, sk)
    _ = (BIG, info)
