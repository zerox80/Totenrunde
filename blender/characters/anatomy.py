"""Skelett und Proportionen der selbst modellierten Figuren.

Alle Maße in Metern, nach anthropometrischen Mittelwerten (Mann 1,80 m, Frau 1,68 m).
Figur schaut nach +Y, linke Körperseite bei -X, Füße (mit Schuhsohle) auf z = 0.
Die Arme stehen in einer A-Pose (45° unter der Waagerechten), die Daumen zeigen nach vorn.

skeleton(kind) liefert ein Dict `sk` mit
  * Skalaren (Höhen, Breiten, Radien) für die Bildhauerei
  * Gelenkpunkten (numpy) und Achsen je Seite ("L"/"R")
  * joints: mathutils-Vektoren mit den Namen, die zombie_rig.bones_from_joints erwartet
"""

import math

import numpy as np

from sculpt.sdf import frame, norm, rot_axis, v3

SIDES = (("L", -1.0), ("R", 1.0))


def _base(kind):
    if kind == "female":
        return dict(
            kind="female",
            s=0.935,  # Längenmaßstab relativ zum Mann
            w=0.92,  # Breitenmaßstab Oberkörper
            sole=0.022,
            head_top=1.68,
            head_c=1.592,
            hs=0.94,  # Kopfmaßstab
            neck_base=1.405,
            sh_x=0.158,
            sh_z=1.348,
            chest_z=1.22,
            waist_z=1.02,
            hip_x=0.086,
            hip_z=0.875,
            knee_x=0.094,
            knee_z=0.465,
            ankle_x=0.104,
            ankle_z=0.072,
            upper_arm=0.278,
            forearm=0.238,
            hand=0.172,
            # Rumpf-Querschnitte (z, Halbbreite, Halbtiefe, y-Mitte)
            torso=[
                (0.875, 0.170, 0.108, -0.014),
                (0.94, 0.168, 0.102, -0.010),
                (1.02, 0.126, 0.088, -0.002),
                (1.10, 0.134, 0.092, 0.000),
                (1.17, 0.142, 0.098, 0.002),
                (1.24, 0.146, 0.100, 0.000),
                (1.31, 0.140, 0.094, -0.006),
                (1.37, 0.112, 0.076, -0.016),
                (1.415, 0.066, 0.056, -0.018),
            ],
            thigh_r=0.082,
            calf_r=0.050,
            arm_r=0.040,
            muscle=0.55,  # Muskeldefinition (0..1)
        )
    return dict(
        kind="male",
        s=1.0,
        w=1.0,
        sole=0.025,
        head_top=1.80,
        head_c=1.705,
        hs=1.0,
        neck_base=1.50,
        sh_x=0.176,
        sh_z=1.442,
        chest_z=1.31,
        waist_z=1.07,
        hip_x=0.088,
        hip_z=0.925,
        knee_x=0.098,
        knee_z=0.49,
        ankle_x=0.108,
        ankle_z=0.075,
        upper_arm=0.30,
        forearm=0.258,
        hand=0.19,
        torso=[
            (0.925, 0.160, 0.104, -0.012),
            (1.00, 0.156, 0.098, -0.006),
            (1.07, 0.143, 0.096, 0.000),
            (1.15, 0.147, 0.099, 0.002),
            (1.23, 0.155, 0.104, 0.002),
            (1.31, 0.158, 0.106, 0.000),
            (1.39, 0.153, 0.100, -0.006),
            (1.45, 0.124, 0.082, -0.016),
            (1.505, 0.072, 0.060, -0.020),
        ],
        thigh_r=0.088,
        calf_r=0.054,
        arm_r=0.047,
        muscle=0.85,
    )


