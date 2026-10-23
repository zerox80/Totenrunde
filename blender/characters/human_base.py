"""Realistische Körper aus dem CC0-Paket "Human Base Meshes" (Blender Studio).

Das Paket liegt in third_party/human_base_meshes (siehe README dort). Diese Datei lädt
einen Körper, richtet ihn auf die Projekt-Konventionen aus (Blick +Y, Füße auf z=0) und
ermittelt aus Querschnitten des Meshes die Gelenkpositionen für das Skelett.
"""

import math
import os

import bpy
from mathutils import Matrix, Vector

from lib import ROOT

BUNDLE = os.path.join(
    ROOT, "third_party", "human_base_meshes", "human-base-meshes-bundle-v1.4.1", "human_base_meshes_bundle.blend"
)

BODIES = {
    "male": "GEO-body_male_realistic",
    "female": "GEO-body_female_realistic",
}


def load_body(kind="male", name="ZombieBody"):
    """Hängt den Körper an die Szene an, dreht ihn um 180° (Blick nach +Y) und setzt ihn
    auf den Ursprung. Liefert das Objekt (mit Multires-Modifier)."""
    if not os.path.exists(BUNDLE):
        raise FileNotFoundError(
            f"Human-Base-Meshes-Paket fehlt: {BUNDLE}\n"
            "Download: https://www.blender.org/download/demo-files/ (Asset Bundles, CC0)"
        )
    src_name = BODIES[kind]
    with bpy.data.libraries.load(BUNDLE, link=False) as (src, dst):
        dst.objects = [src_name]
    obj = dst.objects[0]
    bpy.context.scene.collection.objects.link(obj)
    obj.name = name
    obj.data.name = name
    obj.parent = None
    # Transformation in die Geometrie übernehmen: Ursprung 0, 180° um Z
    rot = Matrix.Rotation(math.pi, 4, "Z")
    obj.data.transform(rot)
    obj.matrix_world = Matrix.Identity(4)
    # Auf den Boden stellen und mittig ausrichten
    xs = [v.co.x for v in obj.data.vertices]
    ys = [v.co.y for v in obj.data.vertices]
    zs = [v.co.z for v in obj.data.vertices]
    off = Vector((-(min(xs) + max(xs)) / 2, -(sum(ys) / len(ys)), -min(zs)))
    obj.data.transform(Matrix.Translation(off))
    obj.data.update()
    return obj


def multires(obj):
    for m in obj.modifiers:
        if m.type == "MULTIRES":
            return m
    return None


def mesh_at_level(obj, level, name):
    """Neues Mesh-Objekt mit der Geometrie der Multires-Stufe `level` (Modifier angewandt)."""
    m = multires(obj)
    if m is not None:
        m.levels = level
        m.render_levels = level
        m.sculpt_levels = level
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(obj.evaluated_get(dg))
    me.name = name
    o = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(o)
    return o


def _slice(verts, z, tol=0.012):
    return [v for v in verts if abs(v.z - z) < tol]


def _centroid(vs):
    if not vs:
        return None
    s = Vector((0, 0, 0))
    for v in vs:
        s += v
    return s / len(vs)


