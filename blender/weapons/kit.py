"""Baukasten für realistische Ego-Waffen.

  * Teile werden mit dem Mesh-Builder (MB) modelliert; jedes Teil bekommt ein "Finish"
    (brüniert, parkerisiert, Polymer, Holz, Messing, ...) als Material-Slot.
  * `harden(obj)`: Fasen an allen Kanten über 30° (Bevel-Modifier, gehärtete Normalen)
    und gewichtete Normalen -> saubere, glänzende Kanten wie bei echten Waffenmodellen.
  * `cut(obj, cutter)`: Boolesche Ausschnitte (Auswurffenster, Rillen, Bohrungen).
  * `bake_weapon(...)`: gemeinsamer UV-Atlas für alle Teile, Cycles backt Farbe (mit
    Kantenabrieb über den Bevel-Knoten und Schmutz in Vertiefungen über AO), Rauheit/Metall
    und Normalen (Fischhaut, Riffelung, Maserung) -> eine 2048er-Textur pro Waffe.
  * Ego-Arme aus tools/cache/fparms.blend: Handgelenk wird platziert, der Unterarm zeigt zum
    Ellbogen und die Finger schließen sich automatisch, bis sie die Waffe berühren.

Konventionen wie bisher: Lauf nach +Y, Visierlinie auf x = 0, z = 0. Objekte: Body (Waffe +
rechte Hand), Slide, Mag, ArmL (linke Hand), leeres Objekt Muzzle.
"""

import math
import os
import time

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Quaternion, Vector
from mathutils.bvhtree import BVHTree

from lib import MB, MODEL_DIR, NB, ROOT, TEX_DIR, empty, export_gltf, hexcol, material, set_active

C = hexcol
OUT = os.path.join(MODEL_DIR, "weapons")
ARMS_BLEND = os.path.join(ROOT, "tools", "cache", "fparms.blend")
UVS = 0.05


def log(*a):
    print("[waffe]", *a, flush=True)


# --------------------------------------------------------------------------------------
# Finishes (Platzhalter-Materialien beim Modellieren; beim Backen durch Shader ersetzt)
# --------------------------------------------------------------------------------------

FINISHES = ["blued", "parkerized", "polymer", "wood", "brass", "steel", "black", "rubber", "alu", "glow", "lens", "orange", "red"]


def F(name):
    """Material-Slot für ein Finish."""
    if name == "glow":
        return material("W_Glow", color=C("#7dff7a"), emission=C("#4dff4a"), emission_strength=10.0, rough=0.3)
    return material("F_" + name, color=C("#777777"))


def mb(name):
    return MB(name, UVS)


# --------------------------------------------------------------------------------------
# Hard-Surface
# --------------------------------------------------------------------------------------


