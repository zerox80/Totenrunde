"""Realistische, kachelbare PBR-Oberflächen (numpy): Farbe, Höhe, Rauheit, Metall.

Jede Funktion bekommt ein `Canvas` und liefert dict(color=(n,n,3) linear, height=(n,n),
rough=(n,n), metal=(n,n) oder float, normal=Stärke, optional alpha). Die Kachelgröße in
Metern steht im Register (für die Wahl der Strukturgrößen) und muss zur Karte passen.
"""

import numpy as np

from .synth import cavity, col, hexc, lerp, ramp, smooth, tint

RECIPES = {}


def recipe(name, size=2048, meters=2.0):
    def deco(fn):
        RECIPES[name] = dict(fn=fn, size=size, meters=meters)
        return fn

    return deco


# --------------------------------------------------------------------------------------
# Bausteine
# --------------------------------------------------------------------------------------


def cracks(cv, cells, salt, width=0.035, density=0.5, warp=0.02):
    """Feine, verzweigte Risse (0..1 = Riss)."""
    w = cv.warp(warp, cells * 1.5, salt + 7)
    vor = cv.voronoi(cells, salt, warp=w)
    line = 1.0 - smooth(vor["edge"], 0.0, width)
    mask = smooth(cv.noise(cells * 0.35, salt + 11, octaves=4), 0.62 - 0.25 * density, 0.7 - 0.25 * density)
    fine = cv.noise(cells * 8, salt + 13, octaves=2)
    return np.clip(line * mask * (0.6 + 0.4 * fine), 0, 1)


def puddles(cv, freq, salt, coverage=0.25):
    """Pfützen-Maske (0..1) mit weichem Rand."""
    n = cv.noise(freq, salt, octaves=3, rough=0.4)
    t = np.percentile(n, 100 * (1 - coverage))
    return smooth(n, t, t + 0.03)


def scratches(cv, salt, count=60, length=(0.02, 0.15), width_px=1.2, angle=None):
    """Feine Kratzer als Liniensegmente (periodisch). 0..1."""
    n = cv.n
    out = np.zeros((n, n), np.float32)
    rng = cv.rng(salt)
    for _ in range(count):
        x0, y0 = rng.random(2)
        ang = rng.uniform(0, np.pi) if angle is None else angle + rng.normal(0, 0.15)
        ln = rng.uniform(*length)
        steps = max(int(ln * n * 1.5), 2)
        t = np.linspace(0, 1, steps)
        # leicht gekrümmt
        bend = rng.normal(0, 0.15)
        xs = x0 + np.cos(ang) * ln * t + np.sin(ang) * bend * ln * t * (1 - t)
        ys = y0 + np.sin(ang) * ln * t - np.cos(ang) * bend * ln * t * (1 - t)
        inten = rng.uniform(0.4, 1.0) * np.sin(np.pi * t) ** 0.5
        xi = (xs * n).astype(np.int32) % n
        yi = (ys * n).astype(np.int32) % n
        np.maximum.at(out, (yi, xi), inten.astype(np.float32))
    return np.clip(cv.blur(out, width_px * 0.5) * 2.5, 0, 1)


def splats(cv, salt, count, radius=(0.01, 0.05), freq=40.0):
    """Unregelmäßige Flecken (Tropfen, Abplatzer) an zufälligen Stellen. 0..1."""
    n = cv.n
    acc = np.zeros((n, n), np.float32)
    rng = cv.rng(salt)
    for _ in range(count):
        cx, cy = rng.random(2)
        r = rng.uniform(*radius)
        dx = (cv.u - cx + 0.5) % 1.0 - 0.5
        dy = (cv.v - cy + 0.5) % 1.0 - 0.5
        d = np.sqrt(dx * dx + dy * dy) / r
        acc = np.maximum(acc, 1.0 - d)
    edge = cv.noise(freq, salt + 3, octaves=4)
    return smooth(acc + (edge - 0.5) * 0.9, 0.2, 0.35)


def grid(cv, nx, ny, gap_u, gap_v, offset_rows=0.0, bevel=0.3):
    """Fliesen/Ziegel-Raster. Liefert (mask 0..1 Fliese, ix, iy, fu, fv, profile)."""
    v = cv.v * ny
    iy = np.floor(v)
    shift = (iy % 2) * offset_rows
    u = cv.u * nx + shift
    ix = np.floor(u) % nx
    fu = u - np.floor(u)
    fv = v - iy
    du = np.minimum(fu, 1 - fu)
    dv = np.minimum(fv, 1 - fv)
    mu = smooth(du, gap_u * 0.5, gap_u * 0.5 + gap_u * bevel + 1e-4)
    mv = smooth(dv, gap_v * 0.5, gap_v * 0.5 + gap_v * bevel + 1e-4)
    return mu * mv, ix, iy, fu, fv, np.minimum(du / max(gap_u, 1e-4), dv / max(gap_v, 1e-4))


def cell_rand(ix, iy, salt):
    """Zufallswert pro Rasterzelle (deterministisch)."""
    h = np.sin(ix * 127.1 + iy * 311.7 + salt * 74.7) * 43758.5453
    return (h - np.floor(h)).astype(np.float32)


def vertical_streaks(cv, salt, freq=6.0, length=8.0):
    """Senkrechte Laufspuren (Wasser, Rost) von oben nach unten auslaufend. 0..1."""
    n = cv.noise(freq, salt, octaves=3, aniso=(1.0, length))
    return smooth(n, 0.6, 0.85)


# --------------------------------------------------------------------------------------
# Beton und Boden
# --------------------------------------------------------------------------------------


@recipe("concrete", meters=3.0)
def concrete(cv):
    big = cv.noise(2, 1, octaves=3)
    mid = cv.noise(10, 2, octaves=5)
    fine = cv.noise(120, 3, octaves=3)
    agg = cv.voronoi(90, 4)
    pits = (1 - smooth(agg["f1"], 0.0, 0.18)) * smooth(cv.noise(14, 5), 0.6, 0.72)
    speck = smooth(agg["id"], 0.75, 0.95) * (1 - smooth(agg["f1"], 0.25, 0.45))
    streak = vertical_streaks(cv, 6, freq=5, length=10) * smooth(cv.noise(3, 7), 0.45, 0.65)
    cr = cracks(cv, 6, 8, width=0.03, density=0.35)
    c = ramp(mid, [(0.0, "#6d6a64"), (0.5, "#85827b"), (1.0, "#9a978f")])
    c = tint(c, big, "#5e5a52", 0.35)
    c = tint(c, (fine - 0.5) * 2, "#aeaba3", 0.25)
    c = tint(c, speck, "#b9b6ae", 0.35)
    c = tint(c, pits, "#3c3a36", 0.7)
    c = tint(c, streak, "#4a4236", 0.45)
    c = tint(c, cr, "#2a2825", 0.8)
    h = mid * 0.35 + fine * 0.25 + speck * 0.1 - pits * 0.5 - cr * 0.6
    rough = 0.88 + (fine - 0.5) * 0.08 - streak * 0.08
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=5.0)


