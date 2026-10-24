"""Karte "Endstation" – ein verlassener Busbahnhof bei Nacht.

Zonen:
  0 Wartehalle (Start)   – Fenster, M14, Doppelflinte, Zweiter Atem
  1 Busbahnhof (Hof)     – Aufsteiger aus dem Boden, MP5, Turbo-Cola, Kiste
  2 Werkstatt            – Strom-Schalter, Doppelschuss, Pump-Gun
  3 Diner                – Panzer-Tonikum, AK-74u
  4 Umspannwerk          – Waffen-Verstärker, Maultier-Malz, Ausdauer-Limo
Rundlauf: Hof -> Werkstatt -> Umspannwerk -> Diner -> Hof.

Statische Geometrie bekommt gebackenes Licht (Lightmaps, siehe maps/lightmap.py).
Aufruf: tools/blender.sh blender/maps/endstation.py [-- nobake]
"""

import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from lib import MB, hexcol  # noqa: E402
from maps import detail as D  # noqa: E402
from maps import furniture as F  # noqa: E402
from maps.mapkit import F_ALL, F_BULLET, F_PLAYER, F_ZOMBIE, MapBuilder, bv, bv_yaw  # noqa: E402

C = hexcol
rnd = random.Random(1337)

M = MapBuilder("endstation")
L = M.lib
F.use(M)
D.use(M)


def face(dx, dy):
    """Yaw (Grad), damit die Modell-Vorderseite (+Y) in Richtung (dx, dy) zeigt."""
    return math.degrees(math.atan2(dy, dx)) - 90.0


# ======================================================================================
# Materialien
# ======================================================================================


def pbr(key, tex, uv, color=None, **kw):
    """Material mit PBR-Texturen (Farbe, Normalen, Rauheit/Metall) aus build_pbr.py."""
    if color is not None:
        kw["color"] = color
    L.add(key, uv=uv, tex=tex, normal=tex + "_n", rough_tex=tex + "_mr", **kw)


pbr("concrete", "concrete", 3.0)
pbr("concrete_dark", "concrete", 3.0, C("#8c8a86"))
pbr("concrete_floor", "concrete_floor", 4.0)
pbr("asphalt", "asphalt", 5.0)
pbr("brick", "brick", 2.4)
pbr("brick_dark", "brick", 2.4, C("#9a8f88"))
pbr("plaster", "plaster", 3.0)
pbr("plaster_cream", "plaster_cream", 3.0)
pbr("paint_lower", "paint_lower", 3.0)
pbr("wall_tiles", "wall_tiles", 1.2)
pbr("tiles", "tiles", 2.4)
pbr("diner_checker", "diner_checker", 2.4)
pbr("ceiling_tile", "ceiling_tile", 2.4)
pbr("planks", "planks", 2.4)
pbr("wood_dark", "wood_dark", 1.5)
pbr("bark", "wood_dark", 1.0, C("#7d746a"))
pbr("rust", "rust", 2.0)
pbr("metal_grey", "metal_paint", 1.5, C("#9a9c9e"))
pbr("metal_green", "metal_paint", 1.5, C("#4f6b55"))
pbr("metal_blue", "metal_paint", 1.5, C("#3f5a7a"))
pbr("metal_red", "metal_paint", 1.5, C("#8a2a22"))
pbr("metal_yellow", "metal_paint", 1.5, C("#c79a2a"))
pbr("metal_white", "metal_paint", 1.5)
pbr("seat_blue", "metal_paint", 1.0, C("#2f4f7a"))
pbr("seat_orange", "metal_paint", 1.0, C("#b0602a"))
pbr("metal_dark", "gunmetal", 1.0)
pbr("dirt", "dirt", 4.0)
pbr("gravel", "gravel", 3.0)
pbr("grass", "grass_dead", 3.5)
L.add("chainlink", uv=2.0, tex="chainlink", normal="chainlink_n", rough_tex="chainlink_mr", alpha_tex=True, double_sided=True)
pbr("fabric_red", "fabric", 1.0, C("#7a2424"))
pbr("fabric_blue", "fabric", 1.0, C("#36507a"))
pbr("rubber", "rubber", 1.0)
L.add("paint_white", uv=2.0, color=C("#b9b5aa"), rough=0.75)
L.add("paint_yellow", uv=2.0, color=C("#b08c30"), rough=0.75)
L.add("glass", uv=2.0, color=C("#1d2a2c"), rough=0.06, metal=0.9)
L.add("chrome", uv=1.0, color=C("#c9cdd2"), rough=0.22, metal=1.0)
L.add("screen_black", uv=1.0, color=C("#060708"), rough=0.85)
L.add("sign_blue", uv=1.0, color=C("#1f3d6b"), rough=0.5)
L.add("sign_green", uv=1.0, color=C("#1e5a34"), rough=0.5)
L.add("sign_red", uv=1.0, color=C("#8a1a14"), rough=0.5)
L.add("sign_yellow", uv=1.0, color=C("#d1a31c"), rough=0.5)
L.add("text_white", uv=1.0, color=C("#e8e6df"), rough=0.6)
L.add("text_black", uv=1.0, color=C("#111111"), rough=0.6)
L.add("window_dark", uv=1.0, color=C("#0b0d10"), rough=0.2, metal=0.5)
L.add("window_lit", uv=1.0, color=C("#ffcf8a"), emission=C("#ffb35a"), emission_strength=1.2)
L.add("decal", uv=1.0, tex="decals", rough_tex="decals_mr", alpha_tex=True, blend=True)
pbr("soot", "rust", 2.0, C("#3a3533"))
L.add("lamp_warm", uv=1.0, color=C("#fff1c9"), emission=C("#ffcf7a"), emission_strength=6.0)
L.add("lamp_cold", uv=1.0, color=C("#eef6ff"), emission=C("#d6ecff"), emission_strength=5.0)
L.add("lamp_dead", uv=1.0, color=C("#a8a9a4"), rough=0.35)
L.add("lamp_red", uv=1.0, color=C("#ff3a2a"), emission=C("#ff2a1a"), emission_strength=8.0)
L.add("lamp_amber", uv=1.0, color=C("#ffb347"), emission=C("#ff9a1f"), emission_strength=6.0)
L.add("lamp_sodium", uv=1.0, color=C("#ffd59a"), emission=C("#ffa040"), emission_strength=8.0)
L.add("sign_exit", uv=1.0, color=C("#2fd26a"), emission=C("#1fe060"), emission_strength=3.0)
L.add("neon_pink", uv=1.0, color=C("#ff5fb0"), emission=C("#ff3d9a"), emission_strength=10.0)
L.add("neon_blue", uv=1.0, color=C("#6fd8ff"), emission=C("#3fc8ff"), emission_strength=10.0)

