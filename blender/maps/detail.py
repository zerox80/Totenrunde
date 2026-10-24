"""Detail-Bausteine für glaubwürdige Räume: abgehängte Decken mit Rasterleuchten, Rohre,
Wartebänke, Schilder mit Schrift, Decals (Blut, Dreck, Papier), Bordsteine, Straßenlaternen,
Freileitungen und Gebäudesilhouetten am Horizont.

Alle Funktionen arbeiten auf der aktiven Karte (siehe `use`) in Blender-Koordinaten.
"""

import math
import os
import random

import bmesh
import bpy
from mathutils import Matrix, Vector

from lib import ASSETS

from .mapkit import F_ALL, F_PLAYER, F_ZOMBIE

M = None
L = None
rnd = random.Random(4242)

# Felder im Decal-Atlas (4×4), siehe textures/pbr.py: DECAL_CELLS
DECAL = {
    "blood_splat": 0, "blood_spray": 1, "blood_drips": 2, "blood_pool": 3,
    "grime": 4, "leak": 5, "debris": 6, "oil": 7,
    "paper": 8, "newspaper": 9, "poster_missing": 10, "poster_ad": 11,
    "cracks": 12, "bullets": 13, "graffiti": 14, "footprints": 15,
}

FONT = os.path.join(ASSETS, "fonts", "Montserrat-ExtraBold.otf")


def use(builder):
    global M, L
    M = builder
    L = builder.lib


def rot2(x, y, yaw):
    a = math.radians(yaw)
    return x * math.cos(a) - y * math.sin(a), x * math.sin(a) + y * math.cos(a)


def lbox(ox, oy, yaw, lx, ly, lz, sx, sy, sz, mat, bevel=0.0, group="static", rot=None):
    """Box in lokalen Koordinaten (Ursprung ox, oy; Drehung yaw um Z)."""
    dx, dy = rot2(lx, ly, yaw)
    b = M.mb(mat, group)
    r = (0, 0, yaw) if rot is None else (rot[0], rot[1], yaw + rot[2])
    b.box((ox + dx, oy + dy, lz), (sx, sy, sz), L[mat], rot=r, bevel=bevel, uv=L.uv[mat])


# --------------------------------------------------------------------------------------
# Decals
# --------------------------------------------------------------------------------------


def decal(center, normal, size, cell, spin=0.0, up=(0, 0, 1), off=0.004, group="decal"):
    """Rechteckiges Decal (size = (breite, höhe)) auf einer Fläche mit Normale `normal`.

    cell: Name oder Index im Atlas. spin dreht das Decal in der Fläche (Grad)."""
    idx = DECAL[cell] if isinstance(cell, str) else cell
    n = Vector(normal).normalized()
    u = Vector(up)
    if abs(n.dot(u)) > 0.95:
        u = Vector((0, 1, 0))
    right = u.cross(n).normalized()
    upv = n.cross(right).normalized()
    a = math.radians(spin)
    r2 = right * math.cos(a) + upv * math.sin(a)
    u2 = -right * math.sin(a) + upv * math.cos(a)
    c = Vector(center) + n * off
    hw, hh = size[0] / 2, size[1] / 2
    cy, cx = divmod(idx, 4)
    # UV-Feld (v nach oben; Bildzeile 0 = oben)
    u0, u1 = cx / 4 + 0.002, (cx + 1) / 4 - 0.002
    v1, v0 = 1 - cy / 4 - 0.002, 1 - (cy + 1) / 4 + 0.002
    pts = [c - r2 * hw - u2 * hh, c + r2 * hw - u2 * hh, c + r2 * hw + u2 * hh, c - r2 * hw + u2 * hh]
    b = M.mb("decal", group)
    b.quad(*pts, L["decal"], uvs=[(u0, v0), (u1, v0), (u1, v1), (u0, v1)])


def floor_decal(x, y, size, cell, z=0.0, spin=None):
    decal((x, y, z), (0, 0, 1), (size, size), cell, spin=rnd.uniform(0, 360) if spin is None else spin, off=0.006)


