"""Karte "Endstation" – ein verlassener Busbahnhof (inspiriert vom Bus-Depot-Gefühl).

Zonen:
  0 Wartehalle (Start)   – Fenster, M14, Doppelflinte, Zweiter Atem
  1 Busbahnhof (Hof)     – Aufsteiger aus dem Boden, MP5, Turbo-Cola, Kiste
  2 Werkstatt            – Strom-Schalter, Doppelschuss, Pump-Gun
  3 Diner                – Panzer-Tonikum, AK-74u
  4 Umspannwerk          – Waffen-Verstärker, Maultier-Malz, Ausdauer-Limo
Rundlauf: Hof -> Werkstatt -> Umspannwerk -> Diner -> Hof.

Aufruf: tools/blender.sh blender/maps/endstation.py
"""

import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from lib import MB, hexcol  # noqa: E402
from maps import furniture as F  # noqa: E402
from maps.mapkit import F_ALL, F_BULLET, F_PLAYER, F_ZOMBIE, MapBuilder, bv, bv_yaw  # noqa: E402

C = hexcol
rnd = random.Random(1337)

M = MapBuilder("endstation")
L = M.lib
F.use(M)


def face(dx, dy):
    """Yaw (Grad), damit die Modell-Vorderseite (+Y) in Richtung (dx, dy) zeigt."""
    return math.degrees(math.atan2(dy, dx)) - 90.0


# ======================================================================================
# Materialien
# ======================================================================================
L.add("concrete", uv=3.0, tex="concrete", normal="concrete_n", rough=0.9)
L.add("concrete_dark", uv=3.0, tex="concrete", normal="concrete_n", rough=0.9, color=C("#8c8a86"))
L.add("asphalt", uv=5.0, tex="asphalt", normal="asphalt_n", rough=0.85)
L.add("brick", uv=3.2, tex="brick", normal="brick_n", rough=0.9)
L.add("brick_dark", uv=3.2, tex="brick", normal="brick_n", rough=0.9, color=C("#9a8f88"))
L.add("plaster", uv=2.8, tex="plaster", normal="plaster_n", rough=0.85)
L.add("plaster_cream", uv=2.8, tex="plaster", normal="plaster_n", rough=0.85, color=C("#f2e2c4"))
L.add("planks", uv=2.4, tex="planks", normal="planks_n", rough=0.8)
L.add("tiles", uv=2.4, tex="tiles", normal="tiles_n", rough=0.5)
L.add("wood_dark", uv=1.5, tex="wood_dark", normal="wood_dark_n", rough=0.7)
L.add("rust", uv=2.0, tex="rust", normal="rust_n", rough=0.9, metal=0.3)
L.add("metal_grey", uv=1.5, tex="metal_paint", normal="metal_paint_n", rough=0.6, metal=0.4, color=C("#9a9c9e"))
L.add("metal_green", uv=1.5, tex="metal_paint", normal="metal_paint_n", rough=0.6, metal=0.3, color=C("#4f6b55"))
L.add("metal_blue", uv=1.5, tex="metal_paint", normal="metal_paint_n", rough=0.6, metal=0.3, color=C("#3f5a7a"))
L.add("metal_red", uv=1.5, tex="metal_paint", normal="metal_paint_n", rough=0.6, metal=0.3, color=C("#8a2a22"))
L.add("metal_yellow", uv=1.5, tex="metal_paint", normal="metal_paint_n", rough=0.6, metal=0.3, color=C("#c79a2a"))
L.add("metal_white", uv=1.5, tex="metal_paint", normal="metal_paint_n", rough=0.5, metal=0.3)
L.add("metal_dark", uv=1.0, tex="gunmetal", normal="gunmetal_n", rough=0.5, metal=0.8)
L.add("dirt", uv=4.0, tex="dirt", normal="dirt_n", rough=0.95)
L.add("gravel", uv=3.0, tex="dirt", normal="dirt_n", rough=0.95, color=C("#a19d96"))
L.add("grass", uv=3.5, tex="grass_dead", normal="grass_dead_n", rough=0.95)
L.add("chainlink", uv=2.0, tex="chainlink", alpha_tex=True, rough=0.6, metal=0.6, double_sided=True)
L.add("fabric_red", uv=1.0, tex="fabric", normal="fabric_n", rough=0.9, color=C("#8a3030"))
L.add("fabric_blue", uv=1.0, tex="fabric", normal="fabric_n", rough=0.9, color=C("#36507a"))
L.add("rubber", uv=1.0, tex="polymer", rough=0.9)
L.add("paint_white", uv=2.0, color=C("#cfcbbf"), rough=0.8)
L.add("paint_yellow", uv=2.0, color=C("#c9a238"), rough=0.8)
L.add("glass", uv=2.0, color=C("#1d2a2c"), rough=0.08, metal=0.9)
L.add("soot", uv=2.0, tex="rust", color=C("#3a3533"), rough=1.0)
L.add("lamp_warm", uv=1.0, color=C("#fff1c9"), emission=C("#ffcf7a"), emission_strength=6.0)
L.add("lamp_cold", uv=1.0, color=C("#eef6ff"), emission=C("#d6ecff"), emission_strength=5.0)
L.add("lamp_red", uv=1.0, color=C("#ff3a2a"), emission=C("#ff2a1a"), emission_strength=8.0)
L.add("neon_pink", uv=1.0, color=C("#ff5fb0"), emission=C("#ff3d9a"), emission_strength=10.0)
L.add("neon_blue", uv=1.0, color=C("#6fd8ff"), emission=C("#3fc8ff"), emission_strength=10.0)

