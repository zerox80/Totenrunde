"""Baut die Ego-Arme (rechts + links) als geriggte Spielmodelle:

  1. Skulptur (arms_sdf) -> Hochpoly per Surface Nets (0,9 mm) mit Etiketten, Schmutz, Blut
  2. Spielmodell: dezimiert (~10k Dreiecke je Arm), gemeinsamer UV-Atlas
  3. Backen (Cycles): Normalen, AO, Farbe, Rauheit -> fparms_albedo/_n/_mr.png (2048)
  4. Skelett je Arm (Hand als Wurzel, Unterarm, je 3 Glieder für Finger und Daumen) mit
     Gewichten aus den Segmenten der Skulptur
  5. Speichern nach tools/cache/fparms.blend (die Waffen-Skripte laden und posieren daraus)

Aufruf: tools/blender.sh blender/fparms/arms_build.py [-- nobake] [-- preview]
"""

import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import bmesh  # noqa: E402
import bpy  # noqa: E402
import numpy as np  # noqa: E402
from mathutils import Vector  # noqa: E402

from lib import ROOT, TEX_DIR, material, reset_scene, script_args, set_active  # noqa: E402
from sculpt import mesher  # noqa: E402
from sculpt.noise import fbm, perlin  # noqa: E402

from fparms import arms_sdf  # noqa: E402
from fparms.arms_shading import TAGS, make_material  # noqa: E402

BLEND = os.path.join(ROOT, "tools", "cache", "fparms.blend")
H = 0.0009
SIZE = 2048


def log(*a):
    print("[arme]", *a, flush=True)


def cached_nets(sc, side, lo, hi, h):
    import hashlib

    src = hashlib.sha1()
    for d in ("blender/sculpt", "blender/fparms"):
        for f in sorted(os.listdir(os.path.join(ROOT, d))):
            if f in ("arms_sdf.py", "sdf.py", "noise.py", "mesher.py"):
                with open(os.path.join(ROOT, d, f), "rb") as fh:
                    src.update(fh.read())
    src.update(repr((side, tuple(np.round(lo, 4)), tuple(np.round(hi, 4)), h)).encode())
    path = os.path.join(ROOT, "tools", "cache", f"arms_{src.hexdigest()[:16]}.npz")
    if os.path.exists(path):
        z = np.load(path)
        log("Netz aus Cache")
        return z["v"], z["q"]
    v, q = mesher.surface_nets(sc, lo, hi, h, log=log)
    v, q = mesher.compact(v, q)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez(path, v=v, q=q)
    return v, q


def fields(v, bones):
    """Schmutz an Fingerkuppen und Handfläche, ein paar Blutspritzer auf Ärmel und Handrücken."""
    dirt = np.zeros(len(v))
    for name, b in bones.items():
        if name.endswith("3"):
            d = np.linalg.norm(v - b["tail"], axis=1)
            dirt += 0.8 * np.exp(-((d / 0.014) ** 2))
    dirt += 0.35 * np.clip(fbm(v, 30.0, 4, seed=3.0), 0, 1)
    dirt = np.clip(dirt * np.clip(0.4 + perlin(v, 80.0, 7.0), 0, 1.4), 0, 1)
    sp = fbm(v, 45.0, 4, seed=11.0)
    blood = np.clip((sp - 0.42) * 6.0, 0, 1) * np.clip(-v[:, 1] * 6.0 + 0.2, 0, 1) * 0.8
    return dirt.astype(np.float32), blood.astype(np.float32)


def high_poly(side):
    sc, bones = arms_sdf.build(side)
    lo, hi = arms_sdf.bounds(side)
    v, q = cached_nets(sc, side, lo, hi, H)
    obj = mesher.to_blender(f"_hi_{side}", v, q)
    _, tags = sc.eval_chunked(v, tags=True)
    for t in TAGS:
        mask = (tags == sc.tag_names.index(t)).astype(np.float32) if t in sc.tag_names else np.zeros(len(v), np.float32)
        mesher.set_attr(obj, "t_" + t, mask)
    dirt, blood = fields(v, bones)
    mesher.set_attr(obj, "dirt", dirt)
    mesher.set_attr(obj, "blood", blood)
    return obj, bones


