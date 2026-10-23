"""Perk-Automaten (eigene Designs): Getränkeautomat mit Leuchtschild, Glasfront mit
Flaschen, Ausgabefach, Kühlrippen. Jeder Perk hat Farbe, Namen und Emblem.
Modelle schauen nach +Y, Ursprung am Boden in der Mitte -> models/perk_<id>.gltf"""

import math

from lib import MB, hexcol

from .common import C, export, fresh, glow, mats, paint, text_obj

PERKS = {
    "revive": ("ZWEITER ATEM", "#2f78c9", "#8fd0ff", "cross"),
    "jugg": ("PANZER-TONIKUM", "#a3151a", "#ff6a55", "shield"),
    "speed": ("TURBO-COLA", "#1f7a2e", "#7dff8a", "bolt"),
    "doubletap": ("DOPPELSCHUSS", "#a8741a", "#ffd36a", "bullets"),
    "staminup": ("AUSDAUER-LIMO", "#c75b12", "#ffb45a", "arrow"),
    "mulekick": ("MAULTIER-MALZ", "#4f6b1c", "#c8f070", "shoe"),
}


def emblem(mb, kind, center, s, mat):
    x, y, z = center
    if kind == "cross":
        mb.box((x, y, z), (s * 0.3, 0.02, s), mat, bevel=0.004)
        mb.box((x, y, z), (s, 0.02, s * 0.3), mat, bevel=0.004)
    elif kind == "shield":
        pts = [(-s * 0.5, s * 0.45), (s * 0.5, s * 0.45), (s * 0.45, -s * 0.1), (0.0, -s * 0.55), (-s * 0.45, -s * 0.1)]
        mb.extrude(pts, 0.025, mat, plane="XZ", center=(x, y, z), bevel=0.004)
    elif kind == "bolt":
        pts = [(-s * 0.1, s * 0.5), (s * 0.3, s * 0.5), (s * 0.05, s * 0.05), (s * 0.3, s * 0.05), (-s * 0.25, -s * 0.55), (-s * 0.02, -s * 0.05), (-s * 0.28, -s * 0.05)]
        mb.extrude(pts, 0.025, mat, plane="XZ", center=(x, y, z))
    elif kind == "bullets":
        for dx in (-s * 0.2, s * 0.2):
            mb.lathe([(0.0, -s * 0.5), (s * 0.13, -s * 0.5), (s * 0.13, s * 0.1), (s * 0.1, s * 0.3), (0.0, s * 0.5)], mat, seg=14, center=(x + dx, y, z))
    elif kind == "arrow":
        pts = [(-s * 0.5, -s * 0.12), (s * 0.1, -s * 0.12), (s * 0.1, -s * 0.35), (s * 0.55, 0.0), (s * 0.1, s * 0.35), (s * 0.1, s * 0.12), (-s * 0.5, s * 0.12)]
        mb.extrude(pts, 0.025, mat, plane="XZ", center=(x, y, z))
    elif kind == "shoe":
        # Hufeisen aus Segmenten
        n = 9
        for i in range(n):
            a0 = math.radians(-30 + 240 * i / n)
            a1 = math.radians(-30 + 240 * (i + 1) / n)
            p0 = (x + math.cos(a0 + math.pi / 2) * s * 0.4, y, z + math.sin(a0 + math.pi / 2) * s * 0.4 - s * 0.05)
            p1 = (x + math.cos(a1 + math.pi / 2) * s * 0.4, y, z + math.sin(a1 + math.pi / 2) * s * 0.4 - s * 0.05)
            mb.cyl_between(p0, p1, s * 0.09, mat, seg=8)