def skeleton(kind="male"):
    sk = _base(kind)
    lift = sk["sole"]
    # alle Höhen um die Schuhsohle anheben
    for key in ("head_top", "head_c", "neck_base", "sh_z", "chest_z", "waist_z", "hip_z", "knee_z", "ankle_z"):
        sk[key] += lift
    sk["torso"] = [(z + lift, rx, ry, cy) for z, rx, ry, cy in sk["torso"]]
    s = sk["s"]
    sk["hc"] = v3((0.0, 0.0, sk["head_c"]))
    sk["chin_z"] = sk["head_c"] - 0.135 * sk["hs"]
    sk["crotch_z"] = sk["hip_z"] - 0.095 * s
    sk["side"] = {}
    for side, sg in SIDES:
        d = {}
        d["sg"] = sg
        # Schulter, Arm (A-Pose, leicht nach vorn), Ellbogen minimal gebeugt
        d["acromion"] = v3((sg * (sk["sh_x"] + 0.019 * s), -0.014, sk["sh_z"] + 0.028 * s))
        d["S"] = v3((sg * sk["sh_x"], -0.012, sk["sh_z"]))
        ang = math.radians(45.0)
        dir1 = norm((sg * math.cos(ang), 0.06, -math.sin(ang)))
        d["E"] = d["S"] + dir1 * sk["upper_arm"]
        dir2 = norm(dir1 + v3((0, 0.13, 0)))
        d["W"] = d["E"] + dir2 * sk["forearm"]
        d["dir1"], d["dir2"] = dir1, dir2
        d["hand_end"] = d["W"] + dir2 * sk["hand"]
        # Hand-Basis: a = Richtung Finger, w = Richtung Daumen (vorn), n = Handfläche
        a = dir2
        w = v3((0, 1, 0)) - a * np.dot(v3((0, 1, 0)), a)
        w = norm(w)
        n = norm(-sg * np.cross(a, w))
        d["ha"], d["hw"], d["hn"] = a, w, n
        # Beine
        d["H"] = v3((sg * sk["hip_x"], 0.0, sk["hip_z"]))
        d["K"] = v3((sg * sk["knee_x"], 0.012 * s, sk["knee_z"]))
        d["A"] = v3((sg * sk["ankle_x"], -0.022 * s, sk["ankle_z"]))
        d["toe"] = v3((sg * (sk["ankle_x"] + 0.006), d["A"][1] + 0.135 * s, lift + 0.02))
        sk["side"][side] = d
    sk["joints"] = _joints(sk)
    return sk


def _joints(sk):
    from mathutils import Vector

    s = sk["s"]
    lift = sk["sole"]
    j = {}
    j["pelvis"] = (0.0, -0.012, sk["hip_z"] + 0.03 * s)
    j["spine"] = (0.0, -0.004, sk["waist_z"] + 0.01 * s)
    j["chest"] = (0.0, -0.008, sk["chest_z"] - 0.07 * s)
    j["neck"] = (0.0, -0.022, sk["neck_base"] - 0.01 * s)
    j["head"] = (0.0, -0.008, sk["head_c"] - 0.105 * sk["hs"])
    j["head_top"] = (0.0, -0.008, sk["head_top"])
    for side, d in sk["side"].items():
        sg = d["sg"]
        j[f"shoulder_in.{side}"] = (sg * 0.03 * s, -0.016, sk["sh_z"] + 0.016 * s)
        j[f"shoulder.{side}"] = tuple(d["S"])
        j[f"elbow.{side}"] = tuple(d["E"])
        j[f"wrist.{side}"] = tuple(d["W"])
        j[f"hand_end.{side}"] = tuple(d["hand_end"])
        j[f"hip.{side}"] = tuple(d["H"])
        j[f"knee.{side}"] = tuple(d["K"])
        j[f"ankle.{side}"] = tuple(d["A"])
        j[f"toe.{side}"] = tuple(d["toe"])
    _ = lift
    return {k: Vector(v) for k, v in j.items()}


def limb_frame(a, b, up):
    """(u, v, w) für eine Gliedmaße a->b; v zeigt möglichst nach up."""
    return frame(v3(b) - v3(a), up)


def bend(direction, axis, deg):
    return rot_axis(axis, math.radians(deg)) @ v3(direction)