# ======================================================================================
# Gelände
# ======================================================================================
M.floor(-90, -90, 90, 90, "dirt", z=0.0, t=0.2)
M.floor(-20, -10, 20, 6, "asphalt", z=0.012, t=0.05)  # Hof
M.floor(-60, -34, 60, -26, "asphalt", z=0.012, t=0.05)  # Straße im Süden
for i in range(-14, 15):
    M.box((i * 4.0, -30.0, 0.017), (2.0, 0.15, 0.01), "paint_white", collide=False)
M.floor(-14, 18, 20, 33, "gravel", z=0.008, t=0.05)  # Umspannwerk
M.floor(-6, 6, 6, 20, "concrete_dark", z=0.008, t=0.05)  # Gasse
# Parkbuchten-Linien im Hof
for i in range(6):
    x = -6 + i * 3.0
    M.box((x, 3.5, 0.018), (0.12, 4.5, 0.01), "paint_white", collide=False)
M.box((1.5, -1.0, 0.018), (40.0, 0.15, 0.01), "paint_yellow", collide=False)
# Grasflächen außen
for (x0, y0, x1, y1) in ((-45, -25, -22, 40), (22, -25, 45, 40), (-45, 34, 45, 50)):
    M.floor(x0, y0, x1, y1, "grass", z=0.006, t=0.05)

# ======================================================================================
# Zone 0: Wartehalle (Start)
# ======================================================================================
H0 = 3.6
# Fußboden + Decke + Dach
M.floor(-8, -22, 8, -10, "tiles", z=0.02, t=0.05)
M.box_mm((-8.2, -22.2, H0), (8.2, -9.8, H0 + 0.25), "concrete_dark", collide=True)
M.box_mm((-8.4, -22.4, H0 + 0.25), (8.4, -9.6, H0 + 0.45), "concrete", collide=False)

# Türen: Tür 0 (Doppeltür Nord -> Hof)
DOOR0 = dict(s=8.0, w=2.4, lo=0.0, hi=2.6, kind="door", door=0)
M.wall((8, -10), (-8, -10), H0, "plaster", "brick", openings=[DOOR0], inside_left=True)
M.wall((-8, -22), (8, -22), H0, "plaster", "brick", openings=[F.window_spec(4.0), F.window_spec(12.0)], inside_left=True)
M.wall((-8, -10), (-8, -22), H0, "plaster", "brick", openings=[F.window_spec(6.0)], inside_left=True)
M.wall((8, -22), (8, -10), H0, "plaster", "brick", openings=[F.window_spec(6.0)], inside_left=True)