def estimate_joints(obj):
    """Gelenkpositionen aus der Geometrie (Figur schaut nach +Y, linke Seite bei -X).

    Liefert dict mit Punkten: pelvis, spine, chest, neck, head, head_top und je Seite
    (L/R): shoulder_in, shoulder, elbow, wrist, hand_end, hip, knee, ankle, toe."""
    verts = [v.co.copy() for v in obj.data.vertices]
    top = max(v.z for v in verts)
    # Schritt: unterste Höhe, auf der Geometrie nahe x=0 liegt (Beine verbinden sich)
    crotch = None
    z = 0.4
    while z < 1.2:
        if any(abs(v.x) < 0.012 for v in _slice(verts, z, 0.01)):
            crotch = z
            break
        z += 0.005
    crotch = crotch or 0.8
    # Hals: schmalste Stelle des Rumpfs zwischen 75 % und 92 % der Höhe
    best_w, neck_z = 9.0, top * 0.85
    z = top * 0.78
    while z < top * 0.92:
        vs = [v for v in _slice(verts, z, 0.008) if abs(v.x) < 0.12]
        if vs:
            w = max(abs(v.x) for v in vs)
            if w < best_w:
                best_w, neck_z = w, z
        z += 0.004
    j = {}
    j["pelvis"] = Vector((0, _centroid(_slice(verts, crotch + 0.12, 0.02)).y, crotch + 0.12))
    j["spine"] = Vector((0, j["pelvis"].y, crotch + 0.26))
    chest_z = crotch + (neck_z - crotch) * 0.72
    chest_vs = [v for v in _slice(verts, chest_z, 0.02) if abs(v.x) < 0.14]
    j["chest"] = Vector((0, _centroid(chest_vs).y if chest_vs else 0.0, chest_z))
    neck_vs = [v for v in _slice(verts, neck_z, 0.01) if abs(v.x) < 0.12]
    ny = _centroid(neck_vs).y if neck_vs else 0.0
    j["neck"] = Vector((0, ny, neck_z - 0.05))
    j["head"] = Vector((0, ny + 0.01, neck_z + 0.04))
    j["head_top"] = Vector((0, ny + 0.01, top))
    for side, s in (("L", -1.0), ("R", 1.0)):
        # Beine: Schwerpunkte der Querschnitte einer Körperhälfte
        def leg_at(zz, tol=0.015):
            vs = [v for v in _slice(verts, zz, tol) if v.x * s > 0.015 and abs(v.x) < 0.3]
            return _centroid(vs)

        hip = leg_at(crotch - 0.03) or Vector((s * 0.1, 0, crotch - 0.03))
        knee_z = crotch * 0.6
        knee = leg_at(knee_z) or Vector((s * 0.1, 0, knee_z))
        ankle = leg_at(0.085, 0.012) or Vector((s * 0.1, 0, 0.085))
        foot = [v for v in verts if v.z < 0.06 and v.x * s > 0.02]
        toe_y = max(v.y for v in foot) if foot else ankle.y + 0.15
        toe = Vector((ankle.x, toe_y - 0.045, 0.025))
        j[f"hip.{side}"] = Vector((hip.x, hip.y, crotch + 0.06))
        j[f"knee.{side}"] = knee
        j[f"ankle.{side}"] = ankle
        j[f"toe.{side}"] = toe
        # Arme: Hand-Ende = tiefster Punkt mit großem |x|; Schulter nahe Halsansatz
        arm_vs = [v for v in verts if v.x * s > 0.24 and v.z > crotch - 0.2 and v.z < neck_z - 0.1]
        hand_end = min(arm_vs, key=lambda v: v.z) if arm_vs else Vector((s * 0.4, 0, 0.7))
        sh_z = neck_z - 0.075
        sh_vs = [v for v in _slice(verts, sh_z - 0.03, 0.02) if v.x * s > 0.1]
        sh_x = max(abs(v.x) for v in sh_vs) - 0.055 if sh_vs else 0.18
        shoulder = Vector((s * sh_x, ny - 0.01, sh_z))
        # Mittellinie des Arms: Schwerpunkte der Querschnitte (Arm-Teil, |x| groß)
        def arm_at(zz):
            vs = [v for v in _slice(verts, zz, 0.012) if v.x * s > 0.2]
            # nur den äußeren Arm nehmen (Rumpf ausschließen)
            if not vs:
                return None
            xs = sorted(abs(v.x) for v in vs)
            cut = xs[0] + 0.02
            vs = [v for v in vs if abs(v.x) >= cut - 0.02]
            return _centroid(vs)

        length = (hand_end - shoulder).length
        elbow_t, wrist_t = 0.43, 0.8
        elbow_guess = shoulder.lerp(hand_end, elbow_t)
        wrist_guess = shoulder.lerp(hand_end, wrist_t)
        elbow = arm_at(elbow_guess.z) or elbow_guess
        wrist = arm_at(wrist_guess.z) or wrist_guess
        j[f"shoulder_in.{side}"] = Vector((s * 0.025, ny - 0.01, sh_z + 0.015))
        j[f"shoulder.{side}"] = shoulder
        j[f"elbow.{side}"] = elbow
        j[f"wrist.{side}"] = wrist
        j[f"hand_end.{side}"] = hand_end
        _ = length
    return j