def low_poly(hi, side, target=10000):
    me = hi.data.copy()
    for name in [a.name for a in me.attributes if a.name.startswith("t_") or a.name in ("dirt", "blood")]:
        me.attributes.remove(me.attributes[name])
    lo = bpy.data.objects.new(f"Arm_{side}", me)
    bpy.context.scene.collection.objects.link(lo)
    ntri = sum(len(p.vertices) - 2 for p in me.polygons)
    mod = lo.modifiers.new("dec", "DECIMATE")
    mod.ratio = min(1.0, target / max(ntri, 1))
    mod.use_collapse_triangulate = True
    set_active(lo)
    bpy.ops.object.modifier_apply(modifier="dec")
    bm = bmesh.new()
    bm.from_mesh(lo.data)
    bmesh.ops.dissolve_degenerate(bm, edges=bm.edges, dist=1e-5)
    bm.to_mesh(lo.data)
    bm.free()
    for p in lo.data.polygons:
        p.use_smooth = True
    log(f"Arm {side}: {len(lo.data.polygons)} Flächen")
    return lo


def unwrap(objs):
    for o in objs:
        o.data.uv_layers.new(name="UVMap")
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=0.003, area_weight=0.0, correct_aspect=True, scale_to_bounds=False)
    bpy.ops.uv.select_all(action="SELECT")
    bpy.ops.uv.average_islands_scale()
    bpy.ops.uv.pack_islands(rotate=True, margin_method="FRACTION", margin=0.004, shape_method="CONCAVE")
    bpy.ops.object.mode_set(mode="OBJECT")


def bake(lo, his, kind, samples, mat):
    scene = bpy.context.scene
    scene.cycles.samples = samples
    for h in his:
        h.data.materials.clear()
        h.data.materials.append(mat)
    img = bpy.data.images.new("_bake", SIZE, SIZE, alpha=False, float_buffer=True)
    img.colorspace_settings.name = "Non-Color"
    m = lo.data.materials[0]
    nt = m.node_tree
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = img
    for n in nt.nodes:
        n.select = False
    tex.select = True
    nt.nodes.active = tex
    set_active(lo)
    for h in his:
        h.select_set(True)
    bpy.context.view_layer.objects.active = lo
    t0 = time.time()
    bpy.ops.object.bake(type=kind, use_selected_to_active=True, cage_extrusion=0.003, max_ray_distance=0.008, margin=12, use_clear=True)
    px = np.empty(SIZE * SIZE * 4, np.float32)
    img.pixels.foreach_get(px)
    nt.nodes.remove(tex)
    bpy.data.images.remove(img)
    log(f"Bake {kind}: {time.time() - t0:.0f}s")
    return px.reshape(SIZE, SIZE, 4)


def save_png(name, rgb, srgb=False):
    from textures.synth import write_png

    write_png(os.path.join(TEX_DIR, name + ".png"), np.clip(rgb, 0, 1)[::-1], srgb=srgb)


# --------------------------------------------------------------------------------------
# Skelett
# --------------------------------------------------------------------------------------


def _seg_dist(P, a, b):
    ab = b - a
    t = np.clip(((P - a) @ ab) / max(ab @ ab, 1e-12), 0.0, 1.0)
    return np.linalg.norm(P - (a + np.outer(t, ab)), axis=1)


def rig(lo, bones, side):
    arm_data = bpy.data.armatures.new(f"Rig_{side}")
    arm = bpy.data.objects.new(f"Rig_{side}", arm_data)
    bpy.context.scene.collection.objects.link(arm)
    set_active(arm)
    bpy.ops.object.mode_set(mode="EDIT")
    eb = {}
    order = ["hand", "forearm"] + [k for k in bones if k not in ("hand", "forearm")]
    for name in order:
        b = bones[name]
        e = arm_data.edit_bones.new(name)
        e.head = Vector(b["head"])
        e.tail = Vector(b["tail"])
        e.align_roll(Vector(b["z"]))
        eb[name] = e
    for name in order:
        par = bones[name]["parent"]
        if par:
            eb[name].parent = eb[par]
            eb[name].use_connect = False
    bpy.ops.object.mode_set(mode="OBJECT")
    # Gewichte: Abstand zur Segment-Oberfläche, weich über die nächsten Segmente verteilt
    V = np.array([v.co[:] for v in lo.data.vertices])
    names = list(bones)
    D = np.stack([_seg_dist(V, np.asarray(bones[n]["head"]), np.asarray(bones[n]["tail"])) - bones[n]["r"] for n in names], axis=1)
    # Polster/Knöchel gehören zum Grundglied bzw. zur Hand; weicher Übergang an Gelenken
    dmin = D.min(axis=1, keepdims=True)
    Wt = np.exp(-np.maximum(D - dmin, 0.0) / 0.0032)
    top = np.argsort(-Wt, axis=1)[:, :3]
    groups = {n: lo.vertex_groups.new(name=n) for n in names}
    for i in range(len(V)):
        idx = top[i]
        w = Wt[i, idx]
        w = w / w.sum()
        for j, wj in zip(idx, w):
            if wj > 0.01:
                groups[names[j]].add([i], float(wj), "REPLACE")
    mod = lo.modifiers.new("Rig", "ARMATURE")
    mod.object = arm
    lo.parent = arm
    return arm