# Einrichtung
for yy in (-14.5, -17.5):
    F.bench(-3.0, yy, 0.0, 2.8)
    F.bench(3.0, yy, 0.0, 2.8)
# Fahrkartenschalter (Südwest-Ecke)
M.box_mm((-7.85, -21.85, 0), (-5.0, -20.6, 1.1), "wood_dark", collide=True)
M.box_mm((-7.85, -20.62, 1.1), (-5.0, -20.58, 2.4), "glass", collide=False)
M.box_mm((-5.02, -21.85, 1.1), (-4.98, -20.6, 2.4), "metal_dark", collide=False)
F.locker_row(6.9, -20.0, 90.0, 4)
F.locker_row(-7.6, -19.0, -90.0, 3)
# Abfahrtstafel
M.box_mm((-2.0, -21.84, 2.3), (2.0, -21.78, 3.1), "metal_dark", collide=False)
M.box_mm((-1.85, -21.79, 2.4), (1.85, -21.77, 3.0), "glass", collide=False)
# Mülleimer & Koffer
F.barrel(6.8, -11.0, "metal_green")
F.crate(-6.9, -11.0, s=0.7)
F.ceiling_lamp(-3.5, -16.0, H0, "cold", flicker=0.0, intensity=45000.0)
F.ceiling_lamp(3.5, -16.0, H0, "cold", flicker=0.6, intensity=45000.0)
F.ceiling_lamp(0.0, -12.0, H0, "cold", intensity=35000.0)
F.ceiling_lamp(0.0, -20.5, H0, "warm", intensity=25000.0)

# ======================================================================================
# Zone 2: Werkstatt
# ======================================================================================
H2 = 4.2
M.floor(-20, 6, -6, 20, "concrete_dark", z=0.02, t=0.05)
M.box_mm((-20.2, 5.8, H2), (-5.8, 20.2, H2 + 0.25), "metal_grey", collide=True)
M.box_mm((-20.4, 5.6, H2 + 0.25), (-5.6, 20.4, H2 + 0.4), "rust", collide=False)
# Südwand mit großem Tor (Schutt-Durchgang 1)
M.wall((-20, 6), (-6, 6), H2, "brick_dark", "brick", openings=[dict(s=7.0, w=3.2, lo=0.0, hi=3.4, kind="hole")], inside_left=True)
M.wall((-20, 20), (-20, 6), H2, "brick_dark", "brick", openings=[F.window_spec(4.0), F.window_spec(10.0)], inside_left=True)
DOOR3 = dict(s=4.0, w=3.0, lo=0.0, hi=3.0, kind="door", door=3)
M.wall((-6, 20), (-20, 20), H2, "brick_dark", "brick", openings=[DOOR3], inside_left=True)
M.wall((-6, 6), (-6, 20), H2, "brick_dark", "brick", openings=[F.window_spec(7.0)], inside_left=True)
F.car_lift(-13.0, 12.5, 0.0)
F.workbench(-18.9, 13.0, -90.0)
F.shelf(-16.3, 19.2, 180.0, 2.4, 2.2, 0.6)
F.tire_stack(-18.8, 7.2, 4)
F.tire_stack(-18.0, 7.0, 3)
F.barrel(-7.0, 7.2, "metal_red")
F.barrel(-7.7, 7.4, "metal_blue")
F.pallet(-7.3, 18.9)
F.crate(-7.3, 18.9, z=0.15, s=0.9)
F.ceiling_lamp(-13.0, 9.0, H2, "warm", needs_power=True, intensity=70000.0, rng=16.0)
F.ceiling_lamp(-13.0, 16.5, H2, "warm", needs_power=True, intensity=70000.0, rng=16.0)
F.ceiling_lamp(-7.0, 16.0, H2, "red", intensity=12000.0, rng=8.0, flicker=0.3)

