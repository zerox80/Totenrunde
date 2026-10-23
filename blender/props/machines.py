"""Waffen-Verstärker (Pack-a-Punch) und Stromkasten."""

import math

from lib import MB

from .common import C, export, fresh, glow, mats, text_obj


def build_pap():
    fresh()
    m = mats()
    purple = glow("PaP_Glow", C("#b36bff"), 10.0)
    soft = glow("PaP_Soft", C("#5a2a8a"), 2.0)
    W, D, H = 1.7, 1.0, 1.25
    mb = MB("PackAPunch", uv_scale=0.5)
    # Unterbau und Gehäuse (Industriemaschine)
    mb.box((0, 0, 0.08), (W + 0.06, D + 0.06, 0.16), m["dark"], bevel=0.015)
    mb.box((0, -0.05, 0.16 + 0.45), (W, D - 0.1, 0.9), m["dark"], bevel=0.03)
    mb.box((0, -0.1, 1.06 + 0.1), (W * 0.9, D * 0.7, 0.2), m["metal"], bevel=0.02)
    # Einzug vorn: Schlitz mit Walzen und Leuchten
    mb.box((0, D / 2 - 0.08, 1.05), (W * 0.62, 0.12, 0.16), m["rubber"])
    for k in range(2):
        mb.cyl((0, D / 2 - 0.02, 0.98 + k * 0.14), 0.045, W * 0.6, m["chrome"], axis="X", seg=16)
    mb.box((0, D / 2 - 0.03, 1.05), (W * 0.58, 0.02, 0.05), purple)
    # Zahnräder an den Seiten, Rohre, Kurbel
    for sx in (-1, 1):
        for k in range(10):
            a = math.tau * k / 10
            mb.box((sx * (W / 2 + 0.02), -0.05 + math.cos(a) * 0.26, 0.62 + math.sin(a) * 0.26), (0.05, 0.08, 0.08), m["rust"], rot=(math.degrees(a), 0, 0))
        mb.cyl((sx * (W / 2 + 0.03), -0.05, 0.62), 0.24, 0.05, m["rust"], axis="X", seg=20)
        mb.cyl((sx * (W / 2 + 0.06), -0.05, 0.62), 0.07, 0.06, m["brass"], axis="X", seg=14)
    for x in (-0.55, -0.2, 0.25, 0.6):
        mb.cyl((x, -0.38, 1.25 + 0.25), 0.05, 0.5, m["metal"], seg=12)
        mb.cyl((x, -0.38, 1.76), 0.065, 0.04, m["rust"], seg=12)
    mb.cyl_between((W / 2 + 0.06, 0.25, 0.9), (W / 2 + 0.25, 0.25, 1.2), 0.025, m["metal"])
    mb.sphere((W / 2 + 0.25, 0.25, 1.2), 0.05, m["rubber"])
    # Leuchtstreifen und Bedienfeld
    for z in (0.3, 0.9):
        mb.box((0, D / 2 - 0.095, z), (W * 0.9, 0.02, 0.03), soft)
    mb.box((-W * 0.32, D / 2 - 0.09, 0.62), (0.3, 0.04, 0.32), m["metal"], bevel=0.01)
    for k in range(3):
        mb.cyl((-W * 0.32 - 0.08 + k * 0.08, D / 2 - 0.06, 0.66), 0.022, 0.03, purple if k == 1 else m["brass"], axis="Y", seg=12)
    mb.to_object(smooth_angle=40)
    text_obj("PaPName", "WAFFEN-VERSTÄRKER", 0.075, 0.01, purple, (0, D / 2 - 0.08, 0.45))
    return export("pap")


def build_power_switch():
    fresh()
    m = mats()
    warn = glow("Power_Warn", C("#ffcc44"), 2.0)
    mb = MB("PowerSwitch", uv_scale=0.4)
    # Kasten an der Wand: Rückseite bei y=0, Vorderseite bei y=0.22
    mb.box((0, 0.11, 0), (0.7, 0.22, 0.9), m["dark"], bevel=0.015)
    mb.box((0, 0.225, 0.0), (0.62, 0.02, 0.82), m["metal"], bevel=0.006)
    # Lager für den Hebel, Kabelrohre nach oben und unten
    mb.box((0.18, 0.25, 0.0), (0.1, 0.05, 0.14), m["rust"], bevel=0.008)
    for z in (0.45, -0.45):
        mb.cyl((-0.2, 0.06, z + (0.3 if z > 0 else -0.3)), 0.035, 0.6, m["metal"], seg=10)
    # Warnschild
    mb.box((-0.12, 0.24, 0.22), (0.26, 0.01, 0.18), warn)
    mb.to_object(smooth_angle=35)
    text_obj("PowerTxt", "STROM", 0.06, 0.006, m["rubber"], (-0.12, 0.25, 0.22))
    return export("power_switch")


def build_all():
    build_pap()
    build_power_switch()
