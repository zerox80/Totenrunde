"""Zombie-Spielmodell aus der eigenen Skulptur (keine fremden Assets).

Ablauf:
  1. Skulptur (zombie_sculpt) -> Hochpoly per Surface Nets (Körper 2 mm, Kopf 1 mm)
     mit Masken je Material, Blut- und Schmutzfeldern als Vertex-Attribute
  2. Spielmodell: Hochpoly dezimiert (~22k Dreiecke), Kopf und Hände mit mehr Texeln
  3. Backen (Cycles): Normalen, AO, Farbe, Rauheit/Metall -> eine 2048er-Textur je Figur
  4. Materialien: Haut, Hemd, Hose, Ausrüstung (teilen sich die Textur; Hemd und Hose
     werden im Spiel eingefärbt)
  5. Binden: Bone-Heat, Kopf abtrennen (platzt beim Kopfschuss), Augen und Halsstumpf
"""

import math
import os
import time

import bmesh
import bpy
import numpy as np
from mathutils import Vector

from lib import MB, TEX_DIR, hexcol, material, set_active
from sculpt import mesher
from sculpt.noise import fbm, perlin

from . import zombie_sculpt
from .zombie_rig import bones_from_joints
from .zombie_shading import TAGS, make_material

C = hexcol

# Material-Gruppen des Spielmodells
GROUPS = {
    "skin": 0, "flesh": 0, "bone": 0, "teeth": 0, "mouth": 0, "nail": 0, "hair": 0,
    "shirt": 1,
    "pants": 2,
    "shoes": 3, "sole": 3, "belt": 3, "buckle": 3,
}
MAT_NAMES = ["Z_Skin", "Z_Shirt", "Z_Pants", "Z_Gear"]


def log(*a):
    print("[zombie]", *a, flush=True)


def _cached_nets(sc, key, lo, hi, h):
    """Surface Nets mit Cache in tools/cache (Schlüssel: Quelltext der Skulptur + Parameter)."""
    import hashlib

    from lib import ROOT

    src = hashlib.sha1()
    for d in ("blender/sculpt", "blender/characters"):
        for f in sorted(os.listdir(os.path.join(ROOT, d))):
            if f.endswith(".py") and f not in ("zombie_body.py", "zombie.py", "zombie_shading.py", "zombie_anims.py"):
                with open(os.path.join(ROOT, d, f), "rb") as fh:
                    src.update(fh.read())
    src.update(repr((key, tuple(np.round(lo, 4)), tuple(np.round(hi, 4)), h)).encode())
    path = os.path.join(ROOT, "tools", "cache", f"nets_{src.hexdigest()[:16]}.npz")
    if os.path.exists(path):
        z = np.load(path)
        log("Netz aus Cache", os.path.basename(path))
        return z["v"], z["q"]
    v, q = mesher.surface_nets(sc, lo, hi, h, log=log)
    v, q = mesher.compact(v, q)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez(path, v=v, q=q)
    return v, q


# --------------------------------------------------------------------------------------
# Felder für Blut und Schmutz (je Hochpoly-Vertex)
# --------------------------------------------------------------------------------------


