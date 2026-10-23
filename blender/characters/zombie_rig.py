"""Skelett des Zombies (Blick nach +Y, linke Seite bei -X).

Die Knochen werden aus den Gelenkpositionen des Körpers erzeugt (siehe human_base.py).
Die Knochennamen sind fest – das Spiel sucht z. B. "Head", "Neck", "Chest", "Hips",
"UpperArm.L" für die Trefferzonen."""

import bpy

from lib import set_active

FPS = 30


def bones_from_joints(j):
    b = [
        ("Hips", j["pelvis"], j["spine"], None),
        ("Spine", j["spine"], j["chest"], "Hips"),
        ("Chest", j["chest"], j["neck"], "Spine"),
        ("Neck", j["neck"], j["head"], "Chest"),
        ("Head", j["head"], j["head_top"], "Neck"),
    ]
    for side in ("L", "R"):
        b += [
            (f"Shoulder.{side}", j[f"shoulder_in.{side}"], j[f"shoulder.{side}"], "Chest"),
            (f"UpperArm.{side}", j[f"shoulder.{side}"], j[f"elbow.{side}"], f"Shoulder.{side}"),
            (f"LowerArm.{side}", j[f"elbow.{side}"], j[f"wrist.{side}"], f"UpperArm.{side}"),
            (f"Hand.{side}", j[f"wrist.{side}"], j[f"hand_end.{side}"], f"LowerArm.{side}"),
            (f"UpperLeg.{side}", j[f"hip.{side}"], j[f"knee.{side}"], "Hips"),
            (f"LowerLeg.{side}", j[f"knee.{side}"], j[f"ankle.{side}"], f"UpperLeg.{side}"),
            (f"Foot.{side}", j[f"ankle.{side}"], j[f"toe.{side}"], f"LowerLeg.{side}"),
        ]
    return b


def build_armature(bones):
    arm_data = bpy.data.armatures.new("ZombieRig")
    arm = bpy.data.objects.new("ZombieRig", arm_data)
    bpy.context.scene.collection.objects.link(arm)
    set_active(arm)
    bpy.ops.object.mode_set(mode="EDIT")
    eb = {}
    for name, head, tail, parent in bones:
        b = arm_data.edit_bones.new(name)
        b.head = tuple(head)
        b.tail = tuple(tail)
        b.roll = 0.0
        if parent:
            b.parent = eb[parent]
            b.use_connect = False
        eb[name] = b
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm
