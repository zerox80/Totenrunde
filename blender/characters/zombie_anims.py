"""Animationen des Zombies, prozedural beschrieben (Sinus-Zyklen und Keyframe-Listen)
in Armature-Achsen und in lokale Knochenrotationen umgerechnet.

Achsen (Figur schaut nach +Y): +X-Rotation schwingt Beine nach vorn, Rumpf vorbeugen = -X,
Arme werden zuerst abgesenkt (um Y) und dann nach vorn geschwungen (um X)."""

import math

import bpy
from mathutils import Euler, Quaternion, Vector

from .zombie_rig import FPS

REST = {}


# Korrektur für Arme, deren Ruhelage keine T-Pose ist (z. B. A-Pose der Base Meshes):
# Rotation von der T-Pose-Richtung (±X) zur tatsächlichen Ruherichtung des Oberarms.
ARM_FIX = {}


def cache_rest(arm):
    for b in arm.data.bones:
        REST[b.name] = b.matrix_local.copy()
    for side, s in (("L", -1.0), ("R", 1.0)):
        b = arm.data.bones.get(f"UpperArm.{side}")
        if b is None:
            continue
        rest_dir = (b.tail_local - b.head_local).normalized()
        ARM_FIX[side] = Vector((s, 0.0, 0.0)).rotation_difference(rest_dir)


def arm_q(rot, order="XYZ"):
    return Euler(tuple(math.radians(a) for a in rot), order).to_quaternion()


def to_local(bone, q_arm):
    r = REST[bone].to_quaternion()
    return r.inverted() @ q_arm @ r


def to_local_loc(bone, v):
    r = REST[bone].to_3x3()
    return r.inverted() @ Vector(v)


