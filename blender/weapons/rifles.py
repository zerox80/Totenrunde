"""Gewehre: M14, Galil, RPD (MG), DSR 50 (Scharfschützengewehr)."""

import math  # noqa: F401

from mathutils import Vector  # noqa: F401

from lib import MB, material

from .common import (  # noqa: F401
    C,
    UVS,
    arm,
    finish,
    fist,
    left_hand_pistol,
    left_hand_support,
    mats,
    pistol_grip,
    right_hand_on_grip,
    ring,
    smg_receiver,
)



def build_m14():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    # Holzschaft (Seitenprofil)
    stock = [(-0.43, -0.03), (-0.2, -0.045), (-0.1, -0.05), (0.33, -0.045), (0.33, -0.085), (0.05, -0.09), (-0.03, -0.085), (-0.1, -0.115), (-0.2, -0.13), (-0.43, -0.16)]
    body.extrude(stock, 0.04, m["wood"], bevel=0.004)
    body.box((0, -0.432, -0.095), (0.042, 0.012, 0.13), m["metal"], uv=UVS)  # Schaftkappe
    # System
    body.extrude([(-0.07, -0.046), (0.15, -0.046), (0.15, -0.014), (0.1, -0.012), (-0.065, -0.012)], 0.032, m["metal"], bevel=0.002)
    body.box((0.018, 0.05, -0.03), (0.008, 0.07, 0.008), m["metal_l"], uv=UVS)  # Spannhebel
    # Diopter
    for x in (-0.007, 0.007):
        body.box((x, -0.05, -0.003), (0.004, 0.012, 0.018), m["black"], uv=UVS)
    body.box((0, -0.05, -0.011), (0.018, 0.012, 0.004), m["black"], uv=UVS)
    # Lauf + Gasrohr + Korn + Mündungsfeuerdämpfer
    body.cyl((0, 0.52, -0.03), 0.0105, 0.42, m["metal"], axis="Y", seg=14, uv=UVS)
    body.cyl((0, 0.45, -0.048), 0.007, 0.25, m["metal"], axis="Y", seg=10, uv=UVS)
    body.box((0, 0.69, -0.012), (0.004, 0.01, 0.022), m["black"], uv=UVS)
    body.cyl((0, 0.71, -0.028), 0.014, 0.05, m["black"], axis="Y", seg=12, uv=UVS)
    body.box((0, 0.24, -0.036), (0.034, 0.16, 0.024), m["wood_d"], bevel=0.003, uv=UVS)  # Handschutz oben
    # Abzug
    body.extrude([(-0.03, -0.09), (0.03, -0.09), (0.028, -0.115), (-0.025, -0.117)], 0.01, m["metal"])
    body.box((0, -0.005, -0.1), (0.006, 0.006, 0.016), m["black"], uv=UVS)
    # Magazin
    mag.extrude([(0.03, -0.09), (0.1, -0.09), (0.11, -0.19), (0.045, -0.195)], 0.026, m["black"], bevel=0.002)
    right_hand_on_grip(body, m, (0.006, -0.11, -0.11), grip_tilt=35, elbow=(0.15, -0.42, -0.3))
    left_hand_support(arml, m, (-0.012, 0.2, -0.1), elbow=(-0.23, -0.12, -0.32), rot=(0, 0, 30))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("m14", None, (0, 0.74, -0.028))


def build_galil():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    smg_receiver(body, m, (-0.13, 0.15), (-0.06, -0.018), 0.036)
    body.box((0, 0.0, -0.014), (0.024, 0.26, 0.012), m["metal"], bevel=0.003, uv=UVS)
    body.extrude([(0.15, -0.025), (0.34, -0.025), (0.34, -0.07), (0.16, -0.075)], 0.046, m["poly"], bevel=0.005)
    for i in range(5):
        body.box((0.024, 0.18 + i * 0.035, -0.048), (0.004, 0.018, 0.025), m["black"], uv=UVS)
        body.box((-0.024, 0.18 + i * 0.035, -0.048), (0.004, 0.018, 0.025), m["black"], uv=UVS)
    body.cyl((0, 0.45, -0.042), 0.01, 0.22, m["metal"], axis="Y", seg=12, uv=UVS)
    body.box((0, 0.41, -0.015), (0.014, 0.03, 0.05), m["metal"], uv=UVS)  # Kornträger
    body.box((0, 0.41, 0.0), (0.004, 0.006, 0.01), m["black"], uv=UVS)
    body.cyl((0, 0.57, -0.042), 0.015, 0.04, m["black"], axis="Y", seg=12, uv=UVS)
    for x in (-0.008, 0.008):
        body.box((x, -0.1, -0.004), (0.004, 0.012, 0.016), m["black"], uv=UVS)
    pistol_grip(body, m, -0.085, -0.06)
    body.extrude([(-0.055, -0.06), (0.0, -0.06), (-0.003, -0.083), (-0.05, -0.085)], 0.01, m["metal"])
    body.box((0, -0.27, -0.035), (0.012, 0.28, 0.012), m["metal"], uv=UVS)
    body.box((0, -0.27, -0.1), (0.012, 0.28, 0.012), m["metal"], rot=(-10, 0, 0), uv=UVS)
    body.box((0, -0.41, -0.07), (0.034, 0.014, 0.09), m["poly"], bevel=0.003, uv=UVS)
    pts = [(0.015 + 0.07 * (i / 8) ** 2, -0.06 - 0.21 * (i / 8)) for i in range(9)]
    back = [(y + 0.03, z + 0.005) for y, z in reversed(pts)]
    mag.extrude(pts + back, 0.026, m["black"], bevel=0.002)
    right_hand_on_grip(body, m, (0.006, -0.105, -0.108), grip_tilt=18, elbow=(0.14, -0.42, -0.3))
    left_hand_support(arml, m, (-0.016, 0.25, -0.075), elbow=(-0.23, -0.06, -0.33), rot=(0, 0, 30))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("galil", None, (0, 0.6, -0.042))


