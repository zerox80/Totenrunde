"""Skulptur der Ego-Arme: Unterarm mit Jackenärmel, taktischer Handschuh (Knöchelschutz,
Fingerpolster, verstärkte Fingerkuppen, Klettband) und – links – eine Armbanduhr.

Lokales System (Ruhepose), Meter:
  Handgelenk W = Ursprung, Handachse a = +Y (zu den Knöcheln), Ellbogen bei -Y.
  rechts: Handfläche n = -X, Daumen w = +Z      links: n = +X, w = +Z
Die Finger sind leicht gekrümmt (gute Gewichte für das Skelett).

`build(side)` liefert (Sculpt, bones). bones: Name -> dict(head, tail, r, parent) – dieselben
Segmente, aus denen die Finger modelliert sind (für Skelett und Gewichte).
"""

import math

import numpy as np

from sculpt.noise import fbm, perlin
from sculpt.sdf import BIG, Box, Ellipsoid, Loft, RoundCone, Sculpt, Sphere, norm, rot_axis

FOREARM = 0.265  # Handgelenk -> Ellbogen
SLEEVE_START = -0.058


def _axes(side):
    a = np.array([0.0, 1.0, 0.0])
    w = np.array([0.0, 0.0, 1.0])
    n = np.array([-1.0, 0.0, 0.0]) if side == "R" else np.array([1.0, 0.0, 0.0])
    return a, n, w


