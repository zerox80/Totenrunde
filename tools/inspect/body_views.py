"""Rendert Vorder-/Seitenansicht eines Körpers aus dem Human-Base-Mesh-Paket mit Raster."""
import math
import sys

import bpy
from mathutils import Vector

args = sys.argv[sys.argv.index("--") + 1:]
path, objname, out = args[0], args[1], args[2]
bpy.ops.wm.read_factory_settings(use_empty=True)
with bpy.data.libraries.load(path, link=False) as (src, dst):
    dst.objects = [objname]
obj = dst.objects[0]
bpy.context.scene.collection.objects.link(obj)
for m in list(obj.modifiers):
    print("MOD", m.type, getattr(m, "levels", None), getattr(m, "total_levels", None))
me = obj.data
ws = [obj.matrix_world @ v.co for v in me.vertices]
mn = Vector((min(v.x for v in ws), min(v.y for v in ws), min(v.z for v in ws)))
mx = Vector((max(v.x for v in ws), max(v.y for v in ws), max(v.z for v in ws)))
print("BBOX", tuple(round(c, 3) for c in mn), tuple(round(c, 3) for c in mx))
print("LOC", tuple(obj.location), "ROT", tuple(obj.rotation_euler), "SCALE", tuple(obj.scale))
print("VGROUPS", [g.name for g in obj.vertex_groups][:40])
print("FACEMAPS/ATTR", [a.name for a in me.attributes][:20])
# Höhenprofil: Breite (x) pro z-Schicht -> Schultern, Hüfte, Knie erkennen
for z in [x * 0.05 for x in range(0, 36)]:
    sl = [v for v in ws if abs(v.z - z) < 0.012]
    if sl:
        xs = [v.x for v in sl]
        ys = [v.y for v in sl]
        print(f"Z {z:.2f} xmin {min(xs):.3f} xmax {max(xs):.3f} ymin {min(ys):.3f} ymax {max(ys):.3f} n {len(sl)}")
# Kamera & Licht
scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.samples = 8
scene.render.resolution_x = 800
scene.render.resolution_y = 900
w = bpy.data.worlds.new("W"); scene.world = w; w.color = (0.5, 0.5, 0.5)
for name, loc, rot in (("front", (0, -4, 0.9), (90, 0, 0)), ("side", (4, 0, 0.9), (90, 0, 90))):
    cd = bpy.data.cameras.new(name); cd.type = "ORTHO"; cd.ortho_scale = 2.0
    cam = bpy.data.objects.new(name, cd); scene.collection.objects.link(cam)
    cam.location = loc; cam.rotation_euler = tuple(math.radians(a) for a in rot)
    scene.camera = cam
    scene.render.filepath = f"{out}_{name}.png"
    bpy.ops.render.render(write_still=True)
