"""Gemeinsame Teile der Waffenmodelle: Materialien, behandschuhte Hände mit Fingern,
Unterarme mit Ärmeln, Formhilfen und Export.

Konventionen (Blender-Koordinaten, Meter):
  * Lauf zeigt nach +Y, Visierlinie liegt auf x=0, z=0 (Kimme bei y≈0)
  * Objekte: "Body" (Waffe + rechte Hand), "Mag" (Magazin), "Slide" (Schlitten/Pumpe),
    "ArmL" (linke Hand), leeres Objekt "Muzzle" an der Mündung
  * Materialien "W_Hands"/"W_Sleeve"/"W_Glow" werden beim Pack-a-Punch nicht ersetzt.
"""

import os

from mathutils import Matrix, Vector

from lib import MODEL_DIR, empty, export_gltf, hexcol, material

C = hexcol
OUT = os.path.join(MODEL_DIR, "weapons")
UVS = 0.12


def mats():
    return dict(
        metal=material("W_Metal", tex="gunmetal", normal="gunmetal_n", metal=0.85, rough=0.42),
        metal_l=material("W_MetalLight", tex="gunmetal", normal="gunmetal_n", metal=0.9, rough=0.35, color=C("#c8ccd0")),
        black=material("W_Black", tex="gunmetal", metal=0.6, rough=0.55, color=C("#6a6a6a")),
        poly=material("W_Polymer", tex="polymer", normal="polymer_n", rough=0.75),
        wood=material("W_Wood", tex="gun_wood", normal="gun_wood_n", rough=0.55),
        wood_d=material("W_WoodDark", tex="gun_wood", normal="gun_wood_n", rough=0.6, color=C("#8a7464")),
        hands=material("W_Hands", tex="polymer", rough=0.8, color=C("#b08a6a")),
        skin=material("W_HandsSkin", tex="skin", rough=0.7, color=C("#e6c8b0")),
        sleeve=material("W_Sleeve", tex="fabric", normal="fabric_n", rough=0.95, color=C("#6c6a52")),
        brass=material("W_Brass", color=C("#b8913d"), metal=1.0, rough=0.3),
        glow=material("W_Glow", color=C("#7dff7a"), emission=C("#4dff4a"), emission_strength=10.0, rough=0.3),
        mag_ak=material("W_MagAk", tex="polymer", rough=0.6, color=C("#c0582e")),
        glass=material("W_Lens", color=C("#16314a"), metal=0.9, rough=0.05),
        bronze=material("W_Bronze", tex="gunmetal", metal=1.0, rough=0.3, color=C("#d9a05a")),
        red=material("W_Red", tex="metal_paint", rough=0.45, metal=0.3, color=C("#a8241c")),
    )


# --------------------------------------------------------------------------------------
# Hände und Arme
# --------------------------------------------------------------------------------------


# --------------------------------------------------------------------------------------
# Hände und Arme
# --------------------------------------------------------------------------------------


def _frame(center, rot_deg):
    """Lokales Koordinatensystem (Matrix) einer Hand: Ursprung, Drehung (Euler XYZ, Grad)."""
    from mathutils import Euler
    import math

    r = Euler(tuple(math.radians(a) for a in rot_deg), "XYZ").to_matrix().to_4x4()
    return Matrix.Translation(Vector(center)) @ r


def finger(mb, m, frame, base, curl_dir, length=0.05, r=0.0092):
    """Ein Finger aus zwei Gliedern, der sich um einen Griff legt (lokale Koordinaten)."""
    b = Vector(base)
    d1 = Vector(curl_dir[0]).normalized()
    d2 = Vector(curl_dir[1]).normalized()
    p1 = b + d1 * length * 0.55
    p2 = p1 + d2 * length * 0.45
    mb.cyl_between(frame @ b, frame @ p1, r, m["hands"], seg=8)
    mb.cyl_between(frame @ p1, frame @ p2, r * 0.92, m["hands"], seg=8)
    mb.sphere(frame @ p1, r * 1.02, m["hands"], seg=8, rings=5)
    mb.sphere(frame @ p2, r * 0.9, m["hands"], seg=8, rings=5)


def grip_hand(mb, m, center, rot=(0, 0, 0), side=1.0, grip_w=0.03, grip_d=0.045):
    """Rechte (side=+1) bzw. linke (side=-1) Hand, die einen senkrechten Griff umschließt.

    Lokal: Griff entlang z, Vorderseite +y. Handfläche liegt außen (x·side), die Finger
    laufen vorne um den Griff herum, der Daumen liegt auf der anderen Seite."""
    f = _frame(center, rot)
    s = side
    q = f.to_quaternion()
    # Handfläche / Handrücken
    mb.box(f @ Vector((s * (grip_w / 2 + 0.011), -0.004, 0.0)), (0.024, grip_d + 0.022, 0.082), m["hands"], rot=q, bevel=0.009, uv=UVS)
    # vier Finger, oben nach unten
    for i in range(4):
        z = 0.03 - i * 0.02
        base = (s * (grip_w / 2 + 0.006), grip_d / 2 - 0.002, z)
        finger(mb, m, f, base, [(-s * 0.35, 1.0, 0.0), (-s * 1.0, 0.25, 0.0)], length=0.05 - i * 0.003, r=0.0088 - i * 0.0004)
    # Daumen über die andere Seite nach vorn
    tb = Vector((s * 0.012, -0.018, 0.04))
    tm = Vector((-s * 0.016, 0.006, 0.045))
    te = Vector((-s * 0.02, 0.034, 0.04))
    mb.cyl_between(f @ tb, f @ tm, 0.0105, m["hands"], seg=8)
    mb.cyl_between(f @ tm, f @ te, 0.0095, m["hands"], seg=8)
    mb.sphere(f @ te, 0.0095, m["hands"], seg=8, rings=5)
    # Handgelenk-Ansatz
    return f @ Vector((s * 0.008, -0.03, -0.035))


