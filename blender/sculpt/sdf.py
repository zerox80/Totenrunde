"""Distanzfeld-Grundformen und das Sculpt-Objekt.

Alle Formen werten numpy-Punktmengen P (N, 3) aus und liefern den (näherungsweisen)
vorzeichenbehafteten Abstand: < 0 innen, > 0 außen. Jede Form kennt ihre Hüllbox, damit
das Sculpt-Objekt sie nur dort auswertet, wo sie etwas bewirken kann.

Operationen werden der Reihe nach angewendet:
  add(form, k)          weiche Vereinigung (k = Verschmelzungsradius in Metern)
  sub(form, k)          weiches Abziehen (Höhlen, Wunden, Augenhöhlen)
  garment(...)          Kleidung: Aufblähen der bisherigen Form in einer Region
  disp(fn, box)         Verschiebung der Oberfläche (Rillen, Falten)
Jede Operation kann ein Material-Etikett (tag) setzen, das an der Oberfläche gewinnt.
"""

import math

import numpy as np

BIG = 1.0e3
# Kleidung und Verschiebungen werden nur bis zu diesem Abstand von der Fläche ausgewertet
NEAR = 0.045


# --------------------------------------------------------------------------------------
# Glatte Operatoren
# --------------------------------------------------------------------------------------


def smin(a, b, k):
    if k <= 0.0:
        return np.minimum(a, b)
    h = np.maximum(k - np.abs(a - b), 0.0) / k
    return np.minimum(a, b) - h * h * k * 0.25


def smax(a, b, k):
    return -smin(-a, -b, k)