def fields(v, sk, info):
    n = len(v)
    blood = np.zeros(n)
    streak = fbm(v, (90.0, 90.0, 6.0), 3, seed=21.0)  # senkrechte Rinnsale
    splat = fbm(v, 14.0, 4, seed=23.0)

    def drip(c, r, length):
        dz = c[2] - v[:, 2]
        hd = np.linalg.norm(v[:, :2] - c[:2], axis=1)
        m = (dz > 0) * np.exp(-((hd / (r * 0.7)) ** 2)) * np.exp(-dz / length)
        return m * np.clip(streak * 1.6 + 0.3, 0, 1)

    for c, r in info["wounds"]:
        d = np.linalg.norm(v - c, axis=1)
        blood += np.exp(-((d / (r * 0.85)) ** 2)) * np.clip(0.7 + splat, 0, 1)
        blood += drip(c, r, 0.18)
    # Mund: Blut am Kinn, über Hals und Brust gelaufen
    hc, hs = sk["hc"], sk["hs"]
    mouth = hc + np.array([0.0, 0.095, -0.104]) * hs
    dm = np.linalg.norm(v - mouth, axis=1)
    blood += 0.55 * np.exp(-((dm / 0.018) ** 2)) * np.clip(splat + 0.6, 0, 1)
    front = np.clip((v[:, 1] - 0.02) / 0.05, 0, 1)
    blood += drip(mouth + np.array([0.0, 0.0, -0.01]), 0.04, 0.3) * front * 0.9
    # Hände bis zum Handgelenk blutig (vom Kratzen und Fressen)
    for d in sk["side"].values():
        hand_c = d["W"] + d["ha"] * 0.08
        dh = np.linalg.norm(v - hand_c, axis=1)
        blood += 0.8 * np.exp(-((dh / 0.1) ** 2)) * np.clip(splat + 0.4, 0, 1)
    blood = np.clip(blood * np.clip(0.75 + 0.5 * perlin(v, 40.0, 25.0), 0, 1.2), 0, 1)
    # Schmutz: Stiefel und Hosenbeine unten, Knie, Hände
    z = v[:, 2]
    dirt = np.clip((0.45 - z) / 0.4, 0, 1) * 0.8
    for d in sk["side"].values():
        dk = np.linalg.norm(v - (d["K"] + np.array([0, 0.05, 0])), axis=1)
        dirt += 0.7 * np.exp(-((dk / 0.07) ** 2))
        dh = np.linalg.norm(v - (d["W"] + d["ha"] * 0.1), axis=1)
        dirt += 0.5 * np.exp(-((dh / 0.08) ** 2))
    dirt = np.clip(dirt * np.clip(0.5 + fbm(v, 9.0, 4, seed=27.0), 0, 1.3), 0, 1)
    return blood.astype(np.float32), dirt.astype(np.float32)


def annotate(obj, v, sc, sk, info):
    """Masken je Etikett (one-hot) und Felder als Vertex-Attribute setzen."""
    _, tags = sc.eval_chunked(v, tags=True)
    for t in TAGS:
        if t in sc.tag_names:
            mask = (tags == sc.tag_names.index(t)).astype(np.float32)
        else:
            mask = np.zeros(len(v), dtype=np.float32)
        mesher.set_attr(obj, "t_" + t, mask)
    blood, dirt = fields(v, sk, info)
    mesher.set_attr(obj, "blood", blood)
    mesher.set_attr(obj, "dirt", dirt)


# --------------------------------------------------------------------------------------
# Spielmodell (Low-Poly)
# --------------------------------------------------------------------------------------


def make_lowpoly(hi, sk, target_tris):
    me = hi.data.copy()
    lo = bpy.data.objects.new("ZombieBody", me)
    bpy.context.scene.collection.objects.link(lo)
    ntri = sum(len(p.vertices) - 2 for p in me.polygons)
    mod = lo.modifiers.new("Dec", "DECIMATE")
    mod.decimate_type = "COLLAPSE"
    mod.ratio = min(1.0, target_tris / ntri)
    mod.use_collapse_triangulate = True
    t0 = time.time()
    dg = bpy.context.evaluated_depsgraph_get()
    me2 = bpy.data.meshes.new_from_object(lo.evaluated_get(dg))
    lo.modifiers.clear()
    old = lo.data
    lo.data = me2
    me2.name = "ZombieBody"
    bpy.data.meshes.remove(old)
    # entartete Dreiecke (kollineare Punkte) entfernen – sie bekämen riesige UV-Inseln
    bm = bmesh.new()
    bm.from_mesh(me2)
    bmesh.ops.dissolve_degenerate(bm, dist=1e-5, edges=bm.edges)
    bad = [f for f in bm.faces if f.calc_area() < 1e-9]
    bmesh.ops.delete(bm, geom=bad, context="FACES")
    bmesh.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me2)
    bm.free()
    me2.shade_smooth()
    log(f"Low-Poly: {ntri} -> {len(me2.polygons)} Dreiecke in {time.time() - t0:.1f} s ({len(bad)} entartete entfernt)")
    return lo


def _seg(p, a, b):
    ab = b - a
    t = np.clip(((p - a) @ ab) / max(float(ab @ ab), 1e-12), 0.0, 1.0)
    proj = a + t[:, None] * ab
    return np.linalg.norm(p - proj, axis=1), proj


