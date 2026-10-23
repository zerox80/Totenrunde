import sys
import bpy
sys.path.insert(0, "/home/mavi/Dokumente/Projekte/Bo2/blender")
from characters import human_base
bpy.ops.wm.read_factory_settings(use_empty=True)
for kind in ("male", "female"):
    o = human_base.load_body(kind, name="b_" + kind)
    me = o.data
    uv = me.uv_layers.active.data
    tiles = {}
    for poly in me.polygons:
        u = sum(uv[i].uv.x for i in poly.loop_indices) / poly.loop_total
        v = sum(uv[i].uv.y for i in poly.loop_indices) / poly.loop_total
        key = (int(u // 1), int(v // 1))
        c = poly.center
        t = tiles.setdefault(key, [0, 9, -9, 9, -9])
        t[0] += 1
        t[1] = min(t[1], c.z); t[2] = max(t[2], c.z)
        t[3] = min(t[3], abs(c.x)); t[4] = max(t[4], abs(c.x))
    for k, t in sorted(tiles.items()):
        print("TILE", kind, k, "faces", t[0], "z", round(t[1], 2), round(t[2], 2), "|x|", round(t[3], 2), round(t[4], 2))
