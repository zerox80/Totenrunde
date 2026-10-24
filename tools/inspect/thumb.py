"""Verkleinert ein (selbst geschriebenes) PNG: tools/blender.sh tools/inspect/thumb.py -- in.png out.png faktor"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
from textures.synth import read_png, write_png  # noqa: E402

src, dst, f = sys.argv[sys.argv.index("--") + 1 :]
a = read_png(src)
f = int(f)
write_png(dst, a[::f, ::f, :3])
