"""Körper als Distanzfeld: Rumpf, Hals, Arme, Hände und Beine aus Knochen- und
Muskelformen, weich verschmolzen.

Reihenfolge: zuerst die großen Grundformen (Rumpf-Loft, Gliedmaßen-Lofts), dann Muskeln
mit kleinerem Verschmelzungsradius, damit ihre Ränder sichtbar bleiben."""

import math

import numpy as np

from sculpt.noise import perlin
from sculpt.sdf import (
    Box,
    Ellipsoid,
    Func,
    Loft,
    RoundCone,
    Sphere,
    capsule,
    ellipsoid_between,
    euler,
    frame,
    norm,
    rot_axis,
    smoothstep,
    v3,
)


def torso(sc, sk):
    s, w, m = sk["s"], sk["w"], sk["muscle"]
    keys = sk["torso"]
    z0, z1 = keys[0][0], keys[-1][0]
    lk = [((z - z0) / (z1 - z0), rx, ry, 0.0, cy) for z, rx, ry, cy in keys]
    sc.add(Loft((0, 0, z0), (0, 0, z1), lk, up=(0, 1, 0)))
    cz, wz, nz = sk["chest_z"], sk["waist_z"], sk["neck_base"]
    female = sk["kind"] == "female"
    for side, d in sk["side"].items():
        sg = d["sg"]
        S, acr = d["S"], d["acromion"]
        # Schultergürtel: Masse vom Hals zur Schulterhöhe
        sc.add(RoundCone((sg * 0.03 * s, -0.022, nz - 0.03 * s), (acr[0] * 0.92, -0.016, acr[2] - 0.012 * s), 0.05 * w, 0.038 * w), k=0.04)
        # Trapezmuskel (Nacken -> Schulter), gibt die schräge Schulterlinie
        sc.add(
            RoundCone((sg * 0.022 * s, -0.05 * s, nz + 0.05 * s), (acr[0] * 0.9, -0.03, acr[2] - 0.004), 0.034 * w, 0.018 * w),
            k=0.035,
        )
        # Schlüsselbein (dezent)
        sc.add(capsule((sg * 0.022 * s, 0.062 * s, nz - 0.045 * s), (acr[0] * 0.86, 0.018, acr[2] - 0.02 * s), 0.0095 * s), k=0.018)
        if not female:
            # Brustmuskel
            R = euler(0, -sg * 14, -sg * 24)
            sc.add(Ellipsoid((sg * 0.066, 0.09, cz + 0.032), (0.08, 0.018 * m + 0.006, 0.056), R), k=0.03)
            # Brustwarzen
            sc.add(Sphere((sg * 0.09, 0.114, cz + 0.006), 0.0045), k=0.005)
        else:
            # Brust (weich, leicht hängend)
            R = euler(-8, 0, -sg * 18)
            sc.add(Ellipsoid((sg * 0.068, 0.088, cz - 0.01), (0.062, 0.052, 0.056), R), k=0.03)
            sc.add(Sphere((sg * 0.074, 0.136, cz - 0.018), 0.006), k=0.006)
        # Latissimus (Rückenflanken)
        R = euler(0, sg * 10, sg * 25)
        sc.add(Ellipsoid((sg * 0.112 * w, -0.04, cz - 0.03), (0.036 * w, 0.055, 0.12 * s), R), k=0.05)
        # Schulterblatt
        R = euler(0, -sg * 12, 0)
        sc.add(Ellipsoid((sg * 0.082 * w, -0.084 * w, cz + 0.06 * s), (0.05 * w, 0.016, 0.066 * s), R), k=0.045)
        # Rückenstrecker (Lende)
        sc.add(capsule((sg * 0.026, -0.082, wz - 0.1 * s), (sg * 0.03, -0.076, wz + 0.15 * s), 0.02 * m + 0.006), k=0.04)
        # Gesäß
        R = euler(-10, 0, sg * 8)
        gx = 0.074 if not female else 0.08
        sc.add(Ellipsoid((sg * gx, -0.072, sk["hip_z"] - 0.035 * s), (0.082, 0.064 if not female else 0.072, 0.098 * s), R), k=0.04)
        # Hüfte (großer Rollhügel + Hüftmuskeln): weicher Übergang Taille -> Oberschenkel
        hw = 0.128 if not female else 0.142
        sc.add(Ellipsoid((sg * hw, -0.012, sk["hip_z"] - 0.04 * s), (0.05, 0.078, 0.14 * s)), k=0.07)
    # Bauch: gerader Bauchmuskel, Unterbauch, Bauchnabel
    sc.add(Ellipsoid((0, 0.064, wz + 0.06 * s), (0.068, 0.036, 0.15 * s)), k=0.035)
    sc.add(Ellipsoid((0, 0.058, wz - 0.06 * s), (0.10 * w, 0.042, 0.075 * s)), k=0.04)
    sc.sub(Sphere((0, 0.103, wz + 0.002), 0.0065), k=0.005)
    # Rippenbogen leicht betonen (ausgemergelt): Rillen an den Flanken
    if sk["kind"] == "male":
        def ribs(P):
            x, y, z = P[:, 0], P[:, 1], P[:, 2]
            band = smoothstep(cz - 0.2, cz - 0.12, z) * smoothstep(cz + 0.06, cz - 0.02, z)
            flank = smoothstep(0.07, 0.13, np.abs(x)) * smoothstep(-0.07, 0.0, y)
            phase = (z - 0.03 * np.abs(x) * 6.0 + 0.4 * np.maximum(y, 0)) / 0.026
            return 0.0012 * band * flank * (0.5 + 0.5 * np.cos(phase * 2 * math.pi))
        sc.disp(ribs, (-0.2, -0.12, cz - 0.22), (0.2, 0.14, cz + 0.08))
    # Wirbelsäulenrinne
    def spine_groove(P):
        x, y, z = P[:, 0], P[:, 1], P[:, 2]
        back = smoothstep(-0.03, -0.07, y)
        vert = smoothstep(sk["hip_z"] - 0.02, sk["hip_z"] + 0.08, z) * smoothstep(nz + 0.02, nz - 0.08, z)
        return 0.0028 * np.exp(-((x / 0.017) ** 2)) * back * vert
    sc.disp(spine_groove, (-0.04, -0.16, sk["hip_z"] - 0.05), (0.04, -0.02, nz + 0.05))