def cut(obj, cutter_mb, solver="EXACT"):
    """Zieht das Mesh von cutter_mb von obj ab (Boolean, angewendet)."""
    c = cutter_mb.to_object(smooth_angle=None)
    mod = obj.modifiers.new("cut", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.object = c
    mod.solver = solver
    # Schneidkörper aus mehreren überlappenden Boxen: Selbstüberschneidung zulassen
    try:
        mod.use_self = True
        mod.use_hole_tolerant = True
    except Exception:
        pass
    set_active(obj)
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(c)


def harden(obj, width=0.0005, segments=2, angle=30.0):
    """Fasen + gewichtete Normalen (wie ein sauber modelliertes Spielmodell)."""
    set_active(obj)
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
    bm.to_mesh(obj.data)
    bm.free()
    for p in obj.data.polygons:
        p.use_smooth = True
    bev = obj.modifiers.new("bev", "BEVEL")
    bev.width = width
    bev.segments = segments
    bev.limit_method = "ANGLE"
    bev.angle_limit = math.radians(angle)
    bev.harden_normals = True
    bev.miter_outer = "MITER_ARC"
    bev.use_clamp_overlap = True
    wn = obj.modifiers.new("wn", "WEIGHTED_NORMAL")
    wn.keep_sharp = True
    wn.weight = 50
    for m in ("bev", "wn"):
        bpy.ops.object.modifier_apply(modifier=m)


def finish_part(builder, width=0.0005, angle=30.0):
    """MB -> Objekt mit Fasen."""
    obj = builder.to_object(smooth_angle=None)
    harden(obj, width=width, angle=angle)
    return obj


def join(objs, name):
    objs = [o for o in objs if o is not None]
    set_active(objs[0])
    for o in objs[1:]:
        o.select_set(True)
    bpy.ops.object.join()
    o = bpy.context.view_layer.objects.active
    o.name = name
    o.data.name = name
    return o


# --------------------------------------------------------------------------------------
# Back-Shader je Finish
# --------------------------------------------------------------------------------------


def _edge_cavity(nb, radius=0.0007):
    """Kantenmaske (konvex, 0..1) und Vertiefungsmaske (AO, 0..1)."""
    bev = nb.n("ShaderNodeBevel")
    bev.samples = 8
    bev.inputs["Radius"].default_value = radius
    geo = nb.n("ShaderNodeNewGeometry")
    d = nb.vmath("DOT_PRODUCT", bev.outputs["Normal"], geo.outputs["Normal"])
    edge = nb.smooth(d, 0.985, 0.85)
    ao = nb.n("ShaderNodeAmbientOcclusion")
    ao.samples = 8
    ao.inputs["Distance"].default_value = 0.006
    cav = nb.sub(1.0, ao.outputs["AO"])
    return edge, cav


def finish_graph(nb, fin, mode):
    """Liefert für mode 'color' eine Farbe, 'rough' (R=Rauheit, G=Metall), 'height' (Relief)."""
    p = nb.obj_coord()
    x, y, z = nb.sep(p)
    edge, cav = _edge_cavity(nb)
    n1, _ = nb.noise3(p, 60.0, detail=4)
    n2, _ = nb.noise3(p, 300.0, detail=3)
    smudge, _ = nb.noise3(p, 25.0, detail=5, rough=0.6)
    scr_n, _ = nb.noise3(nb.vmath("MULTIPLY", p, (1.0, 18.0, 1.0)), 120.0, detail=2)
    scratch = nb.mul(nb.smooth(nb.absv(nb.sub(scr_n, 0.5)), 0.02, 0.0), nb.smooth(n1, 0.55, 0.7))
    wear_n, _ = nb.noise3(p, 140.0, detail=4)
    wear = nb.clamp01(nb.mul(edge, nb.add(0.4, nb.mul(wear_n, 0.9))))
    zero = 0.0

    def out(col, rough, metal, height):
        if mode == "color":
            return col
        if mode == "rough":
            return nb.comb(rough, metal, 0.0)
        return height

    if fin in ("blued", "parkerized", "alu"):
        base = {"blued": ("#16181b", "#22262b"), "parkerized": ("#262824", "#33352f"), "alu": ("#141415", "#1d1d1f")}[fin]
        col = nb.ramp(n1, [(0.3, C(base[0])), (0.7, C(base[1]))])
        col = nb.mix(nb.mul(nb.smooth(smudge, 0.55, 0.75), 0.25), col, C("#3a3d40"))
        bright = C("#8d9094") if fin != "alu" else C("#9a9ca0")
        col = nb.mix(nb.clamp01(nb.mul(wear, 1.4)), col, bright)
        col = nb.mix(nb.mul(scratch, 0.6), col, C("#6a6e72"))
        col = nb.mix(nb.mul(cav, 0.8), col, C("#0b0b0a"))
        r0 = {"blued": 0.38, "parkerized": 0.62, "alu": 0.5}[fin]
        rough = nb.add(r0, nb.mul(nb.sub(smudge, 0.5), 0.15))
        rough = nb.mixf(wear, rough, 0.24)
        rough = nb.mixf(cav, rough, 0.85)
        metal = nb.mixf(wear, 0.85 if fin != "parkerized" else 0.6, 1.0)
        h = nb.add(nb.mul(n2, 0.05), nb.mul(scratch, -0.3))
        return out(col, rough, metal, h)
    if fin == "polymer":
        stip, _ = nb.voronoi3(p, 1600.0)
        col = nb.ramp(n1, [(0.3, C("#111113")), (0.7, C("#19191c"))])
        col = nb.mix(nb.clamp01(nb.mul(wear, 0.8)), col, C("#3b3b3f"))
        col = nb.mix(nb.mul(cav, 0.6), col, C("#060606"))
        rough = nb.add(0.72, nb.mul(stip, 0.1))
        rough = nb.mixf(wear, rough, 0.45)
        return out(col, rough, 0.0, nb.mul(stip, 0.4))
    if fin == "wood":
        grain, _ = nb.noise3(nb.vmath("MULTIPLY", p, (10.0, 1.0, 10.0)), 55.0, detail=6, rough=0.62, distortion=0.8)
        rings = nb.smooth(nb.fract(nb.mul(grain, 16.0)), 0.3, 0.95)
        pores, _ = nb.noise3(nb.vmath("MULTIPLY", p, (40.0, 1.0, 40.0)), 400.0, detail=2)
        tone, _ = nb.noise3(p, 12.0, detail=3)
        col = nb.ramp(tone, [(0.25, C("#33180a")), (0.55, C("#4e2812")), (0.8, C("#64351a"))])
        col = nb.mix(nb.mul(rings, 0.55), col, C("#1c0c04"))
        col = nb.mix(nb.mul(nb.smooth(pores, 0.6, 0.8), 0.35), col, C("#170a03"))
        col = nb.mix(nb.clamp01(nb.mul(wear, 0.9)), col, C("#7a4a28"))
        col = nb.mix(nb.mul(cav, 0.7), col, C("#120803"))
        # Fischhaut (Rautenmuster) für Griffschalen, 1,2 mm Teilung
        k = 2 * math.pi / 0.0024
        da = nb.absv(nb.sin(nb.mul(nb.add(y, z), k)))
        db = nb.absv(nb.sin(nb.mul(nb.sub(y, z), k)))
        checker = nb.minv(da, db)
        rough = nb.add(0.45, nb.mul(rings, 0.1))
        rough = nb.mixf(wear, rough, 0.35)
        return out(col, rough, 0.0, nb.add(nb.mul(rings, 0.08), nb.mul(checker, 0.0)))
    if fin == "wood_checker":
        pass
    if fin == "brass":
        col = nb.ramp(n1, [(0.3, C("#9a7432")), (0.7, C("#b88d45"))])
        col = nb.mix(nb.mul(nb.smooth(smudge, 0.5, 0.8), 0.5), col, C("#5a4220"))
        rough = nb.add(0.28, nb.mul(smudge, 0.2))
        return out(col, rough, 1.0, nb.mul(n2, 0.05))
    if fin == "steel":
        col = nb.ramp(n1, [(0.3, C("#7c7f83")), (0.7, C("#999ca0"))])
        col = nb.mix(nb.mul(cav, 0.7), col, C("#2a2b2c"))
        rough = nb.add(0.25, nb.mul(smudge, 0.15))
        return out(col, rough, 1.0, nb.mul(scratch, -0.2))
    if fin == "black":
        col = nb.ramp(n1, [(0.3, C("#0c0c0d")), (0.7, C("#151516"))])
        col = nb.mix(nb.clamp01(nb.mul(wear, 0.8)), col, C("#45474a"))
        return out(col, nb.mixf(wear, 0.6, 0.35), nb.mixf(wear, 0.3, 0.9), nb.mul(n2, 0.05))
    if fin == "rubber":
        col = nb.ramp(n1, [(0.3, C("#0e0e0e")), (0.7, C("#161616"))])
        col = nb.mix(nb.mul(cav, 0.5), col, C("#050505"))
        return out(col, 0.88, 0.0, nb.mul(n2, 0.2))
    if fin == "orange":
        col = nb.ramp(n1, [(0.3, C("#7a2f12")), (0.7, C("#94401a"))])
        col = nb.mix(nb.clamp01(nb.mul(wear, 0.8)), col, C("#3a2a20"))
        return out(col, 0.55, 0.0, nb.mul(n2, 0.05))
    if fin == "red":
        col = nb.ramp(n1, [(0.3, C("#7a1410")), (0.7, C("#951c15"))])
        col = nb.mix(nb.clamp01(nb.mul(wear, 1.2)), col, C("#a8a8a8"))
        return out(col, nb.mixf(wear, 0.4, 0.25), nb.mixf(wear, 0.2, 1.0), nb.mul(n2, 0.05))
    if fin == "lens":
        return out(C("#0a1a2a"), 0.05, 0.6, zero)
    return out(C("#555555"), 0.6, 0.0, zero)


def checker_height(nb, pitch=0.0024):
    """Fischhaut: Pyramidenraster (für Holz- und Polymergriffe)."""
    p = nb.obj_coord()
    _, y, z = nb.sep(p)
    k = 2 * math.pi / pitch
    da = nb.absv(nb.sin(nb.mul(nb.add(y, z), k)))
    db = nb.absv(nb.sin(nb.mul(nb.sub(y, z), k)))
    return nb.minv(da, db)


def _bake_material(fin, mode, extra_height=None):
    m = bpy.data.materials.new(f"_bake_{fin}_{mode}")
    try:
        m.use_nodes = True
    except Exception:
        pass
    nb = NB(m)
    outn = nb.n("ShaderNodeOutputMaterial")
    if mode in ("color", "rough"):
        em = nb.n("ShaderNodeEmission")
        nb.set(em.inputs["Color"], finish_graph(nb, fin, mode))
        nb.link(em.outputs[0], outn.inputs["Surface"])
    else:
        h = finish_graph(nb, fin, "height")
        if extra_height is not None:
            h = nb.add(h, extra_height(nb))
        bump = nb.n("ShaderNodeBump")
        bump.inputs["Strength"].default_value = 1.0
        bump.inputs["Distance"].default_value = 0.0003
        nb.set(bump.inputs["Height"], h)
        bsdf = nb.n("ShaderNodeBsdfPrincipled")
        nb.link(bump.outputs["Normal"], bsdf.inputs["Normal"])
        nb.link(bsdf.outputs[0], outn.inputs["Surface"])
    return m


def uv_atlas(objs):
    """Gemeinsamer UV-Atlas für alle Waffenteile (vor dem Anfügen der Arme aufrufen)."""
    objs = [o for o in objs if o.type == "MESH"]
    for o in objs:
        while o.data.uv_layers:
            o.data.uv_layers.remove(o.data.uv_layers[0])
        o.data.uv_layers.new(name="UVMap")
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=math.radians(55), island_margin=0.002, area_weight=0.0, correct_aspect=True, scale_to_bounds=False)
    bpy.ops.uv.select_all(action="SELECT")
    bpy.ops.uv.average_islands_scale()
    bpy.ops.uv.pack_islands(rotate=True, margin_method="FRACTION", margin=0.003, shape_method="CONCAVE")
    bpy.ops.object.mode_set(mode="OBJECT")


