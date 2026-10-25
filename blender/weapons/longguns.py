"""Langwaffen und Maschinenpistolen mit realistischen Einzelteilen und Ego-Armen.

Jede Waffe wird in natürlichen Koordinaten modelliert (Visierlinie z = 0) und dann um
`shift` nach vorn verschoben, damit die Abstände Auge–Kimme und Auge–Griff zueinander passen.
Arme und Griffe sind für eine Kamera bei y = -0.17 posiert; im Spiel sitzt die Kamera beim
Zielen bei y = -0.25, damit Kimme und Gehäuse nicht die halbe Sicht verdecken.
"""

import math

from mathutils import Vector

from .kit import F, bake_weapon, finish, join, load_arms, remove_arms, uv_atlas
from .parts import box, curved_mag, ext, lathe, left_support, rail, right_grip, solid, tube


def assemble(wid, body, mag, slide, shift, pose, muzzle):
    """Teile verbinden, verschieben, Arme posieren, backen, exportieren.

    pose(arms, body, mag, slide) -> (rechter Arm, linker Arm), in verschobenen Koordinaten."""
    objs = {"Body": join(body, "Body")}
    if mag:
        objs["Mag"] = join(mag, "Mag")
    if slide:
        objs["Slide"] = join(slide, "Slide")
    for o in objs.values():
        o.data.transform(__import__("mathutils").Matrix.Translation((0, shift, 0)))
    uv_atlas(list(objs.values()))
    arms = load_arms()
    right, left = pose(arms, objs["Body"], objs.get("Mag"), objs.get("Slide"))
    remove_arms(arms)
    objs["Body"] = join([objs["Body"], right], "Body")
    left.name = "ArmL"
    left.data.name = "ArmL"
    bake_weapon(wid, list(objs.values()) + [left])
    finish(wid, (muzzle[0], muzzle[1] + shift, muzzle[2]))


def S(v, shift):
    """Verschiebt einen Punkt nach vorn: (y, z) oder (x, y, z)."""
    return (v[0] + shift, v[1]) if len(v) == 2 else (v[0], v[1] + shift, v[2])


# --------------------------------------------------------------------------------------
# MP5
# --------------------------------------------------------------------------------------


