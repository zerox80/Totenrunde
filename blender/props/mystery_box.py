"""Mysteriöse Kiste (Holztruhe mit Metallbeschlägen und leuchtenden Fragezeichen),
Deckel separat (Ursprung am Scharnier), Teddy."""

from lib import MB

from .common import C, export, fresh, glow, mats, text_obj

W, D, H = 1.25, 0.6, 0.5


def build_box():
    fresh()
    m = mats()
    q = glow("Box_Glow", C("#9fd8ff"), 6.0)
    mb = MB("MysteryBox", uv_scale=0.4)
    mb.box((0, 0, H / 2), (W, D, H), m["wood"], bevel=0.012)
    # Beschläge: Kanten, Ecken, Bänder
    for sx in (-1, 1):
        for sy in (-1, 1):
            mb.box((sx * (W / 2 - 0.02), sy * (D / 2 - 0.02), H / 2), (0.05, 0.05, H + 0.01), m["rust"], bevel=0.006)
    for x in (-0.35, 0.35):
        mb.box((x, 0, H / 2), (0.06, D + 0.012, H + 0.004), m["rust"])
    mb.box((0, D / 2 + 0.008, H - 0.06), (0.12, 0.02, 0.1), m["brass"], bevel=0.005)
    mb.to_object(smooth_angle=35)
    for sx in (-1, 1):
        text_obj(f"Q{sx}", "?", 0.22, 0.01, q, (sx * 0.17, D / 2 + 0.008, H * 0.45))
    return export("mystery_box")


def build_lid():
    fresh()
    m = mats()
    mb = MB("MysteryBoxLid", uv_scale=0.4)
    # Ursprung = Scharnier an der hinteren Oberkante; Deckel reicht nach vorn (+Y)
    mb.box((0, D / 2, 0.04), (W + 0.02, D + 0.02, 0.08), m["wood"], bevel=0.01)
    for x in (-0.35, 0.35):
        mb.box((x, D / 2, 0.045), (0.06, D + 0.03, 0.085), m["rust"])
    for sx in (-1, 1):
        mb.cyl((sx * 0.45, -0.005, 0.0), 0.018, 0.08, m["metal"], axis="X")
    mb.to_object(smooth_angle=35)
    return export("mystery_box_lid")


def build_teddy():
    fresh()
    from lib import material

    fur = material("Teddy_Fur", tex="fabric", normal="fabric_n", color=C("#8a5a34"), rough=1.0)
    dark = material("Teddy_Dark", color=C("#1a120c"), rough=0.4)
    light = material("Teddy_Light", tex="fabric", color=C("#c79a6a"), rough=1.0)
    mb = MB("Teddy", uv_scale=0.1)
    mb.sphere((0, 0, 0.16), 0.12, fur, seg=16, rings=10, scale=(1, 0.85, 1.15))
    mb.sphere((0, 0.02, 0.34), 0.095, fur, seg=16, rings=10)
    mb.sphere((0, 0.1, 0.32), 0.04, light, seg=12, rings=8)
    mb.sphere((0, 0.135, 0.335), 0.012, dark, seg=8, rings=6)
    for sx in (-1, 1):
        mb.sphere((sx * 0.07, 0.0, 0.42), 0.035, fur, seg=10, rings=6, scale=(1, 0.6, 1))
        mb.sphere((sx * 0.035, 0.085, 0.37), 0.012, dark, seg=8, rings=6)
        mb.sphere((sx * 0.13, 0.03, 0.2), 0.045, fur, seg=10, rings=6, scale=(1, 1, 1.6))
        mb.sphere((sx * 0.07, 0.06, 0.04), 0.05, fur, seg=10, rings=6, scale=(1, 1.4, 0.8))
    mb.to_object(smooth_angle=80)
    return export("teddy")


def build_all():
    build_box()
    build_lid()
    build_teddy()
