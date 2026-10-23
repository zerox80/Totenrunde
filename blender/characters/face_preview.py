"""Schnelle Gesichts-Vorschau: Kopf der Skulptur (1 mm) direkt mit dem Zombie-Material
gerendert (ohne Backen) -> tools/preview/face_<kind>_*.png

Aufruf: tools/blender.sh blender/characters/face_preview.py [-- female]
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import bpy  # noqa: E402
import numpy as np  # noqa: E402
from mathutils import Vector  # noqa: E402
from characters import zombie_body, zombie_sculpt  # noqa: E402
from characters.zombie_shading import make_direct_material  # noqa: E402
from lib import PREVIEW_DIR, material, reset_scene, script_args  # noqa: E402
from sculpt import mesher  # noqa: E402

ARGS = script_args()


def cam(name, loc, target, lens=85):
    cd = bpy.data.cameras.new(name)
    cd.lens = lens
    cd.clip_start = 0.01
    o = bpy.data.objects.new(name, cd)
    bpy.context.scene.collection.objects.link(o)
    o.location = loc
    o.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    return o


def light(name, energy, loc, target, size, color):
    ld = bpy.data.lights.new(name, "AREA")
    ld.energy, ld.size, ld.color = energy, size, color
    o = bpy.data.objects.new(name, ld)
    bpy.context.scene.collection.objects.link(o)
    o.location = loc
    o.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()


def main():
    kind = "female" if "female" in ARGS else "male"
    scene = reset_scene()
    sk, sc, info = zombie_sculpt.sculpt(kind)
    hz = sk["neck_base"] - 0.06
    v, q = zombie_body._cached_nets(sc, kind + "_face", np.array((-0.16, -0.16, hz)), np.array((0.16, 0.17, sk["head_top"] + 0.01)), 0.001)
    obj = mesher.to_blender("Head", v, q)
    zombie_body.annotate(obj, v, sc, sk, info)
    obj.data.materials.append(make_direct_material("Direct", sk, info["eyes"], kind))
    eye_m = material("Eye", color=(1.0, 0.75, 0.3, 1), emission=(1.0, 0.6, 0.15, 1), emission_strength=12.0)
    for e in info["eyes"]:
        bpy.ops.mesh.primitive_uv_sphere_add(radius=info["eye_r"], location=tuple(e), segments=32, ring_count=16)
        o = bpy.context.active_object
        o.data.shade_smooth()
        o.data.materials.append(eye_m)
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 64
    scene.cycles.use_denoising = True
    scene.view_settings.view_transform = "AgX"
    w = bpy.data.worlds.new("W")
    scene.world = w
    w.color = (0.04, 0.045, 0.05)
    hc = Vector(sk["hc"])
    light("Key", 40, hc + Vector((0.6, 0.9, 0.5)), hc, 0.6, (1.0, 0.92, 0.82))
    light("Fill", 10, hc + Vector((-0.9, 0.6, 0.1)), hc, 1.0, (0.7, 0.8, 1.0))
    light("Rim", 30, hc + Vector((-0.4, -0.8, 0.5)), hc, 0.5, (0.85, 0.9, 1.0))
    t = hc + Vector((0, 0.03, -0.03))
    shots = [
        ("front", cam("c", t + Vector((0, 0.75, 0.02)), t)),
        ("34", cam("c", t + Vector((0.45, 0.6, 0.04)), t)),
        ("side", cam("c", t + Vector((0.75, 0.02, 0.0)), t)),
    ]
    scene.render.resolution_x = 700
    scene.render.resolution_y = 700
    for name, c in shots:
        scene.camera = c
        scene.render.filepath = os.path.join(PREVIEW_DIR, f"face_{kind[0]}_{name}.png")
        bpy.ops.render.render(write_still=True)


main()
