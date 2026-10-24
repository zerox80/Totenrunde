"""Gebackenes Licht (Lightmaps) für statische Kartengeometrie.

Ablauf (in `bake_lightmaps`):
  1. Zweite UV-Ebene "Lightmap" pro Objekt, alle Objekte einer Gruppe teilen sich einen
     Atlas (Smart-Projektion, gleichmäßige Texeldichte, Packen mit Rand).
  2. Licht der Karte als Blender-Lichter aufbauen (nur Lampen, die immer gleich leuchten),
     Mond als Sonne, Himmel als Welt-Hintergrund.
  3. Cycles backt das einfallende Licht (direkt + indirekt, ohne Albedo) in den Atlas.
  4. Entrauschen (maskierter Weichzeichner), Kodieren als sRGB-PNG mit Skalierung.

Einheiten (so gewählt, dass Bevy und Cycles übereinstimmen):
  * Punktlicht: Cycles-Watt = Bevy-Lumen  -> Ausgabe = E_lux / π
  * Mond:       Sonnenstärke = lux / π
  * Himmel:     Welt-Strahldichte = Bevy-Umgebungshelligkeit (cd/m²)
Im Spiel gilt dann: Lightmap * lightmap_exposure ≈ E / π (wie Bevys direkte Beleuchtung).
"""

import math
import os
import time

import bpy
import numpy as np
from mathutils import Vector

from lib import ROOT
from textures.synth import write_png

LM_DIR = os.path.join(ROOT, "assets", "textures")


def _select_only(objs):
    bpy.ops.object.mode_set(mode="OBJECT") if bpy.context.object and bpy.context.object.mode != "OBJECT" else None
    for o in bpy.context.view_layer.objects:
        o.select_set(False)
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]


def make_uvs(objs, size, margin_px=6):
    """Legt die UV-Ebene "Lightmap" an und packt alle Objekte in einen gemeinsamen Atlas."""
    for o in objs:
        me = o.data
        if "Lightmap" not in me.uv_layers:
            me.uv_layers.new(name="Lightmap")
        me.uv_layers.active = me.uv_layers["Lightmap"]
    _select_only(objs)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=math.radians(50), island_margin=0.0, area_weight=0.0, correct_aspect=True, scale_to_bounds=False)
    bpy.ops.uv.select_all(action="SELECT")
    bpy.ops.uv.average_islands_scale()
    bpy.ops.uv.pack_islands(rotate=True, margin_method="FRACTION", margin=margin_px / size, shape_method="AABB")
    bpy.ops.object.mode_set(mode="OBJECT")


def scene_lights(lights, sun, ambient, light_scale):
    """Baut die Beleuchtung für das Backen auf. Gibt die Liste der gebackenen Lichter zurück."""
    baked = []
    for i, l in enumerate(lights):
        if l["flicker"] > 0.0 or l["needs_power"]:
            continue
        # Bevy-Koordinaten (x, y, z) -> Blender (x, -z, y)
        x, y, z = l["pos"]
        data = bpy.data.lights.new(f"LM_Light_{i}", "POINT")
        data.energy = l["intensity"] * light_scale
        data.color = l["color"]
        data.shadow_soft_size = 0.12
        try:
            data.use_custom_distance = True
            data.cutoff_distance = l["range"]
        except Exception:
            pass
        ob = bpy.data.objects.new(f"LM_Light_{i}", data)
        ob.location = (x, -z, y)
        bpy.context.scene.collection.objects.link(ob)
        l["baked"] = True
        baked.append(ob)
    # Mond
    d = Vector((sun["dir"][0], -sun["dir"][2], sun["dir"][1])).normalized()  # Bevy -> Blender
    sd = bpy.data.lights.new("LM_Moon", "SUN")
    sd.energy = sun["illuminance"] / math.pi
    sd.color = sun["color"]
    sd.angle = math.radians(1.5)
    so = bpy.data.objects.new("LM_Moon", sd)
    so.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
    bpy.context.scene.collection.objects.link(so)
    # Himmel
    world = bpy.data.worlds.new("LM_World")
    world.use_nodes = True
    bg = world.node_tree.nodes.get("Background")
    c = ambient["color"]
    bg.inputs["Color"].default_value = (c[0], c[1], c[2], 1.0)
    bg.inputs["Strength"].default_value = ambient["brightness"]
    bpy.context.scene.world = world
    return baked


def _bake_image(name, size):
    img = bpy.data.images.get(name)
    if img:
        bpy.data.images.remove(img)
    img = bpy.data.images.new(name, size, size, alpha=True, float_buffer=True)
    img.colorspace_settings.name = "Non-Color"
    return img


def _attach(objs, img):
    """Setzt in allen Materialien der Objekte einen aktiven Bild-Knoten auf `img`."""
    mats = {m for o in objs for m in o.data.materials if m}
    for m in mats:
        nt = m.node_tree
        node = nt.nodes.get("LM_Target")
        if node is None:
            node = nt.nodes.new("ShaderNodeTexImage")
            node.name = "LM_Target"
        node.image = img
        for n in nt.nodes:
            n.select = False
        node.select = True
        nt.nodes.active = node
    return mats