def build_mp5():
    sh = -0.015
    body, mag, slide = [], [], []
    rz = -0.026  # Achse des Gehäuserohrs
    # Gehäuse: Blechrohr mit Sicken, oben flach für die Visiere
    body.append(solid("Receiver", lambda b, m: (
        b.cyl((0, 0.04, rz), 0.019, 0.32, m, axis="Y", seg=32),
        b.box((0, 0.04, rz + 0.016), (0.026, 0.32, 0.01), m),
    ), "parkerized", cuts=lambda c, m: [
        c.box((0.019, 0.075, rz + 0.002), (0.012, 0.05, 0.012), m),  # Auswurf
        *[c.box((sx * 0.0195, y, rz - 0.004), (0.003, 0.05, 0.003), m) for sx in (-1, 1) for y in (-0.06, 0.16)],
    ]))
    # Visiere: Trommel hinten (Mitte z = 0), Korntunnel vorne
    body.append(solid("RearSight", lambda b, m: (
        b.box((0, -0.075, -0.012), (0.03, 0.022, 0.016), m),
        b.cyl((0, -0.075, 0.0), 0.0105, 0.02, m, axis="X", seg=20),
    ), "black", cuts=lambda c, m: c.cyl((0, -0.075, 0.0), 0.0022, 0.05, m, axis="Y", seg=12)))
    body.append(solid("FrontSight", lambda b, m: (
        b.box((0, 0.205, -0.012), (0.024, 0.018, 0.014), m),
        b.cyl((0, 0.205, 0.0), 0.012, 0.014, m, axis="Y", seg=24),
        b.box((0, 0.205, -0.004), (0.0022, 0.004, 0.008), m),
    ), "black", cuts=lambda c, m: c.cyl((0, 0.205, 0.0), 0.0095, 0.03, m, axis="Y", seg=24)))
    # Spannhebelrohr links oben + Hebel
    body.append(tube("CockTube", 0.05, 0.215, 0.0065, "parkerized", z=-0.012, x=-0.0035))
    slide.append(box("CockHandle", (-0.016, 0.17, -0.012), (0.018, 0.008, 0.006), "black", rot=(0, 0, -15)))
    # Griffstück (Polymer) mit Abzugsbügel
    body.append(ext("Lower", [(-0.115, -0.044), (0.04, -0.044), (0.04, -0.056), (0.02, -0.064), (-0.06, -0.064), (-0.115, -0.058)], 0.03, "polymer"))
    body.append(ext("Grip", [(-0.06, -0.06), (-0.035, -0.06), (-0.045, -0.085), (-0.05, -0.12), (-0.058, -0.152), (-0.098, -0.15), (-0.09, -0.11), (-0.082, -0.06)], 0.032, "polymer_checker"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.036, -0.06), (0.03, -0.058), (0.03, -0.088), (-0.04, -0.09)], 0.009, m), "polymer",
                      cuts=lambda c, m: c.extrude([(-0.032, -0.064), (0.024, -0.063), (0.024, -0.082), (-0.035, -0.084)], 0.02, m)))
    body.append(ext("Trigger", [(-0.006, -0.062), (0.002, -0.062), (-0.002, -0.08), (-0.009, -0.079)], 0.005, "black"))
    body.append(ext("Selector", [(-0.088, -0.05), (-0.072, -0.049), (-0.074, -0.056), (-0.089, -0.057)], 0.003, "black", offset=-0.0165))
    # Magazinschacht
    body.append(box("MagWell", (0, 0.055, -0.05), (0.03, 0.036, 0.016), "parkerized"))
    # Handschutz: dick, gerippt
    def guard(b, m):
        b.extrude([(0.075, -0.038), (0.205, -0.038), (0.205, -0.066), (0.19, -0.078), (0.095, -0.08), (0.075, -0.07)], 0.046, m)
    body.append(solid("Handguard", guard, "polymer", cuts=lambda c, m: [
        c.box((sx * 0.024, 0.1 + i * 0.022, -0.06), (0.006, 0.011, 0.02), m) for sx in (-1, 1) for i in range(5)
    ]))
    # Lauf mit Dreifachkralle
    body.append(lathe("Barrel", [(0.0, 0.2), (0.0075, 0.2), (0.0075, 0.24), (0.0095, 0.243), (0.0095, 0.262), (0.0068, 0.27), (0.0, 0.27)], "blued", center=(0, 0, rz)))
    # Schiebeschaft: zwei Stangen + Schaftkappe
    body.append(solid("Stock", lambda b, m: (
        b.cyl((0.0135, -0.24, -0.04), 0.004, 0.25, m, axis="Y", seg=12),
        b.cyl((-0.0135, -0.24, -0.04), 0.004, 0.25, m, axis="Y", seg=12),
        b.box((0, -0.12, -0.036), (0.034, 0.012, 0.016), m),
    ), "parkerized"))
    body.append(ext("ButtPad", [(-0.372, -0.012), (-0.358, -0.012), (-0.355, -0.04), (-0.358, -0.1), (-0.372, -0.098)], 0.04, "rubber"))
    # Gebogenes Magazin mit Rippen
    mag.append(curved_mag("Mag", (0.072, -0.055), 0.2, 0.055, 0.03, 0.023, "black", ribs=0))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b, m], S((-0.062, -0.065), sh), S((-0.08, -0.15), sh), depth=0.04, elbow=(0.24, sh - 0.3, -0.38))
        l = left_support(arms, [b, m, r], 0.12 + sh, -0.058, 0.023, 0.021)
        return r, l

    assemble("mp5", body, mag, slide, sh, pose, (0, 0.272, rz))


# --------------------------------------------------------------------------------------
# AK-74u
# --------------------------------------------------------------------------------------


def build_ak74u():
    sh = -0.02
    body, mag, slide = [], [], []
    ax = -0.034  # Laufachse
    body.append(solid("Receiver", lambda b, m: b.box((0, 0.005, -0.04), (0.028, 0.23, 0.032), m), "blued",
                      cuts=[lambda c, m: c.box((0.014, 0.035, -0.03), (0.01, 0.07, 0.012), m),
                            lambda c, m: c.box((0, 0.005, -0.058), (0.02, 0.2, 0.012), m)]))
    body.append(box("RearTrunnion", (0, -0.105, -0.045), (0.03, 0.02, 0.04), "blued"))
    body.append(box("FrontTrunnion", (0, 0.12, -0.036), (0.03, 0.03, 0.034), "blued"))
    body.append(solid("TopCover", lambda b, m: (
        b.cyl((0, -0.005, -0.026), 0.0145, 0.205, m, axis="Y", seg=28),
        *[b.box((0, -0.08 + i * 0.022, -0.0125), (0.012, 0.004, 0.003), m) for i in range(7)],
    ), "blued"))
    # Klappvisier (hinten am Deckel vorne) und Korn auf dem Mündungsaufsatz
    body.append(solid("RearSight", lambda b, m: (
        b.box((0, 0.1, -0.01), (0.022, 0.024, 0.016), m),
        b.box((0, 0.106, -0.001), (0.016, 0.004, 0.008), m),
    ), "black", cuts=lambda c, m: c.box((0, 0.106, 0.0025), (0.0028, 0.02, 0.004), m)))
    body.append(lathe("Booster", [(0.0, 0.27), (0.012, 0.27), (0.016, 0.285), (0.016, 0.32), (0.0115, 0.33), (0.0, 0.33)], "black", center=(0, 0, ax)))
    body.append(solid("FrontSight", lambda b, m: (
        b.box((0, 0.262, ax + 0.02), (0.022, 0.016, 0.03), m),
        b.box((0, 0.262, -0.003), (0.0024, 0.004, 0.008), m),
    ), "black", cuts=lambda c, m: c.box((0, 0.262, -0.003), (0.016, 0.03, 0.007), m)))
    body.append(tube("Barrel", 0.12, 0.275, 0.0085, "blued", z=ax))
    # Gasrohr mit Holz, unterer Handschutz aus Holz mit Rillen
    body.append(tube("GasTube", 0.11, 0.215, 0.011, "wood", z=-0.012, seg=24))
    body.append(ext("LowerGuard", [(0.13, -0.036), (0.235, -0.036), (0.24, -0.05), (0.232, -0.068), (0.14, -0.07), (0.128, -0.058)], 0.04, "wood",
                    cuts=lambda c, m: [c.box((sx * 0.02, 0.15 + i * 0.022, -0.052), (0.004, 0.012, 0.03), m) for sx in (-1, 1) for i in range(4)]))
    # Pistolengriff (Bakelit), Abzugsbügel, Abzug, Sicherungshebel
    body.append(ext("Grip", [(-0.06, -0.056), (-0.032, -0.056), (-0.042, -0.09), (-0.058, -0.15), (-0.094, -0.146), (-0.082, -0.09), (-0.072, -0.056)], 0.03, "orange"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.034, -0.056), (0.04, -0.056), (0.042, -0.064), (0.03, -0.086), (-0.036, -0.088)], 0.008, m), "blued",
                      cuts=lambda c, m: c.extrude([(-0.03, -0.06), (0.032, -0.06), (0.026, -0.081), (-0.031, -0.083)], 0.02, m)))
    body.append(ext("Trigger", [(-0.006, -0.058), (0.002, -0.058), (-0.002, -0.078), (-0.009, -0.077)], 0.005, "black"))
    body.append(ext("Safety", [(-0.085, -0.028), (0.04, -0.03), (0.04, -0.036), (-0.085, -0.036)], 0.003, "blued", offset=0.0158))
    # Klappschaft (Gerippe)
    body.append(solid("Stock", lambda b, m: (
        b.box((0.008, -0.25, -0.03), (0.008, 0.26, 0.009), m),
        b.box((0.008, -0.25, -0.088), (0.008, 0.27, 0.009), m, rot=(-7, 0, 0)),
        b.box((0, -0.385, -0.066), (0.03, 0.012, 0.085), m),
    ), "blued"))
    slide.append(box("Charging", (0.02, 0.07, -0.03), (0.014, 0.012, 0.008), "steel", rot=(0, 0, 10)))
    mag.append(curved_mag("Mag", (0.062, -0.058), 0.19, 0.09, 0.034, 0.024, "orange", ribs=4))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b, m], S((-0.06, -0.06), sh), S((-0.077, -0.148), sh), depth=0.038, elbow=(0.24, sh - 0.3, -0.38))
        l = left_support(arms, [b, m, r], 0.18 + sh, -0.05, 0.02, 0.02)
        return r, l

    assemble("ak74u", body, mag, slide, sh, pose, (0, 0.335, ax))


# --------------------------------------------------------------------------------------
# M14
# --------------------------------------------------------------------------------------