def charts(lo, sk, bones, cut_z):
    """Aufteilung in Körperteil-Karten (wie beim Ausrollen von Hand): Gesicht/Hinterkopf,
    Rumpf vorn/hinten, Arme, Hände, Beine und Füße je als zwei Halbschalen."""
    me = lo.data
    cen = np.empty(len(me.polygons) * 3, dtype=np.float32)
    me.polygons.foreach_get("center", cen)
    c = cen.reshape(-1, 3).astype(np.float64)
    names = [b[0] for b in bones]
    dist = np.stack([_seg(c, np.asarray(b[1]), np.asarray(b[2]))[0] for b in bones], axis=1)
    near = np.argmin(dist, axis=1)
    out = np.zeros(len(c), dtype=np.int32)
    hc = sk["hc"]
    for i, name in enumerate(names):
        m = near == i
        if not m.any():
            continue
        p = c[m]
        base = name.split(".")[0]
        side = name.split(".")[1] if "." in name else "C"
        sid = {"L": 0, "R": 1, "C": 2}[side]
        if base in ("Head", "Neck", "Chest", "Spine", "Hips", "Shoulder"):
            out[m] = np.where(p[:, 1] > -0.008, 10, 11)
            continue
        d = sk["side"][side]
        if base in ("UpperArm", "LowerArm"):
            a, b = (d["S"], d["E"]) if base == "UpperArm" else (d["E"], d["W"])
            _, proj = _seg(p, a, b)
            fwd = d["hw"] if base == "LowerArm" else np.array([0.0, 1.0, 0.0])
            out[m] = 20 + sid * 2 + ((p - proj) @ fwd > 0)
        elif base == "Hand":
            out[m] = 30 + sid * 2 + ((p - d["W"]) @ d["hn"] > 0)
        elif base in ("UpperLeg", "LowerLeg"):
            a, b = (d["H"], d["K"]) if base == "UpperLeg" else (d["K"], d["A"])
            _, proj = _seg(p, a, b)
            out[m] = 40 + sid * 2 + (p[:, 1] - proj[:, 1] > 0)
        elif base == "Foot":
            out[m] = 50 + sid * 2 + ((p[:, 0] - d["A"][0]) * d["sg"] > 0)
        out[m] += 0
    # Kopf getrennt vom Rumpf (Schnitt am Hals), Kopf vorn/hinten
    head = c[:, 2] > cut_z
    out[head] = np.where(c[head, 1] > hc[1] - 0.004, 1, 2)
    out[~head & (out < 10) & (out > 0)] = 10
    return out


def _fix_uv_outliers(me):
    """Flächen, deren UV-Fläche im Verhältnis zur 3D-Fläche extrem groß ist, auf einen Punkt
    zusammenziehen (sonst belegen sie beim Packen viel Platz)."""
    bm = bmesh.from_edit_mesh(me)
    uvl = bm.loops.layers.uv.active
    ratios = []
    for f in bm.faces:
        u = [l[uvl].uv for l in f.loops]
        a = sum(abs((u[k].x - u[0].x) * (u[k + 1].y - u[0].y) - (u[k + 1].x - u[0].x) * (u[k].y - u[0].y)) / 2 for k in range(1, len(u) - 1))
        ratios.append(a / max(f.calc_area(), 1e-12))
    med = float(np.median(ratios))
    n = 0
    for f, r in zip(bm.faces, ratios):
        if r > med * 40:
            c = f.loops[0][uvl].uv.copy()
            for l in f.loops:
                l[uvl].uv = c
            n += 1
    bmesh.update_edit_mesh(me)
    if n:
        log(f"UV: {n} Ausreißer-Flächen zusammengezogen")