def wall_decal(x, y, z, inward_yaw, size, cell, spin=0.0):
    """Decal an einer Wand; inward_yaw = Richtung in den Raum (Grad, 0 = +X)."""
    a = math.radians(inward_yaw)
    decal((x, y, z), (math.cos(a), math.sin(a), 0), size if isinstance(size, tuple) else (size, size), cell, spin=spin)


def scatter_floor(x0, y0, x1, y1, z, count, cells, size=(0.3, 0.8)):
    for _ in range(count):
        floor_decal(rnd.uniform(x0, x1), rnd.uniform(y0, y1), rnd.uniform(*size), rnd.choice(cells), z=z)


# --------------------------------------------------------------------------------------
# Schrift
# --------------------------------------------------------------------------------------


def text_mesh(text, size, extrude=0.0):
    cu = bpy.data.curves.new("txt", "FONT")
    cu.body = text
    cu.size = size
    cu.extrude = extrude
    cu.align_x = "CENTER"
    cu.align_y = "CENTER"
    try:
        cu.font = bpy.data.fonts.load(FONT, check_existing=True)
    except Exception:
        pass
    ob = bpy.data.objects.new("txt", cu)
    bpy.context.scene.collection.objects.link(ob)
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
    bpy.data.objects.remove(ob)
    bpy.data.curves.remove(cu)
    return me


def text(txt, center, face_yaw, size, mat, extrude=0.004, group="nolm"):
    """Schrift, die in Richtung face_yaw (Grad, 0 = +X) zeigt; aufrecht."""
    me = text_mesh(txt, size, extrude)
    a = math.radians(face_yaw)
    d = Vector((math.cos(a), math.sin(a), 0))
    up = Vector((0, 0, 1))
    right = (-d).cross(up)
    m = Matrix((
        (right.x, up.x, d.x, center[0]),
        (right.y, up.y, d.y, center[1]),
        (right.z, up.z, d.z, center[2]),
        (0, 0, 0, 1),
    ))
    M.mb(mat, group).add_mesh(me, m, L[mat])
    bpy.data.meshes.remove(me)


def sign(txt, center, face_yaw, w, h, bg, fg, size=None, depth=0.03, glow=False):
    """Schild: Platte + Schrift. glow: Schrift leuchtet (kein Lightmapping)."""
    x, y, z = center
    a = math.radians(face_yaw)
    lbox(x, y, face_yaw + 90, 0, 0, z, w, depth, h, bg, bevel=0.004)
    tx, ty = x + math.cos(a) * (depth / 2 + 0.002), y + math.sin(a) * (depth / 2 + 0.002)
    text(txt, (tx, ty, z), face_yaw, size or h * 0.55, fg, extrude=0.002, group="glow" if glow else "nolm")


# --------------------------------------------------------------------------------------
# Decke und Licht
# --------------------------------------------------------------------------------------