def build_m14():
    sh = -0.005
    body, mag, slide = [], [], []
    ax = -0.03
    stock = [(-0.44, -0.032), (-0.2, -0.042), (-0.1, -0.045), (-0.05, -0.046), (0.33, -0.044), (0.335, -0.06), (0.32, -0.078),
             (0.12, -0.084), (0.02, -0.084), (-0.03, -0.08), (-0.075, -0.088), (-0.13, -0.12), (-0.25, -0.15), (-0.44, -0.17)]
    body.append(ext("Stock", stock, 0.042, "wood", bevel=0.0012))
    body.append(ext("ButtPlate", [(-0.452, -0.03), (-0.44, -0.03), (-0.44, -0.172), (-0.452, -0.17)], 0.044, "parkerized"))
    body.append(solid("Receiver", lambda b, m: (
        b.box((0, 0.025, -0.03), (0.03, 0.21, 0.034), m),
        b.box((0, -0.075, -0.024), (0.028, 0.03, 0.03), m),
    ), "parkerized", cuts=lambda c, m: c.box((0.0, 0.03, -0.012), (0.018, 0.1, 0.014), m)))
    # Diopter mit Schutzohren (Mitte der Lochkimme bei z = 0)
    body.append(solid("RearSight", lambda b, m: (
        b.box((0, -0.065, -0.007), (0.012, 0.01, 0.016), m),
        b.box((0.011, -0.065, -0.004), (0.004, 0.02, 0.022), m),
        b.box((-0.011, -0.065, -0.004), (0.004, 0.02, 0.022), m),
        b.cyl((0.017, -0.065, -0.01), 0.006, 0.006, m, axis="X", seg=16),
    ), "black", cuts=lambda c, m: c.cyl((0, -0.065, 0.0), 0.0018, 0.03, m, axis="Y", seg=12)))
    body.append(tube("Barrel", 0.13, 0.6, 0.0105, "parkerized", z=ax))
    body.append(tube("GasCyl", 0.4, 0.5, 0.008, "parkerized", z=ax - 0.018))
    body.append(box("Guard", (0, 0.215, -0.025), (0.03, 0.17, 0.016), "black"))
    body.append(lathe("Flash", [(0.0, 0.595), (0.0135, 0.595), (0.014, 0.64), (0.012, 0.66), (0.0, 0.66)], "parkerized", center=(0, 0, ax)))
    body.append(solid("FrontSight", lambda b, m: (
        b.box((0, 0.6, ax + 0.016), (0.016, 0.014, 0.012), m),
        b.box((0, 0.6, -0.004), (0.0024, 0.004, 0.014), m),
        b.box((0.007, 0.6, -0.004), (0.002, 0.008, 0.016), m),
        b.box((-0.007, 0.6, -0.004), (0.002, 0.008, 0.016), m),
    ), "black"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.045, -0.084), (0.02, -0.084), (0.018, -0.11), (-0.04, -0.112)], 0.009, m), "parkerized",
                      cuts=lambda c, m: c.extrude([(-0.04, -0.088), (0.014, -0.088), (0.012, -0.106), (-0.036, -0.107)], 0.02, m)))
    body.append(ext("Trigger", [(-0.016, -0.085), (-0.009, -0.085), (-0.012, -0.103), (-0.019, -0.102)], 0.005, "black"))
    slide.append(ext("OpRod", [(0.02, -0.022), (0.11, -0.022), (0.11, -0.03), (0.02, -0.03)], 0.006, "parkerized", offset=0.019))
    slide.append(box("OpHandle", (0.025, 0.05, -0.024), (0.012, 0.01, 0.012), "parkerized"))
    mag.append(ext("Mag", [(0.03, -0.075), (0.1, -0.075), (0.107, -0.18), (0.04, -0.186)], 0.026, "parkerized"))
    mag.append(ext("MagBase", [(0.036, -0.182), (0.109, -0.176), (0.11, -0.184), (0.036, -0.191)], 0.03, "parkerized"))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b, m], S((-0.075, -0.07), sh), S((-0.13, -0.128), sh), depth=0.042, elbow=(0.24, sh - 0.3, -0.38), trigger=(10, 22, 10))
        l = left_support(arms, [b, m, r], 0.22 + sh, -0.062, 0.021, 0.022)
        return r, l

    assemble("m14", body, mag, slide, sh, pose, (0, 0.66, ax))


# --------------------------------------------------------------------------------------
# Galil
# --------------------------------------------------------------------------------------


def build_galil():
    sh = 0.015
    body, mag, slide = [], [], []
    ax = -0.038
    body.append(solid("Receiver", lambda b, m: (
        b.box((0, 0.01, -0.042), (0.03, 0.27, 0.036), m),
        b.cyl((0, 0.0, -0.026), 0.015, 0.25, m, axis="Y", seg=28),
    ), "parkerized", cuts=lambda c, m: c.box((0.015, 0.04, -0.03), (0.01, 0.075, 0.012), m)))
    body.append(solid("RearSight", lambda b, m: (
        b.box((0, -0.105, -0.008), (0.024, 0.02, 0.018), m),
        b.box((0.009, -0.105, -0.002), (0.004, 0.012, 0.012), m),
        b.box((-0.009, -0.105, -0.002), (0.004, 0.012, 0.012), m),
    ), "black", cuts=lambda c, m: c.cyl((0, -0.105, 0.0), 0.0018, 0.04, m, axis="Y", seg=12)))
    body.append(ext("Handguard", [(0.145, -0.022), (0.33, -0.022), (0.335, -0.035), (0.33, -0.07), (0.16, -0.076), (0.145, -0.065)], 0.046, "polymer",
                    cuts=lambda c, m: [c.box((sx * 0.024, 0.18 + i * 0.03, -0.045), (0.008, 0.016, 0.022), m) for sx in (-1, 1) for i in range(5)]))
    body.append(tube("Barrel", 0.14, 0.52, 0.0095, "parkerized", z=ax))
    body.append(lathe("Flash", [(0.0, 0.5), (0.014, 0.5), (0.0145, 0.56), (0.0, 0.56)], "black", center=(0, 0, ax)))
    body.append(solid("FrontSight", lambda b, m: (
        b.box((0, 0.4, -0.022), (0.016, 0.024, 0.036), m),
        b.box((0, 0.4, -0.002), (0.0024, 0.004, 0.006), m),
        b.box((0.007, 0.4, 0.0), (0.002, 0.012, 0.012), m),
        b.box((-0.007, 0.4, 0.0), (0.002, 0.012, 0.012), m),
    ), "black"))
    body.append(solid("Carry", lambda b, m: (
        b.box((0, 0.075, 0.012), (0.012, 0.11, 0.008), m),
        b.box((0, 0.025, -0.002), (0.01, 0.01, 0.022), m),
        b.box((0, 0.125, -0.002), (0.01, 0.01, 0.022), m),
    ), "black"))
    body.append(ext("Grip", [(-0.06, -0.058), (-0.032, -0.058), (-0.04, -0.09), (-0.052, -0.148), (-0.09, -0.144), (-0.08, -0.09), (-0.074, -0.058)], 0.031, "polymer_checker"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.034, -0.06), (0.03, -0.06), (0.028, -0.088), (-0.036, -0.09)], 0.009, m), "parkerized",
                      cuts=lambda c, m: c.extrude([(-0.03, -0.064), (0.024, -0.064), (0.022, -0.083), (-0.031, -0.085)], 0.02, m)))
    body.append(ext("Trigger", [(-0.006, -0.06), (0.002, -0.06), (-0.002, -0.08), (-0.009, -0.079)], 0.005, "black"))
    body.append(solid("Stock", lambda b, m: (
        b.cyl((0.0, -0.26, -0.03), 0.009, 0.28, m, axis="Y", seg=16),
        b.box((0, -0.24, -0.09), (0.014, 0.25, 0.012), m, rot=(-11, 0, 0)),
        b.box((0, -0.405, -0.072), (0.034, 0.016, 0.1), m),
    ), "parkerized"))
    body.append(ext("ButtPad", [(-0.424, -0.022), (-0.412, -0.022), (-0.412, -0.122), (-0.424, -0.122)], 0.038, "rubber"))
    slide.append(box("Charging", (0.0, 0.06, -0.006), (0.012, 0.012, 0.016), "black", rot=(0, 0, 0)))
    mag.append(curved_mag("Mag", (0.06, -0.062), 0.21, 0.07, 0.032, 0.025, "parkerized", ribs=3))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b, m], S((-0.062, -0.062), sh), S((-0.074, -0.146), sh), depth=0.038, elbow=(0.24, sh - 0.3, -0.38))
        l = left_support(arms, [b, m, r], 0.24 + sh, -0.048, 0.023, 0.025)
        return r, l

    assemble("galil", body, mag, slide, sh, pose, (0, 0.56, ax))