def unwrap(lo, sk, bones, cut_z):
    """Nähte an den Kartengrenzen, winkeltreues Ausrollen, Kopf (x1.7) und Hände (x1.35)
    mit mehr Texeln: die Geometrie wird dazu vorübergehend vergrößert."""
    me = lo.data
    cid = charts(lo, sk, bones, cut_z)
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.faces.ensure_lookup_table()
    for e in bm.edges:
        lf = e.link_faces
        e.seam = bool(len(lf) == 2 and cid[lf[0].index] != cid[lf[1].index])
    bm.to_mesh(me)
    bm.free()
    co = np.empty(len(me.vertices) * 3, dtype=np.float32)
    me.vertices.foreach_get("co", co)
    orig = co.reshape(-1, 3).astype(np.float64)
    co = orig.copy()
    hc = sk["hc"]
    s_head = 1.0 + 0.7 * np.clip((co[:, 2] - (cut_z - 0.03)) / 0.04, 0, 1)
    co = hc + (co - hc) * s_head[:, None]
    for d in sk["side"].values():
        c = d["W"] + d["ha"] * 0.08
        dist = np.linalg.norm(orig - c, axis=1)
        s_hand = 1.0 + 0.35 * np.clip((0.15 - dist) / 0.04, 0, 1)
        co = np.where((dist < 0.16)[:, None], c + (co - c) * s_hand[:, None], co)
    me.vertices.foreach_set("co", co.astype(np.float32).ravel())
    me.update()
    set_active(lo)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.unwrap(method="ANGLE_BASED", margin=0.004)
    bpy.ops.uv.select_all(action="SELECT")
    _fix_uv_outliers(me)
    try:
        bpy.ops.uv.pack_islands(rotate=True, margin=0.004, shape_method="CONCAVE")
    except TypeError:
        bpy.ops.uv.pack_islands(rotate=True, margin=0.004)
    bpy.ops.object.mode_set(mode="OBJECT")
    me.vertices.foreach_set("co", orig.astype(np.float32).ravel())
    me.update()
    log(f"UV: {len(set(cid.tolist()))} Karten")


def assign_groups(lo, sc):
    me = lo.data
    cen = np.empty(len(me.polygons) * 3, dtype=np.float32)
    me.polygons.foreach_get("center", cen)
    nrm = np.empty(len(me.polygons) * 3, dtype=np.float32)
    me.polygons.foreach_get("normal", nrm)
    # etwas nach außen versetzt auswerten (Low-Poly liegt teils innerhalb der Skulptur)
    pts = cen.reshape(-1, 3).astype(np.float64) + nrm.reshape(-1, 3) * 0.001
    _, tags = sc.eval_chunked(pts, tags=True)
    gid = np.array([GROUPS.get(sc.tag_names[t], 0) for t in tags], dtype=np.int32)
    me.polygons.foreach_set("material_index", gid)
    return gid


# --------------------------------------------------------------------------------------
# Backen
# --------------------------------------------------------------------------------------


def _bake_target(obj, size):
    img = bpy.data.images.new("_bake", size, size, alpha=False, float_buffer=True)
    img.colorspace_settings.name = "Non-Color"
    for m in obj.data.materials:
        nt = m.node_tree
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = img
        tex["_bake"] = True
        for n in nt.nodes:
            n.select = False
        tex.select = True
        nt.nodes.active = tex
    return img


def _cleanup(obj):
    for m in obj.data.materials:
        nt = m.node_tree
        for n in [n for n in nt.nodes if n.get("_bake")]:
            nt.nodes.remove(n)


