"""Maschinenpistolen: MP5, AK-74u."""

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



def build_mp5():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    smg_receiver(body, m, (-0.13, 0.16), (-0.05, -0.016), 0.034)
    body.cyl((0, 0.015, -0.028), 0.019, 0.29, m["metal"], axis="Y", seg=16, uv=UVS)
    # Trommelvisier hinten, Korntunnel vorn
    body.cyl((0, -0.075, -0.004), 0.01, 0.012, m["black"], axis="X", seg=12, uv=UVS)
    body.box((0, -0.075, -0.016), (0.012, 0.012, 0.014), m["black"], uv=UVS)
    body.cyl((0, 0.2, -0.005), 0.011, 0.016, m["black"], axis="Y", seg=14, uv=UVS)
    body.box((0, 0.2, -0.018), (0.01, 0.016, 0.02), m["black"], uv=UVS)
    # Handschutz (dick)
    body.extrude([(0.03, -0.035), (0.19, -0.035), (0.19, -0.065), (0.05, -0.08), (0.03, -0.07)], 0.044, m["poly"], bevel=0.006)
    body.cyl((0, 0.24, -0.028), 0.009, 0.1, m["metal"], axis="Y", seg=12, uv=UVS)  # Lauf
    body.cyl((-0.02, 0.12, -0.012), 0.005, 0.12, m["black"], axis="Y", seg=8, uv=UVS)  # Spannhebelrohr
    pistol_grip(body, m, -0.07, -0.05)
    body.extrude([(-0.05, -0.05), (0.0, -0.05), (-0.002, -0.07), (-0.045, -0.072)], 0.01, m["poly"])
    # Schiebeschaft
    for x in (-0.014, 0.014):
        body.box((x, -0.25, -0.03), (0.006, 0.24, 0.008), m["metal"], uv=UVS)
    body.box((0, -0.37, -0.05), (0.04, 0.012, 0.07), m["poly"], bevel=0.003, uv=UVS)
    # gebogenes Magazin
    pts = []
    for i in range(9):
        t = i / 8
        ang = math.radians(10 + 16 * t)
        z = -0.05 - 0.2 * t
        y = 0.02 + 0.06 * t * t
        pts.append((y, z))
    back = [(y + 0.026, z + 0.004) for y, z in reversed(pts)]
    mag.extrude(pts + back, 0.024, m["black"], bevel=0.002)
    right_hand_on_grip(body, m, (0.006, -0.09, -0.098), grip_tilt=18, elbow=(0.14, -0.42, -0.3))
    left_hand_support(arml, m, (-0.016, 0.12, -0.075), elbow=(-0.23, -0.15, -0.32), rot=(0, 0, 30))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("mp5", None, (0, 0.295, -0.028))


def build_ak74u():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    smg_receiver(body, m, (-0.12, 0.13), (-0.058, -0.02), 0.034)
    body.cyl((0, 0.005, -0.022), 0.017, 0.25, m["metal"], axis="Y", seg=16, uv=UVS)  # Deckel
    body.extrude([(0.12, -0.03), (0.25, -0.03), (0.25, -0.07), (0.13, -0.075)], 0.042, m["wood"], bevel=0.005)  # Handschutz
    body.cyl((0, 0.19, -0.012), 0.011, 0.13, m["wood"], axis="Y", seg=12, uv=UVS)  # Gasrohr-Holz
    body.cyl((0, 0.28, -0.04), 0.009, 0.08, m["metal"], axis="Y", seg=12, uv=UVS)
    body.cyl((0, 0.315, -0.04), 0.017, 0.06, m["black"], axis="Y", seg=14, uv=UVS)  # Mündungsaufsatz
    body.box((0, 0.13, -0.006), (0.008, 0.02, 0.012), m["black"], uv=UVS)  # Visier (Klappe)
    body.box((0, 0.29, -0.008), (0.006, 0.01, 0.03), m["black"], uv=UVS)  # Korn
    pistol_grip(body, m, -0.08, -0.058, mat="wood_d")
    body.extrude([(-0.05, -0.058), (0.0, -0.058), (-0.003, -0.08), (-0.048, -0.082)], 0.01, m["metal"])
    # Klappschaft (Gerippe)
    body.box((0, -0.25, -0.03), (0.012, 0.26, 0.01), m["metal"], uv=UVS)
    body.box((0, -0.25, -0.09), (0.012, 0.26, 0.01), m["metal"], rot=(-8, 0, 0), uv=UVS)
    body.box((0, -0.38, -0.065), (0.03, 0.012, 0.075), m["metal"], bevel=0.002, uv=UVS)
    pts = []
    for i in range(9):
        t = i / 8
        pts.append((0.02 + 0.09 * t * t, -0.058 - 0.19 * t))
    back = [(y + 0.032, z + 0.006) for y, z in reversed(pts)]
    mag.extrude(pts + back, 0.024, m["mag_ak"], bevel=0.002)
    right_hand_on_grip(body, m, (0.006, -0.1, -0.105), grip_tilt=18, elbow=(0.14, -0.42, -0.3))
    left_hand_support(arml, m, (-0.016, 0.19, -0.075), elbow=(-0.23, -0.1, -0.33), rot=(0, 0, 30))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("ak74u", None, (0, 0.35, -0.04))