# --------------------------------------------------------------------------------------
# RPD
# --------------------------------------------------------------------------------------


def build_rpd():
    sh = 0.06
    body, mag, slide = [], [], []
    ax = -0.038
    body.append(solid("Receiver", lambda b, m: (
        b.box((0, 0.02, -0.042), (0.036, 0.29, 0.04), m),
        b.box((0, 0.02, -0.018), (0.044, 0.22, 0.012), m),
    ), "blued", cuts=lambda c, m: c.box((0.018, 0.08, -0.04), (0.012, 0.08, 0.014), m)))
    body.append(solid("RearSight", lambda b, m: (
        b.box((0, -0.03, -0.007), (0.026, 0.03, 0.012), m),
        b.box((0, -0.03, 0.0), (0.018, 0.006, 0.006), m),
    ), "black", cuts=lambda c, m: c.box((0, -0.03, 0.002), (0.003, 0.03, 0.004), m)))
    body.append(ext("Guard", [(0.165, -0.032), (0.4, -0.036), (0.405, -0.05), (0.4, -0.068), (0.175, -0.08), (0.165, -0.07)], 0.05, "wood"))
    body.append(tube("Barrel", 0.16, 0.85, 0.012, "blued", z=ax))
    body.append(tube("GasTube", 0.4, 0.6, 0.007, "blued", z=ax - 0.02))
    body.append(lathe("Flash", [(0.0, 0.84), (0.015, 0.84), (0.016, 0.89), (0.0, 0.89)], "black", center=(0, 0, ax)))
    body.append(solid("FrontSight", lambda b, m: (
        b.box((0, 0.82, ax + 0.02), (0.016, 0.02, 0.03), m),
        b.box((0, 0.82, -0.003), (0.0026, 0.004, 0.008), m),
    ), "black"))
    body.append(solid("Bipod", lambda b, m: (
        b.box((0.012, 0.62, ax - 0.025), (0.006, 0.28, 0.006), m),
        b.box((-0.012, 0.62, ax - 0.025), (0.006, 0.28, 0.006), m),
        b.box((0, 0.48, ax - 0.022), (0.03, 0.02, 0.012), m),
    ), "blued"))
    stock = [(-0.46, -0.03), (-0.13, -0.036), (-0.105, -0.062), (-0.135, -0.09), (-0.2, -0.12), (-0.46, -0.16)]
    body.append(ext("Stock", stock, 0.044, "wood", bevel=0.0012))
    body.append(ext("Grip", [(-0.075, -0.06), (-0.045, -0.06), (-0.05, -0.1), (-0.062, -0.15), (-0.1, -0.146), (-0.09, -0.06)], 0.03, "wood"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.045, -0.06), (0.025, -0.06), (0.022, -0.088), (-0.046, -0.09)], 0.009, m), "blued",
                      cuts=lambda c, m: c.extrude([(-0.041, -0.064), (0.019, -0.064), (0.017, -0.083), (-0.042, -0.085)], 0.02, m)))
    slide.append(box("Charging", (0.024, 0.12, -0.04), (0.016, 0.012, 0.01), "steel"))
    mag.append(solid("Drum", lambda b, m: (
        b.cyl((0, 0.07, -0.13), 0.075, 0.068, m, axis="X", seg=40),
        b.cyl((0, 0.07, -0.13), 0.03, 0.074, m, axis="X", seg=24),
        b.box((0, 0.07, -0.06), (0.03, 0.035, 0.04), m),
    ), "blued"))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b, m], S((-0.066, -0.062), sh), S((-0.08, -0.148), sh), depth=0.042, elbow=(0.24, sh - 0.3, -0.38))
        l = left_support(arms, [b, m, r], 0.27 + sh, -0.056, 0.025, 0.024)
        return r, l

    assemble("rpd", body, mag, slide, sh, pose, (0, 0.89, ax))


