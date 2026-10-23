"""Listet Objekte/Meshes einer .blend-Datei (nur Daten lesen, keine Skripte)."""
import sys
import bpy

path = sys.argv[sys.argv.index("--") + 1]
with bpy.data.libraries.load(path, link=False) as (src, dst):
    print("OBJECTS", list(src.objects))
    print("MESHES", list(src.meshes))
    print("COLLECTIONS", list(src.collections))
    print("ARMATURES", list(src.armatures))
    print("TEXTS", list(src.texts))
    dst.objects = [n for n in src.objects]
for o in dst.objects:
    if o is None:
        continue
    info = ""
    if o.type == "MESH":
        me = o.data
        info = f"verts={len(me.vertices)} faces={len(me.polygons)} uv={[u.name for u in me.uv_layers]} mods={[m.type for m in o.modifiers]} dims={tuple(round(d,3) for d in o.dimensions)}"
    print("OBJ", o.name, o.type, info, "parent=", o.parent.name if o.parent else None)
