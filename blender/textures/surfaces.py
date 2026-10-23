"""Kachelbare Oberflächen (Beton, Asphalt, Ziegel, Holz, Metall, Fliesen, Haut, ...).

Alle Rauschfunktionen laufen über eine 4D-Torus-Abbildung und sind dadurch nahtlos."""

from lib import hexcol

from .registry import texture

C = hexcol


def cracks(nb, f, width=0.012, density=0.5, off=0.0):
    """Feine Risse aus Voronoi-Kanten, maskiert durch Rauschen. Liefert 0..1 (1 = Riss)."""
    u, v = nb.uv_xy()
    # Verzerrung der Koordinaten für unregelmäßige Risse
    dn, _ = nb.noise(f * 1.5, detail=3, off=off + 40)
    du = nb.add(u, nb.mul(nb.sub(dn, 0.5), 0.02))
    d, _ = nb.voronoi(f, feature="DISTANCE_TO_EDGE", off=off, uv=(du, v))
    line = nb.sub(1.0, nb.smooth(d, 0.0, width))
    mask_n, _ = nb.noise(f * 0.6, detail=2, off=off + 13)
    t = 0.62 - 0.2 * density
    mask = nb.smooth(mask_n, t, t + 0.06)
    return nb.mul(line, mask)


@texture("concrete")
def tex_concrete(nb):
    n1, _ = nb.noise(5, detail=6, rough=0.55)
    n2, _ = nb.noise(90, detail=2, off=3)
    st, _ = nb.noise(2.5, detail=4, off=7)
    stain = nb.smooth(st, 0.45, 0.68)
    cr = cracks(nb, 4, width=0.01, density=0.45, off=2)
    grime, _ = nb.noise(1.2, detail=5, rough=0.65, off=21)
    pits_d, _ = nb.voronoi(140, off=9)
    pits = nb.sub(1.0, nb.smooth(pits_d, 0.0, 0.12))
    pit_mask, _ = nb.noise(6, off=17)
    pits = nb.mul(pits, nb.smooth(pit_mask, 0.55, 0.7))
    col = nb.ramp(n1, [(0.3, C("#6f6c66")), (0.7, C("#97948c"))])
    col = nb.mix(nb.mul(nb.sub(n2, 0.5), 0.6), col, C("#b0ada5"))
    col = nb.mix(nb.mul(stain, 0.7), col, C("#4a463f"))
    col = nb.mix(nb.mul(nb.smooth(grime, 0.55, 0.8), 0.5), col, C("#3a3631"))
    col = nb.mix(nb.mul(pits, 0.6), col, C("#45423d"))
    col = nb.mix(nb.mul(cr, 0.6), col, C("#34312d"))
    h = nb.add(nb.mul(n1, 0.35), nb.mul(n2, 0.25))
    h = nb.sub(h, nb.mul(cr, 0.5))
    h = nb.sub(h, nb.mul(pits, 0.25))
    return dict(color=col, height=h)


@texture("asphalt")
def tex_asphalt(nb):
    n1, _ = nb.noise(4, detail=5, off=1)
    agg, _ = nb.voronoi(120, off=2)
    agg_rnd_d, agg_col = nb.voronoi(120, off=2)
    speck = nb.mul(nb.sub(1.0, nb.smooth(agg, 0.0, 0.35)), nb.bw(agg_col))
    wear, _ = nb.noise(2, detail=3, off=5)
    oil, _ = nb.noise(1.5, detail=2, off=8)
    oil_m = nb.smooth(oil, 0.62, 0.75)
    cr = cracks(nb, 3, width=0.009, density=0.8, off=4)
    col = nb.ramp(n1, [(0.2, C("#252527")), (0.8, C("#3b3a39"))])
    col = nb.mix(nb.mul(speck, 0.7), col, C("#6a6863"))
    col = nb.mix(nb.mul(nb.smooth(wear, 0.5, 0.8), 0.35), col, C("#55534e"))
    col = nb.mix(nb.mul(oil_m, 0.7), col, C("#151517"))
    col = nb.mix(cr, col, C("#0f0f10"))
    h = nb.add(nb.mul(speck, 0.5), nb.mul(n1, 0.3))
    h = nb.sub(h, nb.mul(cr, 0.6))
    return dict(color=col, height=h)


