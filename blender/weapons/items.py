"""Gegenstände in der Hand: Messer und Perk-Flasche."""

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



def build_knife():
    """Kampfmesser in der rechten Hand (für die Nahkampf-Animation)."""
    m = mats()
    body = MB("Body", UVS)
    blade = [(0.0, -0.004), (0.15, -0.004), (0.185, 0.012), (0.16, 0.018), (0.0, 0.016)]
    body.extrude(blade, 0.004, m["metal_l"], plane="YZ", bevel=0.0)
    body.box((0, 0.004, 0.006), (0.024, 0.01, 0.042), m["black"], bevel=0.002, uv=UVS)  # Parierstange
    body.cyl((0, -0.055, 0.006), 0.013, 0.11, m["poly"], axis="Y", seg=12, uv=UVS)
    for i in range(5):
        body.cyl((0, -0.1 + i * 0.02, 0.006), 0.0142, 0.006, m["poly"], axis="Y", seg=12, uv=UVS)
    from .common import grip_hand

    wrist = grip_hand(body, m, (0.0, -0.06, 0.006), rot=(-90, 0, 0), side=1.0, grip_w=0.028, grip_d=0.028)
    arm(body, m, wrist, (0.12, -0.42, -0.15))
    body.to_object(smooth_angle=35)
    finish("knife", None, (0, 0.185, 0.0))


def build_perk_bottle():
    """Perk-Flasche in der Hand (Trink-Animation)."""
    m = mats()
    body = MB("Body", UVS)
    glass = material("W_BottleGlass", color=C("#6b3a2a"), rough=0.1, metal=0.2)
    label = material("W_Glow", color=C("#ffd36a"), emission=C("#ffb24a"), emission_strength=2.0)
    body.lathe([(0.001, 0.0), (0.032, 0.0), (0.034, 0.01), (0.034, 0.12), (0.024, 0.15), (0.012, 0.175), (0.011, 0.2), (0.013, 0.205), (0.001, 0.206)], glass, seg=20, axis="Z")
    body.cyl((0, 0, 0.07), 0.0352, 0.05, label, seg=20, uv=UVS)
    from .common import grip_hand

    wrist = grip_hand(body, m, (0.0, 0.0, 0.07), rot=(0, 0, 0), side=1.0, grip_w=0.068, grip_d=0.068)
    arm(body, m, wrist, (0.06, -0.3, -0.25))
    body.to_object(smooth_angle=35)
    finish("perk_bottle", None, (0, 0, 0.2))

