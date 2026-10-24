"""HUD-Grafiken (numpy, Schrift aus Blender-Textobjekten gerastert) nach assets/ui:

  * chalk_0..9.png       – Ziffern in Kreide (weiß, im Spiel rot eingefärbt)
  * chalk_tally_1..5.png – Strichliste für Runde 1–5
  * blood_overlay.png    – Blutspritzer am Bildschirmrand (Schaden)
  * points_bar.png       – dunkler, abgeschrägter Balken hinter den Punkten
  * grenade.png          – Granaten-Symbol

Aufruf: tools/blender.sh blender/ui/hud_textures.py
"""

import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import bmesh  # noqa: E402
import bpy  # noqa: E402
import numpy as np  # noqa: E402

from lib import ASSETS, UI_DIR, reset_scene  # noqa: E402
from textures.synth import Canvas, smooth, write_png  # noqa: E402

FONT = os.path.join(ASSETS, "fonts", "Montserrat-Black.otf")


def text_polys(txt):
    """Dreiecke der Schrift (2D, Einheiten der Schriftgröße 1)."""
    cu = bpy.data.curves.new("t", "FONT")
    cu.body = txt
    cu.size = 1.0
    cu.font = bpy.data.fonts.load(FONT, check_existing=True)
    cu.align_x = "CENTER"
    cu.align_y = "CENTER"
    ob = bpy.data.objects.new("t", cu)
    bpy.context.scene.collection.objects.link(ob)
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    tris = [[(v.co.x, v.co.y) for v in f.verts] for f in bm.faces]
    bm.free()
    bpy.data.objects.remove(ob)
    bpy.data.curves.remove(cu)
    bpy.data.meshes.remove(me)
    return np.array(tris, np.float64)


def raster(tris, w, h, scale, ox, oy, ss=3):
    """Rastert Dreiecke (mit Überabtastung) in eine Maske w×h."""
    W, Hh = w * ss, h * ss
    img = np.zeros((Hh, W), np.float32)
    for t in tris:
        p = np.stack([(t[:, 0] * scale + ox) * ss, (oy - t[:, 1] * scale) * ss], axis=1)
        x0, y0 = np.floor(p.min(0)).astype(int)
        x1, y1 = np.ceil(p.max(0)).astype(int)
        x0, y0 = max(x0, 0), max(y0, 0)
        x1, y1 = min(x1, W - 1), min(y1, Hh - 1)
        if x1 < x0 or y1 < y0:
            continue
        xs, ys = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
        (ax, ay), (bx, by), (cx, cy) = p
        d = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy)
        if abs(d) < 1e-9:
            continue
        l1 = ((by - cy) * (xs - cx) + (cx - bx) * (ys - cy)) / d
        l2 = ((cy - ay) * (xs - cx) + (ax - cx) * (ys - cy)) / d
        l3 = 1 - l1 - l2
        inside = (l1 >= 0) & (l2 >= 0) & (l3 >= 0)
        img[y0:y1 + 1, x0:x1 + 1] = np.maximum(img[y0:y1 + 1, x0:x1 + 1], inside.astype(np.float32))
    return img.reshape(h, ss, w, ss).mean(axis=(1, 3))


def chalkify(mask, seed, n=None):
    """Kreide: ausgefranste Ränder, Körnung, Lücken, leichte Schlieren."""
    h, w = mask.shape
    s = max(h, w)
    cv = Canvas(s, seed=seed)
    m = np.zeros((s, s), np.float32)
    m[:h, :w] = mask
    m = cv.sample(m, *cv.warp(0.012, 40, seed + 1))
    m = cv.blur(m, 1.2)
    grain = cv.noise(160, seed + 2, octaves=2)
    streak = cv.noise(25, seed + 3, octaves=3, aniso=(0.25, 1.0))
    holes = smooth(cv.noise(60, seed + 4, octaves=3), 0.62, 0.75)
    edge = smooth(m, 0.25 + grain * 0.4, 0.6 + grain * 0.3)
    a = edge * (0.75 + 0.25 * streak) * (1.0 - holes * 0.7)
    a = np.clip(a * (0.85 + 0.3 * grain), 0, 1)
    return a[:h, :w]


def save_white(name, alpha):
    h, w = alpha.shape
    rgba = np.ones((h, w, 4), np.float32)
    rgba[..., 3] = alpha
    write_png(os.path.join(UI_DIR, name + ".png"), rgba)


def digits():
    for d in range(10):
        tris = text_polys(str(d))
        mask = raster(tris, 120, 150, 150, 60, 80)
        save_white(f"chalk_{d}", chalkify(mask, 100 + d))


def stroke(img, x0, y0, x1, y1, width, rng):
    h, w = img.shape
    ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
    dx, dy = x1 - x0, y1 - y0
    ln2 = dx * dx + dy * dy
    t = np.clip(((xs - x0) * dx + (ys - y0) * dy) / ln2, 0, 1)
    px, py = x0 + t * dx, y0 + t * dy
    d = np.sqrt((xs - px) ** 2 + (ys - py) ** 2)
    # Strich wird zum Ende dünner/druckärmer
    wv = width * (0.85 + 0.25 * np.sin(t * 3.0 + rng.uniform(0, 3)))
    return np.maximum(img, smooth(wv - d, -1.0, 1.0))


def tallies():
    rng = np.random.default_rng(7)
    W, H = 200, 150
    for n in range(1, 6):
        img = np.zeros((H, W), np.float32)
        for i in range(min(n, 4)):
            x = 34 + i * 34 + rng.uniform(-3, 3)
            img = stroke(img, x + rng.uniform(-4, 4), 14 + rng.uniform(-3, 3), x + rng.uniform(-6, 2), 136 + rng.uniform(-3, 3), 7.5, rng)
        if n == 5:
            img = stroke(img, 14, 112, 186, 34, 7.5, rng)
        save_white(f"chalk_tally_{n}", chalkify(img, 200 + n))