def neck(sc, sk):
    s, nz, hs = sk["s"], sk["neck_base"], sk["hs"]
    hc = sk["hc"]
    top = v3((0, -0.002, hc[2] - 0.09 * hs))
    r = 0.056 if sk["kind"] == "male" else 0.047
    sc.add(
        Loft((0, -0.024, nz - 0.04 * s), top, [(0, r * 1.15, r * 1.05, 0, 0), (0.45, r, r * 0.95, 0, 0.003), (1.0, r * 0.95, r * 0.95, 0, 0.0)]),
        k=0.03,
    )
    for sg in (-1.0, 1.0):
        # Kopfwender (vom Ohr zum Brustbein)
        sc.add(
            RoundCone((sg * 0.054 * hs, -0.016, hc[2] - 0.072 * hs), (sg * 0.013, 0.048 * s, nz - 0.03 * s), 0.0125 * hs, 0.0095),
            k=0.012,
        )
    if sk["kind"] == "male":
        sc.add(Ellipsoid((0, 0.046, nz + 0.045), (0.009, 0.009, 0.014)), k=0.02)


def arms(sc, sk):
    s, w, m = sk["s"], sk["w"], sk["muscle"]
    r = sk["arm_r"]
    for side, d in sk["side"].items():
        sg = d["sg"]
        S, E, W = d["S"], d["E"], d["W"]
        d1, d2 = d["dir1"], d["dir2"]
        # Oberarm
        u, v, _ = frame(E - S, (0, 1, 0))
        sc.add(
            Loft(S - d1 * 0.02, E, [(0.0, r, r * 1.05, 0, 0), (0.35, r * 0.93, r * 1.0, 0, 0.0), (0.78, r * 0.84, r * 0.88, 0, -0.002), (1.0, r * 0.8, r * 0.82, 0, -0.003)], up=(0, 1, 0)),
            k=0.03,
        )
        # Deltamuskel
        top = d["acromion"] + v3((0, 0, 0.004))
        sc.add(ellipsoid_between(top, S + d1 * 0.155 * s, 0.05 * w, 0.054 * w, up=(0, 1, 0), bulge=0.4), k=0.025)
        # Bizeps vorn, Trizeps hinten
        sc.add(ellipsoid_between(S + d1 * 0.08 + v * 0.016, S + d1 * 0.27 + v * 0.016, 0.028 * w, 0.026 * m + 0.006, up=v, bulge=0.56), k=0.02)
        sc.add(ellipsoid_between(S + d1 * 0.05 - v * 0.017, S + d1 * 0.25 - v * 0.02, 0.031 * w, 0.024 * m + 0.006, up=v, bulge=0.4), k=0.022)
        # Ellbogen: Olekranon und Knorren
        sc.add(Sphere(E - v * 0.024 + d1 * 0.004, 0.011 * s), k=0.016)
        sc.add(Sphere(E + u * 0.024 * s, 0.0085 * s), k=0.016)
        sc.add(Sphere(E - u * 0.024 * s, 0.0085 * s), k=0.016)
        # Unterarm: Handgelenk breit in Daumenrichtung (hw), flach zur Handfläche
        fr = r * 0.81
        sc.add(
            Loft(
                E - d2 * 0.01,
                W + d2 * 0.004,
                [(0.0, fr, fr * 1.0, 0, 0), (0.22, fr * 1.04, fr * 1.1, 0, 0.003), (0.55, fr * 0.84, fr * 0.92, 0, 0), (0.86, fr * 0.6, fr * 0.78, 0, 0), (1.0, fr * 0.55, fr * 0.74, 0, 0)],
                up=d["hw"],
            ),
            k=0.045,
        )
        # Oberarm-Speichenmuskel (außen am Ellbogen)
        uu, vv, _ = frame(W - E, d["hw"])
        sc.add(ellipsoid_between(E + vv * 0.018 - d2 * 0.01, E + d2 * 0.14 + vv * 0.014, 0.019 * w, 0.019, up=vv, bulge=0.4), k=0.02)
        # Beugermasse (Handflächenseite)
        sc.add(ellipsoid_between(E + d["hn"] * 0.014, E + d2 * 0.15 + d["hn"] * 0.01, 0.022, 0.016, up=d["hn"], bulge=0.4), k=0.02)
        # Handgelenk-Knochen (Elle)
        sc.add(Sphere(W - vv * 0.02 - d["hn"] * 0.004, 0.007 * s), k=0.012)
        hand(sc, sk, d)