def smoothstep(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def v3(x):
    return np.asarray(x, dtype=np.float64)


def norm(v):
    v = v3(v)
    return v / max(np.linalg.norm(v), 1e-12)


def frame(axis, up_hint):
    """Orthonormale Basis (u, v, w) mit w = axis, v möglichst nahe up_hint."""
    w = norm(axis)
    hint = v3(up_hint)
    v = hint - w * np.dot(hint, w)
    if np.linalg.norm(v) < 1e-6:
        alt = np.array([1.0, 0, 0]) if abs(w[0]) < 0.9 else np.array([0, 1.0, 0])
        v = alt - w * np.dot(alt, w)
    v = norm(v)
    u = np.cross(v, w)
    return u, v, w


def rot_axis(axis, angle):
    """Rotationsmatrix (Rodrigues)."""
    a = norm(axis)
    c, s = math.cos(angle), math.sin(angle)
    x, y, z = a
    return np.array(
        [
            [c + x * x * (1 - c), x * y * (1 - c) - z * s, x * z * (1 - c) + y * s],
            [y * x * (1 - c) + z * s, c + y * y * (1 - c), y * z * (1 - c) - x * s],
            [z * x * (1 - c) - y * s, z * y * (1 - c) + x * s, c + z * z * (1 - c)],
        ]
    )


def euler(rx=0.0, ry=0.0, rz=0.0):
    """Rotationsmatrix aus Euler-Winkeln in Grad (Reihenfolge X, dann Y, dann Z)."""
    return rot_axis((0, 0, 1), math.radians(rz)) @ rot_axis((0, 1, 0), math.radians(ry)) @ rot_axis(
        (1, 0, 0), math.radians(rx)
    )


# --------------------------------------------------------------------------------------
# Grundformen
# --------------------------------------------------------------------------------------


class Shape:
    """Basisklasse: __call__(P) -> Abstände, lo/hi = Hüllbox."""

    lo = None
    hi = None

    def __call__(self, P):
        raise NotImplementedError


class Sphere(Shape):
    def __init__(self, c, r):
        self.c, self.r = v3(c), float(r)
        self.lo, self.hi = self.c - r, self.c + r

    def __call__(self, P):
        return np.linalg.norm(P - self.c, axis=1) - self.r


class Ellipsoid(Shape):
    """Ellipsoid mit Halbachsen radii entlang der Spalten von R (Standard: Weltachsen)."""

    def __init__(self, c, radii, R=None):
        self.c = v3(c)
        self.r = v3(radii)
        self.R = np.eye(3) if R is None else v3(R)
        ext = np.abs(self.R) @ self.r
        self.lo, self.hi = self.c - ext, self.c + ext

    def __call__(self, P):
        q = (P - self.c) @ self.R
        k0 = np.linalg.norm(q / self.r, axis=1)
        k1 = np.linalg.norm(q / (self.r * self.r), axis=1)
        return np.where(k1 < 1e-9, -float(self.r.min()), k0 * (k0 - 1.0) / np.maximum(k1, 1e-9))


def ellipsoid_between(a, b, width, depth, up=(0, 1, 0), bulge=0.5):
    """Muskelbauch: Ellipsoid entlang a->b; width quer (u), depth in Richtung up (v).
    bulge verschiebt die Mitte entlang der Achse (0.5 = mittig)."""
    a, b = v3(a), v3(b)
    u, v, w = frame(b - a, up)
    half = np.linalg.norm(b - a) / 2
    c = a + (b - a) * bulge
    return Ellipsoid(c, (width, depth, half), np.stack([u, v, w], axis=1))


class RoundCone(Shape):
    """Kegelstumpf mit Kugelenden (Radius ra bei a, rb bei b) – exakte SDF (I. Quilez)."""

    def __init__(self, a, b, ra, rb):
        self.a, self.b = v3(a), v3(b)
        self.ra, self.rb = float(ra), float(rb)
        r = max(ra, rb)
        self.lo = np.minimum(self.a, self.b) - r
        self.hi = np.maximum(self.a, self.b) + r

    def __call__(self, P):
        a, b, r1, r2 = self.a, self.b, self.ra, self.rb
        ba = b - a
        l2 = float(np.dot(ba, ba))
        if l2 < 1e-12:
            return np.linalg.norm(P - a, axis=1) - max(r1, r2)
        rr = r1 - r2
        a2 = l2 - rr * rr
        il2 = 1.0 / l2
        pa = P - a
        y = pa @ ba
        z = y - l2
        xv = pa * l2 - np.outer(y, ba)
        x2 = np.einsum("ij,ij->i", xv, xv)
        y2 = y * y * l2
        z2 = z * z * l2
        k = math.copysign(1.0, rr) * rr * rr * x2
        d_mid = (np.sqrt(np.maximum(x2 * a2 * il2, 0.0)) + y * rr) * il2 - r1
        d_b = np.sqrt(x2 + z2) * il2 - r2
        d_a = np.sqrt(x2 + y2) * il2 - r1
        out = np.where(np.sign(y) * a2 * y2 < k, d_a, d_mid)
        out = np.where(np.sign(z) * a2 * z2 > k, d_b, out)
        return out


def capsule(a, b, r):
    return RoundCone(a, b, r, r)


def _catmull(ts, vs, t):
    """Catmull-Rom-Interpolation (ts aufsteigend, vs Werte) an den Stellen t (geklemmt)."""
    ts = np.asarray(ts, dtype=np.float64)
    vs = np.asarray(vs, dtype=np.float64)
    t = np.clip(t, ts[0], ts[-1])
    i = np.clip(np.searchsorted(ts, t, side="right") - 1, 0, len(ts) - 2)
    t0, t1 = ts[i], ts[i + 1]
    u = (t - t0) / np.maximum(t1 - t0, 1e-9)
    p1, p2 = vs[i], vs[i + 1]
    p0 = vs[np.maximum(i - 1, 0)]
    p3 = vs[np.minimum(i + 2, len(vs) - 1)]
    # Tangenten (nicht-uniform, auf Intervall skaliert)
    tm0 = ts[np.maximum(i - 1, 0)]
    tp3 = ts[np.minimum(i + 2, len(ts) - 1)]
    m1 = (p2 - p0) / np.maximum(t1 - tm0, 1e-9) * (t1 - t0)
    m2 = (p3 - p1) / np.maximum(tp3 - t0, 1e-9) * (t1 - t0)
    u2, u3 = u * u, u * u * u
    return (2 * u3 - 3 * u2 + 1) * p1 + (u3 - 2 * u2 + u) * m1 + (-2 * u3 + 3 * u2) * p2 + (u3 - u2) * m2


class Loft(Shape):
    """Körper entlang einer Achse a->b mit elliptischen Querschnitten.

    keys: Liste (t, rx, ry, cx, cy) – t in [0, 1] entlang der Achse, rx/ry Halbachsen
    (rx entlang u, ry entlang v), cx/cy Versatz der Mitte. u/v aus frame(axis, up).
    Die Enden sind abgerundet (Halbkugel mit dem Endradius)."""

    def __init__(self, a, b, keys, up=(0, 1, 0)):
        self.a, self.b = v3(a), v3(b)
        self.u, self.v, self.w = frame(self.b - self.a, up)
        self.L = float(np.linalg.norm(self.b - self.a))
        k = np.asarray(keys, dtype=np.float64)
        self.kt = k[:, 0]
        self.krx, self.kry, self.kcx, self.kcy = k[:, 1], k[:, 2], k[:, 3], k[:, 4]
        rmax = float(np.max(np.maximum(k[:, 1], k[:, 2]) + np.abs(k[:, 3]) + np.abs(k[:, 4])))
        self.lo = np.minimum(self.a, self.b) - rmax
        self.hi = np.maximum(self.a, self.b) + rmax

    def __call__(self, P):
        q = P - self.a
        s = q @ self.w
        t = s / self.L
        tc = np.clip(t, self.kt[0], self.kt[-1])
        rx = np.maximum(_catmull(self.kt, self.krx, tc), 1e-4)
        ry = np.maximum(_catmull(self.kt, self.kry, tc), 1e-4)
        cx = _catmull(self.kt, self.kcx, tc)
        cy = _catmull(self.kt, self.kcy, tc)
        x = q @ self.u - cx
        y = q @ self.v - cy
        k0 = np.sqrt((x / rx) ** 2 + (y / ry) ** 2)
        k1 = np.sqrt((x / (rx * rx)) ** 2 + (y / (ry * ry)) ** 2)
        d2 = np.where(k1 < 1e-9, -np.minimum(rx, ry), k0 * (k0 - 1.0) / np.maximum(k1, 1e-9))
        # über die Enden hinaus: abgerundet
        over = np.maximum(self.kt[0] * self.L - s, 0.0) + np.maximum(s - self.kt[-1] * self.L, 0.0)
        inside = np.minimum(d2, 0.0)
        return np.where(over > 0.0, np.sqrt(np.maximum(d2, 0.0) ** 2 + over * over) + inside, d2)


class Box(Shape):
    """Abgerundete Box: halbe Kantenlängen half, Rundung rad, Orientierung R (Spalten = Achsen)."""

    def __init__(self, c, half, rad=0.0, R=None):
        self.c = v3(c)
        self.h = v3(half)
        self.rad = float(rad)
        self.R = np.eye(3) if R is None else v3(R)
        ext = np.abs(self.R) @ (self.h + self.rad)
        self.lo, self.hi = self.c - ext, self.c + ext

    def __call__(self, P):
        q = np.abs((P - self.c) @ self.R) - self.h
        outside = np.linalg.norm(np.maximum(q, 0.0), axis=1)
        inside = np.minimum(np.max(q, axis=1), 0.0)
        return outside + inside - self.rad


class Plane(Shape):
    """Halbraum: innen, wo (P - p0)·n < 0. Hüllbox wird vom Aufrufer begrenzt."""

    def __init__(self, p0, n, lo=(-BIG, -BIG, -BIG), hi=(BIG, BIG, BIG)):
        self.p0, self.n = v3(p0), norm(n)
        self.lo, self.hi = v3(lo), v3(hi)

    def __call__(self, P):
        return (P - self.p0) @ self.n


class Func(Shape):
    """Beliebige Funktion fn(P) mit vorgegebener Hüllbox."""

    def __init__(self, fn, lo, hi):
        self.fn = fn
        self.lo, self.hi = v3(lo), v3(hi)

    def __call__(self, P):
        return self.fn(P)


def chain(points, radii):
    """Liste von Rundkegeln entlang eines Polygonzugs (z. B. Fingerglieder)."""
    return [RoundCone(points[i], points[i + 1], radii[i], radii[i + 1]) for i in range(len(points) - 1)]


# --------------------------------------------------------------------------------------
# Sculpt: geordnete Liste von Operationen
# --------------------------------------------------------------------------------------


class Op:
    __slots__ = ("kind", "shape", "k", "tag", "lo", "hi", "extra")

    def __init__(self, kind, shape, k, tag, pad, extra=None):
        self.kind, self.shape, self.k, self.tag, self.extra = kind, shape, k, tag, extra
        m = k + pad
        self.lo = shape.lo - m
        self.hi = shape.hi + m


class Sculpt:
    """Sammlung von Operationen; eval() wendet sie der Reihe nach an.

    pad: zusätzlicher Rand der Hüllboxen (muss größer als die Auflösung sein)."""

    def __init__(self, pad=0.01):
        self.ops = []
        self.pad = pad
        self.tag_names = ["skin"]

    def tag_id(self, name):
        if name is None:
            return 0
        if name not in self.tag_names:
            self.tag_names.append(name)
        return self.tag_names.index(name)

    # -- Operationen --------------------------------------------------------------------
    def add(self, shape, k=0.0, tag=None):
        if isinstance(shape, (list, tuple)):
            for s in shape:
                self.add(s, k, tag)
            return
        self.ops.append(Op("add", shape, k, self.tag_id(tag), self.pad))

    def sub(self, shape, k=0.0, tag=None):
        """Abziehen; tag (z. B. 'flesh') färbt die neu entstandene Innenfläche."""
        if isinstance(shape, (list, tuple)):
            for s in shape:
                self.sub(s, k, tag)
            return
        self.ops.append(Op("sub", shape, k, self.tag_id(tag) if tag else -1, self.pad))

    def garment(self, region, surface, lo, hi, tag, hem=0.003):
        """Kleidung: surface(P, d, ctx) liefert die Außenfläche des Stoffs aus dem bisherigen
        Abstand d (z. B. d - Versatz), wirksam nur, wo region(P, ctx) < 0. hem rundet den Saum.
        In ctx kann region Zwischenwerte (je Punkt) für surface ablegen. lo/hi begrenzen
        die Auswertung."""
        shape = Func(region, lo, hi)
        self.ops.append(Op("garment", shape, hem, self.tag_id(tag), self.pad, extra=surface))

    def disp(self, fn, lo, hi):
        """Oberflächen-Verschiebung: d += fn(P) (positiv = nach innen)."""
        self.ops.append(Op("disp", Func(fn, lo, hi), 0.0, 0, self.pad))

    def bounds(self):
        lo = np.min([o.shape.lo for o in self.ops if o.kind == "add"], axis=0)
        hi = np.max([o.shape.hi for o in self.ops if o.kind == "add"], axis=0)
        return lo, hi

    # -- Auswertung ---------------------------------------------------------------------
    def eval(self, P, tags=False):
        """Abstände (und optional Etiketten) für P (N, 3)."""
        P = np.asarray(P, dtype=np.float64)
        n = len(P)
        d = np.full(n, BIG)
        tg = np.zeros(n, dtype=np.int16) if tags else None
        if n == 0:
            return (d, tg) if tags else d
        rlo, rhi = P.min(axis=0), P.max(axis=0)
        for op in self.ops:
            if np.any(op.hi < rlo) or np.any(op.lo > rhi):
                continue
            m = np.all((P >= op.lo) & (P <= op.hi), axis=1)
            idx = np.nonzero(m)[0]
            if len(idx) == 0:
                continue
            Q = P[idx]
            cur = d[idx]
            if op.kind == "add":
                s = op.shape(Q)
                if tags:
                    win = s < cur
                    tg[idx[win]] = op.tag
                d[idx] = smin(cur, s, op.k)
            elif op.kind == "sub":
                s = -op.shape(Q)
                if tags and op.tag >= 0:
                    win = s > cur
                    tg[idx[win]] = op.tag
                d[idx] = smax(cur, s, op.k)
            elif op.kind == "garment":
                # Kleidung wirkt nur nahe der Oberfläche (Versatz << NEAR)
                near = np.abs(cur) < NEAR
                if not near.all():
                    idx = idx[near]
                    Q = Q[near]
                    cur = cur[near]
                    if len(idx) == 0:
                        continue
                # erst die Region (billig), dann die Stoff-Fläche nur dort, wo Stoff liegen kann
                ctx = {}
                reg = op.shape.fn(Q, ctx)
                m = reg < cur + op.k
                if not m.any():
                    continue
                idx, Q, cur, reg = idx[m], Q[m], cur[m], reg[m]
                ctx = {key: val[m] for key, val in ctx.items()}
                g = smax(op.extra(Q, cur, ctx), reg, op.k)
                if tags:
                    win = g < cur
                    tg[idx[win]] = op.tag
                d[idx] = np.minimum(cur, g)
            elif op.kind == "disp":
                near = np.abs(cur) < NEAR
                idx = idx[near]
                if len(idx):
                    d[idx] = cur[near] + op.shape(Q[near])
        return (d, tg) if tags else d

    def eval_chunked(self, P, chunk=400000, tags=False):
        out = np.empty(len(P))
        tg = np.empty(len(P), dtype=np.int16) if tags else None
        for i in range(0, len(P), chunk):
            r = self.eval(P[i : i + chunk], tags=tags)
            if tags:
                out[i : i + chunk], tg[i : i + chunk] = r
            else:
                out[i : i + chunk] = r
        return (out, tg) if tags else out

    def gradient(self, P, eps=0.0005):
        """Normierter Gradient (Flächennormale) per zentraler Differenzen."""
        g = np.zeros_like(P)
        for a in range(3):
            e = np.zeros(3)
            e[a] = eps
            g[:, a] = self.eval_chunked(P + e) - self.eval_chunked(P - e)
        return g / np.maximum(np.linalg.norm(g, axis=1, keepdims=True), 1e-9)
