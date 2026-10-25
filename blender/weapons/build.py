"""Baut alle Ego-Waffen nach assets/models/weapons/<id>.gltf.

Aufruf: tools/blender.sh blender/weapons/build.py [-- id1 id2 ...] [-- preview]
"""

import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import bpy  # noqa: E402
from lib import PREVIEW_DIR, reset_scene, script_args  # noqa: E402
from weapons import handguns, longguns, m1911  # noqa: E402
from weapons.kit import OUT  # noqa: E402

ARGS = script_args()

BUILDERS = {
    "m1911": m1911.build,
    "b23r": handguns.build_b23r,
    "python": handguns.build_python,
    "raygun": handguns.build_raygun,
    "m14": longguns.build_m14,
    "galil": longguns.build_galil,
    "rpd": longguns.build_rpd,
    "dsr50": longguns.build_dsr50,
    "mp5": longguns.build_mp5,
    "ak74u": longguns.build_ak74u,
    "olympia": longguns.build_olympia,
    "remington": longguns.build_remington,
    "mgl": longguns.build_mgl,
    "knife": handguns.build_knife,
    "perk_bottle": handguns.build_perk_bottle,
}


def preview_render(name, path, hip=(0.105, -0.092, -0.34), ads=(0.0, 0.0, -0.25)):
    """Kontrollbilder: Ego-Ansicht (wie im Spiel, Hüfte) und Seitenansicht."""
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 32
    scene.cycles.use_denoising = False
    scene.render.resolution_x = 960
    scene.render.resolution_y = 540
    world = bpy.data.worlds.new("W")
    world.use_nodes = True
    bg = world.node_tree.nodes["Background"]
    bg.inputs["Color"].default_value = (0.32, 0.33, 0.36, 1)
    bg.inputs["Strength"].default_value = 0.5
    scene.world = world
    key_d = bpy.data.lights.new("Key", "AREA")
    key_d.energy = 12.0
    key_d.size = 0.6
    key = bpy.data.objects.new("Key", key_d)
    key.location = (0.5, -0.2, 0.6)
    key.rotation_euler = (math.radians(40), math.radians(35), 0)
    scene.collection.objects.link(key)
    cam_data = bpy.data.cameras.new("Cam")
    cam = bpy.data.objects.new("Cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    # Ego-Ansicht: Kamera relativ zur Waffe (Bevy-Hüftversatz -> Blender-Koordinaten)
    cam_data.type = "PERSP"
    cam_data.sensor_fit = "VERTICAL"
    cam_data.angle = math.radians(62.0)
    cam_data.clip_start = 0.005
    cam.location = (-hip[0], hip[2], -hip[1])
    cam.rotation_euler = (math.radians(90), 0, 0)
    scene.render.filepath = path.replace(".png", "_fp.png")
    bpy.ops.render.render(write_still=True)
    # Zielansicht (Kimme und Korn)
    cam.location = (-ads[0], ads[2], -ads[1])
    scene.render.filepath = path.replace(".png", "_ads.png")
    bpy.ops.render.render(write_still=True)
    # Seitenansicht
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = 0.75
    cam.location = (0.9, 0.0, -0.08)
    cam.rotation_euler = (math.radians(90), 0, math.radians(90))
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    # Nur die Waffe (Arme durchsichtig), näher
    hands = bpy.data.materials.get("W_Hands")
    if hands and hands.node_tree:
        nt = hands.node_tree
        out = next(n for n in nt.nodes if n.type == "OUTPUT_MATERIAL")
        tr = nt.nodes.new("ShaderNodeBsdfTransparent")
        nt.links.new(tr.outputs[0], out.inputs["Surface"])
    cam_data.ortho_scale = 0.3
    cam.location = (0.9, 0.06, -0.06)
    scene.render.filepath = path.replace(".png", "_gun.png")
    bpy.ops.render.render(write_still=True)
    cam.location = (-0.9, 0.06, -0.06)
    cam.rotation_euler = (math.radians(90), 0, math.radians(-90))
    scene.render.filepath = path.replace(".png", "_gunL.png")
    bpy.ops.render.render(write_still=True)


def main():
    names = [a for a in ARGS if a in BUILDERS] or list(BUILDERS)
    prev = "preview" in ARGS
    os.makedirs(OUT, exist_ok=True)
    for n in names:
        reset_scene()
        BUILDERS[n]()
        if prev:
            pistol = n in ("m1911", "b23r", "python", "raygun")
            hip = (0.105, -0.092, -0.34) if pistol else ((0.14, -0.14, -0.27) if n == "rpd" else (0.12, -0.11, -0.25))
            ads = (0.0, 0.0, -0.42) if pistol else ((0.0, 0.0, -0.12) if n == "dsr50" else (0.0, 0.0, -0.25))
            preview_render(n, os.path.join(PREVIEW_DIR, f"w_{n}.png"), hip=hip, ads=ads)
        print("WAFFE", n)


main()