def drop_ceiling(x0, y0, x1, y1, z, missing=0.06, hanging=0.02, troffers=(), seed=0, debris_z=0.0):
    """Abgehängte Decke im 0,6-m-Raster (an Weltkoordinaten ausgerichtet).

    troffers: Liste dict(x, y, state='on'|'flicker'|'dead', kind='cold') – Rasterleuchten
    60×120 cm (Mitte auf einer Rasterlinie in y). Fehlende Platten zeigen den dunklen
    Zwischenraum, einige hängen schräg herab oder liegen am Boden."""
    r = random.Random(seed)
    cell = 0.6
    i0, i1 = math.floor(x0 / cell), math.ceil(x1 / cell)
    j0, j1 = math.floor(y0 / cell), math.ceil(y1 / cell)
    lamp_cells = set()
    for t in troffers:
        ci = math.floor(t["x"] / cell)
        cj = math.floor(t["y"] / cell)
        lamp_cells |= {(ci, cj - 1), (ci, cj)}
    b = M.mb("ceiling_tile")
    bm = b.bm
    verts = {}

    def vert(i, j):
        key = (i, j)
        if key not in verts:
            px = min(max(i * cell, x0), x1)
            py = min(max(j * cell, y0), y1)
            verts[key] = bm.verts.new((px, py, z))
        return verts[key]

    faces = []
    holes = []
    for i in range(i0, i1):
        for j in range(j0, j1):
            cx0, cx1 = max(i * cell, x0), min((i + 1) * cell, x1)
            cy0, cy1 = max(j * cell, y0), min((j + 1) * cell, y1)
            if cx1 - cx0 < 0.02 or cy1 - cy0 < 0.02:
                continue
            if (i, j) in lamp_cells:
                continue
            edge = i in (i0, i1 - 1) or j in (j0, j1 - 1)
            roll = r.random()
            if not edge and roll < missing:
                holes.append((cx0, cy0, cx1, cy1))
                continue
            if not edge and roll < missing + hanging:
                # Platte hängt an einer Kante (Scharnier bei x = cx0) schräg herab
                th = r.uniform(30, 65)
                h = (cx1 - cx0) / 2
                c = (cx0 + h * math.cos(math.radians(th)), (cy0 + cy1) / 2, z - h * math.sin(math.radians(th)))
                M.mb("ceiling_tile").box(c, (cx1 - cx0 - 0.01, cy1 - cy0 - 0.01, 0.015), L["ceiling_tile"], rot=(0, th, 0), uv=L.uv["ceiling_tile"])
                holes.append((cx0, cy0, cx1, cy1))
                continue
            f = bm.faces.new([vert(i, j), vert(i, j + 1), vert(i + 1, j + 1), vert(i + 1, j)])
            faces.append(f)
    b._finish_part(bm, [], faces, L["ceiling_tile"], L.uv["ceiling_tile"])
    # T-Profile als feine Leisten an den Rändern der Löcher (sichtbare Kanten)
    for (cx0, cy0, cx1, cy1) in holes:
        for (ax, ay, bx, by) in ((cx0, cy0, cx1, cy0), (cx0, cy1, cx1, cy1)):
            M.box(((ax + bx) / 2, ay, z + 0.01), (bx - ax, 0.024, 0.03), "metal_white", collide=False)
        for (ax, ay, bx, by) in ((cx0, cy0, cx0, cy1), (cx1, cy0, cx1, cy1)):
            M.box((ax, (ay + by) / 2, z + 0.01), (0.024, by - ay, 0.03), "metal_white", collide=False)
        # heruntergefallene Platte (zerbrochen) am Boden
        if r.random() < 0.6:
            px = (cx0 + cx1) / 2 + r.uniform(-0.5, 0.5)
            py = (cy0 + cy1) / 2 + r.uniform(-0.5, 0.5)
            for k in range(r.randint(1, 3)):
                M.box((px + r.uniform(-0.3, 0.3), py + r.uniform(-0.3, 0.3), debris_z + 0.012 + k * 0.016),
                      (r.uniform(0.2, 0.45), r.uniform(0.2, 0.45), 0.014), "ceiling_tile", yaw=r.uniform(0, 90), collide=False)
            floor_decal(px, py, 0.9, "debris", z=debris_z)
    for t in troffers:
        troffer(t["x"], t["y"], z, t.get("state", "on"), t.get("kind", "cold"), t.get("intensity", 26000.0), t.get("range", 9.0))
    return holes


def troffer(x, y, z, state="on", kind="cold", intensity=26000.0, rng=9.0):
    """Rasterleuchte 60×120 cm mit Lamellenraster."""
    M.box((x, y, z + 0.04), (0.62, 1.22, 0.08), "metal_white", collide=False)
    lit = state in ("on", "flicker")
    mat = ("lamp_cold" if kind == "cold" else "lamp_warm") if lit else "lamp_dead"
    M.box((x, y, z - 0.002), (0.56, 1.16, 0.01), mat, collide=False)
    for k in range(-2, 3):
        M.box((x, y + k * 0.23, z - 0.012), (0.56, 0.012, 0.025), "metal_white", collide=False)
    M.box((x, y, z - 0.012), (0.012, 1.16, 0.025), "metal_white", collide=False)
    if lit:
        col = (0.86, 0.93, 1.0) if kind == "cold" else (1.0, 0.8, 0.55)
        M.add_light((x, y, z - 0.25), col, intensity, rng, flicker=0.7 if state == "flicker" else 0.0,
                    shadows=state == "flicker")


def pendant(x, y, ceiling_z, drop=0.9, kind="warm", needs_power=False, intensity=22000.0, rng=8.0, flicker=0.0, shade="metal_dark"):
    """Industrielle Pendelleuchte mit Kabel und Schirm."""
    z = ceiling_z - drop
    M.cyl((x, y, (ceiling_z + z) / 2 + 0.1), 0.006, drop - 0.1, "rubber", seg=6, collide=False)
    M.cyl((x, y, z + 0.06), 0.05, 0.08, shade, seg=12, collide=False)
    M.cyl((x, y, z - 0.06), 0.06, 0.18, shade, seg=16, r2=0.24, collide=False)
    M.cyl((x, y, z - 0.13), 0.07, 0.05, "lamp_warm" if kind == "warm" else "lamp_cold", seg=12, collide=False)
    col = (1.0, 0.75, 0.48) if kind == "warm" else (0.85, 0.92, 1.0)
    M.add_light((x, y, z - 0.25), col, intensity, rng, off_without_power=needs_power, flicker=flicker)


def duct(a, b, z, w=0.6, h=0.35):
    """Lüftungskanal (verzinkt) zwischen zwei Punkten in der Höhe z."""
    ax, ay = a
    bx, by = b
    ln = math.hypot(bx - ax, by - ay)
    yaw = math.degrees(math.atan2(by - ay, bx - ax))
    M.box(((ax + bx) / 2, (ay + by) / 2, z), (ln, w, h), "metal_grey", yaw=yaw, collide=False)
    n = int(ln / 1.2)
    for i in range(n + 1):
        t = i / max(n, 1)
        px, py = ax + (bx - ax) * t, ay + (by - ay) * t
        M.box((px, py, z), (0.04, w + 0.04, h + 0.04), "metal_grey", yaw=yaw, collide=False)


def pipe(points, r=0.04, mat="metal_dark", brackets=0.0, bracket_mat="metal_dark"):
    """Rohrleitung entlang einer Punktliste (3D) mit Bögen; brackets = Abstand der Schellen."""
    b = M.mb(mat)
    for p0, p1 in zip(points[:-1], points[1:]):
        b.cyl_between(p0, p1, r, L[mat], seg=12, uv=L.uv[mat])
    for p in points[1:-1]:
        b.sphere(p, r * 1.02, L[mat], seg=12, rings=6)
    if brackets > 0:
        for p0, p1 in zip(points[:-1], points[1:]):
            d = (Vector(p1) - Vector(p0))
            n = int(d.length / brackets)
            for i in range(1, n + 1):
                c = Vector(p0) + d * (i / (n + 1))
                M.mb(bracket_mat).cyl_between(c - d.normalized() * 0.015, c + d.normalized() * 0.015, r * 1.35, L[bracket_mat], seg=10)


def conduit_run(a, b, z, wall_yaw, r=0.012):
    """Elektro-Leerrohr an der Wand mit Abzweigdosen."""
    ax, ay = a
    bx, by = b
    pipe([(ax, ay, z), (bx, by, z)], r=r, mat="metal_grey", brackets=0.8, bracket_mat="metal_grey")


# --------------------------------------------------------------------------------------
# Möbel und Ausstattung
# --------------------------------------------------------------------------------------


