import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
import bpy, numpy as np
from lib import reset_scene
reset_scene()
from weapons import kit, parts
orig = kit.pose_arm
def dbg(side, arms, wrist, a, n, elbow, targets, **kw):
    print("POSE", side, "W", tuple(round(x,3) for x in wrist), "a", tuple(round(x,3) for x in a))
    o = orig(side, arms, wrist, a, n, elbow, targets, **kw)
    co = np.array([v.co[:] for v in o.data.vertices])
    rig = arms[f"Rig_{side}"]
    print("  rig loc", tuple(round(x,3) for x in rig.matrix_world.translation), "mesh min", co.min(0).round(3), "max", co.max(0).round(3))
    return o
parts.pose_arm = dbg
from weapons import longguns
longguns.bake_weapon = lambda *a, **k: None
longguns.finish = lambda *a, **k: None
longguns.build_mp5()