# ======================================================================================
# Zone 3: Diner
# ======================================================================================
H3 = 3.4
M.floor(6, 6, 20, 18, "tiles", z=0.02, t=0.05)
M.box_mm((5.8, 5.8, H3), (20.2, 18.2, H3 + 0.22), "plaster_cream", collide=True)
M.box_mm((5.6, 5.6, H3 + 0.22), (20.4, 18.4, H3 + 0.4), "metal_white", collide=False)
DOOR2 = dict(s=4.0, w=1.6, lo=0.0, hi=2.4, kind="door", door=2)
M.wall((6, 6), (20, 6), H3, "plaster_cream", "metal_white", openings=[DOOR2, dict(s=8.5, w=2.2, lo=1.0, hi=2.4, kind="block")], inside_left=True)
M.wall((20, 6), (20, 18), H3, "plaster_cream", "metal_white", openings=[F.window_spec(4.0), F.window_spec(9.0)], inside_left=True)
DOOR4 = dict(s=12.0, w=1.4, lo=0.0, hi=2.3, kind="door", door=4)
M.wall((20, 18), (6, 18), H3, "plaster_cream", "metal_white", openings=[DOOR4], inside_left=True)
M.wall((6, 18), (6, 6), H3, "plaster_cream", "metal_white", openings=[F.window_spec(5.5)], inside_left=True)
# Diner-Fenster nach Süden hat Glas statt Barrikade -> blockiert alles
M.collider((14.5, 6.0, 1.7), (2.2, 0.3, 1.4), 0.0, F_PLAYER | F_ZOMBIE)
M.box((14.5, 6.0, 1.7), (2.2, 0.03, 1.4), "glass", collide=False)
# Tresen mit Hockern (Nordseite, parallel zur Wand)
M.box_mm((9.5, 14.6, 0), (17.5, 15.3, 1.05), "metal_white", collide=True)
M.box_mm((9.4, 14.5, 1.05), (17.6, 15.4, 1.12), "metal_red", collide=False)
M.box_mm((9.5, 16.9, 0), (17.5, 17.85, 0.95), "metal_grey", collide=True)
F.shelf(13.5, 17.6, 180.0, 3.0, 1.0, 0.35, "metal_grey")
for i in range(6):
    F.stool(10.2 + i * 1.3, 14.0)
F.booth(7.7, 7.6, 0.0)
F.booth(16.8, 8.0, 0.0)
# Jukebox
M.box_mm((19.0, 12.0, 0), (19.85, 13.0, 1.6), "metal_red", collide=True, bevel=0.03)
M.box_mm((18.95, 12.15, 0.8), (18.98, 12.85, 1.45), "neon_blue", collide=False)
F.ceiling_lamp(10.0, 10.0, H3, "warm", needs_power=True, intensity=40000.0)
F.ceiling_lamp(16.0, 10.0, H3, "warm", needs_power=True, intensity=40000.0)
F.ceiling_lamp(13.0, 16.0, H3, "warm", intensity=18000.0, flicker=0.5)
# Leuchtschild "DINER" über dem Eingang (außen)
for i in range(5):
    M.box((9.0 + i * 0.55, 5.78, 3.0), (0.4, 0.06, 0.5), "neon_pink", collide=False)
M.box((10.1, 5.82, 3.0), (3.0, 0.06, 0.7), "metal_dark", collide=False)
M.add_light((10.1, 5.0, 3.0), (1.0, 0.3, 0.6), 30000.0, 6.0, flicker=0.15)