def beam_seating(x, y, yaw, n=4, seat_mat="seat_blue", broken=()):
    """Wartebank: Sitzschalen auf einem Träger mit zwei Füßen."""
    pitch = 0.56
    ln = n * pitch
    lbox(x, y, yaw, 0, 0.0, 0.36, ln + 0.1, 0.07, 0.05, "metal_dark", bevel=0.008)
    for lx in (-ln / 2 + 0.25, ln / 2 - 0.25):
        lbox(x, y, yaw, lx, 0, 0.18, 0.05, 0.05, 0.36, "metal_dark", bevel=0.006)
        lbox(x, y, yaw, lx, 0, 0.012, 0.08, 0.52, 0.025, "metal_dark", bevel=0.006)
    for i in range(n):
        lx = -ln / 2 + pitch * (i + 0.5)
        if i in broken:
            continue
        lbox(x, y, yaw, lx, 0.02, 0.44, 0.48, 0.44, 0.035, seat_mat, bevel=0.012)
        lbox(x, y, yaw, lx, -0.21, 0.72, 0.48, 0.035, 0.5, seat_mat, bevel=0.012, rot=(-12, 0, 0))
        lbox(x, y, yaw, lx, 0.0, 0.4, 0.06, 0.06, 0.06, "metal_dark")
        if i < n - 1:
            ax = lx + pitch / 2
            lbox(x, y, yaw, ax, 0.02, 0.62, 0.04, 0.38, 0.03, "metal_dark", bevel=0.006)
            lbox(x, y, yaw, ax, 0.18, 0.52, 0.03, 0.03, 0.2, "metal_dark")
    M.collider((x, y, 0.45), (ln + 0.1, 0.55, 0.9) if abs(math.sin(math.radians(yaw))) < 0.5 else (0.55, ln + 0.1, 0.9), 0.0, F_PLAYER | F_ZOMBIE)


def trash_bin(x, y, tipped=False):
    if tipped:
        M.cyl((x, y, 0.22), 0.22, 0.6, "metal_grey", axis="X", seg=16, collide=False)
        floor_decal(x + 0.5, y, 0.8, "debris")
        floor_decal(x + 0.7, y + 0.2, 0.4, "paper")
    else:
        M.cyl((x, y, 0.33), 0.22, 0.66, "metal_grey", seg=16, collide=False, r2=0.2)
        M.cyl((x, y, 0.67), 0.23, 0.03, "metal_dark", seg=16, collide=False)
    M.collider((x, y, 0.33), (0.45, 0.45, 0.66), 0.0, F_PLAYER | F_ZOMBIE)


def wall_clock(x, y, z, face_yaw, stopped_at=(4, 37)):
    a = math.radians(face_yaw)
    d = (math.cos(a), math.sin(a))
    M.cyl((x, y, z), 0.17, 0.05, "metal_dark", axis="X", rot=(0, 0, face_yaw), seg=24, collide=False)
    M.cyl((x + d[0] * 0.026, y + d[1] * 0.026, z), 0.15, 0.004, "paint_white", axis="X", rot=(0, 0, face_yaw), seg=24, collide=False)
    hh, mm = stopped_at
    for ang, ln, w in (((hh % 12 + mm / 60) * 30, 0.08, 0.012), (mm * 6, 0.12, 0.008)):
        r = math.radians(ang)
        lbox(x + d[0] * 0.032, y + d[1] * 0.032, face_yaw + 90, math.sin(r) * ln / 2, 0, z + math.cos(r) * ln / 2, w, 0.004, ln, "metal_dark", rot=(0, ang, 0))


def fire_extinguisher(x, y, face_yaw):
    a = math.radians(face_yaw)
    px, py = x + math.cos(a) * 0.1, y + math.sin(a) * 0.1
    M.cyl((px, py, 1.15), 0.075, 0.5, "metal_red", seg=16, collide=False)
    M.cyl((px, py, 1.43), 0.03, 0.08, "metal_dark", seg=10, collide=False)
    lbox(x, y, face_yaw + 90, 0, 0, 1.2, 0.12, 0.02, 0.2, "metal_dark")


def exit_sign(x, y, z, face_yaw):
    lbox(x, y, face_yaw + 90, 0, 0, z, 0.42, 0.06, 0.16, "metal_white", bevel=0.005)
    a = math.radians(face_yaw)
    lbox(x + math.cos(a) * 0.031, y + math.sin(a) * 0.031, face_yaw + 90, 0, 0, z, 0.38, 0.004, 0.12, "sign_exit")
    text("NOTAUSGANG", (x + math.cos(a) * 0.035, y + math.sin(a) * 0.035, z), face_yaw, 0.045, "text_white", extrude=0.001, group="glow")
    M.add_light((x + math.cos(a) * 0.3, y + math.sin(a) * 0.3, z - 0.1), (0.3, 1.0, 0.45), 900.0, 3.0)