@texture("brick", normal_strength=6.0)
def tex_brick(nb):
    u, v = nb.uv_xy()
    rows, cols = 16.0, 4.0
    r = nb.floor(nb.mul(v, rows))
    shift = nb.mul(nb.mod(r, 2.0), 0.5 / cols)
    us = nb.sub(u, shift)
    c = nb.floor(nb.mul(us, cols))
    fu = nb.fract(nb.mul(us, cols))
    fv = nb.fract(nb.mul(v, rows))
    mortar_u = nb.minv(fu, nb.sub(1.0, fu))
    mortar_v = nb.minv(fv, nb.sub(1.0, fv))
    # Mörtelbreite in Kachel-Einheiten (horizontale Fugen relativ zur Ziegelhöhe)
    mu = nb.smooth(mortar_u, 0.012, 0.03)
    mv = nb.smooth(mortar_v, 0.05, 0.12)
    brick_mask = nb.mul(mu, mv)
    # Ziegelzentrum (periodisch) -> Farbvariation
    cu = nb.fract(nb.add(nb.div(nb.add(c, 0.5), cols), shift))
    cv = nb.div(nb.add(r, 0.5), rows)
    var, _ = nb.noise(23, detail=1, off=3, uv=(cu, cv))
    var2, _ = nb.noise(41, detail=1, off=9, uv=(cu, cv))
    grain, _ = nb.noise(60, detail=4, off=5)
    soot, _ = nb.noise(2, detail=4, off=11)
    brick_col = nb.ramp(var, [(0.25, C("#5a2a1e")), (0.5, C("#7b3b27")), (0.75, C("#8f5a3c"))])
    brick_col = nb.mix(nb.smooth(var2, 0.6, 0.75), brick_col, C("#4a2b22"))
    brick_col = nb.mix(nb.mul(nb.sub(grain, 0.4), 0.5), brick_col, C("#a0705a"))
    mortar_col = nb.ramp(grain, [(0.3, C("#6e6a62")), (0.7, C("#8d887d"))])
    col = nb.mix(brick_mask, mortar_col, brick_col)
    col = nb.mix(nb.mul(nb.smooth(soot, 0.45, 0.8), 0.6), col, C("#221e1c"))
    h = nb.add(nb.mul(brick_mask, 0.6), nb.mul(grain, 0.15))
    return dict(color=col, height=h)


@texture("planks", normal_strength=5.0)
def tex_planks(nb):
    u, v = nb.uv_xy()
    rows = 6.0
    r = nb.floor(nb.mul(v, rows))
    fv = nb.fract(nb.mul(v, rows))
    rnd, _ = nb.white(r, 1.3)
    seg_u = nb.fract(nb.add(u, rnd))
    seg = nb.floor(nb.mul(seg_u, 2.0))
    fu = nb.fract(nb.mul(seg_u, 2.0))
    plank_rnd, _ = nb.white(r, seg, 7.1)
    gap_v = nb.smooth(nb.minv(fv, nb.sub(1.0, fv)), 0.015, 0.05)
    gap_u = nb.smooth(nb.minv(fu, nb.sub(1.0, fu)), 0.004, 0.012)
    gap = nb.mul(gap_v, gap_u)
    # Maserung: stark gestrecktes Rauschen entlang u
    gr, _ = nb.noise(3, 60, detail=5, rough=0.6, distortion=0.4, off=nb_off(plank_rnd))
    rings = nb.fract(nb.mul(gr, 9.0))
    rings = nb.smooth(rings, 0.2, 0.9)
    knot_d, _ = nb.voronoi(5, 18, off=4)
    knot = nb.sub(1.0, nb.smooth(knot_d, 0.0, 0.08))
    col = nb.ramp(plank_rnd, [(0.0, C("#5c4127")), (0.5, C("#7a5634")), (1.0, C("#94714a"))])
    col = nb.mix(nb.mul(rings, 0.35), col, C("#3d2a19"))
    col = nb.mix(nb.mul(knot, 0.6), col, C("#2e1f12"))
    dirt, _ = nb.noise(2.5, detail=4, off=12)
    col = nb.mix(nb.mul(nb.smooth(dirt, 0.5, 0.8), 0.5), col, C("#2b2520"))
    col = nb.mix(gap, C("#130e0a"), col)
    h = nb.add(nb.mul(gap, 0.7), nb.mul(rings, 0.1))
    return dict(color=col, height=h)


