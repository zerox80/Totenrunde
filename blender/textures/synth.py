"""Textur-Synthese mit numpy: kachelbares Rauschen (spektral per FFT), Voronoi-Zellen,
Risse, Weichzeichnen, Höhen -> Normalen/Kavität, Schreiben von PNGs.

Alles ist periodisch über die Kachel (Wrap-around), damit die Texturen nahtlos sind.
Läuft in Blenders Python (numpy vorhanden), braucht aber kein bpy.
"""

import struct
import zlib

import numpy as np

# --------------------------------------------------------------------------------------
# Grundlagen
# --------------------------------------------------------------------------------------


class Canvas:
    """Gemeinsame Koordinaten für eine Textur der Größe n×n (Pixel), u,v in [0,1)."""

    def __init__(self, n, seed=0):
        self.n = n
        self.seed = seed
        ax = (np.arange(n, dtype=np.float32) + 0.5) / n
        self.u, self.v = np.meshgrid(ax, ax)  # v = Zeile (oben -> unten in Bildkoordinaten)
        f = np.fft.fftfreq(n) * n
        self.kx, self.ky = np.meshgrid(f, f)
        self.k = np.sqrt(self.kx**2 + self.ky**2).astype(np.float32)
        self._rng = np.random.default_rng(seed)

    def rng(self, salt):
        return np.random.default_rng(self.seed * 7919 + salt)

    # ---------------------------------------------------------------- Rauschen
    def noise(self, freq, salt=0, octaves=5, rough=0.5, lac=2.0, aniso=(1.0, 1.0)):
        """fBm-Rauschen mit Grundfrequenz `freq` (Strukturen pro Kachel), normiert auf 0..1.

        aniso=(ax, ay) streckt die Strukturen (z. B. (1, 6) = senkrechte Streifen)."""
        rng = self.rng(salt + 101)
        white = rng.standard_normal((self.n, self.n)).astype(np.float32)
        W = np.fft.fft2(white)
        kx, ky = self.kx * aniso[0], self.ky * aniso[1]
        k = np.sqrt(kx**2 + ky**2) + 1e-6
        spec = np.zeros_like(k)
        amp = 1.0
        f = freq
        for _ in range(octaves):
            # Gauß-Band um f (in log-Frequenz), Summe ergibt 1/f-artiges Spektrum
            spec += amp * np.exp(-0.5 * (np.log(k / f) / 0.45) ** 2)
            amp *= rough
            f *= lac
            if f > self.n / 2:
                break
        out = np.real(np.fft.ifft2(W * spec)).astype(np.float32)
        return _normalize(out)

    def band(self, freq, salt=0, width=0.35, aniso=(1.0, 1.0)):
        """Einzelnes Frequenzband (Perlin-ähnlich), 0..1."""
        rng = self.rng(salt + 202)
        white = rng.standard_normal((self.n, self.n)).astype(np.float32)
        kx, ky = self.kx * aniso[0], self.ky * aniso[1]
        k = np.sqrt(kx**2 + ky**2) + 1e-6
        spec = np.exp(-0.5 * (np.log(k / freq) / width) ** 2)
        out = np.real(np.fft.ifft2(np.fft.fft2(white) * spec)).astype(np.float32)
        return _normalize(out)

    def white(self, salt=0):
        return self.rng(salt + 303).random((self.n, self.n), dtype=np.float32)

    # ---------------------------------------------------------------- Zellen
    def voronoi(self, cells, salt=0, jitter=1.0, warp=None):
        """Kachelbares Voronoi mit `cells`×`cells` Zellen.

        Liefert dict(f1, f2, edge (=f2-f1), id (0..1 pro Zelle), cx, cy (Zellmittelpunkt))."""
        g = int(cells)
        rng = self.rng(salt + 404)
        px = (np.arange(g)[None, :] + 0.5 + (rng.random((g, g)) - 0.5) * jitter) / g
        py = (np.arange(g)[:, None] + 0.5 + (rng.random((g, g)) - 0.5) * jitter) / g
        cid = rng.random((g, g)).astype(np.float32)
        u, v = self.u, self.v
        if warp is not None:
            u = (u + warp[0]) % 1.0
            v = (v + warp[1]) % 1.0
        ci = np.floor(u * g).astype(np.int32)
        cj = np.floor(v * g).astype(np.int32)
        f1 = np.full(u.shape, 9.0, np.float32)
        f2 = np.full(u.shape, 9.0, np.float32)
        best = np.zeros(u.shape, np.float32)
        bx = np.zeros(u.shape, np.float32)
        by = np.zeros(u.shape, np.float32)
        for dj in (-1, 0, 1):
            for di in (-1, 0, 1):
                ii = ci + di
                jj = cj + dj
                wi = ii % g
                wj = jj % g
                qx = px[wj, wi] + (ii - wi) / g
                qy = py[wj, wi] + (jj - wj) / g
                d = np.sqrt((u - qx) ** 2 + (v - qy) ** 2) * g
                closer = d < f1
                f2 = np.where(closer, f1, np.minimum(f2, d))
                best = np.where(closer, cid[wj, wi], best)
                bx = np.where(closer, qx % 1.0, bx)
                by = np.where(closer, qy % 1.0, by)
                f1 = np.where(closer, d, f1)
        return dict(f1=f1, f2=f2, edge=f2 - f1, id=best, cx=bx, cy=by)

    def warp(self, amount, freq, salt=0):
        """Verschiebungsfeld (du, dv) für Domain-Warping."""
        return (
            (self.noise(freq, salt + 1, octaves=3) - 0.5) * amount,
            (self.noise(freq, salt + 2, octaves=3) - 0.5) * amount,
        )

    def sample(self, img, du, dv):
        """Bild an verschobenen Koordinaten abtasten (bilinear, periodisch)."""
        n = self.n
        x = (self.u + du) * n - 0.5
        y = (self.v + dv) * n - 0.5
        x0 = np.floor(x).astype(np.int32)
        y0 = np.floor(y).astype(np.int32)
        fx = (x - x0)[..., None] if img.ndim == 3 else x - x0
        fy = (y - y0)[..., None] if img.ndim == 3 else y - y0
        x0 %= n
        y0 %= n
        x1 = (x0 + 1) % n
        y1 = (y0 + 1) % n
        a = img[y0, x0] * (1 - fx) + img[y0, x1] * fx
        b = img[y1, x0] * (1 - fx) + img[y1, x1] * fx
        return a * (1 - fy) + b * fy

    # ---------------------------------------------------------------- Filter
    def blur(self, img, radius_px):
        """Gauß-Weichzeichner (periodisch, per FFT). radius = Sigma in Pixeln."""
        if radius_px <= 0:
            return img
        s = radius_px / self.n
        g = np.exp(-2.0 * (np.pi**2) * (s**2) * (self.k**2) / 1.0)
        if img.ndim == 3:
            return np.stack([self.blur(img[..., c], radius_px) for c in range(img.shape[2])], axis=-1)
        return np.real(np.fft.ifft2(np.fft.fft2(img) * g)).astype(np.float32)