@recipe("concrete_floor", meters=4.0)
def concrete_floor(cv):
    """Werkstattboden: geglätteter Estrich mit Fugen, Ölflecken, Reifenspuren, Pfützen."""
    big = cv.noise(2, 21, octaves=4)
    mid = cv.noise(12, 22, octaves=5)
    fine = cv.noise(160, 23, octaves=2)
    trowel = cv.noise(20, 24, octaves=3, aniso=(1.0, 0.35))
    # Dehnfugen alle 2 m (Kachel = 4 m)
    jm, ix, iy, fu, fv, prof = grid(cv, 2, 2, 0.006, 0.006, bevel=0.6)
    joint = 1 - jm
    oil = splats(cv, 25, 9, radius=(0.02, 0.07), freq=30) * 0.85
    oil = np.maximum(oil, smooth(cv.noise(4, 26, octaves=5), 0.7, 0.8) * 0.7)
    tire = smooth(cv.noise(3, 27, octaves=3, aniso=(0.12, 1.0)), 0.62, 0.8)
    pud = puddles(cv, 3, 28, coverage=0.12)
    dust = smooth(cv.noise(5, 29, octaves=5), 0.55, 0.8)
    cr = cracks(cv, 5, 30, width=0.03, density=0.45)
    c = ramp(mid, [(0.0, "#5f5d59"), (0.5, "#76736d"), (1.0, "#8a8781")])
    c = tint(c, big, "#4d4a45", 0.4)
    c = tint(c, trowel, "#8f8c86", 0.25)
    c = tint(c, (fine - 0.5) * 2, "#9c9993", 0.2)
    c = tint(c, dust, "#a39c8e", 0.3)
    c = tint(c, tire, "#2b2a28", 0.35)
    c = tint(c, oil, "#1a1917", 0.85)
    c = tint(c, joint, "#2a2826", 0.9)
    c = tint(c, cr, "#232120", 0.8)
    c = c * (1 - 0.35 * pud[..., None])
    h = mid * 0.2 + fine * 0.15 + trowel * 0.1 - joint * 1.2 - cr * 0.6
    h = h * (1 - pud) + pud * 0.2
    rough = 0.72 + (mid - 0.5) * 0.1 + dust * 0.15 - oil * 0.35 - trowel * 0.08
    rough = rough * (1 - pud) + 0.06 * pud
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=4.0)


@recipe("asphalt", meters=5.0)
def asphalt(cv):
    big = cv.noise(2, 41, octaves=4)
    mid = cv.noise(15, 42, octaves=4)
    agg = cv.voronoi(260, 43)
    stone = 1 - smooth(agg["f1"], 0.2, 0.55)
    stone_c = agg["id"]
    worn = smooth(cv.noise(4, 44, octaves=5), 0.45, 0.75)
    cr = cracks(cv, 4, 45, width=0.025, density=0.8, warp=0.03)
    seal = smooth(cv.blur(cr, 3.0) * 6, 0.15, 0.5) * smooth(cv.noise(2, 46), 0.4, 0.55)
    pud = puddles(cv, 2.5, 47, coverage=0.18)
    oil = splats(cv, 48, 6, radius=(0.02, 0.06), freq=25) * 0.7
    patch_n = cv.noise(1.5, 49, octaves=2)
    patch = smooth(patch_n, 0.68, 0.7)
    c = ramp(mid, [(0.0, "#1f1f20"), (1.0, "#2f2e2d")])
    c = tint(c, stone * smooth(stone_c, 0.3, 1.0), "#5a5852", 0.75 * worn + 0.15)
    c = tint(c, worn, "#45433f", 0.35)
    c = tint(c, big, "#161616", 0.3)
    c = tint(c, patch, "#151516", 0.6)
    c = tint(c, oil, "#0c0c0d", 0.8)
    c = tint(c, seal, "#0b0b0c", 0.9)
    c = tint(c, cr, "#070707", 0.9)
    c = c * (1 - 0.45 * pud[..., None])
    h = stone * 0.5 + mid * 0.2 - cr * 0.8 + seal * 0.1
    h = h * (1 - pud) + 0.15 * pud
    rough = 0.9 - worn * 0.08 - seal * 0.45 - oil * 0.4
    rough = rough * (1 - pud) + 0.04 * pud
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=4.0)


@recipe("dirt", meters=4.0)
def dirt(cv):
    big = cv.noise(2, 61, octaves=5)
    mid = cv.noise(10, 62, octaves=5)
    fine = cv.noise(90, 63, octaves=3)
    peb = cv.voronoi(70, 64)
    pebble = (1 - smooth(peb["f1"], 0.15, 0.5)) * smooth(peb["id"], 0.55, 0.7)
    small = cv.voronoi(220, 65)
    grit = (1 - smooth(small["f1"], 0.1, 0.4)) * smooth(small["id"], 0.5, 0.8)
    wet = smooth(cv.noise(3, 66, octaves=5), 0.6, 0.78)
    twig = scratches(cv, 67, count=90, length=(0.01, 0.04), width_px=1.5)
    c = ramp(mid, [(0.0, "#2e241a"), (0.5, "#45372a"), (1.0, "#5c4b38")])
    c = tint(c, big, "#3a3227", 0.4)
    c = tint(c, grit, "#6b6156", 0.5)
    c = tint(c, pebble, "#7b766e", 0.85)
    c = tint(c, twig, "#5a4630", 0.6)
    c = tint(c, wet, "#1b150f", 0.55)
    h = mid * 0.4 + fine * 0.15 + pebble * 0.8 + grit * 0.2 + twig * 0.15
    rough = 0.95 - wet * 0.3 - pebble * 0.1
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=6.0)


@recipe("gravel", meters=3.0)
def gravel(cv):
    v1 = cv.voronoi(55, 71, jitter=1.0)
    v2 = cv.voronoi(95, 72, jitter=1.0)
    s1 = 1 - smooth(v1["f1"], 0.25, 0.75)
    s2 = 1 - smooth(v2["f1"], 0.25, 0.75)
    stones = np.maximum(s1, s2 * 0.9)
    sid = np.where(s1 >= s2 * 0.9, v1["id"], v2["id"])
    big = cv.noise(3, 73, octaves=4)
    fine = cv.noise(150, 74, octaves=2)
    c = ramp(sid, [(0.0, "#6e6a64"), (0.4, "#8a857c"), (0.7, "#a29b8f"), (1.0, "#5d5a56")])
    c = c * (0.55 + 0.45 * stones[..., None])
    c = tint(c, (fine - 0.5) * 2, "#b5afa4", 0.2)
    c = tint(c, 1 - stones, "#2e2a24", 0.6)
    c = tint(c, big, "#4e463b", 0.3)
    h = stones * 1.0 + fine * 0.1
    rough = 0.85 + (fine - 0.5) * 0.1
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=7.0)