def nb_off(sock):
    # Platzhalter: Offset muss eine Zahl sein; Variation pro Planke kommt über die Farbe
    return 0.0


@texture("wood_dark", normal_strength=2.5)
def tex_wood_dark(nb):
    gr, _ = nb.noise(2, 40, detail=5, rough=0.6, distortion=0.3, off=2)
    rings = nb.smooth(nb.fract(nb.mul(gr, 7.0)), 0.3, 0.95)
    n2, _ = nb.noise(6, detail=3, off=6)
    col = nb.ramp(n2, [(0.3, C("#2c1a10")), (0.7, C("#432817"))])
    col = nb.mix(nb.mul(rings, 0.4), col, C("#1a0f08"))
    return dict(color=col, height=nb.mul(rings, 0.4))


@texture("metal_paint", normal_strength=3.0)
def tex_metal_paint(nb):
    """Hellgraue Lackierung mit Kratzern und Rostflecken – wird im Material eingefärbt."""
    n1, _ = nb.noise(3, detail=4, off=1)
    scratch_n, _ = nb.noise(2, 120, detail=2, off=3)
    scratch = nb.smooth(nb.absv(nb.sub(scratch_n, 0.5)), 0.012, 0.0)
    scratch_m, _ = nb.noise(4, detail=2, off=5)
    scratch = nb.mul(scratch, nb.smooth(scratch_m, 0.55, 0.7))
    rust_n, _ = nb.noise(5, detail=6, rough=0.65, off=7)
    rust = nb.smooth(rust_n, 0.66, 0.74)
    rust_d, _ = nb.noise(30, detail=3, off=9)
    grime, _ = nb.noise(1.5, detail=4, off=11)
    col = nb.ramp(n1, [(0.3, C("#bdbab1")), (0.7, C("#d4d1c8"))])
    col = nb.mix(nb.mul(nb.smooth(grime, 0.4, 0.85), 0.45), col, C("#77746b"))
    col = nb.mix(nb.mul(scratch, 0.8), col, C("#8d8f91"))
    rust_col = nb.ramp(rust_d, [(0.3, C("#4a2412")), (0.7, C("#7a3d1a"))])
    col = nb.mix(rust, col, rust_col)
    h = nb.sub(0.5, nb.mul(scratch, 0.3))
    h = nb.add(h, nb.mul(rust, nb.mul(rust_d, 0.4)))
    return dict(color=col, height=h)


@texture("gunmetal", size=512, normal_strength=1.5)
def tex_gunmetal(nb):
    brushed, _ = nb.noise(4, 260, detail=2, off=1)
    n1, _ = nb.noise(3, detail=4, off=2)
    wear, _ = nb.noise(6, detail=5, off=4)
    col = nb.ramp(brushed, [(0.3, C("#1e2022")), (0.7, C("#2d3033"))])
    col = nb.mix(nb.mul(nb.smooth(wear, 0.62, 0.78), 0.8), col, C("#5b5f63"))
    col = nb.mix(nb.mul(nb.smooth(n1, 0.3, 0.1), 0.4), col, C("#141414"))
    return dict(color=col, height=nb.mul(brushed, 0.2))


