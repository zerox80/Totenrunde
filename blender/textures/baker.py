"""Backen von Shader-Sockets per Cycles (Emission) und Nachbearbeitung mit numpy
(sRGB-Kodierung, Alpha, Normal-Map aus Höhenwerten)."""

import bmesh
import bpy
import numpy as np

def make_plane():
    me = bpy.data.meshes.new("BakePlane")
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    vs = [bm.verts.new(p) for p in ((0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0))]
    f = bm.faces.new(vs)
    for loop, uv in zip(f.loops, ((0, 0), (1, 0), (1, 1), (0, 1))):
        loop[uvl].uv = uv
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new("BakePlane", me)
    bpy.context.scene.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    return obj


def bake_socket(obj, mat, nb, sock, size, samples, non_color):
    """Backt einen Socket (Farbe oder Float) als Emission in ein neues Bild."""
    nt = mat.node_tree
    for n in [n for n in nt.nodes if n.get("_bake_tmp")]:
        nt.nodes.remove(n)
    em = nb.n("ShaderNodeEmission")
    em["_bake_tmp"] = True
    nb.set(em.inputs["Color"], sock)
    em.inputs["Strength"].default_value = 1.0
    out = nb.n("ShaderNodeOutputMaterial")
    out["_bake_tmp"] = True
    nb.link(em.outputs[0], out.inputs["Surface"])
    img = bpy.data.images.new("bake", size, size, alpha=False, float_buffer=True)
    img.colorspace_settings.name = "Non-Color" if non_color else "Linear Rec.709"
    tex = nb.n("ShaderNodeTexImage")
    tex["_bake_tmp"] = True
    tex.image = img
    for n in nt.nodes:
        n.select = False
    tex.select = True
    nt.nodes.active = tex
    bpy.context.scene.cycles.samples = samples
    bpy.ops.object.bake(type="EMIT", margin=0, use_clear=True)
    px = np.empty(size * size * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    return px.reshape(size, size, 4)


def linear_to_srgb(x):
    x = np.clip(x, 0.0, 1.0)
    return np.where(x <= 0.0031308, x * 12.92, 1.055 * np.power(x, 1.0 / 2.4) - 0.055)


def save_png(path, rgba, size):
    img = bpy.data.images.new("out", size, size, alpha=True, float_buffer=False)
    img.colorspace_settings.name = "Non-Color"
    img.pixels.foreach_set(np.clip(rgba, 0, 1).astype(np.float32).ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    bpy.data.images.remove(img)


def height_to_normal(h, strength):
    dx = (np.roll(h, -1, axis=1) - np.roll(h, 1, axis=1)) * 0.5 * strength
    dy = (np.roll(h, -1, axis=0) - np.roll(h, 1, axis=0)) * 0.5 * strength
    n = np.dstack([-dx, -dy, np.ones_like(h)])
    n /= np.linalg.norm(n, axis=2, keepdims=True)
    return n * 0.5 + 0.5