def build_rpd():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    smg_receiver(body, m, (-0.12, 0.16), (-0.06, -0.016), 0.04)
    body.box((0, 0.02, -0.012), (0.044, 0.2, 0.012), m["metal"], bevel=0.003, uv=UVS)  # Deckel
    body.extrude([(0.16, -0.03), (0.4, -0.035), (0.4, -0.07), (0.17, -0.08)], 0.05, m["wood"], bevel=0.005)
    body.cyl((0, 0.6, -0.04), 0.012, 0.45, m["metal"], axis="Y", seg=14, uv=UVS)
    body.cyl((0, 0.55, -0.06), 0.007, 0.25, m["metal"], axis="Y", seg=10, uv=UVS)
    body.box((0, 0.8, -0.018), (0.005, 0.01, 0.045), m["black"], uv=UVS)
    body.cyl((0, 0.83, -0.04), 0.016, 0.05, m["black"], axis="Y", seg=12, uv=UVS)
    # Zweibein (eingeklappt)
    for x in (-0.014, 0.014):
        body.box((x, 0.63, -0.06), (0.006, 0.28, 0.006), m["metal"], uv=UVS)
    body.box((0, -0.1, -0.002), (0.016, 0.012, 0.014), m["black"], uv=UVS)
    stock = [(-0.46, -0.03), (-0.12, -0.035), (-0.1, -0.06), (-0.13, -0.09), (-0.2, -0.12), (-0.46, -0.16)]
    body.extrude(stock, 0.044, m["wood"], bevel=0.004)
    pistol_grip(body, m, -0.06, -0.06, mat="wood_d", length=0.09)
    body.extrude([(-0.035, -0.06), (0.02, -0.06), (0.018, -0.085), (-0.03, -0.087)], 0.01, m["metal"])
    # Trommelmagazin
    mag.cyl((0, 0.06, -0.12), 0.075, 0.07, m["black"], axis="X", seg=20, uv=UVS)
    mag.cyl((0, 0.06, -0.12), 0.04, 0.074, m["metal"], axis="X", seg=16, uv=UVS)
    mag.box((0, 0.06, -0.055), (0.03, 0.03, 0.03), m["black"], uv=UVS)
    right_hand_on_grip(body, m, (0.006, -0.08, -0.105), grip_tilt=18, elbow=(0.15, -0.42, -0.3))
    left_hand_support(arml, m, (-0.018, 0.27, -0.08), elbow=(-0.24, -0.05, -0.33), rot=(0, 0, 30))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("rpd", None, (0, 0.86, -0.04))


def build_dsr50():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    # Zielfernrohr (Visierlinie durch die Rohrmitte)
    body.cyl((0, 0.06, 0.0), 0.017, 0.26, m["black"], axis="Y", seg=18, uv=UVS)
    body.cyl((0, 0.2, 0.0), 0.024, 0.06, m["black"], axis="Y", seg=18, uv=UVS)
    body.cyl((0, -0.085, 0.0), 0.021, 0.05, m["black"], axis="Y", seg=18, uv=UVS)
    body.cyl((0, 0.232, 0.0), 0.021, 0.004, m["glass"], axis="Y", seg=18, uv=UVS)
    body.cyl((0, 0.08, 0.022), 0.008, 0.02, m["black"], seg=10, uv=UVS)
    for y in (-0.02, 0.13):
        body.box((0, y, -0.025), (0.02, 0.02, 0.03), m["black"], uv=UVS)
    # Bullpup-Gehäuse
    body.extrude([(-0.42, -0.04), (0.32, -0.04), (0.32, -0.085), (0.05, -0.09), (-0.05, -0.12), (-0.42, -0.15)], 0.05, m["poly"], bevel=0.006)
    body.cyl((0, 0.55, -0.06), 0.012, 0.48, m["metal"], axis="Y", seg=14, uv=UVS)
    body.box((0, 0.8, -0.06), (0.04, 0.06, 0.03), m["black"], bevel=0.004, uv=UVS)  # Mündungsbremse
    pistol_grip(body, m, 0.08, -0.09, length=0.09)
    mag.box((0, -0.18, -0.15), (0.03, 0.07, 0.06), m["black"], bevel=0.003, uv=UVS)
    right_hand_on_grip(body, m, (0.006, 0.07, -0.135), grip_tilt=18, elbow=(0.14, -0.3, -0.33))
    left_hand_support(arml, m, (-0.018, 0.25, -0.1), elbow=(-0.24, -0.05, -0.34), rot=(0, 0, 30))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("dsr50", None, (0, 0.84, -0.06))

