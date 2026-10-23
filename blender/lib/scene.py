"""Szenen-Hilfen: Zurücksetzen, Auswahl, leere Objekte, Eltern-Beziehungen."""

import bpy

from . import materials


def reset_scene(fps=30):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    materials._image_cache.clear()
    scene = bpy.context.scene
    scene.render.fps = fps
    scene.unit_settings.system = "METRIC"
    return scene


def empty(name, loc=(0, 0, 0), parent=None, collection=None):
    o = bpy.data.objects.new(name, None)
    o.location = loc
    (collection or bpy.context.scene.collection).objects.link(o)
    if parent:
        o.parent = parent
    return o


def parent_keep(child, parent):
    child.parent = parent
    child.matrix_parent_inverse = parent.matrix_world.inverted()

def set_active(obj):
    bpy.context.view_layer.update()
    bpy.context.view_layer.objects.active = obj
    for o in bpy.data.objects:
        if o.name in bpy.context.view_layer.objects:
            o.select_set(False)
    obj.select_set(True)