def _normalize(x):
    lo, hi = np.percentile(x, 0.5), np.percentile(x, 99.5)
    return np.clip((x - lo) / max(hi - lo, 1e-6), 0.0, 1.0).astype(np.float32)


# --------------------------------------------------------------------------------------
# Hilfsfunktionen
# --------------------------------------------------------------------------------------


def smooth(x, a, b):
    t = np.clip((x - a) / (b - a), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def lerp(a, b, t):
    if isinstance(t, np.ndarray) and (np.ndim(a) == 3 or np.ndim(b) == 3) and t.ndim == 2:
        t = t[..., None]
    return a + (b - a) * t


def hexc(h):
    """'#rrggbb' -> lineares RGB (numpy)."""
    h = h.lstrip("#")
    c = np.array([int(h[i : i + 2], 16) / 255.0 for i in (0, 2, 4)], np.float32)
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4).astype(np.float32)


def ramp(t, stops):
    """Farbverlauf: stops = [(pos, '#hex'), ...] -> (n, n, 3) linear."""
    pos = np.array([p for p, _ in stops], np.float32)
    cols = np.stack([hexc(c) for _, c in stops])
    out = np.empty(t.shape + (3,), np.float32)
    for ch in range(3):
        out[..., ch] = np.interp(t, pos, cols[:, ch])
    return out


def col(h, shape):
    return np.broadcast_to(hexc(h), shape + (3,)).astype(np.float32).copy()