def hand(sc, sk, d):
    """Hand mit fünf gekrümmten Fingern (leicht gespreizte Klaue) und Nägeln."""
    s = sk["s"] * (0.94 if sk["kind"] == "female" else 1.0)
    W, a, w, n = d["W"], d["ha"], d["hw"], d["hn"]
    sg = d["sg"]
    # Mittelhand: Loft vom Handgelenk zu den Knöcheln (u = Daumenrichtung, v = Handfläche)
    sc.add(
        Loft(
            W - a * 0.006 * s,
            W + a * 0.094 * s,
            [(0.0, 0.026 * s, 0.015 * s, 0, 0.001), (0.35, 0.036 * s, 0.0155 * s, 0, 0.002), (0.75, 0.041 * s, 0.0135 * s, 0, 0.0005), (1.0, 0.04 * s, 0.0115 * s, 0, -0.001)],
            up=n,
        ),
        k=0.012,
    )
    # Daumenballen, Kleinfingerballen
    Rn = np.stack([w, n, a], axis=1)
    sc.add(Ellipsoid(W + a * 0.032 * s + w * 0.02 * s + n * 0.008 * s, (0.017 * s, 0.012 * s, 0.03 * s), Rn), k=0.012)
    sc.add(Ellipsoid(W + a * 0.04 * s - w * 0.024 * s + n * 0.006 * s, (0.012 * s, 0.009 * s, 0.03 * s), Rn), k=0.012)
    # Finger: (Versatz quer, Versatz längs, Gliedlängen, Spreizung°, Krümmung° je Gelenk)
    fingers = [
        (0.025, 0.0, (0.042, 0.025, 0.019), 7.0, (12, 22, 14)),
        (0.0085, 0.004, (0.046, 0.029, 0.020), 1.5, (16, 28, 16)),
        (-0.0085, 0.0, (0.043, 0.027, 0.019), -4.0, (20, 32, 18)),
        (-0.0245, -0.01, (0.033, 0.019, 0.017), -11.0, (24, 34, 20)),
    ]
    radii = (0.0098, 0.0089, 0.0079, 0.0068)
    for i, (ox, oa, lens, spread, curl) in enumerate(fingers):
        sm = 0.86 if i == 3 else 1.0
        base = W + a * (0.092 + oa) * s + w * ox * s - n * 0.002 * s
        dirv = norm(a + w * math.tan(math.radians(spread)))
        nn = n.copy()
        pts = [base]
        ns = []
        for L, c in zip(lens, curl):
            axis = norm(np.cross(dirv, nn))
            Rm = rot_axis(axis, math.radians(c))
            dirv = Rm @ dirv
            nn = Rm @ nn
            pts.append(pts[-1] + dirv * L * s)
            ns.append(nn.copy())
        rr = [x * s * sm for x in radii]
        for j in range(3):
            sc.add(RoundCone(pts[j], pts[j + 1], rr[j], rr[j + 1]), k=0.0035 if j == 0 else 0.002)
        # Knöchel
        sc.add(Sphere(base - n * 0.003 * s, 0.0094 * s * sm), k=0.008)
        # Nagel auf dem Endglied (Rückseite = -n)
        tip_a, tip_b = pts[2], pts[3]
        nd = ns[2]
        mid = tip_a + (tip_b - tip_a) * 0.6 - nd * rr[3] * 0.72
        ax = norm(tip_b - tip_a)
        side_ax = norm(np.cross(ax, nd))
        Rnail = np.stack([side_ax, nd, ax], axis=1)
        sc.add(Ellipsoid(mid, (rr[3] * 0.82, 0.0019, np.linalg.norm(tip_b - tip_a) * 0.5), Rnail), k=0.0015, tag="nail")
    # Daumen
    tb = W + a * 0.022 * s + w * 0.024 * s + n * 0.006 * s
    dt = norm(a * 0.8 + w * 0.5 + n * 0.38)
    nn = norm(n - dt * np.dot(n, dt))
    pts = [tb]
    lens = (0.042, 0.031, 0.027)
    curls = (6, 14, 18)
    for L, c in zip(lens, curls):
        axis = norm(np.cross(dt, nn))
        Rm = rot_axis(axis, math.radians(c))
        dt = Rm @ dt
        nn = Rm @ nn
        pts.append(pts[-1] + dt * L * s)
    tr = [0.0128 * s, 0.0108 * s, 0.0096 * s, 0.0078 * s]
    for j in range(3):
        sc.add(RoundCone(pts[j], pts[j + 1], tr[j], tr[j + 1]), k=0.008 if j == 0 else 0.002)
    ax = norm(pts[3] - pts[2])
    side_ax = norm(np.cross(ax, nn))
    mid = pts[2] + (pts[3] - pts[2]) * 0.6 - nn * tr[3] * 0.75
    sc.add(Ellipsoid(mid, (tr[3] * 0.85, 0.0019, np.linalg.norm(pts[3] - pts[2]) * 0.48), np.stack([side_ax, nn, ax], axis=1)), k=0.0015, tag="nail")
    _ = sg


