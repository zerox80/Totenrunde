"""Decal-Atlas über grauem Hintergrund: tools/blender.sh tools/inspect/decal_preview.py"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
from textures.synth import read_png, write_png  # noqa: E402

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
a = read_png(os.path.join(root, "assets", "textures", "decals.png"))[::2, ::2]
bg = 0.55
rgb = a[..., :3] * a[..., 3:4] + bg * (1 - a[..., 3:4])
write_png(os.path.join(root, "tools", "preview", "decals_preview.png"), rgb)