@recipe("grass_dead", meters=3.5)
def grass_dead(cv):
    patch = cv.noise(2.5, 81, octaves=5)
    bare = smooth(cv.noise(3, 82, octaves=5), 0.6, 0.75)
    blades = np.zeros((cv.n, cv.n), np.float32)
    for k, (cnt, ln) in enumerate(((900, (0.006, 0.02)), (500, (0.01, 0.03)))):
        blades = np.maximum(blades, scratches(cv, 83 + k, count=cnt, length=ln, width_px=1.4))
    thatch = cv.noise(60, 85, octaves=3, aniso=(1.0, 0.3))
    ground = dirt(cv)
    c = ramp(patch, [(0.0, "#4a4229"), (0.5, "#6b5f3b"), (1.0, "#8a7c52")])
    c = tint(c, thatch, "#a39468", 0.3)
    c = tint(c, blades, "#b4a676", 0.6)
    c = tint(c, 1 - blades, "#2c2618", 0.35)
    c = lerp(c, ground["color"], bare)
    h = blades * 0.8 + thatch * 0.3
    h = lerp(h, ground["height"], bare)
    rough = np.full_like(h, 0.92)
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=5.0)


# --------------------------------------------------------------------------------------
# Wände
# --------------------------------------------------------------------------------------


@recipe("brick", meters=2.4)
def brick(cv):
    # 32 Lagen à 75 mm, 10 Steine à 240 mm pro 2,4 m
    w = cv.warp(0.0025, 60, 91)
    jm, ix, iy, fu, fv, prof = grid(cv, 10, 32, 0.04, 0.14, offset_rows=0.5, bevel=0.5)
    jm = cv.sample(jm, *w)
    r1 = cell_rand(ix, iy, 1)
    r2 = cell_rand(ix, iy, 2)
    r3 = cell_rand(ix, iy, 3)
    fine = cv.noise(200, 92, octaves=3)
    mid = cv.noise(30, 93, octaves=4)
    chips = smooth(cv.noise(70, 94, octaves=4), 0.72, 0.8) * (1 - smooth(prof, 0.6, 2.0))
    soot = smooth(cv.noise(2, 95, octaves=5), 0.45, 0.8)
    effl = smooth(cv.noise(4, 96, octaves=5), 0.68, 0.82) * vertical_streaks(cv, 97, freq=6, length=6)
    base = ramp(r1, [(0.0, "#4f2219"), (0.35, "#6c2f20"), (0.7, "#7f3d27"), (1.0, "#8e5236")])
    base = tint(base, smooth(r2, 0.85, 0.95), "#3a1d16", 0.7)  # dunkel gebrannte Steine
    base = tint(base, smooth(r3, 0.9, 0.98), "#9a6a4c", 0.6)
    base = tint(base, (mid - 0.5) * 2, "#9c6448", 0.2)
    base = tint(base, (fine - 0.5) * 2, "#a57458", 0.2)
    mortar = ramp(fine, [(0.0, "#55524c"), (1.0, "#7a766d")])
    c = lerp(mortar, base, jm)
    c = tint(c, chips * jm, "#a07a62", 0.5)
    c = tint(c, soot, "#1d1a18", 0.55)
    c = tint(c, effl, "#c9c4b6", 0.5)
    h = jm * 0.8 + fine * 0.12 + mid * 0.1 - chips * 0.4
    rough = 0.9 - effl * 0.05
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=5.0)


def _painted_wall(cv, salt, paint_lo, paint_hi, under="#8c877c", gloss=0.62, peel_amount=0.5, stain=0.5):
    big = cv.noise(2, salt + 1, octaves=4)
    roller = cv.noise(40, salt + 2, octaves=3, aniso=(1.0, 0.25))
    fine = cv.noise(220, salt + 3, octaves=2)
    plaster_n = cv.noise(25, salt + 4, octaves=4)
    w = cv.warp(0.012, 30, salt + 12)
    peel_n = cv.sample(cv.noise(2.5, salt + 5, octaves=3, rough=0.45), *w)
    gate = smooth(cv.noise(1.2, salt + 6, octaves=2), 0.55, 0.75)
    thr = 0.86 - 0.1 * peel_amount
    peel_raw = smooth(peel_n, thr, thr + 0.004)
    flakes = smooth(cv.noise(45, salt + 13, octaves=2), 0.7, 0.72) * smooth(peel_n, thr - 0.08, thr)
    peel = np.maximum(peel_raw, flakes) * gate
    rim = np.clip(smooth(peel_n, thr - 0.02, thr) * gate - peel, 0, 1)
    streak = vertical_streaks(cv, salt + 7, freq=7, length=9) * smooth(cv.noise(2.5, salt + 8), 0.35, 0.6) * stain
    water = smooth(cv.noise(2.2, salt + 9, octaves=6, rough=0.6), 0.62, 0.66) * stain
    water_edge = np.clip(smooth(cv.noise(2.2, salt + 9, octaves=6, rough=0.6), 0.6, 0.62) - water, 0, 1)
    mold = smooth(cv.noise(18, salt + 10, octaves=5), 0.72, 0.85) * smooth(cv.noise(2, salt + 11), 0.6, 0.75)
    c = ramp(big, [(0.0, paint_lo), (1.0, paint_hi)])
    blotch = smooth(cv.noise(1.5, salt + 14, octaves=4), 0.5, 0.9)
    c = tint(c, blotch, "#6b6650", 0.25)
    c = tint(c, (roller - 0.5) * 2, "#ffffff", 0.04)
    c = tint(c, streak, "#5d523e", 0.5)
    c = tint(c, water, "#7a6a4a", 0.35)
    c = tint(c, water_edge, "#5a4a32", 0.5)
    c = tint(c, mold, "#2b3024", 0.6)
    under_c = ramp(plaster_n, [(0.0, under), (1.0, "#a6a196")])
    under_c = tint(under_c, (fine - 0.5) * 2, "#6a665e", 0.4)
    c = lerp(c, under_c, peel)
    c = tint(c, rim, "#2e2b26", 0.55)
    h = roller * 0.06 + fine * 0.05 + (1 - peel) * 0.5 + rim * 0.25 + plaster_n * peel * 0.2
    rough = gloss + (fine - 0.5) * 0.05 + streak * 0.1 + peel * (0.92 - gloss) + mold * 0.2
    return c, h, rough


@recipe("plaster", meters=3.0)
def plaster(cv):
    c, h, r = _painted_wall(cv, 100, "#8e9682", "#a8ae98", gloss=0.7, peel_amount=0.55)
    return dict(color=c, height=h, rough=r, metal=0.0, normal=3.0)


@recipe("plaster_cream", meters=3.0)
def plaster_cream(cv):
    c, h, r = _painted_wall(cv, 120, "#c7b796", "#dccba7", gloss=0.6, peel_amount=0.35, stain=0.6)
    return dict(color=c, height=h, rough=r, metal=0.0, normal=3.0)


@recipe("paint_lower", meters=3.0)
def paint_lower(cv):
    """Dunkle, glänzende Sockelfarbe (Unterteil der Wand) mit Scheuerstellen."""
    c, h, r = _painted_wall(cv, 140, "#2f4a46", "#3a5651", under="#7b776b", gloss=0.38, peel_amount=-0.6, stain=0.3)
    scuff = scratches(cv, 145, count=140, length=(0.02, 0.12), width_px=1.0, angle=0.0)
    kick = smooth(cv.noise(6, 146, octaves=4), 0.6, 0.8)
    c = tint(c, scuff, "#9a9584", 0.5)
    c = tint(c, kick, "#1b1f1d", 0.3)
    r = r + scuff * 0.2
    return dict(color=c, height=h - scuff * 0.1, rough=r, metal=0.0, normal=3.0)


