"""Zombie: eigenes Modell (Skulptur per Code), Skelett und Animationen
-> assets/models/zombie_m.gltf / zombie_f.gltf

Aufruf: tools/blender.sh blender/characters/zombie.py [-- female] [-- preview] [-- nobake]
"""

import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import bpy  # noqa: E402
from characters import zombie_anims, zombie_body, zombie_rig  # noqa: E402
from lib import MODEL_DIR, PREVIEW_DIR, export_gltf, reset_scene, script_args  # noqa: E402

ARGS = script_args()
scene = reset_scene(zombie_rig.FPS)


def _cam(name, loc, target, lens):
    from mathutils import Vector

    cd = bpy.data.cameras.new(name)
    cd.lens = lens
    cd.clip_start = 0.01
    cam = bpy.data.objects.new(name, cd)
    scene.collection.objects.link(cam)
    cam.location = loc
    cam.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    return cam


def _light(name, kind, energy, loc, target, size=1.0, color=(1, 1, 1)):
    from mathutils import Vector

    ld = bpy.data.lights.new(name, kind)
    ld.energy = energy
    ld.color = color
    if kind == "AREA":
        ld.size = size
    o = bpy.data.objects.new(name, ld)
    scene.collection.objects.link(o)
    o.location = loc
    o.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()


def preview(arm, out_dir, tag):
    """Gerenderte Vorschau mit Texturen: Posen und Nahaufnahme des Kopfes."""
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 48
    scene.cycles.use_denoising = True
    scene.render.film_transparent = False
    scene.view_settings.view_transform = "AgX"
    world = bpy.data.worlds.new("W")
    scene.world = world
    world.color = (0.05, 0.055, 0.065)
    _light("Key", "AREA", 260, (1.6, 2.2, 2.4), (0, 0, 1.2), size=1.2, color=(1.0, 0.93, 0.85))
    _light("Fill", "AREA", 60, (-2.2, 1.4, 1.4), (0, 0, 1.1), size=2.0, color=(0.7, 0.8, 1.0))
    _light("Rim", "AREA", 180, (-0.8, -2.4, 2.2), (0, 0, 1.3), size=1.0, color=(0.8, 0.9, 1.0))
    full = _cam("Full", (1.5, 3.1, 1.2), (0, 0, 0.95), 42)
    # Gesicht: Kamera folgt dem Kopfknochen in der Idle-Pose
    from mathutils import Vector

    arm.animation_data.action = bpy.data.actions["Idle"]
    scene.frame_set(1)
    dg = bpy.context.evaluated_depsgraph_get()
    eo = bpy.data.objects["ZombieEyes"].evaluated_get(dg)
    eye = sum((eo.matrix_world @ v.co for v in eo.data.vertices), Vector()) / len(eo.data.vertices)
    neck = arm.matrix_world @ arm.pose.bones["Head"].head
    fwd = Vector((eye.x - neck.x, eye.y - neck.y, 0.0)).normalized()
    side = Vector((fwd.y, -fwd.x, 0.0))
    face = _cam("Face", tuple(eye + fwd * 0.58 + side * 0.28 + Vector((0, 0, 0.02))), tuple(eye + Vector((0, 0, -0.03))), 85)
    face2 = _cam("Face2", tuple(eye + fwd * 0.65), tuple(eye + Vector((0, 0, -0.025))), 85)
    if "ZombieStump" in bpy.data.objects:
        bpy.data.objects["ZombieStump"].hide_render = True
    # Hemd und Hose wie im Spiel einfärben (Basisfarbe x Textur)
    for name, col in (("Z_Shirt", (0.3, 0.27, 0.2, 1)), ("Z_Pants", (0.035, 0.045, 0.07, 1))):
        m = bpy.data.materials.get(name)
        if not m:
            continue
        nt = m.node_tree
        bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
        link = bsdf.inputs["Base Color"].links[0]
        mix = nt.nodes.new("ShaderNodeMix")
        mix.data_type = "RGBA"
        mix.blend_type = "MULTIPLY"
        mix.inputs["Factor"].default_value = 1.0
        nt.links.new(link.from_socket, mix.inputs[6])
        mix.inputs[7].default_value = col
        nt.links.new(mix.outputs[2], bsdf.inputs["Base Color"])
    shots = [("Idle", 1, full), ("Walk", 10, full), ("Attack", 9, full), ("Idle", 1, face), ("Idle", 1, face2)]
    for i, (aname, frame, cam) in enumerate(shots):
        arm.animation_data.action = bpy.data.actions[aname]
        scene.frame_set(frame)
        scene.camera = cam
        scene.render.resolution_x = 640 if cam is full else 700
        scene.render.resolution_y = 800 if cam is full else 700
        scene.render.filepath = os.path.join(out_dir, f"{tag}_pose_{i:02d}_{aname}_{frame}.png")
        bpy.ops.render.render(write_still=True)
    arm.animation_data.action = None
    _ = math


def main():
    kind = "female" if "female" in ARGS else "male"
    tag = "zombie_f" if kind == "female" else "zombie_m"
    parts = zombie_body.build(kind, bake_tex="nobake" not in ARGS, tag=tag)
    arm = zombie_rig.build_armature(parts["bones"])
    zombie_body.bind(parts, arm)
    zombie_anims.build_animations(arm)
    export_gltf(os.path.join(MODEL_DIR, f"{tag}.gltf"), animations=True)
    if "preview" in ARGS:
        preview(arm, PREVIEW_DIR, tag)


main()