def departure_board(x, y, z, face_yaw, lines):
    """Abfahrtstafel mit leuchtender Bernstein-Schrift."""
    lbox(x, y, face_yaw + 90, 0, 0, z, 2.6, 0.12, 1.0, "metal_dark", bevel=0.01)
    a = math.radians(face_yaw)
    fx, fy = x + math.cos(a) * 0.061, y + math.sin(a) * 0.061
    lbox(fx, fy, face_yaw + 90, 0, 0, z, 2.45, 0.004, 0.86, "screen_black")
    text("ABFAHRT", (fx + math.cos(a) * 0.002, fy + math.sin(a) * 0.002, z + 0.33), face_yaw, 0.09, "lamp_amber", extrude=0.0, group="glow")
    for i, ln in enumerate(lines):
        text(ln, (fx + math.cos(a) * 0.002, fy + math.sin(a) * 0.002, z + 0.15 - i * 0.14), face_yaw, 0.065, "lamp_amber", extrude=0.0, group="glow")
    M.add_light((x + math.cos(a) * 0.6, y + math.sin(a) * 0.6, z), (1.0, 0.6, 0.2), 1500.0, 3.5)


def broken_glass(x, y, z=0.0, r=0.6, n=14):
    for _ in range(n):
        px, py = x + rnd.uniform(-r, r), y + rnd.uniform(-r, r)
        s = rnd.uniform(0.03, 0.12)
        M.box((px, py, z + 0.004), (s, s * rnd.uniform(0.4, 1.0), 0.004), "glass", yaw=rnd.uniform(0, 180), collide=False)


def window_shards(cx, cy, z_lo, z_hi, w, yaw):
    """Glasreste im Fensterrahmen (Zacken an den Rändern)."""
    for side in (-1, 1):
        for k in range(3):
            h = rnd.uniform(0.15, 0.4)
            lx = side * (w / 2 - 0.06 - k * 0.08)
            lbox(cx, cy, yaw, lx, 0.0, z_hi - h / 2 - 0.04, 0.07, 0.006, h, "glass", rot=(0, rnd.uniform(-15, 15), 0))
    for k in range(4):
        lbox(cx, cy, yaw, rnd.uniform(-w / 3, w / 3), 0.0, z_lo + 0.08, rnd.uniform(0.08, 0.2), 0.006, rnd.uniform(0.06, 0.15), "glass", rot=(0, rnd.uniform(-30, 30), 0))


def locker_row(x, y, yaw, n=4, open_doors=()):
    w = 0.45
    for i in range(n):
        lx = (i - (n - 1) / 2) * w
        lbox(x, y, yaw, lx, 0, 0.95, w - 0.01, 0.5, 1.9, "metal_grey", bevel=0.008)
        if i in open_doors:
            lbox(x, y, yaw, lx, 0.25, 0.95, w - 0.06, 0.01, 1.8, "metal_dark")
            ox, oy = rot2(lx - w / 2 + 0.02, 0.25 + 0.2, yaw)
            lbox(x + ox, y + oy, yaw + 70, 0.2, 0, 0.95, 0.4, 0.015, 1.8, "metal_grey", bevel=0.004)
        else:
            for k in range(4):
                lbox(x, y, yaw, lx, 0.252, 1.6 + k * 0.04, w - 0.16, 0.006, 0.012, "metal_dark")
            lbox(x, y, yaw, lx + w / 2 - 0.08, 0.26, 1.05, 0.025, 0.02, 0.1, "metal_dark")
    M.collider((x, y, 0.95), (n * w, 0.5, 1.9) if abs(math.sin(math.radians(yaw))) < 0.5 else (0.5, n * w, 1.9), 0.0, F_PLAYER | F_ZOMBIE)


def vent_grille(x, y, z, face_yaw, w=0.6, h=0.3):
    lbox(x, y, face_yaw + 90, 0, 0, z, w, 0.03, h, "metal_grey", bevel=0.004)
    a = math.radians(face_yaw)
    for k in range(int(h / 0.04)):
        lbox(x + math.cos(a) * 0.016, y + math.sin(a) * 0.016, face_yaw + 90, 0, 0, z - h / 2 + 0.03 + k * 0.04, w - 0.04, 0.008, 0.012, "metal_dark", rot=(30, 0, 0))