@recipe("wall_tiles", meters=1.2)
def wall_tiles(cv):
    """Weiße Wandfliesen 150×75 mm im Verband (Diner)."""
    jm, ix, iy, fu, fv, prof = grid(cv, 8, 16, 0.025, 0.05, offset_rows=0.5, bevel=0.8)
    r1 = cell_rand(ix, iy, 11)
    crack = cracks(cv, 6, 151, width=0.02, density=0.3) * jm
    dirt_g = smooth(cv.noise(4, 152, octaves=5), 0.4, 0.8)
    missing = (smooth(r1, 0.975, 0.98)) * jm
    c = ramp(r1, [(0.0, "#c9c6bb"), (1.0, "#dedbd0")])
    c = tint(c, 1 - jm, "#4a463f", 0.85)
    c = tint(c, dirt_g * (1 - jm), "#2a2620", 0.6)
    c = tint(c, dirt_g, "#7c7362", 0.25)
    c = tint(c, crack, "#3a3631", 0.8)
    c = lerp(c, col("#4b4943", c.shape[:2]), missing)
    h = jm * 0.7 + (1 - smooth(prof, 0, 1.5)) * jm * 0.2 - crack * 0.4 - missing * 0.6
    rough = 0.12 + (1 - jm) * 0.75 + dirt_g * 0.15 + missing * 0.7
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=5.0)


# --------------------------------------------------------------------------------------
# Böden innen
# --------------------------------------------------------------------------------------


@recipe("tiles", meters=2.4)
def tiles(cv):
    """Vinyl-Bodenplatten 300 mm (Wartehalle): gesprenkelt, abgenutzt, Fugenschmutz."""
    jm, ix, iy, fu, fv, prof = grid(cv, 8, 8, 0.006, 0.006, bevel=1.5)
    r1 = cell_rand(ix, iy, 21)
    r2 = cell_rand(ix, iy, 22)
    chips = cv.voronoi(260, 161)
    chip = smooth(chips["id"], 0.0, 1.0)
    fleck = (1 - smooth(chips["f1"], 0.3, 0.6)) * smooth(chips["id"], 0.7, 0.85)
    path = smooth(cv.noise(1.5, 162, octaves=3), 0.35, 0.75)
    scuff = scratches(cv, 163, count=220, length=(0.01, 0.06), width_px=1.6)
    dirt_g = smooth(cv.noise(5, 164, octaves=5), 0.45, 0.85)
    broken = np.zeros_like(r2)
    glue = cv.noise(40, 165, octaves=4)
    cr = cracks(cv, 8, 166, width=0.03, density=0.4)
    tone = ramp(r1, [(0.0, "#8d8676"), (0.5, "#9b9483"), (1.0, "#a7a090")])
    tone = np.where(((ix + iy) % 2 == 0)[..., None], tone, tone * 0.86)
    c = tint(tone, (chip - 0.5) * 2, "#bab3a1", 0.15)
    c = tint(c, fleck, "#4f4a40", 0.6)
    c = tint(c, path, "#5e574b", 0.3)
    c = tint(c, dirt_g, "#4a4234", 0.35)
    c = tint(c, scuff, "#2a2722", 0.55)
    c = tint(c, cr * jm, "#2b2824", 0.7)
    c = tint(c, 1 - jm, "#1f1c18", 0.9)
    adh = ramp(glue, [(0.0, "#1e1914"), (1.0, "#3a2e22")])
    c = lerp(c, adh, broken)
    h = jm * 0.5 - scuff * 0.05 - cr * 0.3 - broken * 0.45 + glue * broken * 0.1
    rough = 0.38 + path * 0.25 + dirt_g * 0.15 + scuff * 0.1 + (1 - jm) * 0.4 + broken * 0.5
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=4.0)


@recipe("diner_checker", meters=2.4)
def diner_checker(cv):
    jm, ix, iy, fu, fv, prof = grid(cv, 8, 8, 0.008, 0.008, bevel=1.2)
    r1 = cell_rand(ix, iy, 31)
    checker = ((ix + iy) % 2 == 0)
    path = smooth(cv.noise(1.8, 171, octaves=3), 0.35, 0.75)
    dirt_g = smooth(cv.noise(6, 172, octaves=5), 0.4, 0.85)
    scuff = scratches(cv, 173, count=200, length=(0.01, 0.05), width_px=1.5)
    cr = cracks(cv, 10, 174, width=0.025, density=0.45)
    chip_c = smooth(cv.noise(60, 175, octaves=3), 0.75, 0.8) * (1 - smooth(prof, 0, 3.0))
    white = ramp(r1, [(0.0, "#bdb6a2"), (1.0, "#cfc8b4")])
    black = ramp(r1, [(0.0, "#1b1b1c"), (1.0, "#262628")])
    c = np.where(checker[..., None], white, black)
    c = tint(c, path, "#6b6252", 0.12)
    c = tint(c, dirt_g, "#4c4434", 0.15)
    c = tint(c, scuff, "#5a554c", 0.3)
    c = tint(c, cr * jm, "#2e2a24", 0.75)
    c = tint(c, chip_c, "#4d4840", 0.8)
    c = tint(c, 1 - jm, "#2a2620", 0.9)
    h = jm * 0.5 - cr * 0.3 - chip_c * 0.3 - scuff * 0.05
    rough = 0.25 + path * 0.3 + dirt_g * 0.15 + (1 - jm) * 0.5 + scuff * 0.1
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=4.0)


@recipe("ceiling_tile", meters=2.4)
def ceiling_tile(cv):
    """Akustik-Deckenplatten 600 mm mit T-Profil-Raster und Wasserflecken."""
    jm, ix, iy, fu, fv, prof = grid(cv, 4, 4, 0.04, 0.04, bevel=0.15)
    r1 = cell_rand(ix, iy, 41)
    fiss = cv.noise(160, 181, octaves=3, aniso=(1.0, 0.6))
    holes = cv.voronoi(300, 182)
    hole = (1 - smooth(holes["f1"], 0.0, 0.2)) * smooth(holes["id"], 0.6, 0.62)
    stain_n = cv.noise(3, 183, octaves=6, rough=0.6)
    stain = smooth(stain_n, 0.66, 0.7)
    ring = np.clip(smooth(stain_n, 0.64, 0.66) - stain, 0, 1)
    sag = smooth(r1, 0.8, 1.0)
    c = ramp(fiss, [(0.0, "#a9a597"), (1.0, "#c6c2b4")])
    c = tint(c, sag, "#8f8a7a", 0.35)
    c = tint(c, hole, "#5d5a52", 0.6)
    c = tint(c, stain, "#9a8458", 0.4)
    c = tint(c, ring, "#6b5432", 0.7)
    bar = col("#c9c7c0", c.shape[:2])
    c = lerp(bar, c, jm)
    h = jm * 0.6 + fiss * 0.15 * jm - hole * 0.2
    rough = 0.95 * jm + 0.45 * (1 - jm)
    metal = (1 - jm) * 0.0
    return dict(color=c, height=h, rough=rough, metal=metal, normal=4.0)


