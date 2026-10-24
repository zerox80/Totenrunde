import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
from lib import reset_scene
reset_scene()
from weapons import m1911
for fn in (m1911.slide, m1911.frame, m1911.grips, m1911.barrel, m1911.magazine):
    o = fn()
    import numpy as np
    co = np.array([v.co[:] for v in o.data.vertices]) if len(o.data.vertices) else np.zeros((1,3))
    print("TEIL", fn.__name__, len(o.data.vertices), len(o.data.polygons), co.min(0).round(3), co.max(0).round(3))
