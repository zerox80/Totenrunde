"""Pistolen: M1911 (Startwaffe), 93R Salve, Python, Strahlenpistole."""

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



def build_m1911():
    m = mats()
    body = MB("Body", UVS)
    slide = MB("Slide", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    # Schlitten
    slide.extrude([(-0.035, -0.034), (0.17, -0.034), (0.17, -0.012), (0.155, -0.007), (-0.03, -0.007), (-0.035, -0.012)], 0.024, m["metal"], bevel=0.0015)
    for i in range(7):
        slide.box((0, -0.025 + i * 0.004, -0.02), (0.0245, 0.0016, 0.02), m["black"], uv=UVS)
    slide.box((0, -0.026, -0.004), (0.012, 0.006, 0.006), m["black"], uv=UVS)  # Kimme
    slide.box((0, 0.162, -0.004), (0.004, 0.005, 0.006), m["black"], uv=UVS)  # Korn
    slide.box((0.0125, 0.07, -0.018), (0.002, 0.03, 0.009), m["black"], uv=UVS)  # Auswurf
    # Lauf / Buchse
    body.cyl((0, 0.172, -0.021), 0.0085, 0.008, m["metal_l"], axis="Y", seg=14, uv=UVS)
    # Rahmen
    body.extrude([(-0.03, -0.048), (0.125, -0.048), (0.125, -0.034), (-0.03, -0.034)], 0.022, m["metal"], bevel=0.001)
    # Griff mit Holzschalen
    grip = [(-0.032, -0.046), (0.012, -0.046), (-0.012, -0.142), (-0.057, -0.136)]
    body.extrude(grip, 0.026, m["metal"], bevel=0.002)
    body.extrude([(-0.03, -0.055), (0.006, -0.055), (-0.014, -0.128), (-0.052, -0.124)], 0.032, m["wood"], bevel=0.002)
    # Abzugsbügel + Abzug
    body.extrude([(0.0, -0.048), (0.052, -0.048), (0.05, -0.072), (0.006, -0.074), (0.0, -0.066), (0.006, -0.066), (0.044, -0.065), (0.045, -0.054), (0.004, -0.054)], 0.008, m["metal"])
    body.box((0, 0.018, -0.058), (0.006, 0.006, 0.016), m["black"], rot=(-10, 0, 0), uv=UVS)
    # Hahn
    body.box((0, -0.04, -0.016), (0.008, 0.012, 0.016), m["black"], rot=(-25, 0, 0), bevel=0.002, uv=UVS)
    # Magazin
    mag.extrude([(-0.03, -0.06), (0.002, -0.06), (-0.016, -0.145), (-0.05, -0.14)], 0.02, m["black"])
    mag.box((0, -0.034, -0.146), (0.024, 0.04, 0.008), m["black"], rot=(-12, 0, 0), bevel=0.002, uv=UVS)
    # Hände (beidhändig)
    right_hand_on_grip(body, m, (0.004, -0.032, -0.093), grip_tilt=14, elbow=(0.12, -0.36, -0.31))
    left_hand_pistol(arml, m, (-0.018, -0.03, -0.1), elbow=(-0.17, -0.34, -0.3), tilt=14)
    for mb in (body, slide, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("m1911", None, (0, 0.18, -0.021))


def build_b23r():
    m = mats()
    body = MB("Body", UVS)
    slide = MB("Slide", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    slide.extrude([(-0.04, -0.036), (0.2, -0.036), (0.2, -0.012), (0.185, -0.007), (-0.035, -0.007), (-0.04, -0.012)], 0.026, m["metal"], bevel=0.0015)
    slide.box((0, -0.03, -0.004), (0.012, 0.006, 0.006), m["black"], uv=UVS)
    slide.box((0, 0.19, -0.004), (0.004, 0.005, 0.006), m["black"], uv=UVS)
    for i in range(3):
        slide.box((0, 0.205 + i * 0.008, -0.022), (0.03, 0.004, 0.02), m["black"], uv=UVS)  # Kompensator
    body.extrude([(-0.035, -0.05), (0.15, -0.05), (0.15, -0.036), (-0.035, -0.036)], 0.024, m["metal"])
    body.extrude([(-0.036, -0.048), (0.012, -0.048), (-0.01, -0.145), (-0.06, -0.14)], 0.028, m["poly"], bevel=0.002)
    body.extrude([(0.0, -0.05), (0.055, -0.05), (0.053, -0.075), (0.006, -0.077), (0.0, -0.068), (0.006, -0.068), (0.046, -0.067), (0.047, -0.056), (0.004, -0.056)], 0.008, m["metal"])
    # klappbarer Vordergriff
    body.box((0, 0.11, -0.065), (0.016, 0.014, 0.03), m["poly"], bevel=0.002, uv=UVS)
    body.box((0, 0.11, -0.085), (0.03, 0.012, 0.008), m["poly"], uv=UVS)
    mag.extrude([(-0.032, -0.06), (0.004, -0.06), (-0.025, -0.2), (-0.06, -0.195)], 0.022, m["black"])
    right_hand_on_grip(body, m, (0.004, -0.034, -0.095), grip_tilt=14, elbow=(0.12, -0.36, -0.31))
    left_hand_pistol(arml, m, (0.0, 0.11, -0.085), elbow=(-0.2, -0.25, -0.32), tilt=0, grip_w=0.018, grip_d=0.016)
    for mb in (body, slide, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("b23r", None, (0, 0.23, -0.022))


def build_python():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    # Lauf mit Lüftungsschiene
    body.cyl((0, 0.115, -0.02), 0.0095, 0.17, m["metal_l"], axis="Y", seg=16, uv=UVS)
    body.box((0, 0.115, -0.011), (0.012, 0.17, 0.006), m["metal_l"], uv=UVS)
    for i in range(6):
        body.box((0, 0.045 + i * 0.026, -0.006), (0.004, 0.012, 0.006), m["metal_l"], uv=UVS)
    body.box((0, 0.115, -0.033), (0.014, 0.16, 0.01), m["metal_l"], uv=UVS)  # Ejektorgehäuse
    body.box((0, 0.195, -0.002), (0.004, 0.008, 0.01), m["black"], uv=UVS)  # Korn
    body.box((0, -0.02, -0.004), (0.012, 0.008, 0.006), m["black"], uv=UVS)
    # Rahmen
    body.extrude([(-0.03, -0.05), (0.035, -0.05), (0.035, -0.012), (-0.025, -0.008), (-0.04, -0.02)], 0.024, m["metal_l"], bevel=0.0015)
    body.extrude([(-0.032, -0.045), (0.0, -0.05), (-0.02, -0.135), (-0.065, -0.128), (-0.05, -0.05)], 0.03, m["wood"], bevel=0.003)
    body.extrude([(0.0, -0.05), (0.04, -0.05), (0.038, -0.072), (0.005, -0.074)], 0.008, m["metal_l"])
    body.box((0, -0.042, -0.01), (0.008, 0.014, 0.018), m["black"], rot=(-30, 0, 0), uv=UVS)
    # Trommel (als "Mag")
    mag.cyl((0, 0.008, -0.03), 0.022, 0.045, m["metal_l"], axis="Y", seg=6, uv=UVS)
    for a in range(6):
        ang = a * math.tau / 6
        mag.cyl((math.cos(ang) * 0.014, 0.031, -0.03 + math.sin(ang) * 0.014), 0.004, 0.002, m["brass"], axis="Y", seg=8, uv=UVS)
    right_hand_on_grip(body, m, (0.004, -0.038, -0.09), grip_tilt=20, elbow=(0.12, -0.36, -0.31))
    left_hand_pistol(arml, m, (-0.018, -0.036, -0.098), elbow=(-0.17, -0.34, -0.3), tilt=20)
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("python", None, (0, 0.205, -0.02))


def build_raygun():
    m = mats()
    body = MB("Body", UVS)
    mag = MB("Mag", UVS)
    arml = MB("ArmL", UVS)
    # bauchiger Körper, Ringe, Leuchtzellen – eigenes Retro-Design
    body.lathe([(0.001, -0.06), (0.03, -0.05), (0.038, -0.02), (0.036, 0.03), (0.024, 0.06), (0.012, 0.075), (0.001, 0.078)], m["red"], seg=20, axis="Y", center=(0, 0, -0.04))
    for i, (y, r) in enumerate(((0.08, 0.034), (0.115, 0.028), (0.145, 0.022))):
        body.cyl((0, y, -0.04), r, 0.012, m["bronze"], axis="Y", seg=20, uv=UVS)
        body.cyl((0, y + 0.012, -0.04), r * 0.55, 0.01, m["metal_l"], axis="Y", seg=14, uv=UVS)
    body.cyl((0, 0.17, -0.04), 0.009, 0.05, m["metal_l"], axis="Y", seg=12, uv=UVS)
    body.cyl((0, 0.195, -0.04), 0.012, 0.006, m["glow"], axis="Y", seg=12, uv=UVS)
    # Leuchtröhre oben
    body.cyl((0, 0.0, -0.002), 0.009, 0.09, m["glow"], axis="Y", seg=12, uv=UVS)
    body.box((0, 0.0, -0.012), (0.012, 0.1, 0.01), m["bronze"], uv=UVS)
    body.box((0, -0.05, -0.002), (0.006, 0.01, 0.01), m["black"], uv=UVS)
    body.box((0, 0.06, -0.002), (0.004, 0.008, 0.01), m["black"], uv=UVS)
    # Flosse
    body.extrude([(-0.04, -0.04), (0.02, -0.04), (-0.02, 0.0), (-0.05, 0.0)], 0.006, m["bronze"], center=(0, 0, 0.002), offset=0.0)
    # Griff
    body.extrude([(-0.03, -0.07), (0.01, -0.07), (-0.015, -0.16), (-0.055, -0.155)], 0.03, m["poly"], bevel=0.004)
    body.extrude([(0.0, -0.072), (0.045, -0.07), (0.04, -0.095), (0.005, -0.098)], 0.008, m["bronze"])
    mag.box((0.03, -0.0, -0.06), (0.012, 0.05, 0.03), m["glow"], uv=UVS)
    right_hand_on_grip(body, m, (0.004, -0.035, -0.118), grip_tilt=18, elbow=(0.12, -0.38, -0.32))
    left_hand_pistol(arml, m, (-0.018, -0.033, -0.125), elbow=(-0.17, -0.34, -0.3), tilt=18)
    for mb in (body, mag, arml):
        mb.to_object(smooth_angle=35)
    finish("raygun", None, (0, 0.2, -0.04))

