"""Effekt-Sprites mit Alpha (Mündungsfeuer, Rauch, Flammen, Blut, Funken, Lichtstrahl)."""

from lib import hexcol

from .registry import texture

C = hexcol


def polar(nb):
    u, v = nb.uv_xy()
    x = nb.sub(nb.mul(u, 2.0), 1.0)
    y = nb.sub(nb.mul(v, 2.0), 1.0)
    r = nb.math("SQRT", nb.add(nb.mul(x, x), nb.mul(y, y)))
    ang = nb.math("ARCTAN2", y, x)
    return x, y, r, ang


@texture("fx_glow", size=256, normal_strength=0.0)
def tex_fx_glow(nb):
    x, y, r, a = polar(nb)
    g = nb.pow(nb.clamp01(nb.sub(1.0, r)), 2.2)
    return dict(color=C("#ffffff"), alpha=g)


@texture("fx_muzzle", size=256, normal_strength=0.0)
def tex_fx_muzzle(nb):
    x, y, r, a = polar(nb)
    # Sternform: Radius moduliert mit dem Winkel
    star = nb.add(0.25, nb.mul(nb.pow(nb.absv(nb.cos(nb.mul(a, 3.0))), 6.0), 0.75))
    n1, _ = nb.noise(5, detail=4, off=2)
    star = nb.mul(star, nb.add(0.75, nb.mul(n1, 0.5)))
    m = nb.smooth(nb.div(r, star), 1.0, 0.2)
    core = nb.smooth(r, 0.35, 0.0)
    col = nb.mix(core, C("#ff9a2a"), C("#fff4c8"))
    return dict(color=col, alpha=nb.clamp01(nb.add(m, core)))


@texture("fx_smoke", size=256, normal_strength=0.0)
def tex_fx_smoke(nb):
    x, y, r, a = polar(nb)
    n1, _ = nb.noise(4, detail=6, rough=0.6, off=3)
    m = nb.smooth(nb.add(r, nb.mul(nb.sub(n1, 0.5), 0.6)), 0.95, 0.3)
    col = nb.ramp(n1, [(0.3, C("#5a5652")), (0.7, C("#8d8883"))])
    return dict(color=col, alpha=nb.mul(m, 0.85))


@texture("fx_flame", size=256, normal_strength=0.0)
def tex_fx_flame(nb):
    u, v = nb.uv_xy()
    x = nb.sub(nb.mul(u, 2.0), 1.0)
    n1, _ = nb.noise(3, 2, detail=5, rough=0.6, off=4)
    width = nb.mul(nb.sub(1.0, v), 0.75)
    wob = nb.mul(nb.sub(n1, 0.5), nb.mul(v, 0.9))
    d = nb.absv(nb.add(x, wob))
    shape = nb.smooth(nb.div(d, nb.add(width, 0.02)), 1.0, 0.35)
    shape = nb.mul(shape, nb.smooth(v, 0.0, 0.08))
    heat = nb.mul(shape, nb.sub(1.0, v))
    col = nb.ramp(heat, [(0.0, C("#7a1404")), (0.35, C("#ff5a0a")), (0.7, C("#ffb43a")), (1.0, C("#fff2b0"))])
    return dict(color=col, alpha=shape)


@texture("fx_blood", size=256, normal_strength=0.0)
def tex_fx_blood(nb):
    x, y, r, a = polar(nb)
    n1, _ = nb.noise(3, detail=6, rough=0.7, off=7)
    n2, _ = nb.noise(12, detail=3, off=8)
    spikes = nb.mul(nb.pow(nb.absv(nb.sin(nb.mul(a, 5.0))), 8.0), 0.35)
    rr = nb.add(nb.add(0.45, spikes), nb.mul(nb.sub(n1, 0.5), 0.5))
    m = nb.smooth(nb.div(r, rr), 1.0, 0.85)
    d, _ = nb.voronoi(10, off=9)
    drops = nb.mul(nb.sub(1.0, nb.smooth(d, 0.05, 0.12)), nb.smooth(r, 0.5, 0.7))
    drops = nb.mul(drops, nb.smooth(r, 0.98, 0.9))
    alpha = nb.clamp01(nb.add(m, drops))
    col = nb.ramp(n2, [(0.3, C("#3a0605")), (0.7, C("#6a0c0a"))])
    return dict(color=col, alpha=alpha)


@texture("fx_spark", size=128, normal_strength=0.0)
def tex_fx_spark(nb):
    x, y, r, a = polar(nb)
    line = nb.smooth(nb.absv(y), 0.25, 0.0)
    m = nb.mul(line, nb.smooth(nb.absv(x), 1.0, 0.2))
    return dict(color=C("#ffd890"), alpha=m)


@texture("fx_beam", size=128, normal_strength=0.0)
def tex_fx_beam(nb):
    u, v = nb.uv_xy()
    x = nb.sub(nb.mul(u, 2.0), 1.0)
    side = nb.pow(nb.clamp01(nb.sub(1.0, nb.absv(x))), 1.5)
    fade = nb.mul(nb.smooth(v, 0.0, 0.15), nb.smooth(v, 1.0, 0.4))
    return dict(color=C("#ffffff"), alpha=nb.mul(side, fade))
