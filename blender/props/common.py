"""Gemeinsame Helfer für Requisiten: Materialien, 3D-Schrift, Export."""

import math
import os

import bpy

from lib import MODEL_DIR, export_gltf, hexcol, material, reset_scene

C = hexcol


def mats():
    return dict(
        metal=material("R_Metal", tex="gunmetal", normal="gunmetal_n", metal=0.8, rough=0.45, color=C("#9aa0a6")),
        dark=material("R_DarkMetal", tex="metal_paint", normal="metal_paint_n", metal=0.6, rough=0.55, color=C("#2b2c30")),
        rust=material("R_Rust", tex="rust", normal="rust_n", metal=0.3, rough=0.85),
        chrome=material("R_Chrome", color=C("#d9dde2"), metal=1.0, rough=0.18),
        glass=material("R_Glass", color=C("#3a4a55"), metal=0.0, rough=0.05, alpha=0.35),
        rubber=material("R_Rubber", color=C("#151515"), rough=0.9),
        wood=material("R_Wood", tex="wood_dark", normal="wood_dark_n", rough=0.75),
        brass=material("R_Brass", color=C("#b8913d"), metal=1.0, rough=0.3),
    )


def glow(name, col, strength=6.0):
    return material(name, color=col, emission=col, emission_strength=strength, rough=0.4)


def paint(name, col):
    return material(name, tex="metal_paint", normal="metal_paint_n", color=col, metal=0.35, rough=0.5)


def text_obj(name, text, size, extrude, mat, loc, face="front", bold=True):
    """3D-Schrift als Mesh. face='front': lesbar von +Y (Vorderseite der Modelle)."""
    cu = bpy.data.curves.new(name, "FONT")
    cu.body = text
    cu.size = size
    cu.extrude = extrude
    cu.align_x = "CENTER"
    cu.align_y = "CENTER"
    if bold:
        cu.offset = 0.0
    ob = bpy.data.objects.new(name, cu)
    bpy.context.scene.collection.objects.link(ob)
    ob.location = loc
    if face == "front":
        ob.rotation_euler = (math.radians(90), 0, math.radians(180))
    elif face == "up":
        ob.rotation_euler = (0, 0, math.radians(180))
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
    bpy.data.objects.remove(ob)
    bpy.data.curves.remove(cu)
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    obj.location = loc
    obj.rotation_euler = (math.radians(90), 0, math.radians(180)) if face == "front" else ((0, 0, math.radians(180)) if face == "up" else (0, 0, 0))
    me.materials.clear()
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = False
    return obj


def export(name):
    path = os.path.join(MODEL_DIR, name + ".gltf")
    export_gltf(path)
    return path


def fresh():
    return reset_scene()