# ======================================================================================
# Zone 1: Busbahnhof (Hof)
# ======================================================================================
# Begrenzungen
F.fence((-20, -10), (-8, -10))
F.fence((8, -10), (20, -10))
F.fence((-20, -10), (-20, 6))
F.fence((20, -10), (20, 6))
# Gasse zwischen Werkstatt und Diner: verschüttet
F.fence((-6, 6), (6, 6))
F.rubble(-3.0, 6.8, r=1.6, h=1.2, collide=False)
F.rubble(2.5, 7.5, r=1.4, h=1.0, collide=False)
# Wracks & Deko
F.bus_wreck(-7.0, -1.0, 12.0)
F.car_wreck(11.5, -6.0, 75.0)
F.car_wreck(15.5, 2.5, -20.0, color="metal_blue")
F.bus_shelter(3.5, -8.4, 0.0)
F.jersey(-15.5, 1.5, 90.0)
F.jersey(6.5, 1.0, 20.0)
F.barrel(-17.5, -8.5, "metal_red")
F.barrel(-18.2, -8.0, "metal_green", tipped=True)
F.tire_stack(17.8, -8.6, 2)
F.cone(1.0, 2.0)
F.cone(-1.5, 3.5)
F.cone(9.0, 4.2)
F.crate(-18.6, 4.6, s=0.9)
F.crate(-17.7, 4.8, s=0.7, yaw=20)
F.lamp_post(-14.0, -6.0, 0.0, flicker=0.0)
F.lamp_post(1.5, 3.8, 180.0, flicker=0.7)
F.lamp_post(16.5, -3.0, 90.0, working=False)
# Feuer im Buswrack
M.fires.append({"pos": bv((-7.0, -2.0, 1.3)), "scale": 1.2})
M.fires.append({"pos": bv((12.0, -5.8, 1.2)), "scale": 0.8})
M.add_light((-7.0, -2.0, 2.2), (1.0, 0.45, 0.15), 60000.0, 8.0, flicker=1.0)
M.add_light((12.0, -5.8, 1.9), (1.0, 0.45, 0.15), 30000.0, 6.5, flicker=1.0)

# ======================================================================================
# Zone 4: Umspannwerk
# ======================================================================================
F.fence((-14, 20), (-14, 33))
F.fence((-14, 33), (20, 33))
F.fence((20, 18), (20, 33))
F.fence((-6, 20), (6, 20))
F.fence((6, 18), (6, 20))
# Trafohaus (Pack-a-Punch-Raum)
H4 = 3.4
M.floor(-4, 27, 4, 32.8, "concrete", z=0.02, t=0.05)
M.wall((-4, 27), (4, 27), H4, "brick_dark", "brick", openings=[dict(s=4.0, w=3.0, lo=0.0, hi=2.8, kind="hole")], inside_left=True)
M.wall((-4, 32.8), (-4, 27), H4, "brick_dark", "brick", inside_left=True)
M.wall((4, 27), (4, 32.8), H4, "brick_dark", "brick", inside_left=True)
M.wall((4, 32.8), (-4, 32.8), H4, "brick_dark", "brick", inside_left=True)
M.box_mm((-4.2, 26.8, H4), (4.2, 33.0, H4 + 0.3), "concrete_dark", collide=True)
F.ceiling_lamp(0.0, 30.0, H4, "cold", needs_power=True, intensity=30000.0, rng=10.0)
F.transformer(-9.5, 26.0, 0.0)
F.transformer(10.0, 24.5, 90.0)
F.transformer(12.5, 30.0, 0.0)
F.power_pole(-11.0, 31.5)
F.power_pole(16.0, 21.0)
F.crate(7.5, 31.8, s=0.9)
F.crate(8.4, 31.9, s=0.6, yaw=15)
F.barrel(-12.8, 21.5, "metal_yellow")
F.lamp_post(-2.0, 22.5, 90.0, power=False, flicker=0.4)

# ======================================================================================
# Außenbereich (nicht begehbar, Atmosphäre)
# ======================================================================================
for i, (x, y) in enumerate(((-28, -15), (-31, 8), (-26, 25), (28, -14), (31, 6), (27, 26), (-18, 42), (10, 44), (36, -32), (-36, -30))):
    F.dead_tree(x, y, h=rnd.uniform(5.0, 8.0), seed=i)
F.car_wreck(-24.0, -4.0, 10.0)
F.car_wreck(25.0, -6.0, 100.0, color="metal_red")
F.car_wreck(-12.0, -30.0, 88.0)
F.car_wreck(14.0, -31.0, 94.0, color="metal_blue")
for x in (-40, -20, 0, 20, 40):
    F.power_pole(x, -36.0, 9.0)
F.rubble(-26.0, 18.0, r=2.0, collide=False)
F.rubble(26.0, 20.0, r=2.0, collide=False)