def build(pid):
    name, base_hex, glow_hex, kind = PERKS[pid]
    fresh()
    m = mats()
    body_m = paint(f"P_{pid}_Paint", C(base_hex))
    trim_m = paint(f"P_{pid}_Trim", tuple(c * 0.35 for c in C(base_hex)[:3]) + (1.0,))
    glow_m = glow(f"P_{pid}_Glow", C(glow_hex), 8.0)
    inner_m = glow(f"P_{pid}_Inner", tuple(c * 0.6 for c in C(glow_hex)[:3]) + (1.0,), 2.5)
    label_m = glow(f"P_{pid}_Label", tuple(c * 0.12 for c in C(base_hex)[:3]) + (1.0,), 0.6)
    W, D, H = 1.0, 0.78, 2.0
    mb = MB(f"Perk_{pid}", uv_scale=0.5)
    # Sockel und Korpus
    mb.box((0, 0, 0.05), (W + 0.04, D + 0.04, 0.1), m["dark"], bevel=0.01)
    mb.box((0, -0.02, 0.1 + H / 2), (W, D - 0.04, H), body_m, bevel=0.03)
    # Seitenleisten und Kühlrippen
    for sx in (-1, 1):
        mb.box((sx * (W / 2 + 0.012), 0, 0.1 + H / 2), (0.03, D * 0.9, H * 0.96), trim_m, bevel=0.008)
        for k in range(8):
            mb.box((sx * (W / 2 + 0.03), -0.2, 0.4 + k * 0.08), (0.012, 0.25, 0.03), m["metal"])
    # Glasfront mit beleuchtetem Innenraum und Flaschen
    gz0, gz1 = 0.75, 1.75
    mb.box((0, D / 2 - 0.06, (gz0 + gz1) / 2), (W * 0.74, 0.04, gz1 - gz0), inner_m)
    for row in range(3):
        zr = gz0 + 0.06 + row * 0.32
        mb.box((0, D / 2 - 0.12, zr - 0.02), (W * 0.72, 0.18, 0.015), m["metal"])
        for col in range(5):
            xb = -W * 0.3 + col * W * 0.15
            mb.lathe([(0.001, 0.0), (0.028, 0.0), (0.03, 0.12), (0.014, 0.17), (0.011, 0.21), (0.001, 0.215)], m["glass"] if (row + col) % 4 == 0 else body_m, seg=10, center=(xb, D / 2 - 0.12, zr))
    mb.box((0, D / 2 - 0.005, (gz0 + gz1) / 2), (W * 0.78, 0.02, gz1 - gz0 + 0.04), m["glass"])
    mb.box((0, D / 2 + 0.005, gz1 + 0.03), (W * 0.82, 0.03, 0.05), m["chrome"], bevel=0.008)
    mb.box((0, D / 2 + 0.005, gz0 - 0.03), (W * 0.82, 0.03, 0.05), m["chrome"], bevel=0.008)
    # Ausgabefach und Münzschlitz
    mb.box((0, D / 2 - 0.01, 0.42), (W * 0.5, 0.06, 0.2), m["dark"], bevel=0.01)
    mb.box((0, D / 2 + 0.005, 0.42), (W * 0.44, 0.04, 0.14), m["rubber"])
    mb.box((W * 0.33, D / 2 + 0.002, 1.0), (0.1, 0.03, 0.2), m["chrome"], bevel=0.006)
    mb.box((W * 0.33, D / 2 + 0.02, 1.03), (0.04, 0.01, 0.006), m["rubber"])
    # Leuchtschild oben (Kasten mit Rahmen), Emblem
    mb.box((0, 0.02, 2.28), (W + 0.08, D * 0.7, 0.36), trim_m, bevel=0.02)
    mb.box((0, D * 0.35 + 0.025, 2.28), (W * 0.98, 0.02, 0.3), label_m)
    emblem(mb, kind, (0, D / 2 + 0.015, 1.92), 0.2, glow_m)
    mb.to_object(smooth_angle=40)
    # Name als Leuchtschrift
    text_obj(f"PerkName_{pid}", name, 0.1 if len(name) < 13 else 0.082, 0.006, glow(f"P_{pid}_Text", C(glow_hex), 14.0), (0, D * 0.35 + 0.042, 2.28))
    return export(f"perk_{pid}")


def build_all():
    for pid in PERKS:
        build(pid)