# --------------------------------------------------------------------------------------
# Holz
# --------------------------------------------------------------------------------------


def _wood_grain(cv, salt, freq_along=2.0, rings=9.0):
    g = cv.noise(freq_along, salt, octaves=3, rough=0.45, aniso=(10.0, 1.0))
    w = cv.warp(0.006, 8, salt + 1)
    ring = (np.sin(cv.sample(g, *w) * rings * np.pi * 2) * 0.5 + 0.5).astype(np.float32)
    ring = smooth(ring, 0.2, 0.95)
    fibre = cv.noise(60, salt + 2, octaves=2, aniso=(14.0, 1.0))
    return ring, fibre


@recipe("wood_dark", size=1024, meters=1.5)
def wood_dark(cv):
    ring, fibre = _wood_grain(cv, 201)
    tone = cv.noise(3, 204, octaves=4)
    wear = smooth(cv.noise(4, 205, octaves=5), 0.55, 0.8)
    scr = scratches(cv, 206, count=80, length=(0.02, 0.1))
    c = ramp(tone, [(0.0, "#2a170d"), (1.0, "#432617")])
    c = tint(c, ring, "#170c06", 0.45)
    c = tint(c, fibre, "#4e2e1b", 0.2)
    c = tint(c, wear, "#5e3e26", 0.35)
    c = tint(c, scr, "#6d4a30", 0.5)
    h = ring * 0.2 + fibre * 0.2 - scr * 0.15
    rough = 0.42 + wear * 0.25 + scr * 0.15
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=2.5)


@recipe("planks", size=2048, meters=2.4)
def planks(cv):
    """Verwitterte Bretter (Barrikaden, Kisten): graubraun, Risse, Nagellöcher."""
    rows = 8
    v = cv.v * rows
    iy = np.floor(v)
    fv = v - iy
    off = cell_rand(iy, iy * 0 + 3, 51)
    u = (cv.u + off) % 1.0
    seg = np.floor(u * 2)
    fu = u * 2 - seg
    pr = cell_rand(seg, iy, 52)
    gap = 1 - smooth(np.minimum(fv, 1 - fv), 0.01, 0.04) * smooth(np.minimum(fu, 1 - fu), 0.003, 0.01)
    ring, fibre = _wood_grain(cv, 211, rings=7)
    weather = cv.noise(5, 214, octaves=5)
    split = smooth(cv.noise(20, 215, octaves=2, aniso=(25.0, 1.0)), 0.8, 0.85)
    nails = np.zeros_like(fu)
    for ny in (0.3, 0.7):
        for nx in (0.03, 0.97):
            # Nagelkopf r = 4 mm; Brett 1,2 m × 0,3 m
            d = np.sqrt(((fu - nx) / 0.0033) ** 2 + ((fv - ny) / 0.013) ** 2)
            nails = np.maximum(nails, 1 - smooth(d, 0.7, 1.0))
    c = ramp(pr, [(0.0, "#4e4234"), (0.5, "#6a5a46"), (1.0, "#7d6c56")])
    c = tint(c, weather, "#8a8174", 0.35)
    c = tint(c, ring, "#2e2419", 0.4)
    c = tint(c, fibre, "#9a8d7a", 0.15)
    c = tint(c, split, "#1c1611", 0.8)
    c = tint(c, nails, "#2b241f", 0.9)
    c = tint(c, gap, "#0f0c09", 0.95)
    h = (1 - gap) * 0.6 + ring * 0.15 + fibre * 0.15 - split * 0.4 - nails * 0.2
    rough = 0.88 + (weather - 0.5) * 0.08
    metal = nails * 0.6
    return dict(color=c, height=h, rough=rough, metal=metal, normal=5.0)


# --------------------------------------------------------------------------------------
# Metall
# --------------------------------------------------------------------------------------


@recipe("metal_paint", size=2048, meters=1.5)
def metal_paint(cv):
    """Hellgrauer Lack (wird pro Material eingefärbt) über Stahl: Abplatzer, Rostläufer."""
    big = cv.noise(2, 301, octaves=4)
    fine = cv.noise(150, 302, octaves=2)
    orange = cv.noise(60, 303, octaves=3)  # Orangenhaut
    w = cv.warp(0.01, 40, 310)
    chip_n = cv.sample(cv.noise(3.5, 304, octaves=4, rough=0.5), *w)
    chip = smooth(chip_n, 0.87, 0.875)
    primer = np.clip(smooth(chip_n, 0.858, 0.862) - chip, 0, 1)
    rust_core = smooth(chip_n, 0.92, 0.95)
    bleed = vertical_streaks(cv, 305, freq=8, length=10) * smooth(cv.noise(3, 306), 0.5, 0.65)
    scr = scratches(cv, 307, count=120, length=(0.02, 0.1), width_px=0.9)
    grime = smooth(cv.noise(3, 308, octaves=5), 0.4, 0.85)
    c = ramp(big, [(0.0, "#c2c0ba"), (1.0, "#d6d4ce")])
    c = tint(c, grime, "#6e6a61", 0.45)
    c = tint(c, bleed, "#6b3b1c", 0.55)
    c = tint(c, primer, "#8a4a35", 0.85)
    steel = ramp(fine, [(0.0, "#4b4b4b"), (1.0, "#6a6a68")])
    c = lerp(c, steel, chip)
    rust = ramp(cv.noise(40, 309, octaves=4), [(0.0, "#3d1d0d"), (1.0, "#7a3f1a")])
    c = lerp(c, rust, rust_core)
    c = tint(c, scr * (1 - chip), "#8f8f8c", 0.6)
    h = orange * 0.04 + (1 - chip) * 0.25 - rust_core * 0.1 + rust_core * fine * 0.2 - scr * 0.05
    rough = 0.5 + grime * 0.2 + bleed * 0.15 + (fine - 0.5) * 0.05
    rough = lerp(rough, 0.42, chip * (1 - rust_core))
    rough = lerp(rough, 0.92, rust_core)
    metal = chip * (1 - rust_core) * 0.9
    return dict(color=c, height=h, rough=rough, metal=metal, normal=3.0, tintable=True)


@recipe("rust", size=2048, meters=2.0)
def rust(cv):
    n1 = cv.noise(4, 311, octaves=7, rough=0.65)
    n2 = cv.noise(30, 312, octaves=4)
    flake = cv.voronoi(40, 313)
    flk = smooth(flake["edge"], 0.0, 0.06)
    pits = (1 - smooth(cv.voronoi(120, 314)["f1"], 0.0, 0.25)) * smooth(cv.noise(10, 315), 0.5, 0.7)
    paint_n = cv.noise(3, 316, octaves=5)
    paint = smooth(paint_n, 0.66, 0.67)
    c = ramp(n1, [(0.0, "#2e170b"), (0.4, "#5c2c12"), (0.75, "#8a4a1e"), (1.0, "#a5642c")])
    c = tint(c, (n2 - 0.5) * 2, "#b97a3a", 0.25)
    c = tint(c, 1 - flk, "#1d0f08", 0.5)
    c = tint(c, pits, "#170c06", 0.7)
    c = lerp(c, ramp(n2, [(0.0, "#4b544e"), (1.0, "#5f6a63")]), paint)
    h = n1 * 0.4 + n2 * 0.3 - pits * 0.4 + paint * 0.3 + flk * 0.15
    rough = 0.9 - paint * 0.3
    metal = 0.15 + paint * 0.0
    return dict(color=c, height=h, rough=rough, metal=metal, normal=5.0)


