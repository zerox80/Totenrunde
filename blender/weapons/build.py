"""Baut alle Ego-Waffen nach assets/models/weapons/<id>.gltf.

Aufruf: tools/blender.sh blender/weapons/build.py [-- id1 id2 ...] [-- preview]
"""

import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import bpy  # noqa: E402
from lib import PREVIEW_DIR, reset_scene, script_args  # noqa: E402
from weapons import items, launchers, pistols, rifles, shotguns, smgs  # noqa: E402
from weapons.common import OUT  # noqa: E402

ARGS = script_args()

BUILDERS = {
    "m1911": pistols.build_m1911,
    "b23r": pistols.build_b23r,
    "python": pistols.build_python,
    "raygun": pistols.build_raygun,
    "m14": rifles.build_m14,
    "galil": rifles.build_galil,
    "rpd": rifles.build_rpd,
    "dsr50": rifles.build_dsr50,
    "mp5": smgs.build_mp5,
    "ak74u": smgs.build_ak74u,
    "olympia": shotguns.build_olympia,
    "remington": shotguns.build_remington,
    "mgl": launchers.build_mgl,
    "knife": items.build_knife,
    "perk_bottle": items.build_perk_bottle,
}


def preview_render(name, path):
    """Rendert die Waffe von der Seite (für Kontrollbilder)."""
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 16
    scene.render.resolution_x = 640
    scene.render.resolution_y = 360
    world = bpy.data.worlds.new("W")
    scene.world = world
    world.color = (0.35, 0.37, 0.4)
    cam_data = bpy.data.cameras.new("Cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = 1.3
    cam = bpy.data.objects.new("Cam", cam_data)
    scene.collection.objects.link(cam)
    cam.location = (1.2, 0.18, -0.08)
    cam.rotation_euler = (math.radians(90), 0, math.radians(90))
    scene.camera = cam
    sun_d = bpy.data.lights.new("Sun", "SUN")
    sun_d.energy = 3.0
    sun = bpy.data.objects.new("Sun", sun_d)
    sun.rotation_euler = (math.radians(50), math.radians(-30), math.radians(70))
    scene.collection.objects.link(sun)
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)


def main():
    names = [a for a in ARGS if a in BUILDERS] or list(BUILDERS)
    prev = "preview" in ARGS
    os.makedirs(OUT, exist_ok=True)
    for n in names:
        reset_scene()
        BUILDERS[n]()
        if prev:
            preview_render(n, os.path.join(PREVIEW_DIR, f"w_{n}.png"))
        print("WAFFE", n)


main()