def build(side="R"):
    sc = Sculpt(pad=0.006)
    a, n, w = _axes(side)
    back = -n
    W = np.zeros(3)
    E = W - a * FOREARM
    bones = {}
    s = 1.06  # Handschuh trägt auf

    # ---------------------------------------------------------------- Unterarm (unter dem Ärmel)
    sc.add(RoundCone(E, W - a * 0.01, 0.041, 0.028), k=0.0)
    bones["forearm"] = dict(head=W, tail=E, r=0.045, parent="hand", z=n)

    # ---------------------------------------------------------------- Mittelhand
    knuckle = W + a * 0.094 * s
    sc.add(
        Loft(
            W - a * 0.012 * s,
            knuckle,
            [(0.0, 0.029 * s, 0.017 * s, 0, 0.0), (0.35, 0.038 * s, 0.0165 * s, 0, 0.001), (0.75, 0.0425 * s, 0.0145 * s, 0, 0.0), (1.0, 0.0415 * s, 0.0125 * s, 0, -0.001)],
            up=n,
        ),
        k=0.012,
        tag="glove",
    )
    Rn = np.stack([w, n, a], axis=1)
    sc.add(Ellipsoid(W + a * 0.032 * s + w * 0.021 * s + n * 0.009 * s, (0.018 * s, 0.013 * s, 0.031 * s), Rn), k=0.012, tag="leather")
    sc.add(Ellipsoid(W + a * 0.04 * s - w * 0.025 * s + n * 0.007 * s, (0.013 * s, 0.01 * s, 0.031 * s), Rn), k=0.012, tag="leather")
    bones["hand"] = dict(head=W, tail=knuckle, r=0.032, parent=None, z=n)

    # ---------------------------------------------------------------- Finger
    fingers = [
        ("index", 0.025, 0.0, (0.044, 0.026, 0.021), 6.0, (10, 16, 10), 0.0102),
        ("middle", 0.0085, 0.004, (0.048, 0.03, 0.022), 1.5, (12, 18, 11), 0.0104),
        ("ring", -0.0085, 0.0, (0.045, 0.028, 0.021), -3.5, (14, 20, 12), 0.0098),
        ("pinky", -0.0245, -0.01, (0.035, 0.021, 0.019), -10.0, (16, 22, 13), 0.0088),
    ]
    for name, ox, oa, lens, spread, curl, r0 in fingers:
        base = W + a * (0.093 + oa) * s + w * ox * s - n * 0.002 * s
        dirv = norm(a + w * math.tan(math.radians(spread)))
        nn = n.copy()
        pts = [base]
        zs = []
        for L_, c in zip(lens, curl):
            axis = norm(np.cross(dirv, nn))
            Rm = rot_axis(axis, math.radians(c))
            dirv = Rm @ dirv
            nn = Rm @ nn
            pts.append(pts[-1] + dirv * L_ * s)
            zs.append(nn.copy())
        rr = [r0 * s, r0 * 0.93 * s, r0 * 0.86 * s, r0 * 0.8 * s]
        for j in range(3):
            sc.add(RoundCone(pts[j], pts[j + 1], rr[j], rr[j + 1]), k=0.004 if j == 0 else 0.0025, tag="tip" if j == 2 else "glove")
            bones[f"{name}{j + 1}"] = dict(head=pts[j], tail=pts[j + 1], r=rr[j], parent="hand" if j == 0 else f"{name}{j}", z=zs[j])
        # Polster auf dem Grundglied (Rückseite)
        seg = pts[1] - pts[0]
        mid = pts[0] + seg * 0.5 + back * rr[0] * 0.95
        ax = norm(seg)
        side_ax = norm(np.cross(ax, n))
        sc.add(Box(mid, (rr[0] * 0.5, 0.0006, np.linalg.norm(seg) * 0.28), rad=0.0022, R=np.stack([side_ax, n, ax], axis=1)), k=0.0015, tag="pad")
        # Polster auf dem Mittelglied
        seg2 = pts[2] - pts[1]
        mid2 = pts[1] + seg2 * 0.5 + back * rr[1] * 0.95
        ax2 = norm(seg2)
        side2 = norm(np.cross(ax2, n))
        sc.add(Box(mid2, (rr[1] * 0.45, 0.0005, np.linalg.norm(seg2) * 0.24), rad=0.0018, R=np.stack([side2, n, ax2], axis=1)), k=0.0012, tag="pad")

    # ---------------------------------------------------------------- Daumen
    tb = W + a * 0.018 * s + w * 0.022 * s + n * 0.008 * s
    dt = norm(a * 0.8 + w * 0.55 + n * 0.32)
    nn = norm(n - dt * np.dot(n, dt))
    pts = [tb]
    zs = []
    for L_, c in zip((0.044, 0.032, 0.028), (6, 12, 14)):
        axis = norm(np.cross(dt, nn))
        Rm = rot_axis(axis, math.radians(c))
        dt = Rm @ dt
        nn = Rm @ nn
        pts.append(pts[-1] + dt * L_ * s)
        zs.append(nn.copy())
    tr = [0.0135 * s, 0.0115 * s, 0.0102 * s, 0.0085 * s]
    for j in range(3):
        sc.add(RoundCone(pts[j], pts[j + 1], tr[j], tr[j + 1]), k=0.008 if j == 0 else 0.0025, tag="tip" if j == 2 else "glove")
        bones[f"thumb{j + 1}"] = dict(head=pts[j], tail=pts[j + 1], r=tr[j], parent="hand" if j == 0 else f"thumb{j}", z=zs[j])

    # ---------------------------------------------------------------- Knöchelschutz (Hartschale)
    kc = W + a * 0.087 * s + back * 0.0148 * s
    Rk = np.stack([w, back, a], axis=1)
    sc.add(Box(kc, (0.032 * s, 0.001, 0.0085 * s), rad=0.003, R=Rk), k=0.003, tag="pad")
    for k_ in (-1.5, -0.5, 0.5, 1.5):
        sc.add(Box(kc + w * k_ * 0.0165 * s + back * 0.0035, (0.0055 * s, 0.0006, 0.0075 * s), rad=0.0018, R=Rk), k=0.0015, tag="pad")
    # Handrücken-Polster
    sc.add(Box(W + a * 0.045 * s + back * 0.0135 * s, (0.025 * s, 0.0005, 0.02 * s), rad=0.003, R=Rk), k=0.005, tag="glove")

    # ---------------------------------------------------------------- Stulpe, Klettband
    sc.add(RoundCone(W - a * 0.07, W + a * 0.008, 0.0345, 0.031), k=0.008, tag="glove")
    sc.add(RoundCone(W - a * 0.026, W - a * 0.004, 0.0345, 0.0335), k=0.0015, tag="strap")
    sc.add(Box(W - a * 0.015 + back * 0.0345, (0.014, 0.0008, 0.011), rad=0.0015, R=Rk), k=0.0015, tag="strap")

    # ---------------------------------------------------------------- Ärmel (Jacke)
    keys = [
        (0.0, 0.043, 0.04, 0, 0.0),
        (0.08, 0.047, 0.043, 0, 0.0),
        (0.45, 0.052, 0.048, 0, 0.0),
        (0.85, 0.058, 0.053, 0, 0.0),
        (1.0, 0.06, 0.055, 0, 0.0),
    ]
    sl0 = W + a * SLEEVE_START
    sl1 = W - a * (FOREARM + 0.07)
    sc.add(Loft(sl0, sl1, keys, up=n), k=0.004, tag="sleeve")
    # Bündchen (Saum leicht dicker)
    sc.add(RoundCone(sl0 - a * 0.004, sl0 - a * 0.016, 0.0445, 0.045), k=0.003, tag="cuff")

    # Falten im Ärmel: Ringe (Stauchfalten) mit Rauschen, kräftiger zum Bündchen hin
    def folds(P):
        t = -(P @ a)  # Abstand hinter dem Handgelenk
        q = P - np.outer(P @ a, a)
        ang = np.arctan2(q @ w, q @ n)
        ripple = np.sin(t * 95.0 + 2.2 * np.sin(ang * 2.0 + t * 30.0) + perlin(P, 22.0, 3.0) * 3.0)
        amp = 0.0016 * np.clip((t - 0.06) / 0.03, 0, 1) * (0.55 + 0.45 * np.clip(1.0 - (t - 0.07) / 0.15, 0, 1))
        big = fbm(P, 18.0, 3, seed=5.0) * 0.0018
        return -(ripple * amp + big) * (t > 0.06)

    lo = np.array([-0.08, -(FOREARM + 0.09), -0.08])
    hi = np.array([0.08, SLEEVE_START + 0.002, 0.08])
    sc.disp(folds, lo, hi)

    # ---------------------------------------------------------------- Uhr (links)
    if side == "L":
        wc = W - a * 0.04 + back * 0.0
        # Armband als Ring über der Stulpe, Gehäuse auf der Außenseite (Handrücken)
        sc.add(RoundCone(wc - a * 0.009, wc + a * 0.009, 0.0372, 0.0372), k=0.0015, tag="watchband")
        case_c = wc + back * 0.037
        sc.add(RoundCone(case_c - back * 0.002, case_c + back * 0.0055, 0.0175, 0.016), k=0.0015, tag="watch")
        sc.add(RoundCone(case_c + back * 0.005, case_c + back * 0.0072, 0.0138, 0.0135), k=0.0005, tag="watchface")
        # Krone
        sc.add(Sphere(case_c + w * 0.0185 + back * 0.002, 0.0025), k=0.0005, tag="watch")

    return sc, bones


def bounds(side):
    return np.array([-0.075, -(FOREARM + 0.085), -0.075]), np.array([0.075, 0.2, 0.08])