def tint(c, t, color, amount=1.0):
    """Mischt Farbe `color` (hex) mit Gewicht t*amount über c."""
    return lerp(c, hexc(color)[None, None, :], np.clip(t * amount, 0, 1))


def grad(h):
    dx = (np.roll(h, -1, axis=1) - np.roll(h, 1, axis=1)) * 0.5
    dy = (np.roll(h, -1, axis=0) - np.roll(h, 1, axis=0)) * 0.5
    return dx, dy


def normal_from_height(h, strength):
    """Höhen (Meter-ähnliche Einheit) -> Tangentenraum-Normalen (OpenGL, +Y oben)."""
    dx, dy = grad(h)
    # Bildzeile wächst nach unten, UV-v nach oben: dy-Vorzeichen umdrehen
    n = np.dstack([-dx * strength, dy * strength, np.ones_like(h)])
    n /= np.linalg.norm(n, axis=2, keepdims=True)
    return n * 0.5 + 0.5


def cavity(cv, h, radius):
    """Positive Werte in Vertiefungen (h unter lokalem Mittel)."""
    return np.clip((cv.blur(h, radius) - h), 0.0, None)


def linear_to_srgb(x):
    x = np.clip(x, 0.0, 1.0)
    return np.where(x <= 0.0031308, x * 12.92, 1.055 * np.power(x, 1.0 / 2.4) - 0.055)


# --------------------------------------------------------------------------------------
# PNG schreiben (ohne PIL)
# --------------------------------------------------------------------------------------


def write_png(path, img, srgb=False):
    """img: float (n, m, 3|4) in 0..1 (linear, wenn srgb=True wird kodiert) oder (n, m)."""
    a = np.asarray(img, np.float32)
    if a.ndim == 2:
        a = a[..., None]
    if srgb:
        a = a.copy()
        a[..., :3] = linear_to_srgb(a[..., :3])
    a = np.clip(a * 255.0 + 0.5, 0, 255).astype(np.uint8)
    h, w, c = a.shape
    ctype = {1: 0, 3: 2, 4: 6}[c]
    raw = np.concatenate([np.zeros((h, 1), np.uint8), a.reshape(h, w * c)], axis=1).tobytes()

    def chunk(tag, data):
        body = tag + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, ctype, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 6))
    png += chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(png)


def read_png(path):
    """Minimaler PNG-Leser (8 Bit, ohne Interlace) -> float (h, w, c) in 0..1."""
    with open(path, "rb") as f:
        data = f.read()
    pos = 8
    idat = b""
    w = h = ctype = 0
    while pos < len(data):
        ln = struct.unpack(">I", data[pos : pos + 4])[0]
        tag = data[pos + 4 : pos + 8]
        body = data[pos + 8 : pos + 8 + ln]
        if tag == b"IHDR":
            w, h, depth, ctype = struct.unpack(">IIBB", body[:10])
        elif tag == b"IDAT":
            idat += body
        pos += 12 + ln
    c = {0: 1, 2: 3, 4: 2, 6: 4}[ctype]
    raw = np.frombuffer(zlib.decompress(idat), np.uint8).reshape(h, 1 + w * c)
    out = np.zeros((h, w * c), np.int32)
    prev = np.zeros(w * c, np.int32)
    for y in range(h):
        ft = raw[y, 0]
        line = raw[y, 1:].astype(np.int32)
        if ft == 0:
            cur = line
        elif ft == 1:
            cur = line.copy()
            for x in range(c, w * c):
                cur[x] = (cur[x] + cur[x - c]) & 255
        elif ft == 2:
            cur = (line + prev) & 255
        elif ft == 3:
            cur = line.copy()
            for x in range(w * c):
                left = cur[x - c] if x >= c else 0
                cur[x] = (cur[x] + ((left + prev[x]) >> 1)) & 255
        else:
            cur = line.copy()
            for x in range(w * c):
                a_ = cur[x - c] if x >= c else 0
                b_ = prev[x]
                c_ = prev[x - c] if x >= c else 0
                p = a_ + b_ - c_
                pa, pb, pc = abs(p - a_), abs(p - b_), abs(p - c_)
                pr = a_ if (pa <= pb and pa <= pc) else (b_ if pb <= pc else c_)
                cur[x] = (cur[x] + pr) & 255
        out[y] = cur
        prev = cur
    return out.reshape(h, w, c).astype(np.float32) / 255.0