@texture("polymer", size=512, normal_strength=2.0)
def tex_polymer(nb):
    n1, _ = nb.noise(160, detail=2, off=3)
    n2, _ = nb.noise(4, detail=3, off=5)
    col = nb.ramp(n1, [(0.3, C("#141416")), (0.7, C("#232326"))])
    col = nb.mix(nb.mul(nb.smooth(n2, 0.6, 0.8), 0.4), col, C("#3a3a3d"))
    return dict(color=col, height=nb.mul(n1, 0.3))


@texture("gun_wood", size=512, normal_strength=1.5)
def tex_gun_wood(nb):
    gr, _ = nb.noise(2, 30, detail=5, rough=0.6, distortion=0.5, off=4)
    rings = nb.smooth(nb.fract(nb.mul(gr, 8.0)), 0.25, 0.95)
    n2, _ = nb.noise(5, detail=3, off=8)
    col = nb.ramp(n2, [(0.3, C("#4a2814")), (0.7, C("#6a3b1f"))])
    col = nb.mix(nb.mul(rings, 0.5), col, C("#341a0b"))
    return dict(color=col, height=nb.mul(rings, 0.3))


@texture("rust", normal_strength=5.0)
def tex_rust(nb):
    n1, _ = nb.noise(4, detail=7, rough=0.7, off=1)
    n2, _ = nb.noise(25, detail=4, off=2)
    d, _ = nb.voronoi(18, off=3)
    pits = nb.sub(1.0, nb.smooth(d, 0.0, 0.3))
    paint_n, _ = nb.noise(3, detail=5, off=6)
    paint = nb.smooth(paint_n, 0.58, 0.62)
    col = nb.ramp(n1, [(0.2, C("#3b1d0f")), (0.5, C("#6e3517")), (0.8, C("#94521f"))])
    col = nb.mix(nb.mul(nb.sub(n2, 0.4), 0.6), col, C("#b06a2c"))
    col = nb.mix(nb.mul(pits, 0.5), col, C("#24120a"))
    col = nb.mix(paint, col, C("#5d6660"))
    h = nb.add(nb.mul(n1, 0.4), nb.mul(n2, 0.3))
    h = nb.sub(h, nb.mul(pits, 0.3))
    h = nb.add(h, nb.mul(paint, 0.2))
    return dict(color=col, height=h)


@texture("tiles", normal_strength=6.0)
def tex_tiles(nb):
    u, v = nb.uv_xy()
    n = 8.0
    iu = nb.floor(nb.mul(u, n))
    iv = nb.floor(nb.mul(v, n))
    checker = nb.mod(nb.add(iu, iv), 2.0)
    fu = nb.fract(nb.mul(u, n))
    fv = nb.fract(nb.mul(v, n))
    gu = nb.smooth(nb.minv(fu, nb.sub(1.0, fu)), 0.015, 0.035)
    gv = nb.smooth(nb.minv(fv, nb.sub(1.0, fv)), 0.015, 0.035)
    tile = nb.mul(gu, gv)
    rnd, _ = nb.white(iu, iv, 3.3)
    dirt, _ = nb.noise(3, detail=5, off=4)
    fine, _ = nb.noise(80, detail=2, off=6)
    cr = cracks(nb, 6, width=0.006, density=0.5, off=8)
    black = nb.mix(nb.mul(rnd, 0.3), C("#18181a"), C("#2a2a2c"))
    white = nb.mix(nb.mul(rnd, 0.3), C("#cfc8b4"), C("#ddd6c2"))
    col = nb.mix(checker, white, black)
    col = nb.mix(nb.mul(nb.sub(fine, 0.5), 0.3), col, C("#8f8a7d"))
    col = nb.mix(tile, C("#4e4a43"), col)
    col = nb.mix(nb.mul(nb.smooth(dirt, 0.45, 0.85), 0.6), col, C("#3a342b"))
    col = nb.mix(nb.mul(cr, 0.8), col, C("#1a1814"))
    h = nb.add(nb.mul(tile, 0.6), nb.mul(fine, 0.05))
    h = nb.sub(h, nb.mul(cr, 0.3))
    return dict(color=col, height=h)