def bake_weapon(wid, objs, size=2048, checker=("wood_checker", "polymer_checker")):
    """Backt alle gebackenen Finishes der Objekte in einen Atlas und ersetzt die Materialien
    durch ein Material W_<wid> mit den Texturen w_<wid>_albedo/_n/_mr."""
    from lib import script_args

    if "nobake" in script_args():
        return None
    t0 = time.time()
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.use_denoising = False
    if scene.world is None:
        scene.world = bpy.data.worlds.new("BakeWorld")
    objs = [o for o in objs if o.type == "MESH"]
    # Slots merken, Back-Materialien einsetzen
    slots = []
    for o in objs:
        for i, ms in enumerate(o.material_slots):
            if ms.material and ms.material.name.startswith("F_"):
                slots.append((o, i, ms.material.name[2:]))
    results = {}
    for mode, kind, samples in (("color", "EMIT", 16), ("rough", "EMIT", 16), ("normal", "NORMAL", 8)):
        img = bpy.data.images.new("_wbake", size, size, alpha=False, float_buffer=True)
        img.colorspace_settings.name = "Non-Color"
        made = {}
        for o, i, fin in slots:
            key = fin
            if key not in made:
                base = fin.replace("_checker", "")
                extra = checker_height if fin.endswith("_checker") else None
                m = _bake_material(base, mode, extra)
                tex = m.node_tree.nodes.new("ShaderNodeTexImage")
                tex.image = img
                for n in m.node_tree.nodes:
                    n.select = False
                tex.select = True
                m.node_tree.nodes.active = tex
                made[key] = m
            o.material_slots[i].material = made[key]
        scene.cycles.samples = samples
        bpy.ops.object.select_all(action="DESELECT")
        for o in objs:
            if any(s[0] is o for s in slots):
                o.select_set(True)
        bpy.context.view_layer.objects.active = objs[0]
        bpy.ops.object.bake(type=kind, margin=8, use_clear=True)
        px = np.empty(size * size * 4, np.float32)
        img.pixels.foreach_get(px)
        results[mode] = px.reshape(size, size, 4)[::-1]
        bpy.data.images.remove(img)
        for m in made.values():
            bpy.data.materials.remove(m)
        # Platzhalter wieder einsetzen (für den nächsten Durchgang)
        for o, i, fin in slots:
            o.material_slots[i].material = F(fin)
    from textures.synth import write_png

    name = f"w_{wid}"
    write_png(os.path.join(TEX_DIR, name + "_albedo.png"), np.clip(results["color"][..., :3], 0, 1), srgb=True)
    write_png(os.path.join(TEX_DIR, name + "_n.png"), np.clip(results["normal"][..., :3], 0, 1))
    mr = np.zeros((size, size, 3), np.float32)
    mr[..., 0] = 1.0
    mr[..., 1] = np.clip(results["rough"][..., 0], 0.04, 1)
    mr[..., 2] = np.clip(results["rough"][..., 1], 0, 1)
    write_png(os.path.join(TEX_DIR, name + "_mr.png"), mr)
    wm = material("W_" + wid, tex=name + "_albedo", normal=name + "_n", rough_tex=name + "_mr")
    for o, i, fin in slots:
        o.material_slots[i].material = wm
    # doppelte Slots zusammenfassen
    for o in objs:
        _merge_slots(o)
    log(f"{wid}: Textur gebacken in {time.time() - t0:.0f}s")
    return wm