@recipe("gunmetal", size=1024, meters=1.0)
def gunmetal(cv):
    """Brünierter, abgegriffener Stahl (Karte: dunkle Metallteile)."""
    brushed = cv.noise(4, 321, octaves=3, aniso=(1.0, 30.0))
    big = cv.noise(3, 322, octaves=4)
    wear = smooth(cv.noise(6, 323, octaves=5), 0.62, 0.78)
    spots = smooth(cv.noise(12, 324, octaves=5), 0.72, 0.8)
    scr = scratches(cv, 325, count=60, length=(0.02, 0.1), width_px=0.8)
    c = ramp(brushed, [(0.0, "#1a1b1d"), (1.0, "#2a2c2f")])
    c = tint(c, big, "#121314", 0.4)
    c = tint(c, wear, "#55585b", 0.6)
    c = tint(c, spots, "#3a2416", 0.5)
    c = tint(c, scr, "#77797b", 0.6)
    h = brushed * 0.1 - scr * 0.08 + spots * 0.05
    rough = 0.42 + big * 0.12 - wear * 0.1 + spots * 0.3
    metal = 0.9 - spots * 0.5
    return dict(color=c, height=h, rough=rough, metal=metal, normal=1.5)


@recipe("chainlink", size=1024, meters=2.0)
def chainlink(cv):
    n = 16.0
    a = (cv.u + cv.v) * n
    b = (cv.u - cv.v) * n
    fa = a - np.floor(a)
    fb = b - np.floor(b)
    wa = 1 - smooth(np.minimum(fa, 1 - fa), 0.025, 0.06)
    wb = 1 - smooth(np.minimum(fb, 1 - fb), 0.025, 0.06)
    wire = np.maximum(wa, wb)
    rust_m = smooth(cv.noise(6, 331, octaves=4), 0.55, 0.75)
    c = ramp(cv.noise(30, 332, octaves=3), [(0.0, "#5f6262"), (1.0, "#8a8d8c")])
    c = tint(c, rust_m, "#6b3a1c", 0.8)
    h = wire
    rough = 0.55 + rust_m * 0.35
    metal = 0.8 - rust_m * 0.6
    return dict(color=c, height=h, rough=rough, metal=metal, normal=6.0, alpha=smooth(wire, 0.3, 0.6))


# --------------------------------------------------------------------------------------
# Stoff, Gummi
# --------------------------------------------------------------------------------------


@recipe("fabric", size=1024, meters=1.0)
def fabric(cv):
    u, v = cv.u, cv.v
    f = 110.0
    wu = np.sin(u * np.pi * 2 * f) * 0.5 + 0.5
    wv = np.sin(v * np.pi * 2 * f) * 0.5 + 0.5
    over = (np.floor(u * f) + np.floor(v * f)) % 2
    weave = np.where(over == 0, wu * 0.7 + 0.3, wv * 0.7 + 0.3).astype(np.float32)
    n1 = cv.noise(3, 341, octaves=5)
    stain = smooth(cv.noise(2, 342, octaves=5, rough=0.65), 0.55, 0.75)
    c = ramp(n1, [(0.0, "#8a8984"), (1.0, "#a3a29c")])
    c = c * (0.8 + 0.2 * weave[..., None])
    c = tint(c, stain, "#4b463c", 0.55)
    h = weave * 0.5 + n1 * 0.1
    rough = 0.95 - stain * 0.05
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=4.0, tintable=True)


@recipe("polymer", size=1024, meters=0.5)
def polymer(cv):
    n1 = cv.noise(160, 351, octaves=2)
    n2 = cv.noise(4, 352, octaves=3)
    wear = smooth(cv.noise(8, 353, octaves=5), 0.6, 0.8)
    c = ramp(n1, [(0.0, "#141416"), (1.0, "#1f1f22")])
    c = tint(c, n2, "#2a2a2d", 0.3)
    c = tint(c, wear, "#3a3a3d", 0.4)
    h = n1 * 0.3
    rough = 0.82 - wear * 0.25
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=2.0)


@recipe("rubber", size=1024, meters=1.0)
def rubber(cv):
    n1 = cv.noise(120, 361, octaves=3)
    dust = smooth(cv.noise(4, 362, octaves=5), 0.5, 0.8)
    crack = cracks(cv, 20, 363, width=0.02, density=0.6)
    c = ramp(n1, [(0.0, "#121212"), (1.0, "#1c1c1c")])
    c = tint(c, dust, "#4a4640", 0.45)
    c = tint(c, crack, "#060606", 0.8)
    h = n1 * 0.3 - crack * 0.4
    rough = 0.85 + dust * 0.1
    return dict(color=c, height=h, rough=rough, metal=0.0, normal=3.0)


# --------------------------------------------------------------------------------------
# Decal-Atlas (4×4 Felder): Blut, Dreck, Papier, Plakate, Risse, Einschüsse, Graffiti
# --------------------------------------------------------------------------------------

DECAL_CELLS = [
    "blood_splat", "blood_spray", "blood_drips", "blood_pool",
    "grime", "leak", "debris", "oil",
    "paper", "newspaper", "poster_missing", "poster_ad",
    "cracks", "bullets", "graffiti", "footprints",
]


def _radial(cv, cx=0.5, cy=0.5):
    dx = cv.u - cx
    dy = cv.v - cy
    return np.sqrt(dx * dx + dy * dy), np.arctan2(dy, dx)


def _edge_fade(cv, m=0.04):
    e = np.minimum(np.minimum(cv.u, 1 - cv.u), np.minimum(cv.v, 1 - cv.v))
    return smooth(e, 0.0, m)


def _droplets(cv, salt, n, center, spread, rmin, rmax, direction=None):
    rng = cv.rng(salt)
    acc = np.zeros((cv.n, cv.n), np.float32)
    for _ in range(n):
        if direction is None:
            a = rng.uniform(0, np.pi * 2)
        else:
            a = direction + rng.normal(0, 0.35)
        d = abs(rng.normal(0, spread))
        x = center[0] + np.cos(a) * d
        y = center[1] + np.sin(a) * d
        r = rng.uniform(rmin, rmax) * (1.0 - min(d / (spread * 3), 0.7))
        stretch = 1.0 + d * 6.0
        ca, sa = np.cos(a), np.sin(a)
        dx = cv.u - x
        dy = cv.v - y
        lu = (dx * ca + dy * sa) / stretch
        lv = -dx * sa + dy * ca
        acc = np.maximum(acc, 1.0 - np.sqrt(lu * lu + lv * lv) / r)
    return np.clip(acc, 0, 1)