def _pixels(img, size):
    px = np.empty(size * size * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    return px.reshape(size, size, 4)


def _save(path, rgb, size):
    img = bpy.data.images.new("out", size, size, alpha=False, float_buffer=False)
    img.colorspace_settings.name = "Non-Color"
    rgba = np.ones((size, size, 4), dtype=np.float32)
    rgba[:, :, :3] = np.clip(rgb, 0, 1)
    img.pixels.foreach_set(rgba.ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    bpy.data.images.remove(img)
    log("gespeichert", os.path.basename(path))


def _lin2srgb(x):
    x = np.clip(x, 0.0, 1.0)
    return np.where(x <= 0.0031308, x * 12.92, 1.055 * np.power(x, 1.0 / 2.4) - 0.055)


def bake(lo, his, kind_, size, samples, hi_mat):
    scene = bpy.context.scene
    scene.cycles.samples = samples
    for h in his:
        h.data.materials.clear()
        h.data.materials.append(hi_mat)
    img = _bake_target(lo, size)
    set_active(lo)
    for h in his:
        h.select_set(True)
    bpy.context.view_layer.objects.active = lo
    t0 = time.time()
    bpy.ops.object.bake(
        type=kind_,
        use_selected_to_active=True,
        cage_extrusion=0.007,
        max_ray_distance=0.02,
        margin=12,
        use_clear=True,
    )
    px = _pixels(img, size)
    bpy.data.images.remove(img)
    _cleanup(lo)
    log(f"Bake {kind_} in {time.time() - t0:.1f} s")
    return px


def _trim_above(obj, z):
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    kill = [f for f in bm.faces if all(v.co.z > z for v in f.verts)]
    bmesh.ops.delete(bm, geom=kill, context="FACES")
    bm.to_mesh(obj.data)
    bm.free()


# --------------------------------------------------------------------------------------
# Hauptablauf
# --------------------------------------------------------------------------------------


def build(kind="male", bake_tex=True, size=2048, tag="zombie_m", h=0.002, target_tris=22000):
    t0 = time.time()
    sk, sc, info = zombie_sculpt.sculpt(kind)
    lo_b, hi_b = sc.bounds()
    v, q = _cached_nets(sc, kind + "_body", lo_b - 0.01, hi_b + 0.01, h)
    hi = mesher.to_blender("_hi", v, q)
    log(f"Skulptur + Netz: {time.time() - t0:.1f} s")
    lo = make_lowpoly(hi, sk, target_tris)
    lo.data.materials.clear()
    tmp_mats = [material(f"_tmp_{n}", color=C("#888888")) for n in MAT_NAMES]
    for m in tmp_mats:
        lo.data.materials.append(m)
    assign_groups(lo, sc)
    bones = bones_from_joints(sk["joints"])
    cut_z = sk["neck_base"] + 0.04
    unwrap(lo, sk, bones, cut_z)

    names = dict(albedo=f"{tag}_albedo", normal=f"{tag}_n", mr=f"{tag}_mr")
    if bake_tex:
        his = [hi]
        annotate(hi, v, sc, sk, info)
        # Kopf feiner (1 mm) für Gesichtsdetails; Körpernetz dort entfernen
        hz = sk["neck_base"] + 0.035
        hv, hq = _cached_nets(sc, kind + "_head", np.array((-0.14, -0.16, hz)), np.array((0.14, 0.17, sk["head_top"] + 0.01)), h * 0.5)
        head_hi = mesher.to_blender("_hi_head", hv, hq)
        annotate(head_hi, hv, sc, sk, info)
        _trim_above(hi, hz + 0.012)
        his.append(head_hi)
        log(f"Hochpoly bereit: {time.time() - t0:.1f} s")
        scene = bpy.context.scene
        scene.render.engine = "CYCLES"
        scene.cycles.device = "CPU"
        scene.cycles.use_denoising = False
        world = bpy.data.worlds.new("BakeWorld")
        scene.world = world
        world.light_settings.distance = 0.12
        eyes = info["eyes"]
        m_n = make_material("_hi_normal", "normal", sk, eyes, kind)
        m_c = make_material("_hi_color", "color", sk, eyes, kind)
        m_r = make_material("_hi_rough", "rough", sk, eyes, kind)
        nrm = bake(lo, his, "NORMAL", size, 4, m_n)
        ao = bake(lo, [hi, head_hi], "AO", size, 32, m_n)
        col = bake(lo, his, "EMIT", size, 4, m_c)
        rgh = bake(lo, his, "EMIT", size, 2, m_r)
        aov = np.clip(ao[:, :, 0], 0.0, 1.0)[:, :, None]
        _save(os.path.join(TEX_DIR, names["albedo"] + ".png"), _lin2srgb(col[:, :, :3] * (0.42 + 0.58 * aov)), size)
        _save(os.path.join(TEX_DIR, names["normal"] + ".png"), nrm[:, :, :3], size)
        mr = np.zeros((size, size, 3), dtype=np.float32)
        mr[:, :, 0] = 1.0
        mr[:, :, 1] = np.clip(rgh[:, :, 0], 0.05, 1.0)
        mr[:, :, 2] = np.clip(rgh[:, :, 1], 0.0, 1.0)
        _save(os.path.join(TEX_DIR, names["mr"] + ".png"), mr, size)
        bpy.data.objects.remove(head_hi)
    bpy.data.objects.remove(hi)

    have = os.path.exists(os.path.join(TEX_DIR, names["albedo"] + ".png"))
    mats = {}
    for n in MAT_NAMES:
        if have:
            mats[n] = material(n, color=(1, 1, 1, 1), tex=names["albedo"], normal=names["normal"], rough_tex=names["mr"], rough=0.6)
        else:
            mats[n] = material(n, color=C("#8a8a80"), rough=0.7)
    mats["eyes"] = material("Z_Eyes", color=C("#ffcf5a"), emission=C("#ffb52a"), emission_strength=16.0, rough=0.3)
    mats["stump"] = material("Z_Blood", color=C("#4a0705"), rough=0.25)
    mats["bone"] = material("Z_Bone", color=C("#d8cdb0"), rough=0.5)
    for i, n in enumerate(MAT_NAMES):
        lo.data.materials[i] = mats[n]
    for m in tmp_mats:
        bpy.data.materials.remove(m)
    log(f"Modell fertig: {time.time() - t0:.1f} s")
    return dict(body=lo, joints=sk["joints"], bones=bones, eyes_pos=info["eyes"], eye_r=info["eye_r"], cut_z=cut_z, mats=mats)


# --------------------------------------------------------------------------------------
# Kopf abtrennen, Augen, Halsstumpf, Binden
# --------------------------------------------------------------------------------------


def split_head(body, cut_z, name="ZombieHead"):
    """Trennt alle Flächen oberhalb cut_z als eigenes Objekt ab (Gewichte bleiben)."""
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm_head = bm.copy()
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if f.calc_center_median().z > cut_z], context="FACES")
    bm.to_mesh(body.data)
    bm.free()
    bmesh.ops.delete(bm_head, geom=[f for f in bm_head.faces if f.calc_center_median().z <= cut_z], context="FACES")
    me = bpy.data.meshes.new(name)
    bm_head.to_mesh(me)
    bm_head.free()
    head = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(head)
    for g in body.vertex_groups:
        head.vertex_groups.new(name=g.name)
    for m in body.data.materials:
        me.materials.append(m)
    return head


