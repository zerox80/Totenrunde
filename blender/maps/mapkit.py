"""Baukasten für Karten: statische Geometrie (nach Material zusammengefasst),
Kollisionsboxen und Gameplay-Daten, die als JSON für das Spiel exportiert werden.

Koordinaten: Es wird in Blender-Koordinaten gebaut (X Ost, Y Nord, Z oben).
Ins JSON wird in Bevy-Koordinaten geschrieben: (x, y, z)_bevy = (x, z, -y)_blender.
"""

import json
import math
import os

import bpy
from lib import MAP_DIR, MB, MODEL_DIR, export_gltf, hexcol, material
from mathutils import Matrix, Vector

# Kollisions-Flags (müssen zu src/collision.rs passen)
F_PLAYER = 1
F_ZOMBIE = 2
F_BULLET = 4
F_ALL = F_PLAYER | F_ZOMBIE | F_BULLET


def bv(p):
    """Blender-Punkt -> Bevy-Punkt (Liste)."""
    return [round(p[0], 4), round(p[2], 4), round(-p[1], 4)]


def bv_yaw(deg):
    return round(math.radians(deg), 5)


class MaterialLib:
    """Materialien der Karte mit Textur-Maßstab (Meter pro Kachel)."""

    def __init__(self):
        self.mats = {}
        self.uv = {}

    def add(self, key, uv=2.0, **kw):
        self.mats[key] = material("M_" + key, **kw)
        self.uv[key] = uv

    def __getitem__(self, key):
        return self.mats[key]