# ======================================================================================
# Bewegliche Objekte: Türen und Schutt
# ======================================================================================


def door_panel(name, hinge, width, height, yaw, mat="metal_green", thick=0.06):
    """Türblatt mit Ursprung am Scharnier; Blatt erstreckt sich entlang lokal +X."""
    b = MB(name, uv_scale=1.2)
    hx, hy = hinge
    b.box((width / 2, 0, height / 2), (width - 0.02, thick, height - 0.02), L[mat], bevel=0.01)
    b.box((width - 0.15, 0.05, 1.05), (0.12, 0.04, 0.04), L["metal_dark"])
    b.box((width - 0.15, -0.05, 1.05), (0.12, 0.04, 0.04), L["metal_dark"])
    b.box((width / 2, 0, height * 0.75), (width * 0.6, thick + 0.01, height * 0.3), L["glass"])
    obj = b.to_object(name=name)
    obj.location = (hx, hy, 0.0)
    obj.rotation_euler = (0, 0, math.radians(yaw))
    M.objects.append(obj)
    return obj


doors = []
# Tür 0: Doppeltür Wartehalle -> Hof (Wand y=-10, Öffnung x in [-1.2, 1.2])
door_panel("Door_0_a", (-1.2, -10.0), 1.2, 2.6, 0.0, "metal_green")
door_panel("Door_0_b", (1.2, -10.0), 1.2, 2.6, 180.0, "metal_green")
doors.append(
    {
        "id": 0,
        "cost": 750,
        "kind": "door",
        "zones": [0, 1],
        "prompt": bv((0.0, -10.0, 1.2)),
        "parts": [
            {"node": "Door_0_a", "to_pos": bv((-1.2, -10.0, 0.0)), "to_yaw": bv_yaw(95.0)},
            {"node": "Door_0_b", "to_pos": bv((1.2, -10.0, 0.0)), "to_yaw": bv_yaw(180.0 - 95.0)},
        ],
    }
)
# Schutt 1: Hof -> Werkstatt (Toröffnung bei x=-13, y=6, Breite 3.2)
deb = MB("Debris_1", uv_scale=1.5)
for i in range(14):
    px = rnd.uniform(-1.4, 1.4)
    py = rnd.uniform(-0.6, 0.6)
    s = rnd.uniform(0.3, 0.7)
    deb.ico((px, py, rnd.uniform(0.2, 1.6)), s, L["concrete_dark"], subdiv=1, scale=(1.3, 1.0, 0.7), jitter=0.3, seed=50 + i)
for i in range(6):
    deb.box((rnd.uniform(-1.2, 1.2), rnd.uniform(-0.5, 0.5), rnd.uniform(0.3, 2.4)), (rnd.uniform(1.5, 2.6), 0.15, 0.1), L["planks"], rot=(rnd.uniform(-30, 30), rnd.uniform(-20, 20), rnd.uniform(-40, 40)))
deb.box((0.3, 0.1, 0.45), (0.9, 0.9, 0.9), L["planks"], rot=(0, 0, 25))
deb.cyl((-0.9, -0.2, 0.3), 0.3, 0.9, L["metal_red"], axis="X", rot=(0, 0, 30))
obj = deb.to_object(name="Debris_1")
obj.location = (-13.0, 6.0, 0.0)
M.objects.append(obj)
M.collider((-13.0, 6.0, 1.7), (3.2, 1.6, 3.4), 0.0, F_ALL, kind="door", door=1)
doors.append(
    {
        "id": 1,
        "cost": 1000,
        "kind": "debris",
        "zones": [1, 2],
        "prompt": bv((-13.0, 5.0, 1.2)),
        "parts": [{"node": "Debris_1", "to_pos": bv((-13.0, 6.0, 6.0)), "to_yaw": 0.0}],
    }
)
# Tür 2: Diner-Eingang (Wand y=6, x in [9.2, 10.8])
door_panel("Door_2_a", (9.2, 6.0), 1.6, 2.4, 0.0, "metal_white")
doors.append(
    {
        "id": 2,
        "cost": 1000,
        "kind": "door",
        "zones": [1, 3],
        "prompt": bv((10.0, 6.0, 1.2)),
        "parts": [{"node": "Door_2_a", "to_pos": bv((9.2, 6.0, 0.0)), "to_yaw": bv_yaw(100.0)}],
    }
)
# Tür 3: Werkstatt-Rolltor nach Norden (Wand y=20, x in [-15.5, -12.5])
roll = MB("Door_3_a", uv_scale=1.0)
for k in range(10):
    roll.box((0, 0, 0.15 + k * 0.3), (3.0, 0.08, 0.28), L["metal_grey"])
