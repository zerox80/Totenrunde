"""Ton-Vorschau der Charakter-Skulptur (Workbench-Renderer) -> tools/preview/sc_*.png

Aufruf: tools/blender.sh blender/characters/sculpt_preview.py [-- female] [-- h=2.0] [-- views=body,head,hand]
"""

import math
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import bpy  # noqa: E402
import numpy as np  # noqa: E402
from characters import zombie_sculpt  # noqa: E402
from lib import PREVIEW_DIR, reset_scene, script_args  # noqa: E402
from sculpt import mesher  # noqa: E402

ARGS = script_args()
OPT = dict(a.split("=", 1) for a in ARGS if "=" in a)

TAG_COLORS = {
    "skin": (0.78, 0.74, 0.68),
    "shirt": (0.45, 0.55, 0.75),
    "pants": (0.30, 0.34, 0.45),
    "shoes": (0.25, 0.18, 0.12),
    "flesh": (0.7, 0.15, 0.12),
    "bone": (0.95, 0.92, 0.8),
    "teeth": (0.95, 0.93, 0.85),
    "mouth": (0.25, 0.05, 0.05),
    "nail": (0.85, 0.75, 0.55),
    "hair": (0.15, 0.12, 0.1),
    "belt": (0.12, 0.08, 0.05),
    "sole": (0.08, 0.08, 0.08),
    "buckle": (0.7, 0.7, 0.72),
}


def cam(name, loc, look, lens=50.0, ortho=None):
    cd = bpy.data.cameras.new(name)
    if ortho:
        cd.type = "ORTHO"
        cd.ortho_scale = ortho
    cd.lens = lens
    cd.clip_start = 0.01
    o = bpy.data.objects.new(name, cd)
    bpy.context.scene.collection.objects.link(o)
    o.location = loc
    d = np.asarray(look, dtype=float) - np.asarray(loc, dtype=float)
    from mathutils import Vector

    o.rotation_euler = Vector(d).to_track_quat("-Z", "Y").to_euler()
    return o


def main():
    kind = "female" if "female" in ARGS else "male"
    h = float(OPT.get("h", "2.0")) / 1000.0
    views = OPT.get("views", "body,head,hand").split(",")
    scene = reset_scene()
    t0 = time.time()
    naked = "naked" in ARGS
    sk, sc, _info = zombie_sculpt.sculpt(kind, clothes=not naked, wounds=not naked)
    lo, hi = sc.bounds()
    v, q = mesher.surface_nets(sc, lo - 0.01, hi + 0.01, h)
    obj = mesher.to_blender("Sculpt", v, q)
    # Etiketten als Vertex-Farben
    _, tags = sc.eval_chunked(v, tags=True)
    cols = np.ones((len(v), 4), dtype=np.float32)
    for i, name in enumerate(sc.tag_names):
        cols[tags == i, :3] = TAG_COLORS.get(name, (1, 0, 1))
    ca = obj.data.color_attributes.new("Tags", "FLOAT_COLOR", "POINT")
    ca.data.foreach_set("color", cols.ravel())
    # Augäpfel (im Spiel eigenes, leuchtendes Objekt)
    for e in _info["eyes"]:
        bpy.ops.mesh.primitive_uv_sphere_add(radius=_info["eye_r"], location=tuple(e), segments=24, ring_count=12)
        eo = bpy.context.active_object
        eo.data.shade_smooth()
        ea = eo.data.color_attributes.new("Tags", "FLOAT_COLOR", "POINT")
        ea.data.foreach_set("color", np.tile([1.0, 0.75, 0.25, 1.0], len(eo.data.vertices)).astype(np.float32))
    print(f"Skulptur fertig in {time.time() - t0:.1f} s")

    scene.render.engine = "BLENDER_WORKBENCH"
    sh = scene.display.shading
    sh.light = "STUDIO"
    sh.color_type = "VERTEX"
    sh.show_cavity = True
    sh.cavity_type = "BOTH"
    sh.show_shadows = False
    scene.display.render_aa = "8"
    scene.view_settings.view_transform = "Standard"
    world = bpy.data.worlds.new("W")
    scene.world = world
    world.color = (0.32, 0.33, 0.35)
    hc = sk["hc"]
    shots = []
    if "body" in views:
        top = sk["head_top"]
        mid = (0, 0, top / 2)
        shots += [
            ("body_front", cam("c", (0, 6, top / 2), mid, ortho=top * 1.08), 800, 1000),
            ("body_side", cam("c", (6, 0, top / 2), mid, ortho=top * 1.08), 800, 1000),
            ("body_back", cam("c", (0, -6, top / 2), mid, ortho=top * 1.08), 800, 1000),
            ("body_34", cam("c", (2.4, 3.2, 1.3), (0, 0, 0.95), lens=40), 800, 1000),
        ]
    if "head" in views:
        f = (hc[0], hc[1] + 0.03, hc[2] - 0.04)
        shots += [
            ("head_front", cam("c", (0, 0.75, hc[2] - 0.03), f, lens=85), 800, 800),
            ("head_34", cam("c", (0.5, 0.55, hc[2] + 0.02), f, lens=85), 800, 800),
            ("head_side", cam("c", (0.75, 0.02, hc[2] - 0.03), f, lens=85), 800, 800),
        ]
    if "hand" in views:
        d = sk["side"]["R"]
        hcen = d["W"] + d["ha"] * 0.09
        shots += [
            ("hand_back", cam("c", tuple(hcen - d["hn"] * 0.45 + np.array([0, 0.05, 0])), tuple(hcen), lens=85), 700, 700),
            ("hand_palm", cam("c", tuple(hcen + d["hn"] * 0.45 + np.array([0, 0.05, 0])), tuple(hcen), lens=85), 700, 700),
        ]
    for name, c, w, hgt in shots:
        scene.camera = c
        scene.render.resolution_x = w
        scene.render.resolution_y = hgt
        scene.render.filepath = os.path.join(PREVIEW_DIR, f"sc_{kind[0]}_{name}.png")
        bpy.ops.render.render(write_still=True)
    print("FERTIG", time.time() - t0)


main()
_ = math