WALL_HALL = [(0.0, 0.12, "rubber", 0.012), (0.12, 1.2, "paint_lower", 0.0), (1.2, 1.25, "wood_dark", 0.02), (1.25, 9.0, "plaster", 0.0)]
WALL_SHOP = [(0.0, 1.2, "concrete_dark", 0.02), (1.2, 9.0, "brick_dark", 0.0)]
WALL_DINER = [(0.0, 0.1, "rubber", 0.01), (0.1, 1.15, "wall_tiles", 0.0), (1.15, 1.19, "chrome", 0.012), (1.19, 9.0, "plaster_cream", 0.0)]

# ======================================================================================
# Gelände
# ======================================================================================
M.floor(-90, -90, 90, 90, "dirt", z=0.0, t=0.2, group="ground")
M.floor(-20, -10, 20, 6, "asphalt", z=0.012, t=0.05, group="ground")  # Hof
M.floor(-60, -34, 60, -26, "asphalt", z=0.012, t=0.05, group="ground")  # Straße im Süden
for i in range(-14, 15):
    M.box((i * 4.0, -30.0, 0.017), (2.0, 0.15, 0.01), "paint_white", collide=False)
M.floor(-14, 18, 20, 33, "gravel", z=0.008, t=0.05, group="ground")  # Umspannwerk
M.floor(-6, 6, 6, 20, "concrete_floor", z=0.008, t=0.05)  # Gasse
for (x0, y0, x1, y1) in ((-45, -25, -22, 40), (22, -25, 45, 40), (-45, 34, 45, 50)):
    M.floor(x0, y0, x1, y1, "grass", z=0.006, t=0.05, group="ground")
# Gehweg vor der Wartehalle und entlang der Straße (bündig, damit man nicht stolpert)
D.sidewalk(-8.0, -9.85, 8.0, -8.4, z=0.03)
D.sidewalk(-40.0, -25.8, 40.0, -24.2, z=0.03)
D.curb((-40.0, -25.9), (40.0, -25.9), h=0.06)
D.curb((-8.0, -8.35), (8.0, -8.35), h=0.05)
# Parkbuchten (abgefahrene Farbe)
for i in range(6):
    x = -6 + i * 3.0
    M.box((x, 3.5, 0.018), (0.12, 4.5, 0.005), "paint_white", collide=False)
M.box((1.5, -1.0, 0.018), (40.0, 0.15, 0.005), "paint_yellow", collide=False)

# ======================================================================================
# Zone 0: Wartehalle (Start)
# ======================================================================================
H0 = 3.6
CEIL0 = 3.0
M.floor(-8, -22, 8, -10, "tiles", z=0.02, t=0.05)
M.box_mm((-8.2, -22.2, H0), (8.2, -9.8, H0 + 0.25), "concrete_dark", collide=True)
M.box_mm((-8.4, -22.4, H0 + 0.25), (8.4, -9.6, H0 + 0.45), "concrete", collide=False)