def pallet(x, y, yaw=0.0, z=0.0):
    for k in range(5):
        lbox(x, y, yaw, 0, -0.5 + k * 0.25, z + 0.13, 1.2, 0.1, 0.025, "planks")
    for lx in (-0.55, 0, 0.55):
        lbox(x, y, yaw, lx, 0, z + 0.06, 0.1, 1.0, 0.1, "planks")


def crate(x, y, z=0.0, s=0.8, yaw=0.0, collide=True):
    M.box((x, y, z + s / 2), (s, s, s), "planks", yaw=yaw, collide=collide, bevel=0.015)
    for k in (-1, 1):
        dx, dy = rot2(k * (s / 2 + 0.006), 0, yaw)
        M.box((x + dx, y + dy, z + s / 2), (0.02, s * 0.92, 0.1), "wood_dark", yaw=yaw, collide=False)
        M.box((x + dx, y + dy, z + s / 2), (0.02, 0.1, s * 0.92), "wood_dark", yaw=yaw, collide=False)


def barrel(x, y, mat="metal_blue", tipped=False, z=0.0):
    if tipped:
        M.cyl((x, y, 0.3), 0.29, 0.88, mat, axis="X", seg=20, flags=F_PLAYER | F_ZOMBIE)
        floor_decal(x + 0.7, y, 1.0, "oil")
        return
    M.cyl((x, y, z + 0.44), 0.29, 0.88, mat, seg=20, flags=F_PLAYER | F_ZOMBIE)
    for hz in (0.02, 0.3, 0.58, 0.86):
        M.cyl((x, y, z + hz), 0.297, 0.025, mat, seg=20, collide=False)
    M.cyl((x, y, z + 0.885), 0.27, 0.01, "rust", seg=20, collide=False)


def tire(x, y, z, yaw=0.0, flat=True):
    if flat:
        M.cyl((x, y, z + 0.11), 0.34, 0.22, "rubber", seg=20, collide=False)
        M.cyl((x, y, z + 0.11), 0.2, 0.225, "metal_dark", seg=16, collide=False)
    else:
        M.cyl((x, y, z + 0.34), 0.34, 0.22, "rubber", axis="X", rot=(0, 0, yaw), seg=20, collide=False)


# --------------------------------------------------------------------------------------
# Außen
# --------------------------------------------------------------------------------------


def curb(a, b, w=0.18, h=0.14):
    ax, ay = a
    bx, by = b
    ln = math.hypot(bx - ax, by - ay)
    yaw = math.degrees(math.atan2(by - ay, bx - ax))
    M.box(((ax + bx) / 2, (ay + by) / 2, h / 2), (ln, w, h), "concrete", yaw=yaw, collide=False, bevel=0.015)


def sidewalk(x0, y0, x1, y1, z=0.14, slab=1.5):
    """Gehweg aus Betonplatten (mit Fugen durch Einzelplatten)."""
    nx = max(1, round((x1 - x0) / slab))
    ny = max(1, round((y1 - y0) / slab))
    sx = (x1 - x0) / nx
    sy = (y1 - y0) / ny
    for i in range(nx):
        for j in range(ny):
            cx = x0 + (i + 0.5) * sx
            cy = y0 + (j + 0.5) * sy
            tilt = (rnd.uniform(-0.6, 0.6), rnd.uniform(-0.6, 0.6), 0) if rnd.random() < 0.15 else (0, 0, 0)
            b = M.mb("concrete")
            b.box((cx, cy, z - 0.04), (sx - 0.012, sy - 0.012, 0.08), L["concrete"], rot=tilt, uv=L.uv["concrete"], bevel=0.008)


