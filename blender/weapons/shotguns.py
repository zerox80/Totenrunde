"""Schrotflinten: Doppelflinte, Remington 870."""

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



def build_olympia():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    for x in (-0.0125, 0.0125):
        body.cyl((x, 0.39, -0.022), 0.0125, 0.62, m["metal"], axis="Y", seg=16, uv=UVS)
    body.box((0, 0.39, -0.008), (0.008, 0.62, 0.006), m["metal"], uv=UVS)  # Schiene
    body.sphere((0, 0.695, -0.002), 0.0028, m["brass"], seg=8, rings=4)
    body.extrude([(-0.05, -0.012), (0.09, -0.012), (0.09, -0.05), (-0.04, -0.06), (-0.05, -0.04)], 0.042, m["metal_l"], bevel=0.002)  # Basküle
    body.extrude([(0.09, -0.035), (0.33, -0.035), (0.32, -0.065), (0.1, -0.068)], 0.046, m["wood"], bevel=0.004)  # Vorderschaft
    stock = [(-0.43, -0.03), (-0.05, -0.02), (-0.04, -0.06), (-0.09, -0.08), (-0.13, -0.12), (-0.2, -0.13), (-0.43, -0.17)]
    body.extrude(stock, 0.042, m["wood"], bevel=0.004)
    body.extrude([(-0.03, -0.06), (0.03, -0.06), (0.028, -0.085), (-0.025, -0.087)], 0.01, m["metal"])
    body.box((0, -0.04, -0.005), (0.012, 0.02, 0.008), m["black"], rot=(-20, 0, 0), uv=UVS)  # Hahn
    # zwei Patronen als "Mag" (beim Nachladen sichtbar)
    for x in (-0.0125, 0.0125):
        mag.cyl((x, 0.02, -0.03), 0.011, 0.065, m["red"], axis="Y", seg=12, uv=UVS)
        mag.cyl((x, -0.015, -0.03), 0.0118, 0.01, m["brass"], axis="Y", seg=12, uv=UVS)
    right_hand_on_grip(body, m, (0.006, -0.1, -0.1), grip_tilt=40, elbow=(0.15, -0.42, -0.3))
    left_hand_support(arml, m, (-0.014, 0.21, -0.075), elbow=(-0.24, -0.12, -0.32), rot=(0, 0, 30))
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("olympia", None, (0, 0.71, -0.022))


def build_remington():
    m = mats()
    body = MB("Body", UVS)
    slide = MB("Slide", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    body.cyl((0, 0.4, -0.02), 0.0115, 0.56, m["metal"], axis="Y", seg=16, uv=UVS)
    body.box((0, 0.4, -0.007), (0.006, 0.56, 0.005), m["metal"], uv=UVS)
    body.sphere((0, 0.675, -0.002), 0.003, m["brass"], seg=8, rings=4)
    body.cyl((0, 0.37, -0.046), 0.012, 0.5, m["metal"], axis="Y", seg=14, uv=UVS)  # Röhrenmagazin
    body.extrude([(-0.08, -0.012), (0.12, -0.012), (0.12, -0.058), (-0.06, -0.062), (-0.08, -0.04)], 0.04, m["metal"], bevel=0.002)
    body.box((0.0205, 0.03, -0.034), (0.002, 0.06, 0.018), m["black"], uv=UVS)
    stock = [(-0.43, -0.025), (-0.08, -0.02), (-0.06, -0.06), (-0.1, -0.075), (-0.15, -0.115), (-0.22, -0.13), (-0.43, -0.16)]
    body.extrude(stock, 0.042, m["poly"], bevel=0.004)
    body.extrude([(-0.03, -0.06), (0.04, -0.06), (0.038, -0.088), (-0.025, -0.09)], 0.01, m["metal"])
    # Pumpe (bewegt sich)
    slide.cyl((0, 0.29, -0.047), 0.022, 0.18, m["poly"], axis="Y", seg=14, uv=UVS)
    for i in range(7):
        slide.cyl((0, 0.215 + i * 0.024, -0.047), 0.0235, 0.008, m["poly"], axis="Y", seg=14, uv=UVS)
    mag.cyl((0, 0.1, -0.05), 0.011, 0.06, m["red"], axis="Y", seg=10, uv=UVS)
    mag.cyl((0, 0.065, -0.05), 0.0115, 0.01, m["brass"], axis="Y", seg=10, uv=UVS)
    right_hand_on_grip(body, m, (0.006, -0.11, -0.1), grip_tilt=40, elbow=(0.15, -0.42, -0.3))
    left_hand_support(arml, m, (-0.012, 0.29, -0.065), elbow=(-0.24, -0.05, -0.33), rot=(0, 0, 30))
    for mb in (body, slide, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("remington", None, (0, 0.685, -0.02))