def _blood_color(cv, salt, thick):
    var = cv.noise(8, salt, octaves=4)
    c = ramp(var, [(0.0, "#2a0303"), (0.5, "#4a0606"), (1.0, "#650b09")])
    return lerp(c, np.broadcast_to(hexc("#1a0202"), c.shape), smooth(thick, 0.5, 1.0) * 0.6)


def decal_cell(cv, kind):
    """Liefert (color, alpha, rough, height) für ein Feld."""
    n = cv.n
    zero = np.zeros((n, n), np.float32)
    edge = _edge_fade(cv)
    if kind == "blood_splat":
        r, a = _radial(cv)
        wob = cv.noise(6, 1, octaves=4)
        core = smooth(0.2 + (wob - 0.5) * 0.12 - r, 0.0, 0.02)
        rays = _droplets(cv, 2, 140, (0.5, 0.5), 0.13, 0.006, 0.02)
        streak = _droplets(cv, 3, 25, (0.5, 0.5), 0.09, 0.01, 0.025)
        m = np.maximum(core, smooth(np.maximum(rays, streak), 0.0, 0.15))
        thick = core * (0.6 + 0.4 * wob)
        col_ = _blood_color(cv, 4, thick)
        return col_, m * edge, 0.35 - thick * 0.15, thick
    if kind == "blood_spray":
        m = smooth(_droplets(cv, 11, 420, (0.15, 0.55), 0.25, 0.003, 0.012, direction=0.0), 0.0, 0.2)
        col_ = _blood_color(cv, 12, m * 0.3)
        return col_, m * edge * 0.95, 0.4 + zero, m * 0.1
    if kind == "blood_drips":
        top = smooth(0.32 - abs(cv.v - 0.2) - (cv.noise(10, 21, octaves=3) - 0.5) * 0.15, 0.1, 0.13)
        top = top * smooth(cv.u, 0.08, 0.2) * smooth(1 - cv.u, 0.08, 0.2)
        drips = zero.copy()
        rng = cv.rng(22)
        for _ in range(22):
            x = rng.uniform(0.12, 0.88)
            ln = rng.uniform(0.15, 0.75)
            w = rng.uniform(0.004, 0.012)
            body = smooth(w - abs(cv.u - x - (cv.v - 0.2) * 0.02), 0.0, 0.003) * (cv.v > 0.2) * (cv.v < 0.2 + ln)
            tip = 1 - np.sqrt(((cv.u - x - ln * 0.02) / (w * 1.6)) ** 2 + ((cv.v - 0.2 - ln) / (w * 2.2)) ** 2)
            drips = np.maximum(drips, np.maximum(body, smooth(tip, 0.0, 0.3)))
        m = np.maximum(top, drips)
        col_ = _blood_color(cv, 23, m * 0.5)
        return col_, m * edge, 0.3 + zero, m * 0.2
    if kind == "blood_pool":
        r, a = _radial(cv)
        wob = cv.noise(4, 31, octaves=5)
        shape = 0.33 + (wob - 0.5) * 0.25 - r
        m = smooth(shape, 0.0, 0.012)
        rim = smooth(shape, 0.0, 0.03) * (1 - smooth(shape, 0.03, 0.08))
        sat = smooth(_droplets(cv, 32, 40, (0.5, 0.5), 0.2, 0.005, 0.015), 0.0, 0.2)
        m = np.maximum(m, sat)
        col_ = _blood_color(cv, 33, 0.7 + rim * 0.3)
        return col_, m * edge, 0.08 + rim * 0.1, m * 0.4 + rim * 0.3
    if kind == "grime":
        r, a = _radial(cv)
        nn = cv.noise(5, 41, octaves=6)
        m = smooth(0.42 - r + (nn - 0.5) * 0.35, 0.0, 0.25)
        col_ = ramp(cv.noise(12, 42, octaves=4), [(0.0, "#1c1712"), (1.0, "#3a3024")])
        return col_, m * 0.8 * edge, 0.9 + zero, zero
    if kind == "leak":
        streaks = cv.noise(10, 51, octaves=3, aniso=(1.0, 12.0))
        fall = smooth(1 - cv.v, 0.0, 0.9) * smooth(cv.v, 0.0, 0.08)
        band = smooth(0.4 - abs(cv.u - 0.5), 0.0, 0.25)
        m = smooth(streaks * fall * band * 2.2, 0.15, 0.9)
        col_ = ramp(cv.noise(6, 52, octaves=3), [(0.0, "#2e2416"), (1.0, "#5a4426")])
        return col_, m * edge * 0.85, 0.7 - m * 0.3, zero
    if kind == "debris":
        v1 = cv.voronoi(60, 61)
        bits = (1 - smooth(v1["f1"], 0.1, 0.35)) * smooth(v1["id"], 0.4, 0.5)
        r, a = _radial(cv)
        area = smooth(0.45 - r + (cv.noise(4, 62, octaves=4) - 0.5) * 0.3, 0.0, 0.15)
        leaves = smooth(cv.voronoi(14, 63)["f1"], 0.35, 0.15) * smooth(cv.noise(3, 64), 0.5, 0.7)
        m = np.clip(bits * area + leaves * area, 0, 1)
        col_ = ramp(v1["id"], [(0.0, "#2a241c"), (0.5, "#4a3b28"), (1.0, "#6b5a40")])
        col_ = tint(col_, leaves, "#5a3a1a", 0.8)
        return col_, m * edge, 0.85 + zero, m * 0.6
    if kind == "oil":
        r, a = _radial(cv)
        m = smooth(0.35 - r + (cv.noise(3, 71, octaves=5) - 0.5) * 0.3, 0.0, 0.05)
        col_ = col("#0b0b0a", (n, n))
        return col_, m * 0.85 * edge, 0.15 + zero, zero
    if kind in ("paper", "newspaper", "poster_missing", "poster_ad"):
        # Blatt mit leichter Drehung, ausgefransten Kanten, Knick
        ang = {"paper": 0.12, "newspaper": -0.08, "poster_missing": 0.03, "poster_ad": -0.02}[kind]
        ca, sa = np.cos(ang), np.sin(ang)
        x = (cv.u - 0.5) * ca + (cv.v - 0.5) * sa
        y = -(cv.u - 0.5) * sa + (cv.v - 0.5) * ca
        hw, hh = (0.3, 0.42) if kind != "newspaper" else (0.42, 0.32)
        tear = (cv.noise(30, 81, octaves=3) - 0.5) * 0.02
        m = smooth(hw - abs(x) + tear, 0.0, 0.004) * smooth(hh - abs(y) + tear, 0.0, 0.004)
        lx = (x + hw) / (2 * hw)
        ly = (y + hh) / (2 * hh)
        age = cv.noise(4, 82, octaves=5)
        paper_c = ramp(age, [(0.0, "#a39b84"), (1.0, "#c9c2ac")])
        ink = zero.copy()
        if kind == "paper":
            rows = np.floor(ly * 28)
            fr = ly * 28 - rows
            line = smooth(0.25 - abs(fr - 0.5), 0.0, 0.05) * (rows > 3) * (rows < 26)
            ln = cell_rand(rows, rows * 0, 5)
            line *= (lx > 0.1) * (lx < 0.6 + ln * 0.3)
            ink = np.maximum(ink, line * (0.5 + 0.5 * cv.noise(120, 83, octaves=2)))
            ink = np.maximum(ink, smooth(0.04 - abs(ly - 0.08), 0, 0.01) * (lx > 0.1) * (lx < 0.5))
        elif kind == "newspaper":
            head = (ly > 0.05) * (ly < 0.17) * (lx > 0.05) * (lx < 0.95)
            colw = np.floor(lx * 4)
            fcol = lx * 4 - colw
            rows = np.floor(ly * 60)
            fr = ly * 60 - rows
            body = (ly > 0.22) * (fcol > 0.06) * (fcol < 0.94) * smooth(0.3 - abs(fr - 0.5), 0, 0.1)
            photo = (lx > 0.06) * (lx < 0.48) * (ly > 0.24) * (ly < 0.55)
            ink = np.maximum(head * 0.95, body * 0.6 * (1 - photo))
            ink = np.maximum(ink, photo * (0.4 + 0.5 * cv.noise(20, 84, octaves=3)))
        elif kind == "poster_missing":
            head = (ly > 0.05) * (ly < 0.17) * (lx > 0.08) * (lx < 0.92)
            photo = (lx > 0.2) * (lx < 0.8) * (ly > 0.22) * (ly < 0.62)
            face = smooth(0.13 - np.sqrt(((lx - 0.5) / 0.8) ** 2 + (ly - 0.4) ** 2), 0, 0.02)
            rows = np.floor(ly * 30)
            fr = ly * 30 - rows
            text = (ly > 0.68) * (ly < 0.92) * (lx > 0.12) * (lx < 0.88) * smooth(0.25 - abs(fr - 0.5), 0, 0.1)
            ink = np.maximum(head * 0.95, text * 0.7)
            ink = np.maximum(ink, photo * (0.3 + face * 0.5 + cv.noise(25, 85, octaves=3) * 0.2))
        else:
            grad_ = ramp(ly + (cv.noise(3, 86) - 0.5) * 0.2, [(0.0, "#8a2a1a"), (0.5, "#c47a2a"), (1.0, "#2a3a6a")])
            paper_c = lerp(paper_c, grad_, 0.85)
            circle = smooth(0.18 - np.sqrt((lx - 0.5) ** 2 + (ly - 0.42) ** 2 * 0.6), 0, 0.01)
            paper_c = tint(paper_c, circle, "#e8d6a0", 0.9)
            band = (ly > 0.75) * (ly < 0.85) * (lx > 0.1) * (lx < 0.9)
            ink = band * 0.9
        c = lerp(paper_c, np.broadcast_to(hexc("#1a1816"), paper_c.shape), ink * 0.9)
        fold = smooth(0.004 - abs(y), 0, 0.004) * 0.4
        dirt_ = smooth(cv.noise(5, 87, octaves=5), 0.5, 0.85)
        c = tint(c, dirt_, "#4a4030", 0.5)
        c = tint(c, fold, "#3a352c", 1.0)
        return c, m * edge, 0.85 + zero, m * 0.3 - fold
    if kind == "cracks":
        cr = cracks(cv, 5, 91, width=0.04, density=0.9, warp=0.03)
        r, a = _radial(cv)
        m = cr * smooth(0.48 - r, 0.0, 0.2)
        return col("#151311", (n, n)), m * edge, 0.9 + zero, -m
    if kind == "bullets":
        m = zero.copy()
        h = zero.copy()
        rng = cv.rng(101)
        for _ in range(9):
            x, y = rng.uniform(0.2, 0.8, 2)
            d = np.sqrt((cv.u - x) ** 2 + (cv.v - y) ** 2)
            hole = smooth(0.012 - d, 0.0, 0.004)
            crater = smooth(0.04 + (cv.noise(40, 102) - 0.5) * 0.02 - d, 0.0, 0.01)
            m = np.maximum(m, np.maximum(hole, crater * 0.6))
            h = np.minimum(h, -hole - crater * 0.3)
        c = lerp(col("#5e5a52", (n, n)), col("#0a0908", (n, n)), smooth(-h, 0.5, 1.0))
        return c, m * edge, 0.85 + zero, h
    if kind == "graffiti":
        m = zero.copy()
        rng = cv.rng(111)
        for k in range(7):
            pts = rng.uniform(0.15, 0.85, (6, 2))
            t = np.linspace(0, 1, 120)
            for i in range(len(pts) - 1):
                p0, p1 = pts[i], pts[i + 1]
                for tt in t[::4]:
                    x = p0[0] + (p1[0] - p0[0]) * tt
                    y = p0[1] + (p1[1] - p0[1]) * tt
                    d = np.sqrt((cv.u - x) ** 2 + (cv.v - y) ** 2)
                    m = np.maximum(m, smooth(0.018 - d, 0.0, 0.01))
        drips = cv.noise(30, 112, octaves=2, aniso=(1.0, 15.0))
        m = np.maximum(m, smooth(drips, 0.8, 0.9) * smooth(cv.blur(m, 6), 0.1, 0.3) * 0.8)
        c = col("#7a0d0a", (n, n))
        return c, m * edge * 0.9, 0.6 + zero, zero
    if kind == "footprints":
        m = zero.copy()
        for k, (x, y, s) in enumerate(((0.35, 0.82, 1), (0.6, 0.6, -1), (0.38, 0.36, 1), (0.62, 0.14, -1))):
            dx = (cv.u - x) / 0.07
            dy = (cv.v - y) / 0.12
            sole = smooth(1 - np.sqrt(dx**2 + (dy + 0.3) ** 2 * 1.4), 0.0, 0.1)
            heel = smooth(1 - np.sqrt((dx * 1.2) ** 2 + ((dy - 0.75) * 2.2) ** 2), 0.0, 0.1)
            tread = smooth(np.sin(dy * 30) * 0.5 + 0.5, 0.4, 0.6)
            m = np.maximum(m, np.maximum(sole, heel) * (0.5 + 0.5 * tread) * (1 - k * 0.18))
        col_ = _blood_color(cv, 121, m * 0.2)
        return col_, m * edge * 0.9, 0.5 + zero, zero
    raise ValueError(kind)


@recipe("decals", size=2048, meters=1.0)
def decals(cv):
    from .synth import Canvas

    cell = cv.n // 4
    C3 = np.zeros((cv.n, cv.n, 3), np.float32)
    A = np.zeros((cv.n, cv.n), np.float32)
    R = np.ones((cv.n, cv.n), np.float32)
    H = np.zeros((cv.n, cv.n), np.float32)
    for i, kind in enumerate(DECAL_CELLS):
        sub = Canvas(cell, seed=cv.seed + i * 13)
        c, a, r, h = decal_cell(sub, kind)
        y, x = divmod(i, 4)
        sl = (slice(y * cell, (y + 1) * cell), slice(x * cell, (x + 1) * cell))
        C3[sl] = c
        A[sl] = np.clip(a, 0, 1)
        R[sl] = np.clip(r, 0, 1)
        H[sl] = h
    return dict(color=C3, height=H, rough=R, metal=0.0, normal=2.0, alpha=A)
