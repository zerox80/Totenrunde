"""Kopf als Distanzfeld.

Das Gesicht ist eine Relief-Maske, wie ein Bildhauer sie anlegt:
  * Profilkurve der Mittellinie y_p(z) (Stirn, Nasenwurzel, Oberlippe, Kinn ...)
  * Querkrümmung je Höhe (wie schnell das Gesicht zur Seite nach hinten fällt)
  * Relief-Merkmale: Augenhöhlen, Brauenbogen, Wangenknochen, eingefallene Wangen,
    Nasolabialfalten, Tränenrinne, Kinn, Schläfen
  * Frontumriss (halbe Breite je Höhe) und Unterkieferkante begrenzen die Maske
Darauf sitzen Nase, Lippen, Lider, Ohren als eigene Formen; Schädel und Hinterkopf sind
Ellipsoide. Alle Maße relativ zur Kopfmitte hc, in Metern (Mann; hs skaliert, die Frau hat
weichere Züge). Der Augapfel ist ein eigenes, leuchtendes Objekt.
"""

import math

import numpy as np

from sculpt.sdf import Box, Ellipsoid, Func, RoundCone, Sphere, _catmull, euler, smax, v3

EYE = (0.032, 0.0605, -0.018)
EYE_R = 0.0122
SIDES = (-1.0, 1.0)


class _H:
    """Positionen relativ zur Kopfmitte, gespiegelt für sg = ±1."""

    def __init__(self, sk):
        self.c = sk["hc"]
        self.s = sk["hs"]

    def p(self, x, y, z, sg=1.0):
        return self.c + v3((sg * x, y, z)) * self.s

    def r(self, *vals):
        out = tuple(v * self.s for v in vals)
        return out[0] if len(out) == 1 else out


def eye_centers(sk):
    h = _H(sk)
    return [h.p(*EYE, sg=sg) for sg in (-1.0, 1.0)], EYE_R * sk["hs"]


# --------------------------------------------------------------------------------------
# Relief-Maske
# --------------------------------------------------------------------------------------

# Mittellinie (z, y) ohne Nase und Lippen
PROFILE = [
    (0.095, 0.030), (0.075, 0.060), (0.05, 0.080), (0.025, 0.0905), (0.0, 0.0945), (-0.016, 0.089),
    (-0.04, 0.088), (-0.062, 0.089), (-0.08, 0.094), (-0.092, 0.096), (-0.104, 0.093),
    (-0.117, 0.087), (-0.13, 0.092), (-0.142, 0.085), (-0.155, 0.062),
]
# Querkrümmung c(z): y fällt um c·x² + QUART·x⁴
CURV = [(0.08, 6.5), (0.04, 7.0), (0.0, 7.5), (-0.02, 8.0), (-0.045, 8.5), (-0.07, 11.0), (-0.09, 13.5), (-0.105, 14.5), (-0.125, 16.0), (-0.145, 19.0)]
QUART = 1250.0
# Frontumriss: halbe Gesichtsbreite je Höhe
WIDTH = [(0.09, 0.05), (0.04, 0.06), (0.0, 0.064), (-0.03, 0.068), (-0.06, 0.06), (-0.09, 0.055), (-0.105, 0.052), (-0.12, 0.045), (-0.135, 0.033), (-0.15, 0.021)]


def _g(x, z, cx, cz, sx, sz):
    return np.exp(-(((x - cx) / sx) ** 2) - (((z - cz) / sz) ** 2))


