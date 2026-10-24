"""Kurzwaffen und Handgegenstände: B23R (Salvenpistole), Python (Revolver), Strahlenpistole,
Kampfmesser und Perk-Flasche – mit Ego-Armen."""

import math

from lib import material, hexcol

from .kit import F, bake_weapon, finish, join, load_arms, pose_arm, remove_arms, uv_atlas
from .parts import box, ext, lathe, left_vgrip, pistol_hands, right_grip, solid, tube

C = hexcol


def assemble(wid, body, mag, slide, pose, muzzle, bake=True):
    objs = {"Body": join(body, "Body")}
    if mag:
        objs["Mag"] = join(mag, "Mag")
    if slide:
        objs["Slide"] = join(slide, "Slide")
    uv_atlas(list(objs.values()))
    arms = load_arms()
    right, left = pose(arms, objs["Body"], objs.get("Mag"), objs.get("Slide"))
    remove_arms(arms)
    objs["Body"] = join([objs["Body"], right], "Body")
    extra = []
    if left is not None:
        left.name = "ArmL"
        left.data.name = "ArmL"
        extra = [left]
    if bake:
        bake_weapon(wid, list(objs.values()) + extra)
    finish(wid, muzzle)


TOP = -0.0075


def build_b23r():
    body, mag, slide = [], [], []
    slide.append(solid("SlideBody", lambda b, m: b.extrude([(-0.04, -0.036), (0.2, -0.036), (0.2, TOP), (-0.04, TOP)], 0.025, m), "blued",
                       cuts=[
                           lambda c, m: [c.box((sx * 0.0118, 0.09, TOP), (0.006, 0.28, 0.006), m, rot=(0, 45, 0)) for sx in (-1, 1)],
                           lambda c, m: [c.box((sx * 0.0122, 0.07 + i * 0.03, -0.021), (0.004, 0.02, 0.012), m) for sx in (-1, 1) for i in range(3)],
                           lambda c, m: [c.box((sx * 0.0125, -0.034 + i * 0.0035, -0.022), (0.0016, 0.0012, 0.022), m) for sx in (-1, 1) for i in range(8)],
                           lambda c, m: c.box((0.012, 0.035, -0.016), (0.012, 0.035, 0.01), m),
                       ]))
    slide.append(solid("Sights", lambda b, m: (
        b.box((0, 0.19, TOP + 0.0028), (0.0032, 0.007, 0.0058), m),
        b.extrude([(-0.036, TOP - 0.001), (-0.024, TOP - 0.001), (-0.025, TOP + 0.005), (-0.036, TOP + 0.005)], 0.013, m),
    ), "black", cuts=lambda c, m: c.box((0, -0.03, TOP + 0.005), (0.0028, 0.02, 0.0036), m)))
    body.append(tube("Barrel", -0.02, 0.21, 0.0085, "steel", z=-0.022))
    body.append(solid("Comp", lambda b, m: b.box((0, 0.218, -0.022), (0.03, 0.03, 0.03), m), "black",
                      cuts=lambda c, m: [c.box((0, 0.21 + i * 0.008, -0.01), (0.006, 0.004, 0.02), m) for i in range(3)]))
    body.append(ext("Frame", [(-0.035, -0.05), (0.15, -0.05), (0.15, -0.036), (-0.035, -0.036)], 0.0235, "blued"))
    body.append(ext("Grip", [(-0.038, -0.048), (0.012, -0.048), (-0.01, -0.146), (-0.062, -0.14), (-0.05, -0.08)], 0.029, "polymer_checker"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(0.0, -0.05), (0.058, -0.05), (0.056, -0.077), (0.006, -0.079), (-0.002, -0.07)], 0.008, m), "blued",
                      cuts=lambda c, m: c.extrude([(0.005, -0.054), (0.053, -0.054), (0.052, -0.073), (0.008, -0.074), (0.003, -0.068)], 0.02, m)))
    body.append(ext("Trigger", [(0.014, -0.052), (0.022, -0.052), (0.02, -0.07), (0.013, -0.07)], 0.006, "black"))
    body.append(ext("Hammer", [(-0.04, -0.02), (-0.034, -0.032), (-0.039, -0.037), (-0.051, -0.017), (-0.046, -0.008)], 0.0062, "steel"))
    # Klappbarer Vordergriff
    body.append(solid("FrontGrip", lambda b, m: (
        b.box((0, 0.11, -0.058), (0.018, 0.016, 0.018), m),
        b.extrude([(0.1, -0.066), (0.12, -0.066), (0.118, -0.115), (0.098, -0.115)], 0.018, m),
    ), "polymer"))
    mag.append(ext("Mag", [(-0.032, -0.06), (0.004, -0.06), (-0.026, -0.2), (-0.062, -0.195)], 0.022, "blued"))
    mag.append(ext("MagBase", [(-0.066, -0.194), (-0.024, -0.2), (-0.024, -0.208), (-0.067, -0.202)], 0.026, "black"))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b, m], (-0.012, -0.05), (-0.037, -0.143), depth=0.045, elbow=(0.17, -0.4, -0.3))
        l = left_vgrip(arms, [b, m, r], (0.11, -0.068), (0.108, -0.113), depth=0.02, elbow=(-0.2, -0.28, -0.32))
        return r, l

    assemble("b23r", body, mag, slide, pose, (0, 0.235, -0.022))