def _denoise(px, mask, passes=2):
    """Maskierter Weichzeichner im Atlas (nur innerhalb belegter Texel)."""
    k = np.array([1, 4, 6, 4, 1], np.float32)
    k /= k.sum()
    m = mask.astype(np.float32)
    out = px.copy()
    for _ in range(passes):
        acc = out * m[..., None]
        w = m.copy()
        for axis in (0, 1):
            acc = sum(np.roll(acc, s, axis=axis) * k[s + 2] for s in range(-2, 3))
            w = sum(np.roll(w, s, axis=axis) * k[s + 2] for s in range(-2, 3))
        out = np.where(m[..., None] > 0, acc / np.maximum(w[..., None], 1e-6), out)
    return out


def bake_group(name, objs, size, samples):
    """Backt einen Atlas für die Objekte. Gibt (rgb, mask) zurück (Zeilen von oben nach unten)."""
    t0 = time.time()
    make_uvs(objs, size)
    img = _bake_image("LM_" + name, size)
    mats = _attach(objs, img)
    scene = bpy.context.scene
    scene.cycles.samples = samples
    _select_only(objs)
    bpy.ops.object.bake(type="DIFFUSE", pass_filter={"DIRECT", "INDIRECT"}, margin=12, margin_type="EXTEND", use_clear=True)
    px = np.empty(size * size * 4, np.float32)
    img.pixels.foreach_get(px)
    px = px.reshape(size, size, 4)
    rgb = px[..., :3]
    # Belegte Texel: Rasterisieren der UV-Dreiecke wäre exakt; die Bake-Ränder füllen
    # aber ohnehin alles um die Inseln, daher reicht "nicht schwarz" als Maske.
    mask = rgb.max(-1) > 1e-6
    rgb = _denoise(rgb, mask)
    for m in mats:
        n = m.node_tree.nodes.get("LM_Target")
        if n:
            m.node_tree.nodes.remove(n)
    bpy.data.images.remove(img)
    print(f"LIGHTMAP {name}: {len(objs)} Objekte, {size}px, {time.time() - t0:.0f}s", flush=True)
    # Bildzeilen: Blender speichert von unten nach oben, PNG von oben nach unten
    return rgb[::-1], mask[::-1]


def bake_lightmaps(groups, lights, sun, ambient, light_scale, samples=128, mute_emission=()):
    """groups: Liste von dict(name, objects, size). Liefert die JSON-Einträge für das Spiel."""
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.max_bounces = 4
    scene.cycles.diffuse_bounces = 3
    scene.cycles.glossy_bounces = 0
    scene.cycles.transparent_max_bounces = 4
    scene.cycles.use_denoising = False
    scene.render.threads_mode = "AUTO"
    baked = scene_lights(lights, sun, ambient, light_scale)
    # Leuchtende Lampenflächen beim Backen abschalten (Licht kommt von den Punktlichtern)
    saved = []
    for m in bpy.data.materials:
        if any(m.name.startswith(p) for p in mute_emission) and m.node_tree:
            for n in m.node_tree.nodes:
                if n.type == "BSDF_PRINCIPLED":
                    saved.append((n, n.inputs["Emission Strength"].default_value))
                    n.inputs["Emission Strength"].default_value = 0.0
    results = []
    for g in groups:
        objs = [o for o in g["objects"] if o.type == "MESH" and len(o.data.polygons) > 0]
        if not objs:
            continue
        rgb, mask = bake_group(g["name"], objs, g["size"], samples)
        results.append((g, objs, rgb, mask))
    # Gemeinsame Skalierung, weil Materialien in mehreren Atlanten vorkommen können
    scale = 1e-3
    for _, _, rgb, mask in results:
        if mask.any():
            scale = max(scale, float(np.percentile(rgb.max(-1)[mask], 99.7)))
    entries = []
    for g, objs, rgb, mask in results:
        write_png(os.path.join(LM_DIR, g["image"] + ".png"), np.clip(rgb / scale, 0, 1), srgb=True)
        entries.append({"image": f"textures/{g['image']}.png", "exposure": round(scale, 4), "objects": [o.name for o in objs]})
    print(f"LIGHTMAP Skala {scale:.2f}", flush=True)
    for n, v in saved:
        n.inputs["Emission Strength"].default_value = v
    for ob in baked:
        bpy.data.objects.remove(ob)
    # UV-Ebene 0 wieder aktiv (glTF: TEXCOORD_0 = Material-UVs, TEXCOORD_1 = Lightmap)
    for g in groups:
        for o in g["objects"]:
            if o.type == "MESH" and len(o.data.uv_layers) > 1:
                o.data.uv_layers.active = o.data.uv_layers[0]
                o.data.uv_layers[0].active_render = True
    return entries