DOOR0 = dict(s=8.0, w=2.4, lo=0.0, hi=2.6, kind="door", door=0)
M.wall((8, -10), (-8, -10), H0, "plaster", "brick", openings=[DOOR0], inside_left=True, bands=WALL_HALL)
M.wall((-8, -22), (8, -22), H0, "plaster", "brick", openings=[F.window_spec(4.0), F.window_spec(12.0)], inside_left=True, bands=WALL_HALL)
M.wall((-8, -10), (-8, -22), H0, "plaster", "brick", openings=[F.window_spec(6.0)], inside_left=True, bands=WALL_HALL)
M.wall((8, -22), (8, -10), H0, "plaster", "brick", openings=[F.window_spec(6.0)], inside_left=True, bands=WALL_HALL)
# Außen: Sockel aus Beton um das Gebäude
for (a, b) in (((-8.17, -22.17), (8.17, -22.17)), ((-8.17, -9.83), (8.17, -9.83)), ((-8.17, -22.17), (-8.17, -9.83)), ((8.17, -22.17), (8.17, -9.83))):
    ln = math.hypot(b[0] - a[0], b[1] - a[1])
    yaw = math.degrees(math.atan2(b[1] - a[1], b[0] - a[0]))
    if abs(a[1] - b[1]) < 0.01 and abs(a[1] + 9.83) < 0.01:
        for seg in ((-8.17, -1.25), (1.25, 8.17)):
            M.box(((seg[0] + seg[1]) / 2, a[1], 0.25), (seg[1] - seg[0], 0.06, 0.5), "concrete_dark", collide=False)
        continue
    M.box(((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, 0.25), (ln, 0.06, 0.5), "concrete_dark", yaw=yaw, collide=False)

# Abgehängte Decke mit Rasterleuchten (drei an, eine flackert, drei tot)
D.drop_ceiling(
    -7.85, -21.85, 7.85, -10.15, CEIL0, missing=0.07, hanging=0.025, seed=7, debris_z=0.02,
    troffers=[
        dict(x=-3.3, y=-13.2, state="on"),
        dict(x=3.3, y=-13.2, state="flicker", intensity=30000.0),
        dict(x=0.3, y=-16.2, state="on", intensity=22000.0),
        dict(x=3.3, y=-19.2, state="on"),
        dict(x=-3.3, y=-19.2, state="dead"),
        dict(x=-3.3, y=-16.2, state="dead"),
        dict(x=3.3, y=-16.2, state="dead"),
    ],
)
# Im Zwischenraum: Lüftungskanal und Rohre (durch fehlende Platten sichtbar)
D.duct((-7.6, -12.6), (7.6, -12.6), 3.3)
D.duct((-1.8, -12.6), (-1.8, -21.6), 3.32, w=0.45, h=0.3)
D.pipe([(-7.7, -20.9, 3.45), (7.7, -20.9, 3.45)], r=0.05, mat="rust", brackets=1.5)
D.pipe([(-7.7, -15.2, 3.5), (7.7, -15.2, 3.5)], r=0.03, mat="metal_dark", brackets=1.5)

# Wartebänke (Rücken an Rücken), eine Sitzschale fehlt
for x in (-3.2, 3.2):
    D.beam_seating(x, -14.35, 0.0, 5, "seat_blue", broken=(2,) if x > 0 else ())
    D.beam_seating(x, -14.95, 180.0, 5, "seat_blue")
    D.beam_seating(x, -18.0, 0.0, 5, "seat_orange", broken=(0, 4) if x < 0 else ())
    D.beam_seating(x, -18.6, 180.0, 5, "seat_orange")

# Fahrkartenschalter (Südwest-Ecke) mit Glasaufsatz und Schild
M.box_mm((-7.85, -21.85, 0), (-5.0, -20.6, 1.05), "wood_dark", collide=True, bevel=0.01)
M.box_mm((-7.9, -20.64, 1.05), (-4.95, -20.4, 1.1), "metal_dark", collide=False, bevel=0.005)
for k in range(5):
    M.box((-7.6 + k * 0.62, -20.59, 0.55), (0.04, 0.02, 0.9), "metal_dark", collide=False)
M.box_mm((-7.85, -20.62, 1.1), (-5.0, -20.6, 2.35), "glass", collide=False)
for x in (-7.84, -6.42, -5.01):
    M.box((x, -20.61, 1.72), (0.04, 0.05, 1.25), "metal_dark", collide=False)
M.box_mm((-7.85, -20.65, 2.33), (-5.0, -20.57, 2.4), "metal_dark", collide=False)
M.box_mm((-6.7, -20.64, 1.1), (-6.1, -20.58, 1.22), "metal_dark", collide=False)  # Durchreiche
D.sign("FAHRKARTEN", (-6.42, -20.55, 2.62), 90.0, 1.6, 0.3, "sign_blue", "text_white", size=0.15)
M.box_mm((-7.6, -21.5, 0.0), (-7.1, -21.0, 0.5), "rubber", collide=False)  # Stuhl
D.floor_decal(-5.4, -20.2, 0.5, "paper", z=0.02)
# Schließfächer (eines offen)
D.locker_row(6.9, -20.0, 90.0, 4, open_doors=(1,))
D.locker_row(-7.6, -19.0, -90.0, 3)
# Abfahrtstafel und Uhr
D.departure_board(1.6, -21.78, 2.35, 90.0, ["22:14  NORDBAHNHOF   ENTFÄLLT", "22:31  KLINIKUM     ENTFÄLLT", "22:47  ALTSTADT     ENTFÄLLT", "23:05  ENDSTATION   ---"])
D.wall_clock(2.2, -10.16, 2.55, -90.0)
D.exit_sign(0.0, -10.17, 2.78, -90.0)
D.fire_extinguisher(7.84, -18.4, 180.0)
# Notleuchte über dem Schalter (dunkle Ecke bleibt erkennbar)
M.box((-6.4, -21.8, 2.75), (0.35, 0.08, 0.12), "metal_white", collide=False)
M.box((-6.4, -21.755, 2.75), (0.3, 0.01, 0.08), "lamp_warm", collide=False)
M.add_light((-6.4, -21.3, 2.55), (1.0, 0.82, 0.6), 9000.0, 7.0)
D.vent_grille(-7.84, -13.9, 2.5, 0.0)
D.vent_grille(7.84, -14.5, 0.35, 180.0, w=0.5, h=0.2)
D.sign("WARTEHALLE", (-4.4, -10.17, 2.6), -90.0, 1.8, 0.32, "sign_blue", "text_white", size=0.16)
# Müll und Spuren
D.trash_bin(6.8, -11.0, tipped=True)
D.trash_bin(-1.8, -21.4)
D.crate(-6.9, -11.2, s=0.7, z=0.02)
D.scatter_floor(-7.5, -21.5, 7.5, -10.5, 0.02, 22, ["paper", "newspaper", "debris", "grime"], size=(0.35, 0.8))
D.floor_decal(-4.0, -21.1, 1.4, "blood_pool", z=0.02)
D.floor_decal(-3.6, -20.2, 1.0, "footprints", z=0.02, spin=10)
D.floor_decal(7.1, -16.0, 1.1, "blood_splat", z=0.02)
D.floor_decal(-7.1, -16.2, 0.9, "grime", z=0.02)
D.floor_decal(4.0, -21.0, 0.8, "blood_spray", z=0.02, spin=90)
D.wall_decal(-4.0, -21.85, 2.45, 90.0, (1.2, 1.2), "blood_drips")
D.wall_decal(7.85, -15.0, 1.6, 180.0, (1.0, 1.0), "blood_spray", spin=180)
D.wall_decal(-2.6, -10.15, 1.7, -90.0, (0.55, 0.7), "poster_missing")
D.wall_decal(-1.9, -10.15, 1.62, -90.0, (0.55, 0.7), "poster_missing", spin=-4)
D.wall_decal(5.2, -21.85, 1.75, 90.0, (0.8, 1.0), "poster_ad")
D.wall_decal(-7.85, -17.6, 2.8, 0.0, (1.6, 1.6), "leak")
D.wall_decal(7.85, -12.0, 2.9, 180.0, (1.4, 1.4), "leak")
D.wall_decal(-1.0, -21.85, 1.4, 90.0, (0.9, 0.9), "bullets")
D.wall_decal(4.2, -10.15, 2.2, -90.0, (1.2, 1.2), "cracks")
D.broken_glass(-4.0, -21.3, 0.02, 0.7)
D.broken_glass(4.0, -21.3, 0.02, 0.6)
D.broken_glass(7.3, -16.0, 0.02, 0.6)
D.broken_glass(-7.3, -16.0, 0.02, 0.6)
# Mondlicht durch die Fenster sorgt für kühle Lichtflecken (kommt aus dem Bake)

# ======================================================================================
# Zone 2: Werkstatt
# ======================================================================================
H2 = 4.2
M.floor(-20, 6, -6, 20, "concrete_floor", z=0.02, t=0.05)
M.box_mm((-20.2, 5.8, H2), (-5.8, 20.2, H2 + 0.12), "metal_grey", collide=True)
M.box_mm((-20.4, 5.6, H2 + 0.12), (-5.6, 20.4, H2 + 0.3), "rust", collide=False)
M.wall((-20, 6), (-6, 6), H2, "brick_dark", "brick", openings=[dict(s=7.0, w=3.2, lo=0.0, hi=3.4, kind="hole")], inside_left=True, bands=WALL_SHOP)
M.wall((-20, 20), (-20, 6), H2, "brick_dark", "brick", openings=[F.window_spec(4.0), F.window_spec(10.0)], inside_left=True, bands=WALL_SHOP)
DOOR3 = dict(s=4.0, w=3.0, lo=0.0, hi=3.0, kind="door", door=3)
M.wall((-6, 20), (-20, 20), H2, "brick_dark", "brick", openings=[DOOR3], inside_left=True, bands=WALL_SHOP)
M.wall((-6, 6), (-6, 20), H2, "brick_dark", "brick", openings=[F.window_spec(7.0)], inside_left=True, bands=WALL_SHOP)
# Stahlträger (I-Profil) und Pfetten unter dem Wellblechdach
for y in (8.0, 11.5, 15.0, 18.5):
    M.box((-13.0, y, 3.95), (13.9, 0.16, 0.012), "metal_dark", collide=False)
    M.box((-13.0, y, 3.75), (13.9, 0.16, 0.012), "metal_dark", collide=False)
    M.box((-13.0, y, 3.85), (13.9, 0.012, 0.2), "metal_dark", collide=False)
for x in (-18.0, -15.0, -12.0, -9.0):
    M.box((x, 13.0, 4.1), (0.08, 13.9, 0.14), "rust", collide=False)
D.pipe([(-19.8, 6.4, 3.4), (-19.8, 19.6, 3.4), (-6.4, 19.6, 3.4)], r=0.05, mat="metal_red", brackets=1.8)
D.pipe([(-19.8, 7.0, 0.3), (-19.8, 7.0, 3.4)], r=0.05, mat="metal_red")
D.conduit_run((-6.15, 6.5), (-6.15, 19.5), 2.9, 180.0)
F.car_lift(-13.0, 12.5, 0.0)
F.workbench(-18.9, 13.0, -90.0)
F.shelf(-16.3, 19.2, 180.0, 2.4, 2.2, 0.6)
F.tire_stack(-18.8, 7.2, 4)
F.tire_stack(-18.0, 7.0, 3)
D.barrel(-7.0, 7.2, "metal_red")
D.barrel(-7.7, 7.4, "metal_blue")
D.barrel(-7.2, 8.1, "metal_yellow", tipped=True)
D.pallet(-7.3, 18.9)
D.crate(-7.3, 18.9, z=0.15, s=0.9)
D.crate(-8.4, 19.1, s=0.6, yaw=12)
# Druckluftkompressor
M.cyl((-19.3, 17.2, 0.45), 0.3, 1.1, "metal_red", axis="Y", seg=18, flags=F_PLAYER | F_ZOMBIE)
M.box((-19.3, 17.2, 0.95), (0.5, 0.5, 0.35), "metal_dark", collide=False, bevel=0.02)
D.pipe([(-19.3, 17.6, 1.0), (-19.0, 18.2, 0.9), (-18.4, 18.5, 0.04), (-17.0, 17.6, 0.04)], r=0.012, mat="rubber")
# Leuchten (Strom nötig), rote Notleuchte, Arbeitslampe
D.pendant(-13.0, 9.0, H2, drop=1.2, kind="warm", needs_power=True, intensity=70000.0, rng=16.0)
D.pendant(-13.0, 16.5, H2, drop=1.2, kind="warm", needs_power=True, intensity=70000.0, rng=16.0)
F.ceiling_lamp(-7.0, 16.0, H2, "red", intensity=12000.0, rng=8.0, flicker=0.3)
M.add_light((-18.6, 13.0, 1.6), (1.0, 0.7, 0.4), 9000.0, 5.0)  # Arbeitslampe über der Werkbank
for (x, y, yaw) in ((-6.17, 9.5, 180.0), (-19.83, 17.5, 0.0), (-13.0, 19.83, -90.0)):
    a = math.radians(yaw)
    D.lbox(x, y, yaw + 90, 0, 0, 2.9, 0.34, 0.08, 0.12, "metal_white")
    D.lbox(x + math.cos(a) * 0.045, y + math.sin(a) * 0.045, yaw + 90, 0, 0, 2.9, 0.3, 0.01, 0.08, "lamp_warm")
    M.add_light((x + math.cos(a) * 0.5, y + math.sin(a) * 0.5, 2.7), (1.0, 0.8, 0.55), 11000.0, 8.0)
M.box((-18.8, 13.0, 1.75), (0.15, 0.25, 0.1), "lamp_warm", collide=False)
# Spuren
D.floor_decal(-13.0, 12.5, 2.2, "oil", z=0.02)
D.floor_decal(-11.2, 10.0, 1.2, "oil", z=0.02)
D.floor_decal(-16.5, 16.0, 1.0, "oil", z=0.02)
D.scatter_floor(-19.5, 6.5, -6.5, 19.5, 0.02, 14, ["debris", "grime", "paper"], size=(0.4, 1.0))
D.floor_decal(-6.9, 13.0, 1.2, "blood_pool", z=0.02)
D.wall_decal(-6.15, 13.0, 1.5, 180.0, (1.1, 1.1), "blood_drips")
D.wall_decal(-19.85, 10.0, 3.0, 0.0, (1.5, 1.5), "leak")
D.wall_decal(-12.0, 6.15, 1.4, 90.0, (0.8, 0.8), "graffiti")
D.sign("WERKSTATT", (-15.0, 6.17, 3.3), 90.0, 2.4, 0.4, "sign_yellow", "text_black", size=0.22)
D.sign("RAUCHEN VERBOTEN", (-19.85, 15.5, 2.3), 0.0, 1.2, 0.25, "sign_red", "text_white", size=0.09)

# ======================================================================================
# Zone 3: Diner
# ======================================================================================
H3 = 3.4
M.floor(6, 6, 20, 18, "diner_checker", z=0.02, t=0.05)
M.box_mm((5.8, 5.8, H3), (20.2, 18.2, H3 + 0.22), "plaster_cream", collide=True)
M.box_mm((5.6, 5.6, H3 + 0.22), (20.4, 18.4, H3 + 0.4), "metal_white", collide=False)
DOOR2 = dict(s=4.0, w=1.6, lo=0.0, hi=2.4, kind="door", door=2)
M.wall((6, 6), (20, 6), H3, "plaster_cream", "metal_white", openings=[DOOR2, dict(s=8.5, w=2.2, lo=1.0, hi=2.4, kind="block")], inside_left=True, bands=WALL_DINER)
M.wall((20, 6), (20, 18), H3, "plaster_cream", "metal_white", openings=[F.window_spec(4.0), F.window_spec(9.0)], inside_left=True, bands=WALL_DINER)
DOOR4 = dict(s=12.0, w=1.4, lo=0.0, hi=2.3, kind="door", door=4)
M.wall((20, 18), (6, 18), H3, "plaster_cream", "metal_white", openings=[DOOR4], inside_left=True, bands=WALL_DINER)
M.wall((6, 18), (6, 6), H3, "plaster_cream", "metal_white", openings=[F.window_spec(5.5)], inside_left=True, bands=WALL_DINER)
# Großes Fenster nach Süden: Glas mit Sprung, blockiert alles
M.collider((14.5, 6.0, 1.7), (2.2, 0.3, 1.4), 0.0, F_PLAYER | F_ZOMBIE)
M.box((14.5, 6.0, 1.7), (2.2, 0.03, 1.4), "glass", collide=False)
D.wall_decal(14.1, 6.03, 1.9, 90.0, (0.9, 0.9), "cracks")
# Tresen mit Hockern
M.box_mm((9.5, 14.6, 0), (17.5, 15.3, 1.0), "wall_tiles", collide=True)
M.box_mm((9.4, 14.5, 1.0), (17.6, 15.4, 1.07), "fabric_red", collide=False, bevel=0.01)
M.box_mm((9.38, 14.48, 0.96), (17.62, 15.42, 1.0), "chrome", collide=False)
M.box_mm((9.5, 14.55, 0.0), (17.5, 14.6, 0.1), "chrome", collide=False)
M.box_mm((9.5, 16.9, 0), (17.5, 17.85, 0.95), "metal_grey", collide=True)
F.shelf(13.5, 17.6, 180.0, 3.0, 1.0, 0.35, "metal_grey")
for i in range(6):
    if i == 3:
        M.cyl((10.2 + i * 1.3 + 0.4, 13.6, 0.2), 0.19, 0.08, "fabric_red", axis="X", seg=12, collide=False)
        continue
    F.stool(10.2 + i * 1.3, 14.0)
    M.cyl((10.2 + i * 1.3, 14.0, 0.02), 0.2, 0.03, "chrome", seg=14, collide=False)
F.booth(7.7, 7.6, 0.0)
F.booth(16.8, 8.0, 0.0)
# Kaffeemaschine und Kühlvitrine
M.box((11.0, 17.4, 1.15), (0.5, 0.4, 0.4), "chrome", collide=False, bevel=0.02)
M.box((15.8, 17.4, 1.35), (1.2, 0.5, 0.8), "glass", collide=False)
M.box((15.8, 17.4, 0.97), (1.25, 0.55, 0.06), "chrome", collide=False)
M.add_light((15.8, 17.0, 1.3), (0.7, 0.85, 1.0), 6000.0, 5.0)
M.add_light((10.1, 6.6, 2.6), (1.0, 0.35, 0.65), 9000.0, 7.0)  # Schein der Leuchtschrift durchs Fenster
M.add_light((19.3, 12.5, 1.4), (0.35, 0.8, 1.0), 5000.0, 4.0)  # Jukebox
# Jukebox
M.box_mm((19.0, 12.0, 0), (19.85, 13.0, 1.6), "metal_red", collide=True, bevel=0.03)
M.box_mm((18.95, 12.15, 0.8), (18.98, 12.85, 1.45), "neon_blue", collide=False)
D.pendant(10.0, 10.0, H3, drop=0.7, kind="warm", needs_power=True, intensity=40000.0, shade="metal_red")
D.pendant(16.0, 10.0, H3, drop=0.7, kind="warm", needs_power=True, intensity=40000.0, shade="metal_red")
D.pendant(13.0, 16.0, H3, drop=0.6, kind="warm", intensity=16000.0, flicker=0.5, shade="metal_red")
# Leuchtschrift außen über dem Eingang
M.box((10.1, 5.8, 3.0), (3.0, 0.06, 0.7), "metal_dark", collide=False)
D.text("DINER", (10.1, 5.76, 3.0), -90.0, 0.45, "neon_pink", extrude=0.03, group="glow")
M.add_light((10.1, 5.0, 3.0), (1.0, 0.3, 0.6), 30000.0, 6.0, flicker=0.15)
D.text("SPEISEKARTE", (13.5, 17.83, 2.55), -90.0, 0.16, "text_white", extrude=0.003)
M.box((13.5, 17.85, 2.25), (2.2, 0.03, 0.9), "screen_black", collide=False)
for i, ln in enumerate(["KAFFEE ....... 1,20", "SCHNITZEL .... 7,50", "BURGER ....... 5,90", "KUCHEN ....... 2,40"]):
    D.text(ln, (13.5, 17.83, 2.35 - i * 0.14), -90.0, 0.075, "text_white", extrude=0.001)
D.scatter_floor(6.5, 6.5, 19.5, 17.5, 0.02, 12, ["paper", "debris", "grime"], size=(0.3, 0.7))
D.floor_decal(12.0, 13.4, 1.3, "blood_pool", z=0.02)
D.floor_decal(12.8, 12.6, 1.0, "footprints", z=0.02, spin=200)
D.floor_decal(14.5, 6.6, 1.0, "blood_spray", z=0.02, spin=90)
D.broken_glass(14.5, 6.6, 0.02, 0.9, 20)
D.wall_decal(20.0 - 0.15, 10.0, 2.5, 180.0, (1.0, 1.0), "blood_drips")
D.wall_decal(6.15, 9.0, 1.7, 0.0, (0.6, 0.75), "poster_ad")

# ======================================================================================
# Zone 1: Busbahnhof (Hof)
# ======================================================================================
F.fence((-20, -10), (-8, -10))
F.fence((8, -10), (20, -10))
F.fence((-20, -10), (-20, 6))
F.fence((20, -10), (20, 6))
F.fence((-6, 6), (6, 6))
F.rubble(-3.0, 6.8, r=1.6, h=1.2, collide=False)
F.rubble(2.5, 7.5, r=1.4, h=1.0, collide=False)
F.bus_wreck(-7.0, -1.0, 12.0)
F.car_wreck(11.5, -6.0, 75.0)
F.car_wreck(15.5, 2.5, -20.0, color="metal_blue")
F.bus_shelter(3.5, -8.4, 0.0)
F.jersey(-15.5, 1.5, 90.0)
F.jersey(6.5, 1.0, 20.0)
D.barrel(-17.5, -8.5, "metal_red")
D.barrel(-18.2, -8.0, "metal_green", tipped=True)
F.tire_stack(17.8, -8.6, 2)
F.cone(1.0, 2.0)
F.cone(-1.5, 3.5)
F.cone(9.0, 4.2)
D.crate(-18.6, 4.6, s=0.9)
D.crate(-17.7, 4.8, s=0.7, yaw=20)
D.street_lamp(-14.0, -6.0, 0.0, "on")
D.street_lamp(1.5, 3.8, 180.0, "flicker")
D.street_lamp(16.5, -3.0, 90.0, "dead")
# Haltestellenschild
M.cyl((6.0, -8.9, 1.3), 0.03, 2.6, "metal_grey", seg=8, collide=False)
M.cyl((6.0, -8.9, 2.75), 0.24, 0.03, "sign_yellow", axis="Y", seg=20, collide=False)
M.cyl((6.0, -8.9, 2.75), 0.2, 0.034, "sign_green", axis="Y", seg=20, collide=False)
D.text("H", (6.0, -8.88, 2.75), 90.0, 0.26, "sign_yellow", extrude=0.004)
D.text("H", (6.0, -8.92, 2.75), -90.0, 0.26, "sign_yellow", extrude=0.004)
# Feuer im Buswrack
M.fires.append({"pos": bv((-7.0, -2.0, 1.3)), "scale": 1.2})
M.fires.append({"pos": bv((12.0, -5.8, 1.2)), "scale": 0.8})
M.add_light((-7.0, -2.0, 2.2), (1.0, 0.45, 0.15), 60000.0, 8.0, flicker=1.0, shadows=True)
M.add_light((12.0, -5.8, 1.9), (1.0, 0.45, 0.15), 30000.0, 6.5, flicker=1.0)
D.scatter_floor(-19.0, -9.5, 19.0, 5.5, 0.02, 40, ["debris", "oil", "paper", "newspaper", "grime"], size=(0.4, 1.4))
for (x, y, s) in ((-11.5, -7.5, 2.2), (4.0, 1.0, 2.6), (13.0, 0.5, 1.8), (-2.0, -5.0, 1.6)):
    D.floor_decal(x, y, s, "oil", z=0.015)
D.floor_decal(0.5, -7.6, 1.5, "blood_pool", z=0.04)
D.floor_decal(-1.2, -6.4, 1.2, "footprints", z=0.02, spin=160)
D.wall_decal(-5.0, -9.85, 1.6, 90.0, (1.6, 1.6), "graffiti")
D.wall_decal(5.5, -9.85, 2.2, 90.0, (1.8, 1.8), "leak", spin=0)

# ======================================================================================
# Zone 4: Umspannwerk
# ======================================================================================
F.fence((-14, 20), (-14, 33))
F.fence((-14, 33), (20, 33))
F.fence((20, 18), (20, 33))
F.fence((-6, 20), (6, 20))
F.fence((6, 18), (6, 20))
H4 = 3.4
M.floor(-4, 27, 4, 32.8, "concrete_floor", z=0.02, t=0.05)
M.wall((-4, 27), (4, 27), H4, "brick_dark", "brick", openings=[dict(s=4.0, w=3.0, lo=0.0, hi=2.8, kind="hole")], inside_left=True, bands=WALL_SHOP)
M.wall((-4, 32.8), (-4, 27), H4, "brick_dark", "brick", inside_left=True, bands=WALL_SHOP)
M.wall((4, 27), (4, 32.8), H4, "brick_dark", "brick", inside_left=True, bands=WALL_SHOP)
M.wall((4, 32.8), (-4, 32.8), H4, "brick_dark", "brick", inside_left=True, bands=WALL_SHOP)
M.box_mm((-4.2, 26.8, H4), (4.2, 33.0, H4 + 0.3), "concrete_dark", collide=True)
D.pendant(0.0, 30.0, H4, drop=0.6, kind="cold", needs_power=True, intensity=30000.0, rng=10.0)
D.pipe([(-3.8, 27.4, 3.0), (-3.8, 32.4, 3.0), (3.8, 32.4, 3.0)], r=0.04, mat="metal_dark", brackets=1.2)
D.sign("HOCHSPANNUNG - LEBENSGEFAHR", (0.0, 26.83, 3.05), -90.0, 2.6, 0.3, "sign_yellow", "text_black", size=0.12)
D.wall_decal(2.5, 26.85, 1.5, -90.0, (1.0, 1.0), "graffiti")
F.transformer(-9.5, 26.0, 0.0)
F.transformer(10.0, 24.5, 90.0)
F.transformer(12.5, 30.0, 0.0)
F.power_pole(-11.0, 31.5)
F.power_pole(16.0, 21.0)
D.wires((-11.0, 31.5, 7.5), (16.0, 21.0, 7.5), sag=1.2)
D.wires((-12.0, 31.5, 7.5), (15.0, 21.0, 7.5), sag=1.3)
D.wires((-10.0, 31.5, 7.5), (17.0, 21.0, 7.5), sag=1.1)
D.crate(7.5, 31.8, s=0.9)
D.crate(8.4, 31.9, s=0.6, yaw=15)
D.barrel(-12.8, 21.5, "metal_yellow")
D.street_lamp(-2.0, 22.5, 90.0, "flicker", intensity=60000.0)
D.street_lamp(15.5, 26.0, 180.0, "on", intensity=110000.0)
for (x, y) in ((-9.5, 24.6), (10.0, 23.0), (12.5, 28.6)):
    D.sign("VORSICHT", (x, y, 1.6), -90.0, 0.7, 0.24, "sign_yellow", "text_black", size=0.1)
D.scatter_floor(-13.5, 20.5, 19.5, 32.5, 0.012, 18, ["debris", "grime", "paper"], size=(0.5, 1.2))
D.floor_decal(3.0, 23.0, 1.4, "blood_pool", z=0.012)

# ======================================================================================
# Außenbereich (nicht begehbar, Atmosphäre)
# ======================================================================================
for i, (x, y) in enumerate(((-28, -15), (-31, 8), (-26, 25), (28, -14), (31, 6), (27, 26), (-18, 42), (10, 44), (36, -32), (-36, -30), (-24, -8), (25, 18), (-30, 30))):
    D.dead_tree(x, y, h=rnd.uniform(5.0, 8.5), seed=i)
F.car_wreck(-24.0, -4.0, 10.0)
F.car_wreck(25.0, -6.0, 100.0, color="metal_red")
F.car_wreck(-12.0, -30.0, 88.0)
F.car_wreck(14.0, -31.0, 94.0, color="metal_blue")
for x in (-40, -20, 0, 20, 40):
    F.power_pole(x, -36.0, 9.0)
for x0, x1 in ((-40, -20), (-20, 0), (0, 20), (20, 40)):
    for dz, k in ((8.55, 0.0), (8.55, -1.0), (8.55, 1.0)):
        D.wires((x0 + k, -36.0, dz), (x1 + k, -36.0, dz), sag=0.9, r=0.01)
F.rubble(-26.0, 18.0, r=2.0, collide=False)
F.rubble(26.0, 20.0, r=2.0, collide=False)
# Stadt am Horizont (im Nebel)
for i, (x, y, w, d, h, yaw) in enumerate(((-60, 55, 22, 14, 24, 0), (-25, 70, 30, 16, 38, 5), (15, 62, 18, 12, 18, -8), (45, 58, 26, 14, 30, 0),
                                          (-75, 10, 14, 30, 20, 90), (75, -5, 14, 26, 28, -90), (70, 35, 12, 18, 16, -80),
                                          (-55, -60, 30, 14, 22, 180), (20, -65, 24, 14, 34, 180), (60, -55, 18, 12, 15, 175))):
    D.silhouette_building(x, y, w, d, h, yaw=face(-x, -y), seed=i)

# ======================================================================================
# Bewegliche Objekte: Türen und Schutt
# ======================================================================================


def door_panel(name, hinge, width, height, yaw, mat="metal_green", thick=0.06):
    """Türblatt mit Ursprung am Scharnier; Blatt erstreckt sich entlang lokal +X.

    Stahltür mit Drahtglasfenster, Trittblech, Panikstange und Scharnieren."""
    b = MB(name, uv_scale=1.2)
    hx, hy = hinge
    w, h = width - 0.02, height - 0.02
    gy0, gy1 = height * 0.58, height * 0.88
    gx0, gx1 = width * 0.22, width * 0.78
    # Blatt um das Fenster herum (vier Teile), damit das Glas durchsichtig wirkt
    b.box((width / 2, 0, gy0 / 2), (w, thick, gy0), L[mat], bevel=0.006)
    b.box((width / 2, 0, (gy1 + h) / 2), (w, thick, h - gy1), L[mat], bevel=0.006)
    b.box(((0.01 + gx0) / 2, 0, (gy0 + gy1) / 2), (gx0 - 0.01, thick, gy1 - gy0), L[mat])
    b.box(((gx1 + width - 0.01) / 2, 0, (gy0 + gy1) / 2), (width - 0.01 - gx1, thick, gy1 - gy0), L[mat])
    b.box((width / 2, 0, (gy0 + gy1) / 2), (gx1 - gx0, 0.008, gy1 - gy0), L["glass"])
    for s in (-1, 1):
        # Rahmenleisten um das Glas
        b.box((width / 2, s * thick / 2, gy0), (gx1 - gx0 + 0.04, 0.012, 0.03), L["metal_dark"])
        b.box((width / 2, s * thick / 2, gy1), (gx1 - gx0 + 0.04, 0.012, 0.03), L["metal_dark"])
        b.box((gx0, s * thick / 2, (gy0 + gy1) / 2), (0.03, 0.012, gy1 - gy0), L["metal_dark"])
        b.box((gx1, s * thick / 2, (gy0 + gy1) / 2), (0.03, 0.012, gy1 - gy0), L["metal_dark"])
        # Trittblech und Panikstange
        b.box((width / 2, s * (thick / 2 + 0.003), 0.15), (w - 0.04, 0.004, 0.25), L["chrome"])
        b.box((width * 0.55, s * (thick / 2 + 0.045), 1.0), (width * 0.7, 0.03, 0.04), L["metal_dark"], bevel=0.008)
        for px in (width * 0.22, width * 0.88):
            b.box((px, s * (thick / 2 + 0.025), 1.0), (0.05, 0.05, 0.08), L["metal_dark"])
    for z in (0.25, height - 0.3):
        b.cyl((0.0, 0.0, z), 0.012, 0.12, L["chrome"], seg=8)
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
        "sun": {"dir": [-0.45, -0.55, 0.7], "color": [0.55, 0.64, 0.9], "illuminance": 1300.0},
        "fog": {"color": [0.03, 0.035, 0.048], "start": 6.0, "end": 85.0},
        "ambient": {"color": [0.4, 0.47, 0.62], "brightness": 130.0},
    }
)