def build_python():
    body, mag = [], []
    ax = -0.022
    body.append(tube("Barrel", 0.03, 0.2, 0.0095, "blued", z=ax))
    body.append(solid("Rib", lambda b, m: (
        b.box((0, 0.115, ax + 0.012), (0.011, 0.17, 0.008), m),
        *[b.box((0, 0.05 + i * 0.026, ax + 0.019), (0.004, 0.014, 0.006), m) for i in range(6)],
        b.box((0, 0.11, ax - 0.012), (0.014, 0.16, 0.012), m),
    ), "blued"))
    body.append(box("FrontSight", (0, 0.19, -0.004), (0.004, 0.012, 0.01), "black"))
    body.append(solid("Frame", lambda b, m: b.extrude([(-0.032, -0.052), (0.034, -0.052), (0.034, -0.008), (-0.026, -0.006), (-0.042, -0.02)], 0.024, m), "blued",
                      cuts=lambda c, m: c.box((0, 0.005, -0.03), (0.03, 0.05, 0.034), m)))
    body.append(ext("RearSight", [(-0.03, -0.007), (-0.018, -0.007), (-0.018, 0.0), (-0.03, 0.0)], 0.012, "black"))
    body.append(ext("Grip", [(-0.034, -0.046), (0.0, -0.052), (-0.016, -0.135), (-0.064, -0.128), (-0.052, -0.05)], 0.032, "wood_checker"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(0.0, -0.052), (0.04, -0.052), (0.038, -0.074), (0.004, -0.076)], 0.008, m), "blued",
                      cuts=lambda c, m: c.extrude([(0.004, -0.056), (0.035, -0.056), (0.033, -0.07), (0.007, -0.071)], 0.02, m)))
    body.append(ext("Trigger", [(0.01, -0.054), (0.018, -0.054), (0.015, -0.07), (0.008, -0.069)], 0.006, "steel"))
    body.append(ext("Hammer", [(-0.034, -0.012), (-0.028, -0.024), (-0.036, -0.028), (-0.05, -0.008), (-0.046, 0.0)], 0.007, "steel"))
    mag.append(solid("Cylinder", lambda b, m: b.cyl((0, 0.006, ax - 0.008), 0.021, 0.044, m, axis="Y", seg=36), "blued",
                     cuts=lambda c, m: [
                         *[c.cyl((math.cos(k * math.tau / 6 + 0.52) * 0.023, 0.006, ax - 0.008 + math.sin(k * math.tau / 6 + 0.52) * 0.023), 0.0045, 0.03, m, axis="Y", seg=12) for k in range(6)],
                         *[c.cyl((math.cos(k * math.tau / 6) * 0.0135, 0.03, ax - 0.008 + math.sin(k * math.tau / 6) * 0.0135), 0.0048, 0.01, m, axis="Y", seg=16) for k in range(6)],
                     ]))

    def pose(arms, b, m, s):
        return pistol_hands(arms, [b, m], (-0.016, -0.05), (-0.04, -0.132), depth=0.046)

    assemble("python", body, mag, None, pose, (0, 0.205, ax))


