"""Kontaktbogen aus Vorschaubildern: tools/blender.sh tools/inspect/montage.py -- out.png cols scale in1.png in2.png ..."""
import os
import sys

import bpy
import numpy as np

args = sys.argv[sys.argv.index("--") + 1:]
out, cols, scale, files = args[0], int(args[1]), int(args[2]), args[3:]
tiles = []
for f in files:
    if not os.path.exists(f):
        continue
    img = bpy.data.images.load(f)
    w, h = img.size
    px = np.empty(w * h * 4, np.float32)
    img.pixels.foreach_get(px)
    px = px.reshape(h, w, 4)[::scale, ::scale]
    tiles.append(px)
th, tw = tiles[0].shape[:2]
rows = (len(tiles) + cols - 1) // cols
sheet = np.ones((rows * th, cols * tw, 4), np.float32)
for i, t in enumerate(tiles):
    r, c = divmod(i, cols)
    sheet[(rows - 1 - r) * th:(rows - r) * th, c * tw:(c + 1) * tw] = t[:th, :tw]
o = bpy.data.images.new("sheet", cols * tw, rows * th, alpha=True)
o.pixels.foreach_set(sheet.ravel())
o.filepath_raw = out
o.file_format = "PNG"
o.save()
