"""Rendert die Ego-Arme (tools/cache/fparms.blend) zur Kontrolle: tools/blender.sh tools/inspect/arms_preview.py [-- pose]"""
import math
import os
import sys

import bpy

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
bpy.ops.wm.open_mainfile(filepath=os.path.join(ROOT, "tools", "cache", "fparms.blend"))
scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.samples = 24
scene.cycles.use_denoising = False
scene.render.resolution_x = 1200
scene.render.resolution_y = 700
w = bpy.data.worlds.new("W")
w.use_nodes = True
w.node_tree.nodes["Background"].inputs["Color"].default_value = (0.3, 0.32, 0.36, 1)
w.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.6
scene.world = w
bpy.data.objects["Rig_L"].location.x = 0.18
bpy.data.objects["Rig_R"].location.x = -0.02
cam_d = bpy.data.cameras.new("C")
cam_d.lens = 50
cam = bpy.data.objects.new("C", cam_d)
scene.collection.objects.link(cam)
scene.camera = cam
ld = bpy.data.lights.new("K", "AREA")
ld.energy = 30
ld.size = 0.5
lo = bpy.data.objects.new("K", ld)
lo.location = (0.4, 0.2, 0.5)
lo.rotation_euler = (math.radians(30), math.radians(30), 0)
scene.collection.objects.link(lo)
views = {"top": ((0.08, -0.05, 0.75), (0, 0, 0)), "side": ((-0.6, 0.0, 0.05), (math.radians(90), 0, math.radians(-90))),
         "persp": ((0.45, 0.35, 0.35), None)}
for name, (loc, rot) in views.items():
    cam.location = loc
    if rot is None:
        d = (bpy.data.objects["Rig_R"].location + __import__("mathutils").Vector((0.09, -0.02, 0))) - cam.location
        cam.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
    else:
        cam.rotation_euler = rot
    scene.render.filepath = os.path.join(ROOT, "tools", "preview", f"arms_{name}.png")
    bpy.ops.render.render(write_still=True)