def _line(x, z, a, b, w):
    """Gauß um eine Strecke a->b in der (x, z)-Ebene."""
    ax, az = a
    bx, bz = b
    dx, dz = bx - ax, bz - az
    t = np.clip(((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz), 0.0, 1.0)
    px, pz = ax + t * dx, az + t * dz
    return np.exp(-(((x - px) ** 2 + (z - pz) ** 2) / (w * w)))


def _features(ax, z, female, gaunt):
    """Relief-Versatz (y) aus anatomischen Merkmalen; ax = |x|."""
    ex, _, ez = EYE
    f = np.zeros_like(z)
    # Augenregion: Lidfläche etwas zurück, tiefe Höhle unter dem Brauenbogen und innen
    f -= 0.0055 * _g(ax, z, ex, ez, 0.015, 0.009)
    f -= 0.006 * _g(ax, z, ex - 0.002, ez + 0.012, 0.016, 0.006) * gaunt
    f -= 0.006 * _g(ax, z, 0.017, ez + 0.002, 0.006, 0.008)
    # Brauenbogen
    f += (0.0045 if not female else 0.002) * _g(ax, z, 0.03, 0.002, 0.024, 0.008)
    # Wangenknochen (Jochbein-Vorsprung)
    f += 0.0075 * _g(ax, z, 0.05, -0.04, 0.016, 0.013)
    # eingefallene Wange unter dem Jochbein
    f -= 0.009 * _g(ax, z, 0.048, -0.074, 0.015, 0.02) * gaunt
    # Tränenrinne (Bogen unter dem Auge) und Tränensack darüber
    f -= 0.0022 * _line(ax, z, (0.018, -0.03), (0.042, -0.04), 0.004) * gaunt
    f += 0.0016 * _g(ax, z, 0.033, -0.031, 0.012, 0.004)
    # Nasolabialfalte: Rinne mit Wulst außen
    f -= 0.0022 * _line(ax, z, (0.019, -0.072), (0.031, -0.106), 0.0028)
    f += 0.0026 * _line(ax, z, (0.026, -0.07), (0.037, -0.103), 0.007)
    # Kinn (rundlicher Vorsprung)
    f += 0.003 * _g(ax, z, 0.0, -0.132, 0.016, 0.01)
    # Schläfen
    f -= 0.006 * _g(ax, z, 0.062, 0.012, 0.012, 0.02) * gaunt
    # Mundpartie leicht gewölbt (Zahnbogen)
    f += 0.003 * _g(ax, z, 0.0, -0.095, 0.022, 0.016)
    return f


_ZP = np.asarray(PROFILE)[::-1]
_CV = np.asarray(CURV)[::-1]
_WD = np.asarray(WIDTH)[::-1]


def _yface(x, z, female, gaunt):
    yp = _catmull(_ZP[:, 0], _ZP[:, 1], z)
    c = _catmull(_CV[:, 0], _CV[:, 1], z)
    return yp - c * x * x - QUART * x**4 + _features(np.abs(x), z, female, gaunt)


def face_mask(sk, gaunt=1.0):
    """Distanzfeld der Gesichtsmaske (Weltkoordinaten)."""
    hc, hs = sk["hc"], sk["hs"]
    female = sk["kind"] == "female"

    def fn(P):
        q = (P - hc) / hs
        x, y, z = q[:, 0], q[:, 1], q[:, 2]
        e = 0.0008
        yf = _yface(x, z, female, gaunt)
        gx = (_yface(x + e, z, female, gaunt) - _yface(x - e, z, female, gaunt)) / (2 * e)
        gz = (_yface(x, z + e, female, gaunt) - _yface(x, z - e, female, gaunt)) / (2 * e)
        d_rel = (y - yf) / np.sqrt(1.0 + gx * gx + gz * gz)
        w = _catmull(_WD[:, 0], _WD[:, 1], z)
        d_out = np.abs(x) - w
        zb = -0.152 + 0.45 * np.maximum(0.07 - y, 0.0) + 6.0 * x * x
        d = smax(d_rel, d_out, 0.028)
        d = smax(d, zb - z, 0.01)
        d = smax(d, -0.035 - y, 0.03)
        d = smax(d, z - 0.075, 0.04)
        return d * hs

    lo = hc + v3((-0.09, -0.05, -0.17)) * hs
    hi = hc + v3((0.09, 0.13, 0.11)) * hs
    return Func(fn, lo, hi)


# --------------------------------------------------------------------------------------
# Kopf
# --------------------------------------------------------------------------------------


def build(sc, sk, gaunt=1.0):
    h = _H(sk)
    P, r = h.p, h.r
    female = sk["kind"] == "female"
    ex, ey, ez = EYE
    # --- Schädel, Hinterkopf, Gesichtsmaske ---
    sc.add(Ellipsoid(P(0, -0.016, 0.006), r(0.074, 0.097, 0.095)), k=0.03)
    sc.add(Ellipsoid(P(0, -0.066, -0.02), r(0.06, 0.042, 0.058)), k=0.035)
    sc.add(face_mask(sk, gaunt), k=0.022)
    for sg in SIDES:
        # Unterkieferast vom Kieferwinkel zum Ohr, Kaumuskel
        jaw = P(0.054 if not female else 0.049, -0.006, -0.103, sg)
        sc.add(RoundCone(jaw, P(0.063, -0.012, -0.05, sg), r(0.0115), r(0.0095)), k=0.02)
        sc.add(Ellipsoid(P(0.052, 0.0, -0.078, sg), r(0.011, 0.022, 0.026)), k=0.025)
    # --- Augen: Lidhülle um den Augapfel, Lidspalt, Lidfalte ---
    for sg in SIDES:
        c = P(ex, ey, ez, sg)
        sc.add(Sphere(c, r(EYE_R + 0.0024)), k=0.0045)
        sc.sub(
            Ellipsoid(P(ex, ey + EYE_R + 0.0026, ez + 0.0006, sg), r(0.0128, 0.0068, 0.0041), euler(0, -sg * 7, 0)),
            k=0.0012,
            tag="mouth",
        )
        sc.sub(RoundCone(P(ex - 0.011, 0.077, ez + 0.0095, sg), P(ex + 0.012, 0.072, ez + 0.0088, sg), r(0.0012), r(0.001)), k=0.003)
    # --- Nase ---
    sc.add(RoundCone(P(0, 0.084, -0.016), P(0, 0.108, -0.064), r(0.0062), r(0.0082)), k=0.009)
    sc.add(Ellipsoid(P(0, 0.097, -0.056), r(0.0098, 0.011, 0.016)), k=0.008)
    sc.add(Sphere(P(0, 0.1055, -0.0695), r(0.0098) if not female else r(0.0088)), k=0.006)
    for sg in SIDES:
        sc.add(Sphere(P(0.0128, 0.0945, -0.0738, sg), r(0.0072)), k=0.006)
        sc.sub(Ellipsoid(P(0.0063, 0.101, -0.0815, sg), r(0.0033, 0.0055, 0.003)), k=0.002, tag="mouth")
        sc.sub(RoundCone(P(0.0185, 0.093, -0.069, sg), P(0.0195, 0.095, -0.078, sg), r(0.0013), r(0.0012)), k=0.003)
    sc.add(Ellipsoid(P(0, 0.096, -0.0795), r(0.0045, 0.006, 0.004)), k=0.004)  # Columella
    # --- Mund: Höhle, Zähne, Lippen, Spalt ---
    sc.sub(Ellipsoid(P(0, 0.064, -0.1), r(0.026, 0.03, 0.0095)), k=0.0, tag="mouth")
    teeth(sc, h)
    lip_u = 0.0052 if not female else 0.0058
    lip_l = 0.0062 if not female else 0.0072
    for sg in SIDES:
        sc.add(RoundCone(P(0, 0.0985, -0.0952), P(0.0115, 0.0955, -0.0957, sg), r(lip_u), r(lip_u * 0.92)), k=0.003)
        sc.add(RoundCone(P(0.0115, 0.0955, -0.0957, sg), P(0.0245, 0.085, -0.0995, sg), r(lip_u * 0.92), r(0.0024)), k=0.003)
        sc.add(RoundCone(P(0, 0.096, -0.1055), P(0.012, 0.0935, -0.1048, sg), r(lip_l), r(lip_l * 0.9)), k=0.003)
        sc.add(RoundCone(P(0.012, 0.0935, -0.1048, sg), P(0.0225, 0.0855, -0.101, sg), r(lip_l * 0.9), r(0.0024)), k=0.003)
        sc.add(RoundCone(P(0.0045, 0.0975, -0.081, sg), P(0.005, 0.1005, -0.0905, sg), r(0.0018), r(0.0022)), k=0.003)
        sc.sub(Sphere(P(0.0255, 0.0835, -0.1, sg), r(0.0022)), k=0.003)
    sc.sub(Ellipsoid(P(0, 0.098, -0.1004), r(0.0215, 0.016, 0.0018)), k=0.0012, tag="mouth")
    # --- Ohren ---
    for sg in SIDES:
        R = euler(-12, sg * 8, sg * 27)
        sc.add(Ellipsoid(P(0.077, -0.014, -0.035, sg), r(0.0085, 0.021, 0.031), R), k=0.007)
        sc.add(Ellipsoid(P(0.078, -0.009, -0.061, sg), r(0.0052, 0.0085, 0.0095)), k=0.004)
        sc.add(Sphere(P(0.0775, 0.0, -0.037, sg), r(0.004)), k=0.003)
        sc.sub(Ellipsoid(P(0.0845, -0.008, -0.04, sg), r(0.0058, 0.011, 0.014), R), k=0.004)
        sc.sub(Ellipsoid(P(0.085, -0.017, -0.024, sg), r(0.0036, 0.012, 0.017), R), k=0.003)
        sc.sub(Sphere(P(0.0795, -0.004, -0.04, sg), r(0.004)), k=0.002, tag="mouth")


def teeth(sc, h):
    """Obere und untere Zahnreihe entlang eines Bogens (harte Vereinigung)."""
    P, r = h.p, h.r
    for row, zc in ((0, -0.0982), (1, -0.1032)):
        for x, wd in ((0.0042, 0.0041), (0.0122, 0.0036), (0.0188, 0.0034), (0.0236, 0.0042), (0.0268, 0.0046)):
            for sg in SIDES:
                y = 0.0885 - (x / 0.026) ** 2 * 0.0145 - row * 0.0015
                ang = math.degrees(math.atan(2 * x / 0.026**2 * 0.0145))
                R = euler(0, 0, -sg * ang)
                hgt = 0.0042 if x < 0.02 else 0.0034
                sc.add(Box(P(x, y, zc, sg), r(wd * 0.5, 0.0022, hgt * 0.5), r(0.0009), R), k=0.0, tag="teeth")
