"""Bauteile für realistische Waffen (Einzelteile mit Fasen; Schnitte nur auf geschlossenen
Einzelteilen) und Griff-Posen für die Ego-Arme."""

import math

from mathutils import Vector

from .kit import F, cut, harden, mb, pose_arm


def solid(name, build, fin=None, bevel=0.0005, cuts=None, angle=30.0):
    """Ein Teil: build(mb, mat) fügt Geometrie hinzu; cuts(mb, mat) optional abziehen."""
    b = mb(name)
    m = F(fin) if fin else None
    build(b, m)
    o = b.to_object(smooth_angle=None)
    if cuts is not None:
        # mehrere Schnitt-Gruppen nacheinander (robuster als ein großer Schneidkörper)
        for fn in (cuts if isinstance(cuts, (list, tuple)) else [cuts]):
            c = mb("_c")
            fn(c, m)
            cut(o, c)
    harden(o, width=bevel, angle=angle)
    return o


def ext(name, profile, width, fin, offset=0.0, bevel=0.0005, cuts=None, plane="YZ"):
    return solid(name, lambda b, m: b.extrude(profile, width, m, offset=offset, plane=plane), fin, bevel, cuts)


def tube(name, y0, y1, r, fin, z=0.0, x=0.0, seg=28, r2=None, bevel=0.0004):
    return solid(name, lambda b, m: b.cyl((x, (y0 + y1) / 2, z), r, y1 - y0, m, axis="Y", seg=seg, r2=r2), fin, bevel)


def lathe(name, prof, fin, center=(0, 0, 0), seg=28, bevel=0.0004):
    """Rotationskörper entlang Y: prof = [(radius, y), ...]."""
    return solid(name, lambda b, m: b.lathe(prof, m, seg=seg, axis="Y", center=center), fin, bevel, angle=40.0)


def box(name, c, s, fin, rot=None, bevel=0.0005):
    return solid(name, lambda b, m: b.box(c, s, m, rot=rot), fin, bevel)


def rail(name, y0, y1, z, fin="alu", w=0.0212):
    """Picatinny-Schiene mit Querrillen (Teilung 10 mm)."""
    def build(b, m):
        b.box((0, (y0 + y1) / 2, z + 0.003), (w * 0.7, y1 - y0, 0.006), m)
        n = int((y1 - y0) / 0.01)
        for i in range(n):
            y = y0 + 0.0025 + i * 0.01
            b.box((0, y + 0.0025, z + 0.0075), (w, 0.0052, 0.004), m)
    return solid(name, build, fin, 0.0003)


def curved_mag(name, top, length, sweep, depth, width, fin, ribs=0, base=True):
    """Gebogenes Stangenmagazin: top = (y, z) vorne oben; sweep = Vorlauf der Spitze (m)."""
    front, back = [], []
    n = 12
    for i in range(n + 1):
        t = i / n
        y = top[0] + sweep * t * t
        z = top[1] - length * t
        # Normale der Kurve (nach hinten)
        dy, dz = 2 * sweep * t, -length
        ln = math.hypot(dy, dz)
        nx, nz = -dz / ln, dy / ln
        front.append((y, z))
        back.append((y - nx * depth, z - nz * depth))
    pts = front + list(reversed(back))

    def build(b, m):
        b.extrude(pts, width, m)
        if base:
            (yb, zb), (yf, zf) = back[-1], front[-1]
            b.extrude([(yb - 0.004, zb + 0.004), (yf + 0.004, zf + 0.004), (yf + 0.004, zf - 0.006), (yb - 0.004, zb - 0.006)], width + 0.004, m)
        for k in range(ribs):
            t = (k + 1) / (ribs + 1)
            i = int(t * n)
            (y1, z1), (y2, z2) = front[i], back[i]
            b.extrude([(y2, z2 + 0.003), (y1, z1 + 0.003), (y1, z1 - 0.003), (y2, z2 - 0.003)], width + 0.003, m)
    return solid(name, build, fin, 0.0005)