# --------------------------------------------------------------------------------------
# DSR-50 (Bullpup mit Zielfernrohr)
# --------------------------------------------------------------------------------------


def build_dsr50():
    sh = 0.035
    body, mag, slide = [], [], []
    # Zielfernrohr als hohles Rohr: beim Zielen blickt man hindurch (Tunnelbild mit Fadenkreuz)
    body.append(solid("Scope", lambda b, m: (
        b.lathe([(0.016, -0.105), (0.0205, -0.105), (0.0205, -0.062), (0.0128, -0.045), (0.0128, 0.15), (0.0242, 0.19),
                 (0.0242, 0.255), (0.0212, 0.255), (0.0212, 0.195), (0.0112, 0.155), (0.0112, -0.04), (0.016, -0.06), (0.016, -0.105)],
                m, seg=40, axis="Y", caps=False),
        b.cyl((0, 0.075, 0.018), 0.009, 0.016, m, seg=20),
        b.cyl((0.018, 0.075, 0.0), 0.009, 0.016, m, axis="X", seg=20),
    ), "alu", angle=40.0))
    body.append(solid("Reticle", lambda b, m: (
        b.box((0, 0.17, 0.0), (0.024, 0.0005, 0.00045), m),
        b.box((0, 0.17, 0.0), (0.00045, 0.0005, 0.024), m),
        b.box((0, 0.17, -0.007), (0.0012, 0.0005, 0.01), m),
        b.box((0.007, 0.17, 0.0), (0.01, 0.0005, 0.0012), m),
        b.box((-0.007, 0.17, 0.0), (0.01, 0.0005, 0.0012), m),
    ), "black", bevel=0.00005))
    body.append(solid("Mounts", lambda b, m: (
        b.box((0, -0.01, -0.027), (0.018, 0.02, 0.018), m),
        b.box((0, 0.13, -0.027), (0.018, 0.02, 0.018), m),
    ), "black"))
    body.append(rail("Rail", -0.12, 0.3, -0.039))
    body.append(ext("Chassis", [(-0.42, -0.04), (0.33, -0.04), (0.335, -0.06), (0.33, -0.085), (0.12, -0.09), (0.05, -0.09), (-0.05, -0.1),
                               (-0.12, -0.13), (-0.42, -0.14)], 0.05, "polymer",
                    cuts=lambda c, m: [c.box((sx * 0.025, 0.18 + i * 0.035, -0.065), (0.008, 0.018, 0.02), m) for sx in (-1, 1) for i in range(4)]))
    body.append(ext("CheekRest", [(-0.38, -0.04), (-0.12, -0.04), (-0.14, -0.026), (-0.36, -0.024)], 0.03, "rubber"))
    body.append(tube("Barrel", 0.3, 0.8, 0.0125, "blued", z=-0.06))
    body.append(solid("Brake", lambda b, m: b.box((0, 0.82, -0.06), (0.042, 0.06, 0.032), m), "black",
                      cuts=lambda c, m: [c.box((0, 0.8 + i * 0.016, -0.06), (0.05, 0.007, 0.022), m) for i in range(3)]))
    body.append(ext("Grip", [(0.06, -0.088), (0.09, -0.088), (0.08, -0.12), (0.07, -0.18), (0.035, -0.176), (0.045, -0.12), (0.048, -0.088)], 0.032, "polymer_checker"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(0.09, -0.09), (0.15, -0.09), (0.15, -0.118), (0.088, -0.12)], 0.009, m), "polymer",
                      cuts=lambda c, m: c.extrude([(0.094, -0.094), (0.145, -0.094), (0.145, -0.113), (0.092, -0.115)], 0.02, m)))
    body.append(ext("Trigger", [(0.112, -0.09), (0.12, -0.09), (0.116, -0.11), (0.109, -0.109)], 0.005, "black"))
    body.append(ext("ButtPad", [(-0.435, -0.038), (-0.42, -0.038), (-0.42, -0.142), (-0.435, -0.142)], 0.05, "rubber"))
    slide.append(box("Bolt", (0.028, -0.06, -0.06), (0.03, 0.012, 0.012), "steel"))
    mag.append(ext("Mag", [(-0.23, -0.12), (-0.13, -0.12), (-0.13, -0.17), (-0.23, -0.17)], 0.034, "black"))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b, m], S((0.07, -0.09), sh), S((0.052, -0.178), sh), depth=0.04, elbow=(0.24, sh - 0.2, -0.38))
        l = left_support(arms, [b, m, r], 0.27 + sh, -0.065, 0.025, 0.024, elbow=(-0.27, sh - 0.05, -0.4))
        return r, l

    assemble("dsr50", body, mag, slide, sh, pose, (0, 0.85, -0.06))


