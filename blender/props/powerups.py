"""Power-up-Symbole (goldgrün leuchtend), Ursprung in der Mitte -> models/pu_*.gltf"""

import math

from lib import MB

from .common import C, export, fresh, glow, text_obj


def gold():
    return glow("PU_Gold", C("#e6c45a"), 3.5)


def build(kind):
    fresh()
    g = gold()
    mb = MB(f"PU_{kind}", uv_scale=0.2)
    if kind == "maxammo":
        mb.box((0, 0, 0), (0.34, 0.2, 0.2), g, bevel=0.015)
        mb.box((0, 0, 0.11), (0.1, 0.06, 0.02), g)
        for k in range(4):
            mb.lathe([(0.001, 0.0), (0.02, 0.0), (0.02, 0.1), (0.015, 0.13), (0.001, 0.15)], g, seg=10, center=(-0.09 + k * 0.06, 0.0, 0.1))
    elif kind == "instakill":
        mb.sphere((0, 0, 0.03), 0.14, g, seg=18, rings=12, scale=(1, 0.95, 1))
        mb.box((0, 0.02, -0.12), (0.16, 0.14, 0.08), g, bevel=0.02)
        for sx in (-1, 1):
            mb.sphere((sx * 0.05, 0.12, 0.03), 0.04, glow("PU_Dark", C("#1a1a0a"), 0.2), seg=10, rings=6)
        mb.box((0, 0.13, -0.05), (0.03, 0.02, 0.04), glow("PU_Dark", C("#1a1a0a"), 0.2))
    elif kind == "double":
        mb.to_object()
        text_obj("x2", "x2", 0.32, 0.05, g, (0, 0, 0))
        return export("pu_double")
    elif kind == "nuke":
        mb.sphere((0, 0, 0), 0.13, g, seg=18, rings=12, scale=(1.8, 1, 1))
        for k in range(4):
            a = math.tau * k / 4
            mb.box((-0.27, math.cos(a) * 0.06, math.sin(a) * 0.06), (0.1, 0.012, 0.08), g, rot=(math.degrees(a), 0, 0))
        mb.cyl((-0.22, 0, 0), 0.06, 0.08, g, axis="X", seg=14)
    elif kind == "carpenter":
        mb.cyl_between((0, 0, -0.18), (0, 0, 0.12), 0.025, g, seg=10)
        mb.box((0, 0, 0.14), (0.24, 0.07, 0.07), g, bevel=0.01)
        mb.box((-0.15, 0, 0.16), (0.08, 0.05, 0.03), g, rot=(0, -25, 0))
    elif kind == "firesale":
        pts = [(-0.18, -0.12), (0.12, -0.12), (0.2, 0.0), (0.12, 0.12), (-0.18, 0.12)]
        mb.extrude(pts, 0.03, g, plane="XZ", center=(0, 0, 0), bevel=0.006)
        mb.to_object()
        text_obj("pct", "%", 0.14, 0.02, glow("PU_Red", C("#ff4a3a"), 4.0), (-0.03, 0.02, 0.0))
        return export("pu_firesale")
    mb.to_object(smooth_angle=50)
    return export(f"pu_{kind}")


def build_all():
    for k in ("maxammo", "instakill", "double", "nuke", "carpenter", "firesale"):
        build(k)