def street_lamp(x, y, yaw, state="on", intensity=90000.0, flicker=0.0):
    """Mastleuchte mit Ausleger und Natriumdampf-Licht (orange)."""
    M.cyl((x, y, 0.3), 0.14, 0.6, "concrete", seg=12, collide=False)
    M.cyl((x, y, 3.4), 0.08, 6.2, "metal_grey", seg=12, r2=0.055, flags=F_PLAYER | F_ZOMBIE)
    hx, hy = rot2(1.3, 0, yaw)
    b = M.mb("metal_grey")
    b.cyl_between((x, y, 6.3), (x + hx * 0.6, y + hy * 0.6, 6.7), 0.045, L["metal_grey"], seg=10)
    b.cyl_between((x + hx * 0.6, y + hy * 0.6, 6.7), (x + hx, y + hy, 6.75), 0.04, L["metal_grey"], seg=10)
    lbox(x + hx * 1.15, y + hy * 1.15, yaw, 0, 0, 6.72, 0.62, 0.26, 0.12, "metal_dark", bevel=0.03)
    lit = state in ("on", "flicker")
    lbox(x + hx * 1.15, y + hy * 1.15, yaw, 0, 0, 6.655, 0.5, 0.18, 0.012, "lamp_sodium" if lit else "glass")
    if lit:
        M.add_light((x + hx * 1.15, y + hy * 1.15, 6.4), (1.0, 0.62, 0.28), intensity, 18.0, flicker=0.6 if state == "flicker" else flicker,
                    shadows=False)


def wires(p0, p1, sag=0.6, r=0.012, seg=10):
    """Durchhängendes Kabel zwischen zwei Punkten (Kettenlinie angenähert)."""
    a, b = Vector(p0), Vector(p1)
    pts = []
    for i in range(seg + 1):
        t = i / seg
        p = a.lerp(b, t)
        p.z -= sag * 4 * t * (1 - t)
        pts.append(tuple(p))
    bb = M.mb("rubber", "nolm")
    for q0, q1 in zip(pts[:-1], pts[1:]):
        bb.cyl_between(q0, q1, r, L["rubber"], seg=5, caps=False)


def silhouette_building(x, y, w, d, h, yaw=0.0, lit_windows=0.04, seed=0):
    """Gebäudeklotz am Horizont mit Fensterraster (wenige erleuchtet)."""
    r = random.Random(seed)
    lbox(x, y, yaw, 0, 0, h / 2, w, d, h, "concrete_dark", group="far")
    floors = int(h / 3.2)
    cols = int(w / 3.0)
    a = math.radians(yaw)
    fx, fy = -math.sin(a), math.cos(a)  # Vorderseite (+Y lokal)
    for f in range(1, floors):
        for c in range(cols):
            lx = -w / 2 + (c + 0.5) * w / cols
            z = f * 3.2 + 0.4
            mat = "window_lit" if r.random() < lit_windows else "window_dark"
            lbox(x, y, yaw, lx, d / 2 + 0.02, z, 1.4, 0.04, 1.5, mat, group="far")


def dead_tree(x, y, h=6.0, seed=0):
    r = random.Random(seed)
    b = M.mb("bark")
    b.cyl((x, y, h * 0.35), 0.25, h * 0.7, L["bark"], r2=0.12, seg=10)
    b.cyl((x, y, h * 0.85), 0.12, h * 0.3, L["bark"], r2=0.03, seg=8)

    def branch(p, d, ln, rad, depth):
        q = (p[0] + d[0] * ln, p[1] + d[1] * ln, p[2] + d[2] * ln)
        b.cyl_between(p, q, rad, L["bark"], seg=6, r2=rad * 0.6)
        if depth > 0:
            for _ in range(2):
                nd = Vector((d[0] + r.uniform(-0.7, 0.7), d[1] + r.uniform(-0.7, 0.7), d[2] + r.uniform(-0.1, 0.5))).normalized()
                branch(q, nd, ln * r.uniform(0.5, 0.75), rad * 0.6, depth - 1)

    for i in range(6):
        z = h * r.uniform(0.35, 0.8)
        ang = r.uniform(0, math.tau)
        d = Vector((math.cos(ang), math.sin(ang), r.uniform(0.3, 0.9))).normalized()
        branch((x, y, z), d, r.uniform(1.2, 2.2), 0.08, 2)
    M.collider((x, y, h / 2), (0.55, 0.55, h), 0.0, F_PLAYER | F_ZOMBIE)