def support_hand(mb, m, center, rot=(0, 0, 0), width=0.045):
    """Linke Stützhand unter einem Handschutz: Handfläche links unten, Finger um die
    Unterseite gelegt, Daumen links oben."""
    f = _frame(center, rot)
    q = f.to_quaternion()
    mb.box(f @ Vector((-width / 2 - 0.006, 0.0, -0.012)), (0.024, 0.08, 0.06), m["hands"], rot=q, bevel=0.009, uv=UVS)
    for i in range(4):
        y = 0.03 - i * 0.02
        base = (-width / 2 + 0.002, y, -0.03)
        finger(mb, m, f, base, [(1.0, 0.0, -0.35), (1.0, 0.0, 0.6)], length=0.052 - i * 0.003, r=0.0088 - i * 0.0004)
    tb = Vector((-width / 2 - 0.008, -0.03, 0.0))
    te = Vector((-width / 2 + 0.002, 0.03, 0.012))
    mb.cyl_between(f @ tb, f @ te, 0.0105, m["hands"], seg=8)
    mb.sphere(f @ te, 0.0098, m["hands"], seg=8, rings=5)
    return f @ Vector((-width / 2 - 0.012, -0.035, -0.02))


def arm(mb, m, wrist, elbow, r_wrist=0.026, r_elbow=0.036, sleeve=True):
    """Unterarm vom Handgelenk zum Ellbogen, mit Ärmel und Bündchen."""
    w, e = Vector(wrist), Vector(elbow)
    d = (e - w).normalized()
    mb.cyl_between(w, w + d * 0.11, r_wrist, m["hands"], seg=12, r2=r_wrist * 1.12)
    if sleeve:
        s0 = w + d * 0.09
        mb.cyl_between(s0, e + d * 0.22, r_elbow + 0.008, m["sleeve"], seg=12, r2=r_elbow + 0.016)
        mb.cyl_between(s0 - d * 0.004, s0 + d * 0.022, r_wrist + 0.016, m["sleeve"], seg=12)
    else:
        mb.cyl_between(w, e, r_wrist, m["hands"], seg=12, r2=r_elbow)


def right_hand_on_grip(mb, m, grip_center, grip_tilt=15.0, elbow=(0.13, -0.40, -0.30)):
    wrist = grip_hand(mb, m, grip_center, rot=(grip_tilt, 0, 0), side=1.0)
    arm(mb, m, wrist, elbow)


def left_hand_support(mb, m, center, elbow=(-0.2, -0.34, -0.3), rot=(0, 0, 25)):
    wrist = support_hand(mb, m, center, rot=(rot[0], 0, 0))
    arm(mb, m, wrist, elbow)


def left_hand_pistol(mb, m, center, elbow=(-0.17, -0.34, -0.3), tilt=14.0, grip_w=0.06, grip_d=0.06):
    """Linke Hand umschließt bei Pistolen die rechte Hand (Beidhand-Griff) oder einen
    senkrechten Vordergriff."""
    wrist = grip_hand(mb, m, center, rot=(tilt, 0, 0), side=-1.0, grip_w=grip_w, grip_d=grip_d)
    arm(mb, m, wrist, elbow)


def finish(name, objs, muzzle):
    empty("Muzzle", loc=muzzle)
    export_gltf(os.path.join(OUT, name + ".gltf"))


def ring(mb, mat, y, r, z=0.0, depth=0.01, seg=16):
    mb.cyl((0, y, z), r, depth, mat, axis="Y", seg=seg, uv=UVS)


# --------------------------------------------------------------------------------------
# Waffen
# --------------------------------------------------------------------------------------


def smg_receiver(body, m, length=(-0.12, 0.2), h=(-0.052, -0.012), w=0.036, mat="metal"):
    y0, y1 = length
    z0, z1 = h
    body.box(((0), (y0 + y1) / 2, (z0 + z1) / 2), (w, y1 - y0, z1 - z0), m[mat], bevel=0.004, uv=UVS)


def pistol_grip(body, m, y, z_top=-0.05, mat="poly", length=0.1, tilt=18):
    body.extrude([(y - 0.02, z_top), (y + 0.018, z_top), (y + 0.002, z_top - length), (y - 0.04, z_top - length + 0.006)], 0.03, m[mat], bevel=0.003)


def fist(mb, m, center, size=(0.052, 0.085, 0.1), rot=(0, 0, 0)):
    """Geschlossene Hand (Handschuh) um einen Griff: Handfläche + Fingerwulst + Daumen."""
    c = Vector(center)
    mb.box(c, size, m["hands"], rot=rot, bevel=0.012, uv=UVS)
    # Fingerknöchel-Reihe vorne
    R = Matrix.Rotation(0, 3, "Z")
    from bhelp import _mat3

    R = _mat3(rot)
    for i in range(4):
        off = R @ Vector((0.0, size[1] * 0.42, size[2] * (0.32 - i * 0.21)))
        mb.box(c + off, (size[0] * 1.02, 0.028, 0.022), m["hands"], rot=rot, bevel=0.008, uv=UVS)