class MapBuilder:
    def __init__(self, name):
        from lib import reset_scene

        reset_scene()
        self.name = name
        self.lib = MaterialLib()
        self.builders = {}
        self.colliders = []
        self.zones = []
        self.doors = []
        self.barricades = []
        self.spawners = []
        self.wallbuys = []
        self.perks = []
        self.box_spots = []
        self.pap = None
        self.power = None
        self.lights = []
        self.fires = []
        self.decor = []  # zusätzliche Modelle (z. B. Requisiten), die das Spiel lädt
        self.objects = []  # eigenständige Blender-Objekte (Türen usw.)
        self.player_start = None
        self.nav = None
        self.extra = {}

    # ------------------------------------------------------------------ Geometrie
    def mb(self, mat_key, group="static"):
        key = (group, mat_key)
        if key not in self.builders:
            self.builders[key] = MB(f"{group}_{mat_key}", uv_scale=self.lib.uv[mat_key])
        return self.builders[key]

    def collider(self, center, size, yaw_deg=0.0, flags=F_ALL, kind="solid", door=None):
        """center/size in Blender-Koordinaten, Rotation um Z (Grad)."""
        c = {
            "c": bv(center),
            "h": [round(size[0] / 2, 4), round(size[2] / 2, 4), round(size[1] / 2, 4)],
            "yaw": bv_yaw(yaw_deg),
            "f": flags,
            "kind": kind,
        }
        if door is not None:
            c["door"] = door
        self.colliders.append(c)

    def box(self, center, size, mat, yaw=0.0, collide=True, flags=F_ALL, bevel=0.0, group="static", uv_rot=0.0, uv=None):
        b = self.mb(mat, group)
        b.box(center, size, self.lib[mat], rot=(0, 0, yaw), bevel=bevel, uv=uv or self.lib.uv[mat], uv_rot=uv_rot)
        if collide:
            self.collider(center, size, yaw, flags)

    def box_mm(self, mn, mx, mat, **kw):
        c = [(a + b) / 2 for a, b in zip(mn, mx)]
        s = [abs(b - a) for a, b in zip(mn, mx)]
        self.box(c, s, mat, **kw)

    def cyl(self, center, r, h, mat, seg=14, collide=True, flags=F_ALL, group="static", axis="Z", r2=None, rot=None):
        b = self.mb(mat, group)
        b.cyl(center, r, h, self.lib[mat], axis=axis, seg=seg, r2=r2, rot=rot, uv=self.lib.uv[mat])
        if collide:
            if axis == "Z":
                self.collider(center, (r * 1.8, r * 1.8, h), 0.0, flags)
            elif axis == "X":
                self.collider(center, (h, r * 2, r * 2), 0.0, flags)
            else:
                self.collider(center, (r * 2, h, r * 2), 0.0, flags)

    def floor(self, x0, y0, x1, y1, mat, z=0.0, t=0.1, collide=False, uv_rot=0.0):
        self.box_mm((x0, y0, z - t), (x1, y1, z), mat, collide=collide, uv_rot=uv_rot)

    def wall(self, a, b, h, mat_in, mat_out, t=0.3, openings=(), inside_left=True, z0=0.0, trim=None):
        """Wand von a nach b (2D). openings: Liste von dict(s=Mitte entlang Wand, w=Breite,
        lo=Unterkante, hi=Oberkante, kind='window'|'door'|'hole', door=id).

        inside_left: die Innenseite liegt links der Laufrichtung a->b.
        Die Wand wird aus zwei Schichten gebaut (innen/außen), damit beide Seiten
        unterschiedliche Materialien bekommen."""
        ax, ay = a
        bx, by = b
        dx, dy = bx - ax, by - ay
        length = math.hypot(dx, dy)
        ux, uy = dx / length, dy / length
        # Normale nach innen
        nx, ny = (-uy, ux) if inside_left else (uy, -ux)
        yaw = math.degrees(math.atan2(uy, ux))
        ops = sorted(openings, key=lambda o: o["s"])
        # Segmente entlang der Wand
        segs = []
        cur = 0.0
        for o in ops:
            s0, s1 = o["s"] - o["w"] / 2, o["s"] + o["w"] / 2
            if s0 > cur:
                segs.append((cur, s0, z0, z0 + h))
            if o["lo"] > 0.0:
                segs.append((s0, s1, z0, z0 + o["lo"]))
            if o["hi"] < h:
                segs.append((s0, s1, z0 + o["hi"], z0 + h))
            cur = s1
        if cur < length:
            segs.append((cur, length, z0, z0 + h))

        def place(s0, s1, zlo, zhi, off, thick, mat, collide):
            sm = (s0 + s1) / 2
            cx = ax + ux * sm + nx * off
            cy = ay + uy * sm + ny * off
            self.box(
                (cx, cy, (zlo + zhi) / 2),
                (s1 - s0, thick, zhi - zlo),
                mat,
                yaw=yaw,
                collide=collide,
            )

        for s0, s1, zlo, zhi in segs:
            if s1 - s0 < 1e-3 or zhi - zlo < 1e-3:
                continue
            # Innen- und Außenschicht (visuell), eine Kollisionsbox über die ganze Stärke
            place(s0, s1, zlo, zhi, t / 4, t / 2, mat_in, False)
            place(s0, s1, zlo, zhi, -t / 4, t / 2, mat_out, False)
            sm = (s0 + s1) / 2
            self.collider(
                (ax + ux * sm, ay + uy * sm, (zlo + zhi) / 2),
                (s1 - s0, t, zhi - zlo),
                yaw,
                F_ALL,
            )
        # Laibungen und Rahmen für Öffnungen
        for o in ops:
            s0, s1 = o["s"] - o["w"] / 2, o["s"] + o["w"] / 2
            sm = o["s"]
            cx, cy = ax + ux * sm, ay + uy * sm
            lo, hi = z0 + o["lo"], z0 + o["hi"]
            fr = trim or "wood_dark"
            ft = 0.07
            # Rahmen (links, rechts, oben, ggf. unten)
            for s in (s0 + ft / 2, s1 - ft / 2):
                self.box((ax + ux * s, ay + uy * s, (lo + hi) / 2), (ft, t + 0.04, hi - lo), fr, yaw=yaw, collide=False)
            self.box((cx, cy, hi - ft / 2), (o["w"], t + 0.04, ft), fr, yaw=yaw, collide=False)
            if o["lo"] > 0.0:
                self.box((cx, cy, lo + ft / 2), (o["w"] + 0.1, t + 0.12, ft), fr, yaw=yaw, collide=False)
            kind = o.get("kind", "hole")
            size = (o["w"], t, hi - lo)
            if kind == "window":
                # Unsichtbare Sperre: Spieler und Zombies (Zombies klettern skriptgesteuert)
                self.collider((cx, cy, (lo + hi) / 2), size, yaw, F_PLAYER | F_ZOMBIE, kind="window")
            elif kind == "door":
                self.collider((cx, cy, (lo + hi) / 2), size, yaw, F_ALL, kind="door", door=o["door"])
            elif kind == "block":
                self.collider((cx, cy, (lo + hi) / 2), size, yaw, F_PLAYER | F_ZOMBIE, kind="solid")
        return dict(ux=ux, uy=uy, nx=nx, ny=ny, yaw=yaw)

    def wall_point(self, a, b, s, inside_left=True, off=0.0):
        """Punkt auf der Wand bei Abstand s, um off nach innen verschoben; liefert (x, y, yaw_innen)."""
        ax, ay = a
        bx, by = b
        dx, dy = bx - ax, by - ay
        length = math.hypot(dx, dy)
        ux, uy = dx / length, dy / length
        nx, ny = (-uy, ux) if inside_left else (uy, -ux)
        x = ax + ux * s + nx * off
        y = ay + uy * s + ny * off
        yaw_in = math.degrees(math.atan2(ny, nx))  # Richtung nach innen
        return x, y, yaw_in

    # ------------------------------------------------------------------ Objekte
    def new_object(self, name, mb, origin=(0, 0, 0)):
        """Erzeugt aus einem eigenen MB ein separates (beweglich) Objekt mit Ursprung origin."""
        obj = mb.to_object(name=name)
        obj.data.transform(Matrix.Translation(-Vector(origin)))
        obj.location = origin
        self.objects.append(obj)
        return obj

    # ------------------------------------------------------------------ Gameplay
    def zone(self, zid, name, rects, open_=False):
        self.zones.append(
            {
                "id": zid,
                "name": name,
                "open": open_,
                "rects": [[r[0], -r[3], r[2], -r[1]] for r in rects],  # Bevy x0,z0,x1,z1
            }
        )

    def add_light(self, pos, color, intensity, rng, power=False, shadows=False, flicker=0.0, off_without_power=False):
        self.lights.append(
            {
                "pos": bv(pos),
                "color": list(color),
                "intensity": intensity,
                "range": rng,
                "power": power,
                "shadows": shadows,
                "flicker": flicker,
                "needs_power": off_without_power,
            }
        )

    # ------------------------------------------------------------------ Export
    def finish(self, extra=None):
        objs = []
        for (group, mat_key), b in self.builders.items():
            obj = b.to_object(smooth_angle=30.0)
            objs.append(obj)
        model_path = os.path.join(MODEL_DIR, self.name + ".gltf")
        export_gltf(model_path)
        data = {
            "name": self.name,
            "model": "models/" + self.name + ".gltf",
            "nav": self.nav,
            "player_start": self.player_start,
            "colliders": self.colliders,
            "zones": self.zones,
            "doors": self.doors,
            "barricades": self.barricades,
            "spawners": self.spawners,
            "wallbuys": self.wallbuys,
            "perks": self.perks,
            "box_spots": self.box_spots,
            "pap": self.pap,
            "power": self.power,
            "lights": self.lights,
            "fires": self.fires,
            "decor": self.decor,
        }
        if extra:
            data.update(extra)
        os.makedirs(MAP_DIR, exist_ok=True)
        with open(os.path.join(MAP_DIR, self.name + ".json"), "w") as f:
            json.dump(data, f, indent=1)
        print("KARTE:", self.name, "Kollider:", len(self.colliders), "Objekte:", len(objs) + len(self.objects))