@texture("plaster", normal_strength=3.0)
def tex_plaster(nb):
    n1, _ = nb.noise(6, detail=6, off=1)
    n2, _ = nb.noise(50, detail=3, off=2)
    damp, _ = nb.noise(1.5, detail=5, rough=0.6, off=3)
    damp_m = nb.smooth(damp, 0.47, 0.6)
    peel_n, _ = nb.noise(4, detail=6, rough=0.65, off=5)
    peel = nb.smooth(peel_n, 0.6, 0.62)
    peel_edge = nb.sub(nb.smooth(peel_n, 0.57, 0.6), peel)
    col = nb.ramp(n1, [(0.3, C("#9fa48b")), (0.7, C("#b4b89f"))])
    col = nb.mix(nb.mul(nb.sub(n2, 0.4), 0.3), col, C("#c8cbb5"))
    col = nb.mix(nb.mul(damp_m, 0.6), col, C("#6f6545"))
    col = nb.mix(peel, col, C("#7d7465"))
    col = nb.mix(nb.mul(peel_edge, 0.6), col, C("#3d3a33"))
    h = nb.add(nb.mul(n2, 0.2), nb.mul(nb.sub(1.0, peel), 0.3))
    return dict(color=col, height=h)


@texture("dirt", normal_strength=4.0)
def tex_dirt(nb):
    n1, _ = nb.noise(4, detail=6, rough=0.6, off=1)
    n2, _ = nb.noise(40, detail=3, off=2)
    d, pc = nb.voronoi(60, off=3)
    pebble = nb.mul(nb.sub(1.0, nb.smooth(d, 0.1, 0.4)), nb.smooth(nb.bw(pc), 0.45, 0.6))
    wet, _ = nb.noise(2, detail=4, off=5)
    col = nb.ramp(n1, [(0.3, C("#3a2d20")), (0.7, C("#57452f"))])
    col = nb.mix(nb.mul(nb.sub(n2, 0.4), 0.5), col, C("#6b5841"))
    col = nb.mix(nb.mul(pebble, 0.8), col, C("#77706a"))
    col = nb.mix(nb.mul(nb.smooth(wet, 0.55, 0.75), 0.6), col, C("#241b13"))
    h = nb.add(nb.mul(n1, 0.3), nb.mul(n2, 0.2))
    h = nb.add(h, nb.mul(pebble, 0.5))
    return dict(color=col, height=h)


@texture("grass_dead", normal_strength=4.0)
def tex_grass_dead(nb):
    blades, _ = nb.noise(70, 14, detail=3, off=1)
    blades2, _ = nb.noise(110, 20, detail=2, off=2)
    patch, _ = nb.noise(3, detail=4, off=3)
    col = nb.ramp(patch, [(0.3, C("#4d4630")), (0.7, C("#7a6d45"))])
    col = nb.mix(nb.mul(nb.smooth(blades, 0.5, 0.8), 0.6), col, C("#9a8a5a"))
    col = nb.mix(nb.mul(nb.smooth(blades2, 0.45, 0.2), 0.7), col, C("#2a2618"))
    h = nb.add(nb.mul(blades, 0.5), nb.mul(blades2, 0.3))
    return dict(color=col, height=h)


@texture("fabric", size=512, normal_strength=3.0)
def tex_fabric(nb):
    u, v = nb.uv_xy()
    weave = nb.mul(nb.add(nb.sin(nb.mul(u, 6.2832 * 90)), 1.0), nb.add(nb.sin(nb.mul(v, 6.2832 * 90)), 1.0))
    weave = nb.mul(weave, 0.25)
    n1, _ = nb.noise(3, detail=5, off=1)
    stain, _ = nb.noise(2, detail=5, rough=0.65, off=2)
    blood, _ = nb.noise(2.5, detail=6, rough=0.7, off=6)
    col = nb.ramp(n1, [(0.3, C("#8c8b86")), (0.7, C("#a5a49e"))])
    col = nb.mix(nb.mul(weave, 0.25), col, C("#6e6d68"))
    col = nb.mix(nb.mul(nb.smooth(stain, 0.5, 0.75), 0.6), col, C("#4b463c"))
    col = nb.mix(nb.smooth(blood, 0.6, 0.63), col, C("#5a1410"))
    return dict(color=col, height=nb.add(nb.mul(weave, 0.3), nb.mul(n1, 0.2)))