def legs(sc, sk):
    s, m = sk["s"], sk["muscle"]
    tr, cr = sk["thigh_r"], sk["calf_r"]
    female = sk["kind"] == "female"
    for side, d in sk["side"].items():
        sg = d["sg"]
        H, K, A = d["H"], d["K"], d["A"]
        # Oberschenkel (startet im Becken)
        sc.add(
            Loft(
                H + v3((sg * 0.002, -0.006, 0.01 * s)),
                K,
                [(0.0, tr * 0.8, tr * 0.95, 0, -0.008), (0.12, tr, tr * 1.03, 0, 0.0), (0.5, tr * 0.86, tr * 0.88, 0, 0.004), (0.84, tr * 0.68, tr * 0.7, 0, 0.004), (1.0, tr * 0.62, tr * 0.64, 0, 0.0)],
                up=(0, 1, 0),
            ),
            k=0.045,
        )
        # Muskeln: seitlicher/innerer Quadrizeps, gerader Schenkelmuskel, Beuger, Adduktoren
        sc.add(ellipsoid_between(H + v3((sg * 0.05, 0.016, -0.09 * s)), K + v3((sg * 0.034, 0.012, 0.11 * s)), 0.036, 0.042, up=(sg, 0.4, 0), bulge=0.48), k=0.03)
        sc.add(Ellipsoid(K + v3((-sg * 0.03, 0.026, 0.1 * s)), (0.03, 0.028 * m + 0.006, 0.055 * s)), k=0.025)
        sc.add(ellipsoid_between(H + v3((0, 0.05, -0.07 * s)), K + v3((0, 0.04, 0.12 * s)), 0.034, 0.028 * m + 0.006, up=(0, 1, 0), bulge=0.45), k=0.03)
        sc.add(ellipsoid_between(H + v3((0, -0.05, -0.09 * s)), K + v3((0, -0.035, 0.08 * s)), 0.044, 0.034, up=(0, -1, 0), bulge=0.4), k=0.03)
        sc.add(Ellipsoid(H + v3((-sg * 0.045, -0.002, -0.11 * s)), (0.04, 0.052, 0.11 * s)), k=0.035)
        # Knie: Kniescheibe und Gelenkknorren
        sc.add(Ellipsoid(K + v3((0, 0.047, 0.014)), (0.024 * s, 0.012, 0.028 * s)), k=0.014)
        sc.add(Sphere(K + v3((sg * 0.026 * s, 0.0, 0.004)), 0.026 * s), k=0.03)
        sc.add(Sphere(K + v3((-sg * 0.024 * s, 0.0, 0.008)), 0.028 * s), k=0.03)
        # Unterschenkel (Wade hinten)
        sc.add(
            Loft(
                K,
                A + v3((0, 0, -0.01)),
                [(0.0, cr * 0.95, cr, 0, 0), (0.2, cr, cr * 1.08, 0, 0.012), (0.36, cr * 0.97, cr * 1.04, 0, 0.012), (0.66, cr * 0.75, cr * 0.78, 0, 0.004), (0.9, cr * 0.54, cr * 0.6, 0, 0.002), (1.0, cr * 0.52, cr * 0.64, 0, 0.0)],
                up=(0, -1, 0),
            ),
            k=0.035,
        )
        # Wadenmuskel (zwei Köpfe, innen tiefer), Schienbeinkante, Knöchel
        sc.add(ellipsoid_between(K + v3((sg * 0.02, -0.034, -0.03 * s)), K + v3((sg * 0.014, -0.032, -0.19 * s)), 0.024, 0.024 * m + 0.006, up=(0, -1, 0), bulge=0.45), k=0.022)
        sc.add(ellipsoid_between(K + v3((-sg * 0.02, -0.034, -0.04 * s)), K + v3((-sg * 0.016, -0.03, -0.23 * s)), 0.026, 0.026 * m + 0.006, up=(0, -1, 0), bulge=0.45), k=0.022)
        sc.add(capsule(K + v3((0, 0.034, -0.06 * s)), A + v3((0, 0.022, 0.09)), 0.011 * s), k=0.022)
        sc.add(Sphere(A + v3((sg * 0.022, -0.004, -0.008)), 0.012 * s), k=0.012)
        sc.add(Sphere(A + v3((-sg * 0.02, 0.002, 0.002)), 0.012 * s), k=0.012)
        # Fuß (wird vom Schuh verdeckt, aber für Bone-Heat und Silhouette vorhanden)
        foot = v3((A[0] + sg * 0.004, A[1] + 0.07 * s, sk["sole"] + 0.03))
        sc.add(Ellipsoid(foot, (0.032 * s, 0.095 * s, 0.022)), k=0.03)
        _ = female


def build(sc, sk):
    torso(sc, sk)
    neck(sc, sk)
    arms(sc, sk)
    legs(sc, sk)
    _ = (Func, perlin, capsule, euler)
