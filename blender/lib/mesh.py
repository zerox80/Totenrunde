"""Mesh-Builder: Geometrie aus Primitiven (Boxen, Zylinder, Kugeln, extrudierte Profile,
Rotationskörper) mit Materialslots, Box-UVs, Fasen und Vertex-Gruppen."""

import math

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

from .materials import material
from .paths import TAU

def _mat3(rot):
    if rot is None:
        return Matrix.Identity(3)
    if isinstance(rot, Quaternion):
        return rot.to_matrix()
    if isinstance(rot, Matrix):
        return rot.to_3x3()
    return Euler(tuple(math.radians(a) for a in rot), "XYZ").to_matrix()


class MB:
    """Mesh-Builder: sammelt Geometrie in einem bmesh mit Material-Slots.

    uv_mode: 'box' = Box-Projektion in Objekt-/Weltkoordinaten (uv_scale Meter pro Kachel).
    """

    def __init__(self, name, uv_scale=1.0):
        self.name = name
        self.bm = bmesh.new()
        self.uv_layer = self.bm.loops.layers.uv.verify()
        self.mats = []
        self.uv_scale = uv_scale
        self.groups = {}  # Vertex-Gruppen: name -> Liste von BMVerts

    def mi(self, mat):
        if mat is None:
            mat = material("Default")
        if mat.name not in [m.name for m in self.mats]:
            self.mats.append(mat)
        return [m.name for m in self.mats].index(mat.name)

    # ---------- interne Helfer ----------
    def _bm(self, bevel):
        """Gefaste Teile werden in einem eigenen bmesh gebaut und danach zusammengeführt."""
        return bmesh.new() if bevel > 0.0 else self.bm

    def _finish_part(self, bm, verts, faces, mat, uv_scale=None, bevel=0.0, group=None, uv_rot=0.0):
        idx = self.mi(mat)
        scale = uv_scale or self.uv_scale
        if bm is not self.bm:
            if bevel > 0.0:
                bmesh.ops.bevel(
                    bm,
                    geom=list(bm.verts) + list(bm.edges),
                    offset=bevel,
                    segments=1,
                    affect="EDGES",
                    profile=0.5,
                    clamp_overlap=True,
                )
            faces = list(bm.faces)
            for f in faces:
                f.material_index = idx
            self.box_uv(faces, scale, uv_rot, bm.loops.layers.uv.verify())
            n0 = len(self.bm.verts)
            me = bpy.data.meshes.new("_tmp_part")
            bm.to_mesh(me)
            bm.free()
            self.bm.from_mesh(me)
            bpy.data.meshes.remove(me)
            self.bm.verts.ensure_lookup_table()
            new_verts = list(self.bm.verts[n0:])
            if group:
                self.groups.setdefault(group, []).extend(new_verts)
            return new_verts
        for f in faces:
            f.material_index = idx
        self.box_uv(faces, scale, uv_rot)
        if group:
            self.groups.setdefault(group, []).extend(list({v for f in faces for v in f.verts}))
        return faces

    def box_uv(self, faces, scale, rot=0.0, uvl=None):
        uvl = uvl or self.uv_layer
        cr, sr = math.cos(rot), math.sin(rot)
        for f in faces:
            f.normal_update()
            n = f.normal
            ax = max(range(3), key=lambda i: abs(n[i]))
            for loop in f.loops:
                co = loop.vert.co
                if ax == 0:
                    u, v = (co.y if n.x > 0 else -co.y), co.z
                elif ax == 1:
                    u, v = (-co.x if n.y > 0 else co.x), co.z
                else:
                    u, v = co.x, (co.y if n.z > 0 else -co.y)
                u, v = u * cr - v * sr, u * sr + v * cr
                loop[uvl].uv = (u / scale, v / scale)

    # ---------- Primitive ----------
    def box(self, center, size, mat, rot=None, bevel=0.0, uv=None, group=None, uv_rot=0.0):
        m = Matrix.Translation(Vector(center)) @ _mat3(rot).to_4x4() @ Matrix.Diagonal((*size, 1.0))
        bm = self._bm(bevel)
        res = bmesh.ops.create_cube(bm, size=1.0, matrix=m)
        verts = res["verts"]
        faces = list({f for v in verts for f in v.link_faces})
        return self._finish_part(bm, verts, faces, mat, uv, bevel, group, uv_rot)

    def box_minmax(self, mn, mx, mat, **kw):
        c = [(a + b) / 2 for a, b in zip(mn, mx)]
        s = [abs(b - a) for a, b in zip(mn, mx)]
        return self.box(c, s, mat, **kw)

    def cyl(self, center, radius, depth, mat, axis="Z", seg=16, r2=None, rot=None, caps=True, uv=None, group=None, bevel=0.0):
        """Zylinder/Kegelstumpf. axis: 'X','Y','Z' oder Rotation per rot."""
        r2 = radius if r2 is None else r2
        base = Matrix.Identity(4)
        if axis == "X":
            base = Matrix.Rotation(math.radians(90), 4, "Y")
        elif axis == "Y":
            base = Matrix.Rotation(math.radians(-90), 4, "X")
        m = Matrix.Translation(Vector(center)) @ _mat3(rot).to_4x4() @ base
        bm = self._bm(bevel)
        res = bmesh.ops.create_cone(
            bm, cap_ends=caps, cap_tris=False, segments=seg, radius1=radius, radius2=r2, depth=depth, matrix=m
        )
        verts = res["verts"]
        faces = list({f for v in verts for f in v.link_faces})
        return self._finish_part(bm, verts, faces, mat, uv, bevel, group)

    def cyl_between(self, p0, p1, radius, mat, seg=12, r2=None, caps=True, uv=None, group=None):
        p0, p1 = Vector(p0), Vector(p1)
        d = p1 - p0
        length = d.length
        q = Vector((0, 0, 1)).rotation_difference(d.normalized())
        return self.cyl((p0 + p1) / 2, radius, length, mat, rot=q, seg=seg, r2=r2, caps=caps, uv=uv, group=group)

    def sphere(self, center, radius, mat, seg=16, rings=8, scale=(1, 1, 1), rot=None, uv=None, group=None):
        m = Matrix.Translation(Vector(center)) @ _mat3(rot).to_4x4() @ Matrix.Diagonal((*scale, 1.0))
        res = bmesh.ops.create_uvsphere(self.bm, u_segments=seg, v_segments=rings, radius=radius, matrix=m)
        verts = res["verts"]
        faces = list({f for v in verts for f in v.link_faces})
        return self._finish_part(self.bm, verts, faces, mat, uv, 0.0, group)

    def ico(self, center, radius, mat, subdiv=1, scale=(1, 1, 1), rot=None, uv=None, group=None, jitter=0.0, seed=0):
        import random

        m = Matrix.Translation(Vector(center)) @ _mat3(rot).to_4x4() @ Matrix.Diagonal((*scale, 1.0))
        res = bmesh.ops.create_icosphere(self.bm, subdivisions=subdiv, radius=radius, matrix=m)
        verts = res["verts"]
        if jitter > 0:
            rnd = random.Random(seed)
            c = Vector(center)
            for v in verts:
                d = v.co - c
                v.co = c + d * (1.0 + rnd.uniform(-jitter, jitter))
        faces = list({f for v in verts for f in v.link_faces})
        return self._finish_part(self.bm, verts, faces, mat, uv, 0.0, group)

    def extrude(self, pts, depth, mat, plane="YZ", offset=0.0, rot=None, center=(0, 0, 0), uv=None, bevel=0.0, group=None):
        """Extrudiert ein 2D-Polygon (Liste von (a,b)).

        plane='YZ': a=Y (vorne), b=Z (oben), Extrusion entlang X (zentriert um offset).
        plane='XY': a=X, b=Y, Extrusion entlang Z.  plane='XZ': a=X, b=Z, Extrusion entlang Y.
        """
        bm = self._bm(bevel)
        h = depth / 2.0
        m = Matrix.Translation(Vector(center)) @ _mat3(rot).to_4x4()

        def to3(a, b, d):
            if plane == "YZ":
                return m @ Vector((offset + d, a, b))
            if plane == "XY":
                return m @ Vector((a, b, offset + d))
            return m @ Vector((a, offset + d, b))

        # Orientierung so, dass Normalen nach außen zeigen
        area = 0.0
        for i in range(len(pts)):
            a0, b0 = pts[i]
            a1, b1 = pts[(i + 1) % len(pts)]
            area += a0 * b1 - a1 * b0
        p = list(pts) if area > 0 else list(reversed(pts))
        front = [bm.verts.new(to3(a, b, h)) for a, b in p]
        back = [bm.verts.new(to3(a, b, -h)) for a, b in p]
        faces = []
        if plane == "YZ":
            faces.append(bm.faces.new(front))
            faces.append(bm.faces.new(list(reversed(back))))
        else:
            faces.append(bm.faces.new(list(reversed(front)) if plane == "XZ" else front))
            faces.append(bm.faces.new(back if plane == "XZ" else list(reversed(back))))
        n = len(p)
        for i in range(n):
            j = (i + 1) % n
            quad = [front[i], back[i], back[j], front[j]] if plane != "XZ" else [front[j], back[j], back[i], front[i]]
            faces.append(bm.faces.new(quad))
        bm.normal_update()
        # Sicherstellen, dass die Normalen nach außen zeigen
        bmesh.ops.recalc_face_normals(bm, faces=faces)
        verts = front + back
        return self._finish_part(bm, verts, faces, mat, uv, bevel, group)

    def quad(self, p0, p1, p2, p3, mat, uv=None, group=None, uvs=None):
        bm = self.bm
        vs = [bm.verts.new(Vector(p)) for p in (p0, p1, p2, p3)]
        f = bm.faces.new(vs)
        faces = self._finish_part(self.bm, vs, [f], mat, uv, 0.0, group)
        if uvs:
            for loop, uvc in zip(f.loops, uvs):
                loop[self.uv_layer].uv = uvc
        return faces

    def lathe(self, profile, mat, seg=16, axis="Z", center=(0, 0, 0), rot=None, uv=None, group=None, caps=True):
        """Rotationskörper: profile = Liste (radius, höhe) von unten nach oben."""
        bm = self.bm
        base = Matrix.Identity(4)
        if axis == "X":
            base = Matrix.Rotation(math.radians(90), 4, "Y")
        elif axis == "Y":
            base = Matrix.Rotation(math.radians(-90), 4, "X")
        m = Matrix.Translation(Vector(center)) @ _mat3(rot).to_4x4() @ base
        rings = []
        for r, z in profile:
            ring = []
            for i in range(seg):
                a = TAU * i / seg
                ring.append(bm.verts.new(m @ Vector((math.cos(a) * r, math.sin(a) * r, z))))
            rings.append(ring)
        faces = []
        for k in range(len(rings) - 1):
            a, b = rings[k], rings[k + 1]
            for i in range(seg):
                j = (i + 1) % seg
                faces.append(bm.faces.new([a[i], a[j], b[j], b[i]]))
        if caps:
            if profile[0][0] > 1e-4:
                faces.append(bm.faces.new(list(reversed(rings[0]))))
            if profile[-1][0] > 1e-4:
                faces.append(bm.faces.new(rings[-1]))
        bmesh.ops.recalc_face_normals(bm, faces=faces)
        verts = [v for r in rings for v in r]
        return self._finish_part(self.bm, verts, faces, mat, uv, 0.0, group)

    def add_mesh(self, me, matrix, mat, uv=None, group=None, keep_uv=False):
        """Fügt ein fertiges Mesh (z. B. Schrift) mit Transformation und Material hinzu."""
        n0 = len(self.bm.verts)
        nf0 = len(self.bm.faces)
        tmp = me.copy()
        tmp.transform(matrix)
        self.bm.from_mesh(tmp)
        bpy.data.meshes.remove(tmp)
        self.bm.faces.ensure_lookup_table()
        self.bm.verts.ensure_lookup_table()
        faces = list(self.bm.faces[nf0:])
        idx = self.mi(mat)
        for f in faces:
            f.material_index = idx
        if not keep_uv:
            self.box_uv(faces, uv or self.uv_scale)
        if group:
            self.groups.setdefault(group, []).extend(list(self.bm.verts[n0:]))
        return faces

    def transform_last(self, faces, matrix):
        verts = list({v for f in faces for v in f.verts})
        bmesh.ops.transform(self.bm, matrix=matrix, verts=verts)

    # ---------- Abschluss ----------
    def to_object(self, collection=None, smooth_angle=35.0, name=None, merge=0.0):
        bm = self.bm
        if merge > 0:
            bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=merge)
        bm.normal_update()
        # Glätten mit Kantenwinkel (ersetzt Auto-Smooth)
        if smooth_angle is not None and smooth_angle > 0:
            lim = math.radians(smooth_angle)
            for f in bm.faces:
                f.smooth = True
            for e in bm.edges:
                if len(e.link_faces) == 2:
                    a, b = e.link_faces
                    e.smooth = a.normal.angle(b.normal, 0.0) < lim
                else:
                    e.smooth = False
        me = bpy.data.meshes.new(name or self.name)
        bm.to_mesh(me)
        for m in self.mats:
            me.materials.append(m)
        obj = bpy.data.objects.new(name or self.name, me)
        (collection or bpy.context.scene.collection).objects.link(obj)
        # Vertex-Gruppen übernehmen
        if self.groups:
            index_of = {}
            for gname, verts in self.groups.items():
                vg = obj.vertex_groups.new(name=gname)
                vg.add([v.index for v in verts if v.is_valid], 1.0, "REPLACE")
        bm.free()
        return obj
