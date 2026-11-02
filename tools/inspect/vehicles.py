"""Kontrollbild der Fahrzeuge (Autowracks und Bus) ohne die ganze Karte zu backen.

Aufruf: tools/blender.sh tools/inspect/vehicles.py  ->  tools/preview/vehicles.png
"""

import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))

import bpy  # noqa: E402
from lib import PREVIEW_DIR, hexcol  # noqa: E402
from maps import furniture as F  # noqa: E402
from maps.mapkit import MapBuilder  # noqa: E402

M = MapBuilder("vehicles_preview")
L = M.lib
F.use(M)


def pbr(key, tex, uv, color=None, **kw):
    if color is not None:
        kw["color"] = color
    L.add(key, uv=uv, tex=tex, normal=tex + "_n", rough_tex=tex + "_mr", **kw)


pbr("asphalt", "asphalt", 5.0)
pbr("rust", "rust", 2.0)
pbr("soot", "rust", 2.0, hexcol("#3a3533"))
pbr("rubber", "rubber", 1.0)
for key, col in (("metal_grey", "#9a9c9e"), ("metal_blue", "#3f5a7a"), ("metal_red", "#8a2a22"), ("metal_white", None)):
    pbr(key, "metal_paint", 1.5, hexcol(col) if col else None)
pbr("metal_dark", "gunmetal", 1.0)
L.add("paint_white", uv=2.0, color=hexcol("#b9b5aa"), rough=0.75)
L.add("glass", uv=2.0, color=hexcol("#1d2a2c"), rough=0.06, metal=0.9)
L.add("chrome", uv=1.0, color=hexcol("#c9cdd2"), rough=0.22, metal=1.0)
L.add("screen_black", uv=1.0, color=hexcol("#060708"), rough=0.85)
L.add("window_dark", uv=1.0, color=hexcol("#0b0d10"), rough=0.2, metal=0.5)

M.floor(-20, -20, 20, 20, "asphalt", z=0.0, t=0.05)
F.car_wreck(-3.0, 0.0, 20.0)
F.car_wreck(1.0, -1.0, -15.0, color="metal_blue")
F.car_wreck(4.5, 1.5, 70.0, color="metal_red")
F.bus_wreck(0.0, 9.0, 90.0)
for b in M.builders.values():
    b.to_object(smooth_angle=30.0)

scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.samples = 48
scene.render.resolution_x = 1280
scene.render.resolution_y = 720
world = bpy.data.worlds.new("W")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.35, 0.37, 0.42, 1)
world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.6
scene.world = world
sun = bpy.data.objects.new("Sun", bpy.data.lights.new("Sun", "SUN"))
sun.data.energy = 3.0
sun.rotation_euler = (math.radians(50), math.radians(10), math.radians(30))
scene.collection.objects.link(sun)
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
scene.collection.objects.link(cam)
scene.camera = cam
cam.location = (-9.0, -11.0, 5.0)
target = (1.0, 3.0, 0.8)
d = [t - c for t, c in zip(target, cam.location)]
cam.rotation_euler = (math.atan2(math.hypot(d[0], d[1]), -d[2]), 0.0, math.atan2(d[1], d[0]) - math.pi / 2)
cam.data.lens = 28
scene.render.filepath = os.path.join(PREVIEW_DIR, "vehicles.png")
bpy.ops.render.render(write_still=True)