def _merge_slots(o):
    set_active(o)
    try:
        bpy.ops.object.material_slot_remove_unused()
    except Exception:
        pass
    names = [s.material.name if s.material else None for s in o.material_slots]
    first = {}
    remap = {}
    for i, n in enumerate(names):
        if n in first:
            remap[i] = first[n]
        else:
            first[n] = i
    if remap:
        for p in o.data.polygons:
            if p.material_index in remap:
                p.material_index = remap[p.material_index]
        bpy.ops.object.material_slot_remove_unused()


# --------------------------------------------------------------------------------------
# Arme posieren
# --------------------------------------------------------------------------------------

FINGERS = ("thumb", "index", "middle", "ring", "pinky")
LIMITS = {"thumb": (35, 55, 60), "index": (95, 105, 75), "middle": (95, 105, 75), "ring": (95, 105, 75), "pinky": (95, 105, 75)}


def load_arms():
    if not os.path.exists(ARMS_BLEND):
        raise RuntimeError("tools/cache/fparms.blend fehlt – erst blender/fparms/arms_build.py ausführen")
    with bpy.data.libraries.load(ARMS_BLEND, link=False) as (src, dst):
        dst.objects = [n for n in src.objects if n in ("Arm_R", "Arm_L", "Rig_R", "Rig_L")]
    for o in dst.objects:
        bpy.context.scene.collection.objects.link(o)
    return {o.name: o for o in dst.objects}