def blood_overlay():
    w, h = 1024, 576
    cv = Canvas(1024, seed=33)
    u = cv.u[:h, :w] * 1.0
    v = cv.v[:h, :w] * (1024 / 576)
    v = (np.arange(h)[:, None] + 0.5) / h * np.ones((1, w))
    u = (np.arange(w)[None, :] + 0.5) / w * np.ones((h, 1))
    ex = np.minimum(u, 1 - u) * 1.6
    ey = np.minimum(v, 1 - v)
    edge = np.minimum(ex, ey)
    n1 = cv.noise(6, 34, octaves=5)[:h, :w]
    n2 = cv.noise(30, 35, octaves=3)[:h, :w]
    base = smooth(0.15 - edge + (n1 - 0.5) * 0.2, 0.0, 0.09)
    # Spritzer
    rng = np.random.default_rng(36)
    spl = np.zeros((h, w), np.float32)
    ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
    for _ in range(140):
        side = rng.integers(4)
        if side == 0:
            cx, cy = rng.uniform(0, w), rng.uniform(0, h * 0.25)
        elif side == 1:
            cx, cy = rng.uniform(0, w), rng.uniform(h * 0.75, h)
        elif side == 2:
            cx, cy = rng.uniform(0, w * 0.2), rng.uniform(0, h)
        else:
            cx, cy = rng.uniform(w * 0.8, w), rng.uniform(0, h)
        r = rng.uniform(2, 14)
        d = np.sqrt((xs - cx) ** 2 + (ys - cy) ** 2)
        spl = np.maximum(spl, smooth(r - d, -1.5, 1.5))
    # Laufspuren von oben
    drips = np.zeros((h, w), np.float32)
    for _ in range(26):
        cx = rng.uniform(0, w)
        ln = rng.uniform(40, 220)
        wd = rng.uniform(2.0, 6.0)
        body = smooth(wd - np.abs(xs - cx), -1, 1) * (ys < ln)
        tip = smooth(wd * 1.4 - np.sqrt((xs - cx) ** 2 + ((ys - ln) * 0.8) ** 2), -1, 1)
        drips = np.maximum(drips, np.maximum(body, tip) * smooth(ln + 10 - ys, 0, 30))
    a = np.clip(np.maximum(base, np.maximum(spl * 0.95, drips * 0.9)), 0, 1)
    a = a * (0.8 + 0.2 * n2)
    col = np.zeros((h, w, 4), np.float32)
    dark = smooth(a, 0.5, 1.0)
    col[..., 0] = 0.32 - dark * 0.16 + n2 * 0.05
    col[..., 1] = 0.01
    col[..., 2] = 0.01
    col[..., 3] = a
    write_png(os.path.join(UI_DIR, "blood_overlay.png"), col, srgb=True)


def points_bar():
    w, h = 320, 64
    cv = Canvas(512, seed=41)
    u = (np.arange(w)[None, :] + 0.5) / w * np.ones((h, 1))
    v = (np.arange(h)[:, None] + 0.5) / h * np.ones((1, w))
    # Parallelogramm (rechts abgeschrägt), nach links ausblendend
    shape = smooth(u - (0.0 + v * 0.0), 0.0, 0.02) * smooth((1.0 - v * 0.12) - u, 0.0, 0.015)
    shape *= smooth(v, 0.04, 0.12) * smooth(1 - v, 0.04, 0.12)
    fade = 0.7 + 0.3 * smooth(u, 0.0, 0.4)
    grunge = cv.noise(20, 42, octaves=4)[:h, :w]
    a = shape * fade * (0.75 + 0.25 * grunge)
    col = np.zeros((h, w, 4), np.float32)
    col[..., 0] = 0.02 + grunge * 0.02
    col[..., 1] = 0.02 + grunge * 0.02
    col[..., 2] = 0.025 + grunge * 0.02
    col[..., 3] = a * 0.9
    # heller Akzentstrich links
    accent = smooth(0.018 - u, 0.0, 0.004) * shape
    col[..., :3] = col[..., :3] * (1 - accent[..., None]) + np.array([0.85, 0.8, 0.7]) * accent[..., None]
    col[..., 3] = np.maximum(col[..., 3], accent)
    write_png(os.path.join(UI_DIR, "points_bar.png"), col, srgb=True)


def grenade():
    w, h = 64, 64
    ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
    body = smooth(1.0 - np.sqrt(((xs - 30) / 17) ** 2 + ((ys - 38) / 21) ** 2), 0.0, 0.08)
    top = smooth(1.0 - np.maximum(np.abs(xs - 30) / 8, np.abs(ys - 14) / 5), 0.0, 0.15)
    lever = smooth(1.0 - np.maximum(np.abs(xs - 43) / 4, np.abs(ys - 30) / 14), 0.0, 0.2)
    ring = smooth(2.2 - np.abs(np.sqrt((xs - 18) ** 2 + (ys - 13) ** 2) - 6), 0.0, 1.0)
    a = np.clip(np.maximum.reduce([body, top, lever, ring]), 0, 1)
    # Rillen
    grooves = (np.sin(ys * 0.9) > 0.6) * body * 0.35
    rgba = np.ones((h, w, 4), np.float32)
    rgba[..., :3] -= grooves[..., None]
    rgba[..., 3] = a
    write_png(os.path.join(UI_DIR, "grenade.png"), rgba)


def main():
    reset_scene()
    os.makedirs(UI_DIR, exist_ok=True)
    digits()
    tallies()
    blood_overlay()
    points_bar()
    grenade()
    print("HUD-Grafiken fertig")


main()
