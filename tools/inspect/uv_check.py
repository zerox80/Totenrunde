"""Prüft die UVs eines exportierten Modells: Flächen mit extremem UV/3D-Flächenverhältnis."""
import sys
import bpy
import numpy as np
path = sys.argv[sys.argv.index("--") + 1]
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=path)
for o in bpy.data.objects:
    if o.type != "MESH":
        continue
    me = o.data
    uv = me.uv_layers.active.data
    ratios, cents, areas = [], [], []
    for p in me.polygons:
        u = [uv[i].uv for i in p.loop_indices]
        a = 0.0
        for k in range(1, len(u) - 1):
            a += abs((u[k].x - u[0].x) * (u[k + 1].y - u[0].y) - (u[k + 1].x - u[0].x) * (u[k].y - u[0].y)) / 2
        ratios.append(a / max(p.area, 1e-12))
        cents.append(tuple(round(c, 3) for c in o.matrix_world @ p.center))
        areas.append(a)
    r = np.array(ratios)
    med = np.median(r)
    bad = np.nonzero(r > med * 8)[0]
    print("UVCHECK", o.name, "faces", len(r), "median", round(float(med), 2), "bad", len(bad), "uv-area bad", round(float(np.array(areas)[bad].sum()), 3))
    for i in bad[:8]:
        print("UVCHECK   face", i, "ratio", round(float(r[i] / med), 1), "center", cents[i], "3d area", me.polygons[i].area)
# größte UV-Flächen
for o in bpy.data.objects:
    if o.type != "MESH":
        continue
    me = o.data
    uv = me.uv_layers.active.data
    best = []
    for p in me.polygons:
        u = [uv[i].uv for i in p.loop_indices]
        a = sum(abs((u[k].x - u[0].x) * (u[k + 1].y - u[0].y) - (u[k + 1].x - u[0].x) * (u[k].y - u[0].y)) / 2 for k in range(1, len(u) - 1))
        best.append((a, p.index, tuple(round(c, 3) for c in p.center), p.area, [tuple(round(x, 3) for x in v) for v in u]))
    best.sort(reverse=True)
    for b in best[:3]:
        print("UVBIG", o.name, b)
