"""Backt alle Texturen nach assets/textures (Farbe + Normal-Map).

Aufruf: tools/blender.sh blender/textures/build.py [-- name1 name2 ...]
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import time  # noqa: E402

import bpy  # noqa: E402
import numpy as np  # noqa: E402
from lib import NB, TEX_DIR, reset_scene, script_args  # noqa: E402
from textures import sprites, surfaces  # noqa: E402,F401  (registriert die Texturen)
from textures.baker import bake_socket, height_to_normal, linear_to_srgb, make_plane, save_png  # noqa: E402
from textures.registry import TEXTURES  # noqa: E402


def main():
    wanted = set(script_args())
    scene = reset_scene()
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.use_denoising = False
    scene.render.bake.margin = 0
    os.makedirs(TEX_DIR, exist_ok=True)
    obj = make_plane()
    for name, spec in TEXTURES.items():
        if wanted and name not in wanted:
            continue
        t0 = time.time()
        mat = bpy.data.materials.new("bake_" + name)
        try:
            mat.use_nodes = True
        except Exception:
            pass
        obj.data.materials.clear()
        obj.data.materials.append(mat)
        nb = NB(mat)
        res = spec["fn"](nb)
        size = spec["size"]
        col = bake_socket(obj, mat, nb, res["color"], size, spec["samples"], non_color=False)
        rgba = np.ones((size, size, 4), dtype=np.float32)
        rgba[:, :, :3] = linear_to_srgb(col[:, :, :3])
        if "alpha" in res:
            a = bake_socket(obj, mat, nb, res["alpha"], size, spec["samples"], non_color=True)
            rgba[:, :, 3] = np.clip(a[:, :, 0], 0, 1)
        save_png(os.path.join(TEX_DIR, name + ".png"), rgba, size)
        if "height" in res and spec["normal"] > 0:
            h = bake_socket(obj, mat, nb, res["height"], size, spec["samples"], non_color=True)[:, :, 0]
            nrm = height_to_normal(h, spec["normal"] * size / 256.0)
            out = np.ones((size, size, 4), dtype=np.float32)
            out[:, :, :3] = nrm
            save_png(os.path.join(TEX_DIR, name + "_n.png"), out, size)
        bpy.data.materials.remove(mat)
        print(f"TEXTUR {name}: {time.time() - t0:.1f}s")


main()