roll.box((0, 0.05, 0.05), (3.0, 0.1, 0.1), L["metal_dark"])
obj = roll.to_object(name="Door_3_a")
obj.location = (-10.0, 20.0, 0.0)
M.objects.append(obj)
doors.append(
    {
        "id": 3,
        "cost": 1250,
        "kind": "door",
        "zones": [2, 4],
        "prompt": bv((-10.0, 20.0, 1.2)),
        "parts": [{"node": "Door_3_a", "to_pos": bv((-10.0, 20.0, 2.9)), "to_yaw": 0.0}],
    }
)
# Tür 4: Diner-Hintertür (Wand y=18, x in [7.3, 8.7])
door_panel("Door_4_a", (8.7, 18.0), 1.4, 2.3, 180.0, "metal_grey")
doors.append(
    {
        "id": 4,
        "cost": 1250,
        "kind": "door",
        "zones": [3, 4],
        "prompt": bv((8.0, 18.0, 1.2)),
        "parts": [{"node": "Door_4_a", "to_pos": bv((8.7, 18.0, 0.0)), "to_yaw": bv_yaw(180.0 + 100.0)}],
    }
)
M.doors = doors

# ======================================================================================
# Gameplay-Daten
# ======================================================================================
M.nav = {"min": [-34.0, -46.0], "max": [34.0, 40.0], "cell": 0.5}
M.player_start = {"pos": bv((0.0, -16.0, 0.05)), "yaw": 0.0}

M.zone(0, "Wartehalle", [(-7.85, -21.85, 7.85, -10.15)], open_=True)
M.zone(1, "Busbahnhof", [(-19.85, -9.85, 19.85, 5.85)])
M.zone(2, "Werkstatt", [(-19.85, 6.15, -6.15, 19.85)])
M.zone(3, "Diner", [(6.15, 6.15, 19.85, 17.85)])
M.zone(4, "Umspannwerk", [(-13.85, 20.15, 6.0, 32.85), (6.0, 18.15, 19.85, 32.85)])

# Barrikaden: Mittelpunkt der Öffnung, Blickrichtung nach innen
barr = []


def barricade(bid, zone, x, y, inward, sill=0.75, top=2.15, width=1.3, spawn_dist=5.5):
    ix, iy = inward
    yaw = face(ix, iy)
    barr.append(
        {
            "id": bid,
            "zone": zone,
            "pos": bv((x, y, (sill + top) / 2)),
            "yaw": bv_yaw(yaw),
            "width": width,
            "sill": sill,
            "top": top,
            "inside": bv((x + ix * 0.85, y + iy * 0.85, 0.0)),
            "outside": bv((x - ix * 0.85, y - iy * 0.85, 0.0)),
        }
    )
    M.spawners.append(
        {"zone": zone, "kind": "window", "barricade": bid, "pos": bv((x - ix * spawn_dist, y - iy * spawn_dist, 0.0))}
    )


barricade(0, 0, -4.0, -22.0, (0, 1))
barricade(1, 0, 4.0, -22.0, (0, 1))
barricade(2, 0, -8.0, -16.0, (1, 0))
barricade(3, 0, 8.0, -16.0, (-1, 0))
barricade(4, 2, -20.0, 16.0, (1, 0))
barricade(5, 2, -20.0, 10.0, (1, 0))
barricade(6, 2, -6.0, 13.0, (-1, 0), spawn_dist=3.5)
barricade(7, 3, 20.0, 10.0, (-1, 0))
barricade(8, 3, 20.0, 15.0, (-1, 0))
barricade(9, 3, 6.0, 12.5, (1, 0), spawn_dist=3.5)
M.barricades = barr