@texture("skin", normal_strength=4.0)
def tex_skin(nb):
    n1, _ = nb.noise(4, detail=6, rough=0.6, off=1)
    n2, _ = nb.noise(30, detail=3, off=2)
    blotch, _ = nb.noise(3, detail=5, off=3)
    bruise, _ = nb.noise(2.2, detail=4, off=4)
    vein_d, _ = nb.voronoi(9, feature="DISTANCE_TO_EDGE", off=5)
    veins = nb.sub(1.0, nb.smooth(vein_d, 0.0, 0.025))
    vein_m, _ = nb.noise(3, detail=2, off=6)
    veins = nb.mul(veins, nb.smooth(vein_m, 0.5, 0.7))
    wound, _ = nb.noise(3, detail=6, rough=0.7, off=8)
    wound_m = nb.smooth(wound, 0.63, 0.66)
    col = nb.ramp(n1, [(0.3, C("#868b72")), (0.7, C("#a2a58b"))])
    col = nb.mix(nb.mul(nb.smooth(blotch, 0.5, 0.75), 0.6), col, C("#5e6447"))
    col = nb.mix(nb.mul(nb.smooth(bruise, 0.52, 0.66), 0.65), col, C("#5d4146"))
    col = nb.mix(nb.mul(veins, 0.5), col, C("#3f3341"))
    col = nb.mix(nb.mul(nb.sub(n2, 0.4), 0.25), col, C("#aaa58c"))
    col = nb.mix(wound_m, col, C("#4f0b0b"))
    h = nb.add(nb.mul(n2, 0.3), nb.mul(veins, 0.2))
    h = nb.sub(h, nb.mul(wound_m, 0.3))
    return dict(color=col, height=h)


@texture("pap_camo", size=512, normal_strength=0.0)
def tex_pap_camo(nb):
    d, cc = nb.voronoi(7, off=1)
    e, _ = nb.voronoi(7, feature="DISTANCE_TO_EDGE", off=1)
    n1, _ = nb.noise(5, detail=4, distortion=1.5, off=3)
    hue = nb.fract(nb.add(nb.bw(cc), nb.mul(n1, 0.6)))
    col = nb.ramp(
        hue,
        [
            (0.0, C("#2a0a5e")),
            (0.25, C("#7a1fd0")),
            (0.5, C("#d23a8a")),
            (0.7, C("#1c6fe0")),
            (0.9, C("#32d1c4")),
            (1.0, C("#2a0a5e")),
        ],
    )
    edge = nb.sub(1.0, nb.smooth(e, 0.0, 0.05))
    col = nb.mix(nb.mul(edge, 0.8), col, C("#f4e7ff"))
    return dict(color=col)


@texture("chainlink", size=512, normal_strength=0.0)
def tex_chainlink(nb):
    u, v = nb.uv_xy()
    n = 8.0
    a = nb.fract(nb.mul(nb.add(u, v), n))
    b = nb.fract(nb.mul(nb.sub(u, v), n))
    wa = nb.smooth(nb.minv(a, nb.sub(1.0, a)), 0.05, 0.02)
    wb = nb.smooth(nb.minv(b, nb.sub(1.0, b)), 0.05, 0.02)
    wire = nb.maxv(wa, wb)
    n1, _ = nb.noise(6, detail=3, off=2)
    col = nb.ramp(n1, [(0.3, C("#5d5f5f")), (0.7, C("#8c8f8e"))])
    rust = nb.smooth(n1, 0.6, 0.7)
    col = nb.mix(rust, col, C("#6b3a1c"))
    return dict(color=col, alpha=wire)