def build_raygun():
    body, mag = [], []
    ax = -0.04
    body.append(lathe("Hull", [(0.0, -0.065), (0.022, -0.06), (0.036, -0.035), (0.04, 0.0), (0.037, 0.035), (0.026, 0.065), (0.014, 0.08), (0.0, 0.082)], "red", center=(0, 0, ax), seg=40))
    for i, (y, r) in enumerate(((0.082, 0.034), (0.115, 0.028), (0.145, 0.022))):
        body.append(tube(f"Ring{i}", y, y + 0.012, r, "brass", z=ax, seg=40))
        body.append(tube(f"Core{i}", y + 0.012, y + 0.02, r * 0.55, "steel", z=ax, seg=32))
    body.append(tube("Emitter", 0.15, 0.19, 0.009, "steel", z=ax, seg=24))
    body.append(solid("Fin", lambda b, m: b.extrude([(-0.045, ax + 0.03), (0.03, ax + 0.034), (-0.01, ax + 0.06), (-0.05, ax + 0.06)], 0.005, m), "brass"))
    body.append(ext("Grip", [(-0.03, -0.072), (0.01, -0.072), (-0.012, -0.165), (-0.056, -0.16)], 0.03, "polymer_checker"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(0.0, -0.074), (0.046, -0.072), (0.042, -0.098), (0.004, -0.1)], 0.008, m), "brass",
                      cuts=lambda c, m: c.extrude([(0.004, -0.078), (0.04, -0.077), (0.037, -0.094), (0.007, -0.095)], 0.02, m)))
    body.append(box("Sight", (0, 0.0, -0.002), (0.012, 0.09, 0.008), "brass"))
    glow = material("W_Glow", color=C("#7dff7a"), emission=C("#4dff4a"), emission_strength=10.0, rough=0.3)
    from .kit import harden, mb

    g = mb("Glow")
    g.cyl((0, 0.0, 0.004), 0.007, 0.08, glow, axis="Y", seg=20)
    g.cyl((0, 0.19, ax), 0.012, 0.006, glow, axis="Y", seg=24)
    go = g.to_object(smooth_angle=None)
    harden(go, 0.0003)
    body.append(go)
    m2 = mb("MagGlow")
    m2.box((0.032, 0.0, -0.062), (0.012, 0.05, 0.03), glow)
    mo = m2.to_object(smooth_angle=None)
    harden(mo, 0.0004)
    mag.append(mo)

    def pose(arms, b, m, s):
        return pistol_hands(arms, [b, m], (-0.01, -0.072), (-0.034, -0.162), depth=0.044)

    assemble("raygun", body, mag, None, pose, (0, 0.2, ax))


def build_knife():
    body = []
    body.append(solid("Blade", lambda b, m: b.extrude([(0.0, -0.005), (0.15, -0.005), (0.188, 0.012), (0.16, 0.019), (0.0, 0.017)], 0.0045, m), "steel"))
    body.append(box("Guard", (0, 0.004, 0.006), (0.024, 0.01, 0.044), "black"))
    body.append(solid("Handle", lambda b, m: b.cyl((0, -0.055, 0.006), 0.0135, 0.11, m, axis="Y", seg=24), "rubber",
                      cuts=lambda c, m: [c.cyl((0, -0.1 + i * 0.016, 0.006), 0.016, 0.004, m, axis="Y", seg=24) for i in range(6)]))
    body.append(lathe("Pommel", [(0.0, -0.122), (0.014, -0.122), (0.0145, -0.11), (0.0, -0.11)], "black", center=(0, 0, 0.006)))

    def pose(arms, b, m, s):
        r = pose_arm("R", arms, wrist=(0.018, -0.075, -0.06), a=(-0.15, 0.25, 0.96), n=(-1.0, 0.0, 0.0), elbow=(0.12, -0.42, -0.15), targets=[b])
        return r, None

    assemble("knife", body, None, None, pose, (0, 0.188, 0.0))


def build_perk_bottle():
    body = []
    glass = material("W_BottleGlass", color=C("#5a2e20"), rough=0.08, metal=0.1)
    label = material("W_Glow", color=C("#ffd36a"), emission=C("#ffb24a"), emission_strength=2.0)
    from .kit import harden, mb

    g = mb("Bottle")
    g.lathe([(0.001, 0.0), (0.03, 0.0), (0.033, 0.006), (0.034, 0.12), (0.024, 0.15), (0.012, 0.175), (0.011, 0.2), (0.0135, 0.204), (0.001, 0.206)], glass, seg=32, axis="Z")
    g.cyl((0, 0, 0.07), 0.0352, 0.05, label, seg=32)
    go = g.to_object(smooth_angle=None)
    harden(go, 0.0004, angle=50)
    body.append(go)
    body.append(solid("Cap", lambda b, m: b.cyl((0, 0, 0.207), 0.0138, 0.008, m, seg=24), "red"))

    def pose(arms, b, m, s):
        r = pose_arm("R", arms, wrist=(0.03, -0.08, 0.06), a=(-0.25, 0.97, 0.0), n=(-1.0, 0.0, 0.0), elbow=(0.06, -0.33, -0.22), targets=[b])
        return r, None

    assemble("perk_bottle", body, None, None, pose, (0, 0, 0.2))
