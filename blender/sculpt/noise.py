"""Gradientenrauschen (Perlin) und fBm in numpy, vektorisiert über Punktmengen (N, 3)."""

import numpy as np

_perm = np.random.default_rng(1337).permutation(256).astype(np.int64)
_P = np.concatenate([_perm, _perm, _perm]).astype(np.int32)
_G = np.array(
    [
        [1, 1, 0], [-1, 1, 0], [1, -1, 0], [-1, -1, 0],
        [1, 0, 1], [-1, 0, 1], [1, 0, -1], [-1, 0, -1],
        [0, 1, 1], [0, -1, 1], [0, 1, -1], [0, -1, -1],
    ],
    dtype=np.float64,
)


def _seed_off(seed):
    r = np.random.default_rng(int(seed * 1000) + 7)
    return r.uniform(0, 200, 3)


# Gradienten als Tabellen (16 Einträge, 12 Kantenrichtungen + 4 Wiederholungen)
_GX = np.array([1, -1, 1, -1, 1, -1, 1, -1, 0, 0, 0, 0, 1, 0, -1, 0], dtype=np.float32)
_GY = np.array([1, 1, -1, -1, 0, 0, 0, 0, 1, -1, 1, -1, 1, -1, 1, -1], dtype=np.float32)
_GZ = np.array([0, 0, 0, 0, 1, 1, -1, -1, 1, 1, -1, -1, 0, 1, 0, -1], dtype=np.float32)


def _grad(h, x, y, z):
    h = h & 15
    return _GX[h] * x + _GY[h] * y + _GZ[h] * z


def perlin(P, freq=1.0, seed=0.0):
    """Perlin-Rauschen in [-1, 1] (typisch ±0.7). P: (N, 3), freq: Skalar oder (3,)."""
    p = (np.asarray(P, dtype=np.float64) * freq + _seed_off(seed)).astype(np.float32)
    pi = np.floor(p)
    f = p - pi
    i = pi.astype(np.int32) & 255
    u = f * f * f * (f * (f * 6 - 15) + 10)
    xi, yi, zi = i[:, 0], i[:, 1], i[:, 2]
    fx, fy, fz = f[:, 0], f[:, 1], f[:, 2]
    fx1, fy1, fz1 = fx - 1, fy - 1, fz - 1
    a0 = _P[xi] + yi
    a1 = _P[xi + 1] + yi
    h00, h01 = _P[a0] + zi, _P[a0 + 1] + zi
    h10, h11 = _P[a1] + zi, _P[a1 + 1] + zi
    ux, uy, uz = u[:, 0], u[:, 1], u[:, 2]
    g000 = _grad(_P[h00], fx, fy, fz)
    g100 = _grad(_P[h10], fx1, fy, fz)
    g010 = _grad(_P[h01], fx, fy1, fz)
    g110 = _grad(_P[h11], fx1, fy1, fz)
    g001 = _grad(_P[h00 + 1], fx, fy, fz1)
    g101 = _grad(_P[h10 + 1], fx1, fy, fz1)
    g011 = _grad(_P[h01 + 1], fx, fy1, fz1)
    g111 = _grad(_P[h11 + 1], fx1, fy1, fz1)
    x00 = g000 + ux * (g100 - g000)
    x10 = g010 + ux * (g110 - g010)
    x01 = g001 + ux * (g101 - g001)
    x11 = g011 + ux * (g111 - g011)
    y0 = x00 + uy * (x10 - x00)
    y1 = x01 + uy * (x11 - x01)
    return (y0 + uz * (y1 - y0)).astype(np.float64)


def fbm(P, freq=1.0, octaves=4, gain=0.5, lac=2.0, seed=0.0):
    """Fraktales Rauschen, normiert auf etwa [-1, 1]."""
    total = np.zeros(len(P))
    amp, norm = 1.0, 0.0
    f = np.asarray(freq, dtype=np.float64)
    for o in range(octaves):
        total += amp * perlin(P, f, seed + o * 3.17)
        norm += amp
        amp *= gain
        f = f * lac
    return total / norm * 1.4


def ridged(P, freq=1.0, octaves=3, seed=0.0):
    """Grate (1 - |n|): scharfe Linien für Risse und Falten, in [0, 1]."""
    total = np.zeros(len(P))
    amp, norm = 1.0, 0.0
    f = np.asarray(freq, dtype=np.float64)
    for o in range(octaves):
        total += amp * (1.0 - np.abs(perlin(P, f, seed + o * 5.3)) * 1.4)
        norm += amp
        amp *= 0.5
        f = f * 2.0
    return np.clip(total / norm, 0.0, 1.0)