def add_eyes_and_stump(body, eyes_pos, eye_r, cut_z, mats):
    mb = MB("ZombieEyes", uv_scale=0.1)
    for e in eyes_pos:
        mb.sphere(tuple(e), eye_r, mats["eyes"], seg=14, rings=9)
    eyes = mb.to_object(name="ZombieEyes", smooth_angle=80)
    neck_vs = [v.co for v in body.data.vertices if abs(v.co.z - cut_z) < 0.006]
    cy = sum(v.y for v in neck_vs) / max(len(neck_vs), 1)
    c = Vector((0.0, cy, cut_z - 0.004))
    rs = sorted((Vector((v.x, v.y - cy, 0))).length for v in neck_vs)
    r = rs[len(rs) // 2] * 0.92 if rs else 0.05
    sb = MB("ZombieStump", uv_scale=0.1)
    sb.cyl(c, r * 0.97, 0.012, mats["stump"], seg=16)
    sb.sphere(c + Vector((0, -0.01, 0.006)), r * 0.35, mats["bone"], seg=10, rings=6, scale=(1, 1, 0.5))
    stump = sb.to_object(name="ZombieStump", smooth_angle=60)
    return eyes, stump


def bind(parts, arm):
    body = parts["body"]
    set_active(arm)
    body.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    missing = [v.index for v in body.data.vertices if len(v.groups) == 0]
    log("Vertices ohne Gewicht:", len(missing))
    if missing:
        _fill_weights(body, missing)
    head = split_head(body, parts["cut_z"])
    head.parent = arm
    head.modifiers.new("Armature", "ARMATURE").object = arm
    parts["head"] = head
    eyes, stump = add_eyes_and_stump(body, parts["eyes_pos"], parts["eye_r"], parts["cut_z"], parts["mats"])
    parts["eyes"], parts["stump"] = eyes, stump
    for key, bone in (("eyes", "Head"), ("stump", "Neck")):
        o = parts[key]
        vg = o.vertex_groups.new(name=bone)
        vg.add(list(range(len(o.data.vertices))), 1.0, "REPLACE")
        o.parent = arm
        o.modifiers.new("Armature", "ARMATURE").object = arm


def _fill_weights(body, missing):
    """Vertices ohne Gewicht übernehmen die Gewichte des nächsten gewichteten Vertex."""
    from mathutils.kdtree import KDTree

    me = body.data
    ok = [v for v in me.vertices if len(v.groups) > 0]
    kd = KDTree(len(ok))
    for i, v in enumerate(ok):
        kd.insert(v.co, i)
    kd.balance()
    groups = body.vertex_groups
    for idx in missing:
        _, i, _ = kd.find(me.vertices[idx].co)
        if i is None:
            continue
        for g in ok[i].groups:
            groups[g.group].add([idx], g.weight, "REPLACE")