def lerp_keys(keys, t):
    """keys: Liste (zeit, wert-tupel), stückweise weich (smoothstep) interpoliert."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t0 <= t <= t1:
            u = (t - t0) / max(t1 - t0, 1e-6)
            u = u * u * (3 - 2 * u)
            return tuple(a + (b - a) * u for a, b in zip(v0, v1))
    return keys[-1][1]


def S(phase, amp, off=0.0):
    return amp * math.sin(math.tau * (phase + off))


def Cs(phase, amp, off=0.0):
    return amp * math.cos(math.tau * (phase + off))


def side_sign(bone):
    return -1.0 if bone.endswith(".L") else 1.0


def pose_to_quats(pose):
    """pose: dict Knochen -> Werte. Arme: (schwung, runter, drehung); Ellbogen: beuge;
    Beine: (schwung, spreizen, drehung); Knie: beuge; sonst Euler XYZ in Armature-Achsen."""
    out = {}
    for bone, v in pose.items():
        if bone.startswith("UpperArm"):
            s = side_sign(bone)
            swing, down, twist = v
            # erst absenken (um Y), dann nach vorn schwingen (um X) – formuliert ab T-Pose
            q = arm_q((swing, s * down, twist * s), "YXZ")
            fix = ARM_FIX.get(bone[-1])
            if fix is not None:
                q = q @ fix.inverted()
        elif bone.startswith("LowerArm"):
            s = side_sign(bone)
            bend = v[0] if isinstance(v, tuple) else v
            q = arm_q((0.0, 0.0, s * bend))
            fix = ARM_FIX.get(bone[-1])
            if fix is not None:
                q = fix @ q @ fix.inverted()
        elif bone.startswith("UpperLeg"):
            s = side_sign(bone)
            swing, spread, twist = v
            q = arm_q((swing, -s * spread, twist), "YXZ")
        elif bone.startswith("LowerLeg"):
            bend = v[0] if isinstance(v, tuple) else v
            q = arm_q((-bend, 0.0, 0.0))
        elif bone.startswith("Hand"):
            s = side_sign(bone)
            q = arm_q((v[0], v[1] * s, v[2] * s))
            fix = ARM_FIX.get(bone[-1])
            if fix is not None:
                q = fix @ q @ fix.inverted()
        else:
            q = arm_q(v)
        out[bone] = q
    return out


def make_action(arm, name, frames, pose_fn, loop=True):
    """pose_fn(t) -> (pose_dict, hips_loc) mit t in [0,1]."""
    act = bpy.data.actions.new(name)
    act.use_fake_user = True
    arm.animation_data.action = act
    pbs = arm.pose.bones
    for pb in pbs:
        pb.rotation_mode = "QUATERNION"
    n = frames if loop else frames
    for f in range(0, frames + 1):
        t = f / frames
        if loop and f == frames:
            t = 0.0
        pose, hips_loc = pose_fn(t)
        quats = pose_to_quats(pose)
        for pb in pbs:
            q = quats.get(pb.name, Quaternion())
            pb.rotation_quaternion = to_local(pb.name, q)
            pb.keyframe_insert("rotation_quaternion", frame=f + 1)
            if pb.name == "Hips":
                pb.location = to_local_loc("Hips", hips_loc)
                pb.keyframe_insert("location", frame=f + 1)
            else:
                pb.location = (0, 0, 0)
    track = arm.animation_data.nla_tracks.new()
    track.name = name
    track.strips.new(name, 1, act)
    track.mute = True
    arm.animation_data.action = None
    print("Animation", name, frames / FPS, "s")
    return act


# ---- Grundhaltungen ----
def base_pose():
    return {
        "Spine": (-12.0, 0.0, 0.0),
        "Chest": (-8.0, 0.0, 0.0),
        "Neck": (8.0, 0.0, 0.0),
        "Head": (-6.0, 10.0, 0.0),
        "UpperArm.L": (10.0, 78.0, 0.0),
        "UpperArm.R": (5.0, 78.0, 0.0),
        "LowerArm.L": 15.0,
        "LowerArm.R": 20.0,
        "UpperLeg.L": (8.0, 2.0, 0.0),
        "UpperLeg.R": (4.0, 2.0, 0.0),
        "LowerLeg.L": 12.0,
        "LowerLeg.R": 10.0,
        "Foot.L": (0.0, 0.0, 0.0),
        "Foot.R": (0.0, 0.0, 0.0),
    }


def idle(t):
    p = base_pose()
    p["Spine"] = (-14 + S(t, 2), S(t, 2, 0.25), 0)
    p["Head"] = (-6 + S(t, 4, 0.1), 14 + S(t, 5), S(t, 6, 0.3))
    p["UpperArm.L"] = (8 + S(t, 5), 78 + S(t, 3, 0.2), 0)
    p["UpperArm.R"] = (4 + S(t, 4, 0.5), 80, 0)
    return p, (0, S(t, 0.01), Cs(t, 0.012) - 0.02)


def walk(t):
    """Schlurfender Gang mit Hinken; ein Arm greift nach vorn."""
    p = base_pose()
    ph = t
    p["Hips"] = (S(ph, 3, 0.25), S(ph, 5), S(ph, 8))
    p["Spine"] = (-20 + S(ph * 2, 2), S(ph, -3), S(ph, -5))
    p["Chest"] = (-8, S(ph, -3), S(ph, -6))
    p["Neck"] = (10, 0, 0)
    p["Head"] = (-4 + S(ph * 2, 4), 16 + S(ph, 4), S(ph, 5, 0.2))
    # Beine: links schleift nach (kleinerer Schritt)
    p["UpperLeg.L"] = (Cs(ph, 20) + 4, 3, 0)
    p["UpperLeg.R"] = (Cs(ph, -27) + 6, 3, 0)
    knee_l = 14 + max(0.0, S(ph, 30, 0.15))
    knee_r = 12 + max(0.0, S(ph, -48, 0.15))
    p["LowerLeg.L"] = knee_l
    p["LowerLeg.R"] = knee_r
    p["Foot.L"] = (S(ph, 10, 0.1) - 8, 0, 0)
    p["Foot.R"] = (S(ph, -12, 0.1), 0, 0)
    # Arme: links nach vorn greifend, rechts schlaff pendelnd
    p["UpperArm.L"] = (72 + S(ph, 8, 0.3), 80, 10)
    p["LowerArm.L"] = 25 + S(ph, 8)
    p["UpperArm.R"] = (S(ph, 22, 0.5) + 5, 82, 0)
    p["LowerArm.R"] = 20 + S(ph, 10)
    bob = abs(math.cos(math.tau * ph)) * 0.035
    return p, (S(ph, 0.035), 0.0, bob - 0.04)


def run(t):
    p = base_pose()
    ph = t
    p["Hips"] = (S(ph, 4, 0.25) - 4, S(ph, 4), S(ph, 10))
    p["Spine"] = (-22, S(ph, -3), S(ph, -6))
    p["Chest"] = (-10, 0, S(ph, -8))
    p["Neck"] = (16, 0, 0)
    p["Head"] = (2 + S(ph * 2, 5), 8 + S(ph, 6), 0)
    p["UpperLeg.L"] = (Cs(ph, 38) + 10, 3, 0)
    p["UpperLeg.R"] = (Cs(ph, -38) + 10, 3, 0)
    p["LowerLeg.L"] = 30 + max(0.0, S(ph, 55, 0.12))
    p["LowerLeg.R"] = 30 + max(0.0, S(ph, -55, 0.12))
    p["Foot.L"] = (S(ph, 15, 0.1), 0, 0)
    p["Foot.R"] = (S(ph, -15, 0.1), 0, 0)
    # beide Arme nach vorn, leicht schlenkernd
    p["UpperArm.L"] = (78 + S(ph, 12), 76, 8)
    p["UpperArm.R"] = (74 + S(ph, -12), 76, 8)
    p["LowerArm.L"] = 30 + S(ph, 10)
    p["LowerArm.R"] = 34 + S(ph, -10)
    bob = abs(math.cos(math.tau * ph)) * 0.06
    return p, (S(ph, 0.03), 0.02, bob - 0.07)


def sprint(t):
    p = base_pose()
    ph = t
    p["Hips"] = (-10 + S(ph, 4, 0.25), S(ph, 4), S(ph, 12))
    p["Spine"] = (-26, 0, S(ph, -8))
    p["Chest"] = (-12, 0, S(ph, -10))
    p["Neck"] = (24, 0, 0)
    p["Head"] = (6 + S(ph * 2, 6), S(ph, 8), 0)
    p["UpperLeg.L"] = (Cs(ph, 55) + 14, 4, 0)
    p["UpperLeg.R"] = (Cs(ph, -55) + 14, 4, 0)
    p["LowerLeg.L"] = 40 + max(0.0, S(ph, 70, 0.1))
    p["LowerLeg.R"] = 40 + max(0.0, S(ph, -70, 0.1))
    p["Foot.L"] = (S(ph, 20, 0.1), 0, 0)
    p["Foot.R"] = (S(ph, -20, 0.1), 0, 0)
    # wild rudernde Arme
    p["UpperArm.L"] = (Cs(ph, -55) + 40, 70, 10)
    p["UpperArm.R"] = (Cs(ph, 55) + 40, 70, 10)
    p["LowerArm.L"] = 55 + S(ph, 15)
    p["LowerArm.R"] = 55 + S(ph, -15)
    bob = abs(math.cos(math.tau * ph)) * 0.08
    return p, (0.0, 0.05, bob - 0.1)


def attack(t):
    """Zweihändiger Schlag: ausholen, zuschlagen (Treffer bei ~50 %), zurück."""
    p = base_pose()
    k_r = [(0.0, (75, 78, 0)), (0.3, (160, 60, 20)), (0.5, (40, 70, -10)), (0.68, (20, 75, 0)), (1.0, (75, 78, 0))]
    k_l = [(0.0, (70, 78, 0)), (0.35, (140, 55, 10)), (0.55, (35, 72, -5)), (0.72, (25, 78, 0)), (1.0, (70, 78, 0))]
    k_e = [(0.0, (30,)), (0.3, (70,)), (0.5, (10,)), (1.0, (30,))]
    k_sp = [(0.0, (-18, 0, 0)), (0.3, (-4, 0, 15)), (0.5, (-30, 0, -15)), (0.7, (-26, 0, -5)), (1.0, (-18, 0, 0))]
    p["UpperArm.R"] = lerp_keys(k_r, t)
    p["UpperArm.L"] = lerp_keys(k_l, t)
    p["LowerArm.R"] = lerp_keys(k_e, t)[0]
    p["LowerArm.L"] = lerp_keys(k_e, max(0.0, t - 0.05))[0]
    p["Spine"] = lerp_keys(k_sp, t)
    p["Chest"] = (-8, 0, lerp_keys(k_sp, t)[2] * 0.6)
    p["Head"] = lerp_keys([(0.0, (0, 10, 0)), (0.3, (-10, 0, 0)), (0.5, (10, 5, 0)), (1.0, (0, 10, 0))], t)
    p["UpperLeg.L"] = (18, 4, 0)
    p["UpperLeg.R"] = (-8, 4, 0)
    p["LowerLeg.L"] = 25
    p["LowerLeg.R"] = 15
    lean = lerp_keys([(0.0, (0.0,)), (0.3, (-0.03,)), (0.5, (0.06,)), (1.0, (0.0,))], t)[0]
    return p, (0, lean, -0.05)


def tear(t):
    """Bretter abreißen: greifen, ziehen, loslassen."""
    p = base_pose()
    arms = lerp_keys(
        [(0.0, (70, 78, 0)), (0.3, (98, 70, -5)), (0.45, (95, 70, -5)), (0.7, (35, 70, 15)), (1.0, (70, 78, 0))], t
    )
    p["UpperArm.L"] = arms
    p["UpperArm.R"] = (arms[0] - 6, arms[1], arms[2])
    p["LowerArm.L"] = lerp_keys([(0.0, (30,)), (0.3, (10,)), (0.45, (35,)), (0.7, (80,)), (1.0, (30,))], t)[0]
    p["LowerArm.R"] = p["LowerArm.L"]
    p["Spine"] = lerp_keys([(0.0, (-15, 0, 0)), (0.3, (-28, 0, 0)), (0.7, (-2, 0, 10)), (1.0, (-15, 0, 0))], t)
    p["Head"] = (-10, 8, 0)
    p["UpperLeg.L"] = (14, 4, 0)
    p["UpperLeg.R"] = (-6, 4, 0)
    p["LowerLeg.L"] = 22
    p["LowerLeg.R"] = 14
    back = lerp_keys([(0.0, (0.0,)), (0.3, (0.05,)), (0.7, (-0.08,)), (1.0, (0.0,))], t)[0]
    return p, (0, back, -0.05)


def climb(t):
    """Durch ein Fenster klettern, tief gebückt (Fensteröffnung nur ~1,4 m hoch).
    Das Spiel hebt die Figur gleichzeitig auf Brüstungshöhe und schiebt sie nach vorn."""
    p = base_pose()
    p["UpperArm.L"] = lerp_keys([(0.0, (80, 75, 0)), (0.3, (60, 70, 0)), (0.6, (20, 75, 0)), (1.0, (70, 78, 0))], t)
    p["UpperArm.R"] = lerp_keys([(0.0, (85, 75, 0)), (0.35, (55, 70, 0)), (0.65, (15, 75, 0)), (1.0, (65, 78, 0))], t)
    p["LowerArm.L"] = lerp_keys([(0.0, (20,)), (0.3, (60,)), (0.6, (20,)), (1.0, (25,))], t)[0]
    p["LowerArm.R"] = lerp_keys([(0.0, (20,)), (0.35, (60,)), (0.65, (20,)), (1.0, (25,))], t)[0]
    p["Hips"] = lerp_keys([(0.0, (-10, 0, 0)), (0.35, (-35, 0, 0)), (0.7, (-25, 0, 0)), (1.0, (0, 0, 0))], t)
    p["Spine"] = lerp_keys([(0.0, (-30, 0, 0)), (0.4, (-40, 0, 0)), (0.75, (-25, 0, 0)), (1.0, (-14, 0, 0))], t)
    p["Chest"] = (-15, 0, 0)
    p["Neck"] = (-5, 0, 0)
    p["Head"] = (20, 10, 0)
    p["UpperLeg.R"] = lerp_keys([(0.0, (30, 4, 0)), (0.25, (110, 6, 0)), (0.55, (70, 4, 0)), (1.0, (6, 2, 0))], t)
    p["LowerLeg.R"] = lerp_keys([(0.0, (40,)), (0.25, (120,)), (0.55, (80,)), (1.0, (10,))], t)[0]
    p["UpperLeg.L"] = lerp_keys([(0.0, (20, 4, 0)), (0.45, (40, 4, 0)), (0.7, (105, 6, 0)), (1.0, (8, 2, 0))], t)
    p["LowerLeg.L"] = lerp_keys([(0.0, (35,)), (0.45, (70,)), (0.7, (115,)), (1.0, (12,))], t)[0]
    down = lerp_keys([(0.0, (-0.12,)), (0.35, (-0.3,)), (0.7, (-0.28,)), (1.0, (-0.04,))], t)[0]
    return p, (0, 0, down)


def rise(t):
    """Aus dem Boden steigen: Arme nach oben, hochziehen, aufrichten.
    Das Spiel hebt den Zombie gleichzeitig von unter dem Boden an."""
    p = base_pose()
    p["UpperArm.L"] = lerp_keys([(0.0, (170, 70, 0)), (0.35, (150, 70, 0)), (0.65, (60, 75, 0)), (1.0, (12, 78, 0))], t)
    p["UpperArm.R"] = lerp_keys([(0.0, (160, 70, 0)), (0.4, (140, 70, 0)), (0.7, (50, 75, 0)), (1.0, (6, 78, 0))], t)
    p["LowerArm.L"] = lerp_keys([(0.0, (20,)), (0.4, (60,)), (0.7, (40,)), (1.0, (15,))], t)[0]
    p["LowerArm.R"] = lerp_keys([(0.0, (30,)), (0.45, (70,)), (0.75, (40,)), (1.0, (20,))], t)[0]
    p["Spine"] = lerp_keys([(0.0, (-40, 0, 10)), (0.5, (-35, 0, -10)), (0.8, (-20, 0, 0)), (1.0, (-12, 0, 0))], t)
    p["Head"] = lerp_keys([(0.0, (-20, 0, 0)), (0.5, (10, 15, 0)), (1.0, (-6, 10, 0))], t)
    p["UpperLeg.L"] = lerp_keys([(0.0, (90, 8, 0)), (0.6, (60, 6, 0)), (1.0, (8, 2, 0))], t)
    p["UpperLeg.R"] = lerp_keys([(0.0, (70, 8, 0)), (0.6, (40, 6, 0)), (1.0, (4, 2, 0))], t)
    p["LowerLeg.L"] = lerp_keys([(0.0, (120,)), (0.6, (80,)), (1.0, (12,))], t)[0]
    p["LowerLeg.R"] = lerp_keys([(0.0, (110,)), (0.6, (70,)), (1.0, (10,))], t)[0]
    down = lerp_keys([(0.0, (-0.45,)), (0.6, (-0.25,)), (1.0, (0.0,))], t)[0]
    return p, (0, 0, down)


def death_fwd(t):
    p = base_pose()
    u = min(1.0, t * 1.4)
    p["Hips"] = lerp_keys([(0.0, (0, 0, 0)), (0.35, (-20, 5, 10)), (0.75, (-85, 8, 15)), (1.0, (-88, 8, 15))], u)
    p["Spine"] = lerp_keys([(0.0, (-15, 0, 0)), (0.5, (-25, 0, 0)), (1.0, (5, 0, 0))], u)
    p["Head"] = lerp_keys([(0.0, (0, 10, 0)), (0.6, (-30, 20, 0)), (1.0, (20, 40, 0))], u)
    p["UpperLeg.L"] = lerp_keys([(0.0, (5, 2, 0)), (0.4, (40, 5, 0)), (1.0, (10, 8, 0))], u)
    p["UpperLeg.R"] = lerp_keys([(0.0, (5, 2, 0)), (0.4, (30, 5, 0)), (1.0, (0, 12, 0))], u)
    p["LowerLeg.L"] = lerp_keys([(0.0, (12,)), (0.4, (70,)), (1.0, (20,))], u)[0]
    p["LowerLeg.R"] = lerp_keys([(0.0, (10,)), (0.4, (60,)), (1.0, (35,))], u)[0]
    p["UpperArm.L"] = lerp_keys([(0.0, (20, 78, 0)), (0.6, (120, 50, 0)), (1.0, (160, 40, 0))], u)
    p["UpperArm.R"] = lerp_keys([(0.0, (20, 78, 0)), (0.6, (100, 60, 0)), (1.0, (140, 30, 20))], u)
    p["LowerArm.L"] = 30
    p["LowerArm.R"] = 50
    hl = lerp_keys([(0.0, (0, 0, 0)), (0.35, (0, 0.1, -0.25)), (0.75, (0, 0.55, -0.78)), (1.0, (0, 0.6, -0.8))], u)
    return p, hl


def death_back(t):
    p = base_pose()
    u = min(1.0, t * 1.4)
    p["Hips"] = lerp_keys([(0.0, (0, 0, 0)), (0.3, (15, -5, -10)), (0.75, (82, -5, -20)), (1.0, (85, -5, -20))], u)
    p["Spine"] = lerp_keys([(0.0, (-15, 0, 0)), (0.4, (10, 0, 0)), (1.0, (5, 0, 0))], u)
    p["Head"] = lerp_keys([(0.0, (0, 10, 0)), (0.5, (25, 0, 0)), (1.0, (10, -30, 0))], u)
    p["UpperLeg.L"] = lerp_keys([(0.0, (5, 2, 0)), (0.35, (35, 6, 0)), (1.0, (-5, 15, 0))], u)
    p["UpperLeg.R"] = lerp_keys([(0.0, (5, 2, 0)), (0.35, (20, 6, 0)), (1.0, (20, 6, 0))], u)
    p["LowerLeg.L"] = lerp_keys([(0.0, (12,)), (0.35, (60,)), (1.0, (10,))], u)[0]
    p["LowerLeg.R"] = lerp_keys([(0.0, (10,)), (0.35, (50,)), (1.0, (40,))], u)[0]
    p["UpperArm.L"] = lerp_keys([(0.0, (20, 78, 0)), (0.5, (60, 20, 0)), (1.0, (10, 10, 0))], u)
    p["UpperArm.R"] = lerp_keys([(0.0, (20, 78, 0)), (0.5, (80, 30, 0)), (1.0, (40, 0, 0))], u)
    p["LowerArm.L"] = 20
    p["LowerArm.R"] = 40
    hl = lerp_keys([(0.0, (0, 0, 0)), (0.3, (0, -0.1, -0.2)), (0.75, (0, -0.6, -0.78)), (1.0, (0, -0.65, -0.8))], u)
    return p, hl


def crawl(t):
    """Kriechen (Beine verloren): Arme ziehen den Körper vorwärts."""
    p = base_pose()
    ph = t
    p["Hips"] = (-80, S(ph, 5), S(ph, 6))
    p["Spine"] = (-5, 0, S(ph, -6))
    p["Chest"] = (5, 0, 0)
    p["Neck"] = (-30, 0, 0)
    p["Head"] = (-25, S(ph, 8), 0)
    p["UpperArm.L"] = (Cs(ph, 40) + 120, 60, 0)
    p["UpperArm.R"] = (Cs(ph, -40) + 120, 60, 0)
    p["LowerArm.L"] = 40 + S(ph, 25)
    p["LowerArm.R"] = 40 + S(ph, -25)
    p["UpperLeg.L"] = (-5, 6, 0)
    p["UpperLeg.R"] = (-8, 6, 0)
    p["LowerLeg.L"] = 10 + S(ph, 8)
    p["LowerLeg.R"] = 12 + S(ph, -8)
    return p, (0, 0.55, -0.78)


def build_animations(arm):
    arm.animation_data_create()
    cache_rest(arm)
    make_action(arm, "Idle", 60, idle)
    make_action(arm, "Walk", 36, walk)
    make_action(arm, "Run", 24, run)
    make_action(arm, "Sprint", 16, sprint)
    make_action(arm, "Attack", 27, attack, loop=False)
    make_action(arm, "Tear", 39, tear, loop=False)
    make_action(arm, "Climb", 45, climb, loop=False)
    make_action(arm, "Rise", 54, rise, loop=False)
    make_action(arm, "DeathFwd", 36, death_fwd, loop=False)
    make_action(arm, "DeathBack", 36, death_back, loop=False)
    make_action(arm, "Crawl", 36, crawl)