def _bvh(objs):
    dg = bpy.context.evaluated_depsgraph_get()
    verts, polys = [], []
    for o in objs:
        oe = o.evaluated_get(dg)
        me = oe.to_mesh()
        mw = o.matrix_world
        off = len(verts)
        verts.extend([mw @ v.co for v in me.vertices])
        polys.extend([[off + i for i in p.vertices] for p in me.polygons])
        oe.to_mesh_clear()
    return BVHTree.FromPolygons(verts, polys)


def _contact(bvh, pts, radii, slack=1.0):
    for p, r in zip(pts, radii):
        loc, nrm, _, dist = bvh.find_nearest(p)
        if loc is None:
            continue
        # nur Abstand: die Waffen bestehen aus überlappenden Teilen, deren innere Flächen
        # einen Innen/Außen-Test über die Normale unzuverlässig machen
        if dist < r * slack:
            return True
    return False


def pose_arm(side, arms, wrist, a, n, elbow, targets, curls=None, grip_slack=1.0, skip=()):
    """Platziert einen Arm und schließt die Finger um die Ziel-Objekte.

    wrist: Handgelenk; a: Richtung zu den Knöcheln; n: Richtung der Handfläche;
    elbow: Punkt, auf den der Unterarm zeigt. curls: feste Winkel je Finger (Grad, 3 Werte)
    statt automatischem Greifen. Liefert ein Mesh-Objekt in Waffen-Koordinaten."""
    rig = arms[f"Rig_{side}"]
    arm = arms[f"Arm_{side}"]
    a = Vector(a).normalized()
    n = (Vector(n) - a * Vector(n).dot(a)).normalized()
    w = a.cross(n) if side == "R" else n.cross(a)
    x_col = -n if side == "R" else n
    R = Matrix((x_col, a, w)).transposed()
    rig.matrix_world = Matrix.Translation(Vector(wrist)) @ R.to_4x4()
    bpy.context.view_layer.update()
    pb = rig.pose.bones
    for b in pb:
        b.rotation_mode = "QUATERNION"
        b.rotation_quaternion = Quaternion()
    # Unterarm auf den Ellbogen ausrichten
    fb = rig.data.bones["forearm"]
    rest_dir = (rig.matrix_world.to_3x3() @ (fb.tail_local - fb.head_local)).normalized()
    want = (Vector(elbow) - Vector(wrist)).normalized()
    q_world = rest_dir.rotation_difference(want)
    Mb = (rig.matrix_world.to_3x3() @ fb.matrix_local.to_3x3()).to_quaternion()
    pb["forearm"].rotation_quaternion = Mb.inverted() @ q_world @ Mb
    bvh = _bvh(targets) if targets else None
    mw = rig.matrix_world
    for f in FINGERS:
        if f in skip:
            continue
        bones = [rig.data.bones[f"{f}{j}"] for j in (1, 2, 3)]
        pts = [mw @ bones[0].head_local] + [mw @ b.tail_local for b in bones]
        axes = [(mw.to_3x3() @ b.matrix_local.to_3x3().col[0]).normalized() for b in bones]
        radii = [_bone_r(f, j) for j in range(3)]
        angles = [0.0, 0.0, 0.0]

        def rotate(j, deg):
            Rm = Matrix.Rotation(math.radians(deg), 3, axes[j])
            for k in range(j + 1, 4):
                pts[k] = pts[j] + Rm @ (pts[k] - pts[j])
            for k in range(j + 1, 3):
                axes[k] = Rm @ axes[k]

        def samples(j):
            sp, sr = [], []
            for k in range(j, 3):
                for t in (0.35, 0.7, 1.0):
                    sp.append(pts[k].lerp(pts[k + 1], t))
                    sr.append(radii[k] * (1.0 - 0.15 * t))
            return sp, sr

        if curls and f in curls:
            for j in range(3):
                rotate(j, curls[f][j])
                angles[j] = curls[f][j]
        elif bvh is not None:
            for j in range(3):
                step = 3.0
                sp, sr = samples(j)
                if _contact(bvh, sp, sr, grip_slack):
                    while angles[j] > -25 and _contact(bvh, *samples(j), grip_slack):
                        rotate(j, -step)
                        angles[j] -= step
                    continue
                while angles[j] < LIMITS[f][j]:
                    rotate(j, step)
                    if _contact(bvh, *samples(j), grip_slack):
                        rotate(j, -step)
                        break
                    angles[j] += step
        for j in range(3):
            pb[f"{f}{j + 1}"].rotation_quaternion = Quaternion((1, 0, 0), math.radians(angles[j]))
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(arm.evaluated_get(dg))
    me.transform(arm.matrix_world)
    obj = bpy.data.objects.new(f"Posed_{side}", me)
    bpy.context.scene.collection.objects.link(obj)
    return obj


_RADII = {}


def _bone_r(f, j):
    base = {"thumb": (0.0143, 0.0122, 0.0108), "index": (0.0108, 0.01, 0.0093), "middle": (0.011, 0.0103, 0.0095),
            "ring": (0.0104, 0.0097, 0.009), "pinky": (0.0093, 0.0087, 0.008)}
    return base[f][j]


def remove_arms(arms):
    for o in arms.values():
        if o.name in bpy.data.objects:
            bpy.data.objects.remove(o)


def finish(wid, muzzle):
    """Leeres Muzzle-Objekt anlegen und exportieren (bei 'nobake' nur Vorschau, kein Export)."""
    from lib import script_args

    empty("Muzzle", loc=muzzle)
    if "nobake" in script_args():
        return
    for o in list(bpy.data.objects):
        if o.type == "MESH" and o.name not in ("Body", "Slide", "Mag", "ArmL"):
            bpy.data.objects.remove(o)
    export_gltf(os.path.join(OUT, wid + ".gltf"))