# Aufsteiger (Zombies kommen aus dem Boden)
for (x, y) in ((-12.0, -7.0), (-15.5, 4.0), (-1.0, 4.5), (5.0, -2.5), (13.5, -1.5), (17.5, 4.5), (8.0, -8.5), (-17.5, -4.0)):
    M.spawners.append({"zone": 1, "kind": "riser", "pos": bv((x, y, 0.0))})
for (x, y) in ((-10.5, 23.0), (-6.5, 30.5), (2.0, 22.5), (7.5, 27.0), (15.5, 27.0), (14.5, 21.0), (-12.0, 29.0)):
    M.spawners.append({"zone": 4, "kind": "riser", "pos": bv((x, y, 0.0))})

# Wandwaffen (Position an der Wand, Blick in den Raum)
M.wallbuys = [
    {"weapon": "m14", "pos": bv((-7.84, -12.5, 1.55)), "yaw": bv_yaw(face(1, 0))},
    {"weapon": "olympia", "pos": bv((7.84, -12.5, 1.55)), "yaw": bv_yaw(face(-1, 0))},
    {"weapon": "mp5", "pos": bv((-5.0, -9.84, 1.55)), "yaw": bv_yaw(face(0, 1))},
    {"weapon": "remington", "pos": bv((-6.16, 9.5, 1.55)), "yaw": bv_yaw(face(-1, 0))},
    {"weapon": "ak74u", "pos": bv((6.16, 15.5, 1.55)), "yaw": bv_yaw(face(1, 0))},
    {"weapon": "b23r", "pos": bv((-3.84, 29.5, 1.55)), "yaw": bv_yaw(face(1, 0))},
]

# Perk-Automaten
M.perks = [
    {"perk": "revive", "pos": bv((-5.6, -10.75, 0.02)), "yaw": bv_yaw(face(0, -1))},
    {"perk": "speed", "pos": bv((-18.9, -6.5, 0.02)), "yaw": bv_yaw(face(1, 0))},
    {"perk": "doubletap", "pos": bv((-18.9, 18.6, 0.02)), "yaw": bv_yaw(face(1, -0.4))},
    {"perk": "jugg", "pos": bv((19.0, 17.0, 0.02)), "yaw": bv_yaw(face(-1, -0.3))},
    {"perk": "mulekick", "pos": bv((18.6, 31.8, 0.02)), "yaw": bv_yaw(face(-0.5, -1))},
    {"perk": "staminup", "pos": bv((-12.8, 31.8, 0.02)), "yaw": bv_yaw(face(0.5, -1))},
]

# Mysteriöse-Kiste-Standorte
M.box_spots = [
    {"pos": bv((-17.8, -1.5, 0.02)), "yaw": bv_yaw(face(1, 0)), "start": True},
    {"pos": bv((-9.0, 7.0, 0.02)), "yaw": bv_yaw(face(0, 1)), "start": False},
    {"pos": bv((6.65, 9.0, 0.02)), "yaw": bv_yaw(face(1, 0)), "start": False},
    {"pos": bv((-12.8, 24.5, 0.02)), "yaw": bv_yaw(face(1, 0)), "start": False},
    {"pos": bv((-0.5, -21.35, 0.02)), "yaw": bv_yaw(face(0, 1)), "start": False},
]

M.pap = {"pos": bv((0.0, 31.6, 0.02)), "yaw": bv_yaw(face(0, -1))}
M.power = {"pos": bv((-6.16, 17.5, 1.4)), "yaw": bv_yaw(face(-1, 0))}

M.finish(
    extra={
        "sun": {"dir": [-0.45, -0.55, 0.7], "color": [0.55, 0.62, 0.85], "illuminance": 900.0},
        "fog": {"color": [0.055, 0.06, 0.075], "start": 8.0, "end": 70.0},
        "ambient": {"color": [0.45, 0.5, 0.65], "brightness": 120.0},
    }
)
