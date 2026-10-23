"""Materialien (Principled BSDF), die sauber nach glTF exportieren."""

import os

import bpy

from .nodes import NB
from .paths import TEX_DIR

_image_cache = {}


def load_image(name, non_color=False):
    """Lädt assets/textures/<name>.png (gecached)."""
    key = (name, non_color)
    if key in _image_cache:
        return _image_cache[key]
    path = os.path.join(TEX_DIR, name + ".png")
    img = bpy.data.images.load(path, check_existing=True)
    if non_color:
        img.colorspace_settings.name = "Non-Color"
    _image_cache[key] = img
    return img


def material(
    name,
    color=(0.8, 0.8, 0.8, 1.0),
    rough=0.7,
    metal=0.0,
    tex=None,
    normal=None,
    normal_strength=1.0,
    emission=None,
    emission_strength=1.0,
    alpha_tex=False,
    blend=False,
    alpha=1.0,
    double_sided=False,
    rough_tex=None,
):
    """Principled-Material, das sauber nach glTF exportiert.

    color: lineares RGBA (z. B. hexcol('#aabbcc')). Bei tex wird color als Faktor multipliziert.
    tex / normal / rough_tex: Namen von Texturen in assets/textures (ohne .png);
    rough_tex enthält Rauheit im Grün- und Metall im Blaukanal.
    """
    if name in bpy.data.materials:
        return bpy.data.materials[name]
    m = bpy.data.materials.new(name)
    try:
        m.use_nodes = True
    except Exception:
        pass
    nb = NB(m)
    bsdf = nb.n("ShaderNodeBsdfPrincipled")
    out = nb.n("ShaderNodeOutputMaterial")
    nb.link(bsdf.outputs[0], out.inputs["Surface"])
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metal
    if tex:
        t = nb.n("ShaderNodeTexImage")
        t.image = load_image(tex)
        if tuple(color[:3]) != (1.0, 1.0, 1.0):
            mixed = nb.mix(1.0, t.outputs["Color"], color, blend="MULTIPLY")
            nb.link(mixed, bsdf.inputs["Base Color"])
        else:
            nb.link(t.outputs["Color"], bsdf.inputs["Base Color"])
        if alpha_tex:
            nb.link(t.outputs["Alpha"], bsdf.inputs["Alpha"])
    else:
        bsdf.inputs["Base Color"].default_value = color
    if rough_tex:
        # glTF: metallicRoughness-Textur (G = Rauheit, B = Metall)
        rt = nb.n("ShaderNodeTexImage")
        rt.image = load_image(rough_tex, non_color=True)
        sep = nb.n("ShaderNodeSeparateColor")
        nb.link(rt.outputs["Color"], sep.inputs[0])
        nb.link(sep.outputs[1], bsdf.inputs["Roughness"])
        nb.link(sep.outputs[2], bsdf.inputs["Metallic"])
    if normal:
        nt = nb.n("ShaderNodeTexImage")
        nt.image = load_image(normal, non_color=True)
        nm = nb.n("ShaderNodeNormalMap")
        nm.inputs["Strength"].default_value = normal_strength
        nb.link(nt.outputs["Color"], nm.inputs["Color"])
        nb.link(nm.outputs["Normal"], bsdf.inputs["Normal"])
    if emission is not None:
        bsdf.inputs["Emission Color"].default_value = emission
        bsdf.inputs["Emission Strength"].default_value = emission_strength
    if alpha < 1.0:
        bsdf.inputs["Alpha"].default_value = alpha
    if blend or alpha < 1.0:
        try:
            m.surface_render_method = "BLENDED"
        except Exception:
            pass
    elif alpha_tex:
        # glTF: alphaMode MASK
        try:
            m.surface_render_method = "DITHERED"
        except Exception:
            pass
    m.use_backface_culling = not double_sided
    return m
