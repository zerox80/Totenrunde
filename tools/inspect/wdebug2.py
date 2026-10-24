import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
import bpy
import numpy as np
from lib import reset_scene
reset_scene()
from weapons import m1911, kit
kit.bake_weapon = lambda wid, objs, **k: None
kit_finish = kit.finish
m1911.bake_weapon = lambda wid, objs, **k: None
m1911.finish = lambda wid, muzzle: None
m1911.build()
for o in bpy.data.objects:
    if o.type == "MESH":
        co = np.array([(o.matrix_world @ v.co)[:] for v in o.data.vertices])
        print("OBJ", o.name, len(co), co.min(0).round(3), co.max(0).round(3), [m.name for m in o.data.materials])
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "cache", "m1911_debug.blend"))