def main():
    args = script_args()
    reset_scene()
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.use_denoising = False
    t0 = time.time()
    his, los, all_bones = {}, {}, {}
    for side in ("R", "L"):
        hi, bones = high_poly(side)
        his[side] = hi
        all_bones[side] = bones
        los[side] = low_poly(hi, side)
    log(f"Netze: {time.time() - t0:.0f}s")
    mat = material("W_Hands", tex="fparms_albedo", normal="fparms_n", rough_tex="fparms_mr") if os.path.exists(os.path.join(TEX_DIR, "fparms_albedo.png")) else material("W_Hands", color=(0.1, 0.1, 0.1, 1))
    for lo in los.values():
        lo.data.materials.clear()
        lo.data.materials.append(mat)
    unwrap([los["R"], los["L"]])
    if "nobake" not in args:
        world = bpy.data.worlds.new("BakeWorld")
        scene.world = world
        world.light_settings.distance = 0.03
        # Beide Arme in einen Atlas: Kopien zusammenfügen, links zur Seite schieben
        his["L"].location.x += 0.4
        copies = []
        for side in ("R", "L"):
            c = los[side].copy()
            c.data = los[side].data.copy()
            bpy.context.scene.collection.objects.link(c)
            if side == "L":
                c.location.x += 0.4
            copies.append(c)
        set_active(copies[0])
        copies[1].select_set(True)
        bpy.ops.object.join()
        joined = copies[0]
        bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)
        his["L"].select_set(True)
        set_active(his["L"])
        bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)
        tmp = material("_bake_tmp", color=(0.5, 0.5, 0.5, 1))
        joined.data.materials.clear()
        joined.data.materials.append(tmp)
        res = {}
        for kind, mode, samples in (("NORMAL", "normal", 4), ("AO", "normal", 32), ("EMIT", "color", 4), ("EMIT", "rough", 2)):
            m = make_material(f"_hi_{mode}", mode)
            res[(kind, mode)] = bake(joined, [his["R"], his["L"]], kind, samples, m)
        bpy.data.objects.remove(joined)
        nrm = res[("NORMAL", "normal")][..., :3]
        ao = np.clip(res[("AO", "normal")][..., 0], 0, 1)[..., None]
        col = res[("EMIT", "color")][..., :3]
        rgh = res[("EMIT", "rough")]
        save_png("fparms_albedo", col * (0.35 + 0.65 * ao), srgb=True)
        save_png("fparms_n", nrm)
        mr = np.zeros((SIZE, SIZE, 3), np.float32)
        mr[..., 0] = ao[..., 0]
        mr[..., 1] = np.clip(rgh[..., 0], 0.05, 1)
        mr[..., 2] = np.clip(rgh[..., 1], 0, 1)
        save_png("fparms_mr", mr)
        # Material mit den neuen Texturen neu aufbauen
        bpy.data.materials.remove(mat)
        from lib import materials as libmat

        libmat._image_cache.clear()
        for img in list(bpy.data.images):
            if img.name.startswith("fparms"):
                bpy.data.images.remove(img)
        mat = material("W_Hands", tex="fparms_albedo", normal="fparms_n", rough_tex="fparms_mr")
        for lo in los.values():
            lo.data.materials.clear()
            lo.data.materials.append(mat)
    for side in ("R", "L"):
        bpy.data.objects.remove(his[side])
        rig(los[side], all_bones[side], side)
    os.makedirs(os.path.dirname(BLEND), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=BLEND)
    log(f"gespeichert: {BLEND} ({time.time() - t0:.0f}s)")


main()