# --------------------------------------------------------------------------------------
# Olympia (Doppelflinte)
# --------------------------------------------------------------------------------------


def build_olympia():
    sh = -0.07
    body, mag, slide = [], [], []
    stock = [(-0.44, -0.03), (-0.2, -0.034), (-0.08, -0.036), (-0.03, -0.034), (-0.03, -0.07), (-0.075, -0.082), (-0.13, -0.112), (-0.25, -0.14), (-0.44, -0.165)]
    body.append(ext("Stock", stock, 0.04, "wood", bevel=0.0012))
    body.append(ext("ButtPad", [(-0.452, -0.028), (-0.44, -0.028), (-0.44, -0.167), (-0.452, -0.165)], 0.042, "rubber"))
    body.append(solid("Action", lambda b, m: (
        b.box((0, 0.02, -0.035), (0.046, 0.1, 0.044), m),
        b.box((0, -0.04, -0.03), (0.022, 0.05, 0.012), m),
    ), "steel"))
    for sx in (-1, 1):
        body.append(tube(f"Barrel{sx}", 0.07, 0.68, 0.0115, "blued", z=-0.026, x=sx * 0.0118, seg=28))
    body.append(box("Rib", (0, 0.37, -0.0115), (0.008, 0.6, 0.006), "blued"))
    body.append(solid("Bead", lambda b, m: b.sphere((0, 0.665, -0.007), 0.0028, m, seg=12, rings=6), "brass"))
    body.append(ext("Forend", [(0.075, -0.04), (0.29, -0.04), (0.3, -0.05), (0.285, -0.064), (0.09, -0.066), (0.075, -0.056)], 0.046, "wood"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.03, -0.056), (0.04, -0.056), (0.038, -0.082), (-0.026, -0.086)], 0.008, m), "blued",
                      cuts=lambda c, m: c.extrude([(-0.026, -0.06), (0.034, -0.06), (0.032, -0.077), (-0.022, -0.081)], 0.02, m)))
    body.append(ext("Triggers", [(0.0, -0.057), (0.008, -0.057), (0.004, -0.075), (-0.003, -0.074)], 0.012, "black"))
    body.append(box("Lever", (0, -0.025, -0.008), (0.008, 0.035, 0.008), "steel", rot=(0, 0, 15)))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b], S((-0.04, -0.045), sh), S((-0.1, -0.1), sh), depth=0.042, elbow=(0.3, sh - 0.22, -0.46), trigger=(10, 22, 10))
        l = left_support(arms, [b, r], 0.2 + sh, -0.053, 0.023, 0.013)
        return r, l

    assemble("olympia", body, mag, slide, sh, pose, (0, 0.68, -0.026))


# --------------------------------------------------------------------------------------
# Remington 870 (Vorderschaftrepetierer)
# --------------------------------------------------------------------------------------


def build_remington():
    sh = -0.06
    body, mag, slide = [], [], []
    stock = [(-0.44, -0.03), (-0.2, -0.036), (-0.09, -0.038), (-0.06, -0.04), (-0.06, -0.07), (-0.095, -0.085), (-0.14, -0.11), (-0.25, -0.14), (-0.44, -0.165)]
    body.append(ext("Stock", stock, 0.04, "wood", bevel=0.0012))
    body.append(ext("ButtPad", [(-0.455, -0.028), (-0.44, -0.028), (-0.44, -0.167), (-0.455, -0.165)], 0.042, "rubber"))
    body.append(solid("Receiver", lambda b, m: b.extrude([(-0.065, -0.012), (0.12, -0.012), (0.12, -0.068), (-0.065, -0.07)], 0.03, m), "blued",
                      cuts=lambda c, m: c.box((0.014, 0.04, -0.035), (0.01, 0.07, 0.025), m)))
    body.append(tube("Barrel", 0.11, 0.6, 0.0115, "blued", z=-0.025))
    body.append(tube("MagTube", 0.12, 0.5, 0.0105, "blued", z=-0.05))
    body.append(box("Rib", (0, 0.35, -0.0115), (0.008, 0.5, 0.004), "blued"))
    body.append(solid("Bead", lambda b, m: b.sphere((0, 0.59, -0.007), 0.0028, m, seg=12, rings=6), "brass"))
    body.append(tube("Cap", 0.5, 0.525, 0.012, "blued", z=-0.05))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.05, -0.068), (0.015, -0.068), (0.012, -0.095), (-0.045, -0.098)], 0.009, m), "black",
                      cuts=lambda c, m: c.extrude([(-0.046, -0.072), (0.01, -0.072), (0.008, -0.09), (-0.042, -0.093)], 0.02, m)))
    body.append(ext("Trigger", [(-0.022, -0.069), (-0.015, -0.069), (-0.018, -0.088), (-0.025, -0.087)], 0.005, "black"))
    slide.append(solid("Pump", lambda b, m: (
        b.cyl((0, 0.26, -0.052), 0.022, 0.18, m, axis="Y", seg=28),
    ), "wood_checker", cuts=lambda c, m: [c.cyl((0, 0.18 + i * 0.016, -0.052), 0.024, 0.004, m, axis="Y", seg=28) for i in range(10)]))
    slide.append(ext("ActionBar", [(0.12, -0.04), (0.18, -0.042), (0.18, -0.046), (0.12, -0.044)], 0.003, "blued", offset=0.0145))

    def pose(arms, b, m, s):
        r = right_grip(arms, [b], S((-0.07, -0.05), sh), S((-0.13, -0.105), sh), depth=0.042, elbow=(0.3, sh - 0.22, -0.46), trigger=(10, 22, 10))
        l = left_support(arms, [b, s, r], 0.26 + sh, -0.052, 0.022, 0.022)
        return r, l

    assemble("remington", body, mag, slide, sh, pose, (0, 0.6, -0.025))


