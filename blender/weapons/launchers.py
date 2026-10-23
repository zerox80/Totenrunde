"""Granatwerfer: Kriegsmaschine (Revolver-Granatwerfer)."""

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



def build_mgl():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    # Reflexvisier
    body.box((0, -0.02, -0.02), (0.03, 0.05, 0.012), m["black"], uv=UVS)
    for x in (-0.014, 0.014):
        body.box((x, -0.02, -0.004), (0.003, 0.004, 0.03), m["black"], uv=UVS)
    body.box((0, -0.02, 0.012), (0.03, 0.004, 0.003), m["black"], uv=UVS)
    body.box((0, -0.02, -0.004), (0.026, 0.002, 0.026), m["glass"], uv=UVS)
    body.box((0, 0.05, -0.05), (0.05, 0.3, 0.03), m["black"], uv=UVS)  # Schiene
    body.cyl((0, 0.27, -0.09), 0.024, 0.24, m["black"], axis="Y", seg=16, uv=UVS)  # Lauf
    body.box((0, 0.3, -0.13), (0.03, 0.03, 0.08), m["poly"], bevel=0.004, uv=UVS)  # Vordergriff
    pistol_grip(body, m, -0.06, -0.12, length=0.09)
    body.extrude([(-0.22, -0.06), (-0.09, -0.06), (-0.09, -0.15), (-0.22, -0.18)], 0.04, m["poly"], bevel=0.004)
    body.cyl((0, -0.05, -0.09), 0.01, 0.1, m["metal"], axis="Y", seg=10, uv=UVS)
    # Revolvertrommel
    mag.cyl((0, 0.06, -0.09), 0.068, 0.12, m["metal"], axis="Y", seg=6, uv=UVS)
    for a in range(6):
        ang = a * math.tau / 6 + 0.5
        mag.cyl((math.cos(ang) * 0.045, 0.122, -0.09 + math.sin(ang) * 0.045), 0.018, 0.004, m["brass"], axis="Y", seg=10, uv=UVS)
    right_hand_on_grip(body, m, (0.006, -0.08, -0.165), grip_tilt=18, elbow=(0.14, -0.42, -0.34))
    left_hand_support(arml, m, (-0.014, 0.3, -0.17), elbow=(-0.22, -0.02, -0.36), rot=(0, 0, 10))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("mgl", None, (0, 0.39, -0.09))