# --------------------------------------------------------------------------------------
# Griff-Posen
# --------------------------------------------------------------------------------------


def right_grip(arms, targets, top, bot, depth=0.042, side=0.014, t=0.42, elbow=(0.18, -0.36, -0.27), trigger=(18, 30, 12), inward=-0.12):
    """Rechte Hand an einem Griff (Pistolengriff oder Kolbenhals).

    top/bot: (y, z) der Griff-Mittellinie oben/unten; depth: Tiefe vorn-hinten."""
    gy, gz = bot[0] - top[0], bot[1] - top[1]
    ln = math.hypot(gy, gz)
    gy, gz = gy / ln, gz / ln
    py, pz = -gz, gy
    if py < 0:
        py, pz = -py, -pz
    P = Vector((0.0, top[0] + (bot[0] - top[0]) * t, top[1] + (bot[1] - top[1]) * t))
    K = P + Vector((side, py * (depth / 2 - 0.004), pz * (depth / 2 - 0.004)))
    a = Vector((inward, py, pz)).normalized()
    W = K - a * 0.099
    return pose_arm("R", arms, wrist=W, a=a, n=(-1.0, 0.0, 0.0), elbow=elbow, targets=targets,
                    curls={"index": trigger} if trigger else None)


def left_support(arms, targets, y, zc, hw, hh, elbow=None, tilt=0.35, fwd=(0.15, 0.97, 0.12), x_shift=-0.3):
    """Linke Hand unter einem Handschutz/Vorderschaft (Mitte zc, halbe Breite hw, halbe Höhe hh)."""
    n = Vector((tilt, 0.0, 1.0)).normalized()
    a = Vector(fwd)
    a = (a - n * a.dot(n)).normalized()
    palm = Vector((hw * x_shift, y, zc - hh - 0.004))
    W = palm - a * 0.05 - n * 0.016
    if elbow is None:
        elbow = (-0.27, y - 0.32, zc - 0.36)
    return pose_arm("L", arms, wrist=W, a=a, n=n, elbow=elbow, targets=targets)


def left_vgrip(arms, targets, top, bot, depth=0.032, side=-0.013, t=0.45, elbow=(-0.2, -0.2, -0.3)):
    """Linke Hand an einem senkrechten Vordergriff (Handfläche links am Griff)."""
    gy, gz = bot[0] - top[0], bot[1] - top[1]
    ln = math.hypot(gy, gz)
    gy, gz = gy / ln, gz / ln
    py, pz = -gz, gy
    if py < 0:
        py, pz = -py, -pz
    P = Vector((0.0, top[0] + (bot[0] - top[0]) * t, top[1] + (bot[1] - top[1]) * t))
    K = P + Vector((side, py * (depth / 2 - 0.004), pz * (depth / 2 - 0.004)))
    a = Vector((0.12, py, pz)).normalized()
    W = K - a * 0.099
    return pose_arm("L", arms, wrist=W, a=a, n=(1.0, 0.0, 0.0), elbow=elbow, targets=targets)


def pistol_hands(arms, targets, top, bot, depth=0.044, elbow_r=(0.17, -0.4, -0.3), elbow_l=(-0.22, -0.38, -0.34), trigger=(18, 30, 12)):
    """Beidhändiger Pistolengriff: rechts am Griff, links stützt von unten/links."""
    r = right_grip(arms, targets, top, bot, depth=depth, elbow=elbow_r, trigger=trigger)
    gy, gz = bot[0] - top[0], bot[1] - top[1]
    mid = (top[0] + (bot[0] - top[0]) * 0.6, top[1] + (bot[1] - top[1]) * 0.6)
    W = Vector((-0.056, mid[0] - 0.012, mid[1] - 0.048))
    l = pose_arm("L", arms, wrist=W, a=(0.22, 0.85, 0.45), n=(0.85, 0.0, 0.5), elbow=elbow_l, targets=targets + [r])
    return r, l