# --------------------------------------------------------------------------------------
# MGL (Granatwerfer mit Trommel)
# --------------------------------------------------------------------------------------


def build_mgl():
    sh = -0.06
    body, mag, slide = [], [], []
    ax = -0.05
    body.append(tube("Barrel", 0.11, 0.33, 0.027, "parkerized", z=ax, seg=36))
    body.append(lathe("Muzzle", [(0.0215, 0.33), (0.029, 0.33), (0.029, 0.345), (0.0215, 0.345)], "black", center=(0, 0, ax)))
    body.append(box("Frame", (0, 0.0, ax + 0.045), (0.034, 0.26, 0.012), "parkerized"))
    body.append(box("FrameLow", (0, -0.02, ax - 0.06), (0.03, 0.2, 0.014), "parkerized"))
    body.append(rail("Rail", -0.12, 0.1, ax + 0.051))
    body.append(solid("Sight", lambda b, m: (
        b.box((0, 0.0, -0.012), (0.03, 0.04, 0.016), m),
        b.box((0, 0.0, -0.002), (0.024, 0.006, 0.006), m),
    ), "black", cuts=lambda c, m: c.box((0, 0.0, 0.0), (0.004, 0.03, 0.005), m)))
    body.append(box("Post", (0, 0.24, -0.006), (0.004, 0.006, 0.014), "black"))
    body.append(ext("Grip", [(-0.09, -0.11), (-0.06, -0.11), (-0.068, -0.15), (-0.08, -0.2), (-0.118, -0.196), (-0.108, -0.11)], 0.032, "polymer_checker"))
    body.append(ext("ForeGrip", [(0.14, -0.08), (0.17, -0.08), (0.168, -0.17), (0.138, -0.17)], 0.03, "polymer_checker"))
    body.append(solid("Stock", lambda b, m: (
        b.cyl((0, -0.25, ax + 0.02), 0.012, 0.22, m, axis="Y", seg=20),
        b.box((0, -0.37, ax - 0.02), (0.034, 0.02, 0.11), m),
    ), "polymer"))
    body.append(solid("TrigGuard", lambda b, m: b.extrude([(-0.065, -0.108), (0.0, -0.108), (-0.002, -0.134), (-0.066, -0.136)], 0.009, m), "polymer",
                      cuts=lambda c, m: c.extrude([(-0.061, -0.112), (-0.004, -0.112), (-0.006, -0.13), (-0.062, -0.131)], 0.02, m)))
    def drum(b, m):
        b.cyl((0, 0.03, ax), 0.06, 0.15, m, axis="Y", seg=36)
    mag.append(solid("Drum", drum, "parkerized", cuts=lambda c, m: [
        *[c.cyl((math.cos(k * math.tau / 6) * 0.042, 0.105, ax + math.sin(k * math.tau / 6) * 0.042), 0.0205, 0.03, m, axis="Y", seg=20) for k in range(6)],
        *[c.cyl((math.cos((k + 0.5) * math.tau / 6) * 0.062, 0.03, ax + math.sin((k + 0.5) * math.tau / 6) * 0.062), 0.008, 0.12, m, axis="Y", seg=12) for k in range(6)],
    ]))

    def pose(arms, b, m, s):
        from .parts import left_vgrip

        r = right_grip(arms, [b, m], S((-0.085, -0.112), sh), S((-0.1, -0.198), sh), depth=0.04, elbow=(0.24, sh - 0.32, -0.38))
        l = left_vgrip(arms, [b, m, r], S((0.155, -0.082), sh), S((0.153, -0.17), sh), elbow=(-0.27, sh - 0.18, -0.4))
        return r, l

    assemble("mgl", body, mag, slide, sh, pose, (0, 0.345, ax))
