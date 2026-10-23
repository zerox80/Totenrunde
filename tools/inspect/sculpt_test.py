import os, sys, time
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
import bpy, bmesh
import numpy as np
from sculpt import sdf, mesher
from lib import reset_scene
reset_scene()
s = sdf.Sculpt()
s.add(sdf.Sphere((0, 0, 0.5), 0.2))
s.add(sdf.RoundCone((0, 0, 0.5), (0.4, 0, 0.9), 0.08, 0.02), k=0.05)
s.add(sdf.Loft((0, 0, 0), (0, 0, 0.4), [(0, 0.1, 0.06, 0, 0), (0.5, 0.06, 0.05, 0, 0.02), (1, 0.12, 0.08, 0, 0)]), k=0.04)
s.sub(sdf.Sphere((0.15, 0.1, 0.55), 0.06), k=0.01, tag="flesh")
lo, hi = s.bounds()
v, q = mesher.surface_nets(s, lo - 0.02, hi + 0.02, 0.0025)
o = mesher.to_blender("T", v, q)
bm = bmesh.new(); bm.from_mesh(o.data)
nm = [e for e in bm.edges if not e.is_manifold]
print("RESULT verts", len(bm.verts), "faces", len(bm.faces), "nonmanifold edges", len(nm))
# Normalen nach außen? Volumen positiv
print("RESULT volume", bm.calc_volume(signed=True))
