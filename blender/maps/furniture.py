"""Requisiten-Bausteine für Karten (Bänke, Kisten, Wracks, Zäune, Lampen, ...).

Jede Funktion fügt Geometrie und Kollision zur aktiven Karte hinzu (siehe `use`)."""

import math
import random

from lib import MB

from .mapkit import F_ALL, F_BULLET, F_PLAYER, F_ZOMBIE

M = None
L = None
rnd = random.Random(1337)


def use(builder):
    """Legt die Karte fest, zu der die Bausteine Geometrie hinzufügen."""
    global M, L
    M = builder
    L = builder.lib




def rot2(x, y, yaw):
    a = math.radians(yaw)
    return x * math.cos(a) - y * math.sin(a), x * math.sin(a) + y * math.cos(a)


def local_box(ox, oy, yaw, lx, ly, lz, sx, sy, sz, mat, collide=True, flags=F_ALL, bevel=0.0):
    """Box in lokalen Koordinaten eines Objekts (Ursprung ox,oy, Drehung yaw)."""
    dx, dy = rot2(lx, ly, yaw)
    M.box((ox + dx, oy + dy, lz), (sx, sy, sz), mat, yaw=yaw, collide=collide, flags=flags, bevel=bevel)


def bench(x, y, yaw, length=2.4):
    for sx in (-length / 2 + 0.15, length / 2 - 0.15):
        local_box(x, y, yaw, sx, 0, 0.22, 0.08, 0.45, 0.44, "metal_dark", collide=False)
    local_box(x, y, yaw, 0, 0, 0.46, length, 0.46, 0.06, "wood_dark", collide=False)
    local_box(x, y, yaw, 0, -0.22, 0.72, length, 0.06, 0.42, "wood_dark", collide=False)
    M.collider((x, y, 0.45), (length, 0.5, 0.9), yaw, F_PLAYER | F_ZOMBIE)


def crate(x, y, z=0.0, s=0.8, yaw=0.0, collide=True):
    M.box((x, y, z + s / 2), (s, s, s), "planks", yaw=yaw, collide=collide, bevel=0.02)
    for k in (-1, 1):
        dx, dy = rot2(k * (s / 2 + 0.005), 0, yaw)
        M.box((x + dx, y + dy, z + s / 2), (0.02, s * 0.9, s * 0.15), "wood_dark", yaw=yaw, collide=False)


def barrel(x, y, mat="metal_blue", z=0.0, tipped=False):
    if tipped:
        M.cyl((x, y, 0.3), 0.3, 0.9, mat, axis="X", seg=14, flags=F_PLAYER | F_ZOMBIE)
    else:
        M.cyl((x, y, z + 0.45), 0.3, 0.9, mat, seg=14, flags=F_PLAYER | F_ZOMBIE)
        M.cyl((x, y, z + 0.9), 0.3, 0.02, "rust", seg=14, collide=False)


def tire_stack(x, y, n=3):
    for i in range(n):
        M.cyl((x + rnd.uniform(-0.04, 0.04), y + rnd.uniform(-0.04, 0.04), 0.12 + i * 0.24), 0.36, 0.24, "rubber", seg=16, collide=False)
    M.collider((x, y, n * 0.12), (0.72, 0.72, n * 0.24), 0.0, F_PLAYER | F_ZOMBIE)


def cone(x, y):
    M.cyl((x, y, 0.3), 0.14, 0.6, "metal_red", r2=0.03, seg=10, collide=False)
    M.box((x, y, 0.015), (0.36, 0.36, 0.03), "metal_red", collide=False)


def jersey(x, y, yaw, length=3.0):
    local_box(x, y, yaw, 0, 0, 0.2, length, 0.6, 0.4, "concrete", collide=False, bevel=0.03)
    local_box(x, y, yaw, 0, 0, 0.6, length, 0.25, 0.45, "concrete", collide=False, bevel=0.03)
    c = abs(math.cos(math.radians(yaw)))
    M.collider((x, y, 0.42), (length * c + 0.6 * (1 - c), length * (1 - c) + 0.6 * c, 0.85), 0.0, F_PLAYER | F_ZOMBIE)


def lamp_post(x, y, yaw=0.0, working=True, flicker=0.0, power=False):
    M.cyl((x, y, 3.2), 0.07, 6.4, "metal_dark", seg=10, flags=F_PLAYER | F_ZOMBIE | F_ZOMBIE)
    M.cyl((x, y, 0.2), 0.16, 0.4, "concrete", seg=10, collide=False)
    hx, hy = rot2(0, 1.1, yaw)
    M.box((x + hx / 2, y + hy / 2, 6.35), (0.08, 1.2, 0.08), "metal_dark", yaw=yaw, collide=False)
    M.box((x + hx, y + hy, 6.25), (0.35, 0.55, 0.18), "metal_dark", yaw=yaw, collide=False)
    M.box((x + hx, y + hy, 6.15), (0.28, 0.45, 0.04), "lamp_warm" if working else "glass", yaw=yaw, collide=False)
    if working:
        M.add_light((x + hx, y + hy, 5.9), (1.0, 0.72, 0.42), 120000.0, 14.0, flicker=flicker, power=power)


def local_ext(ox, oy, yaw, pts, width, mat, lx=0.0, z=0.0):
    """Seitenprofil (lokal: a = vorn, b = oben) quer extrudiert, um lx zur Seite versetzt."""
    M.mb(mat).extrude(pts, width, M.lib[mat], plane="YZ", offset=lx, rot=(0, 0, yaw), center=(ox, oy, z), uv=M.lib.uv[mat])


def local_cyl(ox, oy, yaw, lx, ly, lz, r, h, mat, seg=16):
    """Quer liegender Zylinder (Achse lokal X), z. B. Räder."""
    dx, dy = rot2(lx, ly, yaw)
    M.cyl((ox + dx, oy + dy, lz), r, h, mat, axis="X", rot=(0, 0, yaw), seg=seg, collide=False)


def wheel(ox, oy, yaw, lx, ly, r, w, rim="metal_grey"):
    """Reifen mit Felge und Nabe (lx > 0: rechte Seite)."""
    side = 1.0 if lx >= 0 else -1.0
    local_cyl(ox, oy, yaw, lx, ly, r, r, w, "rubber", seg=20)
    local_cyl(ox, oy, yaw, lx + side * 0.012, ly, r, r * 0.6, w, rim, seg=16)
    local_cyl(ox, oy, yaw, lx + side * 0.02, ly, r, r * 0.18, w, "metal_dark", seg=10)


def car_wreck(x, y, yaw, color="rust"):
    """Verlassene Limousine: Seitenprofil mit Haube, schrägen Scheiben und Kofferraum,
    dunkle Scheiben mit Säulen in Wagenfarbe, Radkästen, Felgen, Stoßstangen, Lichter.
    Manchen fehlt ein Rad."""
    def ext(pts, width, mat, lx=0.0):
        local_ext(x, y, yaw, pts, width, mat, lx)

    def lb(lx, ly, lz, sx, sy, sz, mat, bevel=0.0):
        local_box(x, y, yaw, lx, ly, lz, sx, sy, sz, mat, collide=False, bevel=bevel)

    # Karosserie bis zur Gürtellinie
    ext([(-2.22, 0.33), (2.2, 0.33), (2.26, 0.5), (2.22, 0.7), (1.98, 0.79), (1.05, 0.9), (-1.3, 0.94), (-2.14, 0.9),
         (-2.26, 0.74), (-2.26, 0.45)], 1.76, color)
    # Fahrgastzelle: Scheiben dunkel, Dach und Säulen in Wagenfarbe
    ext([(-1.28, 0.92), (1.02, 0.89), (0.42, 1.36), (-0.62, 1.38)], 1.5, "window_dark")
    ext([(-0.66, 1.37), (0.46, 1.35), (0.4, 1.42), (-0.6, 1.44)], 1.55, color)
    for side in (-1, 1):
        lx = side * 0.75
        ext([(0.98, 0.89), (1.06, 0.9), (0.47, 1.37), (0.39, 1.37)], 0.05, color, lx)
        ext([(-0.14, 0.92), (-0.03, 0.92), (-0.03, 1.38), (-0.14, 1.38)], 0.05, color, lx)
        ext([(-1.32, 0.93), (-1.1, 0.93), (-0.57, 1.38), (-0.68, 1.38)], 0.05, color, lx)
        # Spiegel, Türfugen und Griffe
        lb(side * 0.95, 0.86, 0.98, 0.16, 0.07, 0.1, color)
        for ly in (0.95, -0.05, -1.15):
            lb(side * 0.882, ly, 0.62, 0.012, 0.02, 0.5, "soot")
        for ly in (0.3, -0.75):
            lb(side * 0.885, ly, 0.78, 0.02, 0.16, 0.03, "metal_dark")
    # Radkästen (dunkle Öffnung hinter den Rädern) und Räder
    for ly in (1.36, -1.36):
        local_cyl(x, y, yaw, 0.0, ly, 0.36, 0.41, 1.78, "soot", seg=20)
    missing = rnd.randrange(6)
    for i, (lx, ly) in enumerate([(-0.78, 1.36), (0.78, 1.36), (-0.78, -1.36), (0.78, -1.36)]):
        if i != missing:
            wheel(x, y, yaw, lx, ly, 0.33, 0.24)
    # Stoßstangen, Grill, Scheinwerfer, Rückleuchten, Kennzeichen
    lb(0, 2.25, 0.42, 1.8, 0.14, 0.2, "metal_dark", bevel=0.04)
    lb(0, -2.27, 0.44, 1.8, 0.14, 0.2, "metal_dark", bevel=0.04)
    lb(0, 2.255, 0.63, 0.9, 0.04, 0.14, "metal_dark")
    for side in (-1, 1):
        lb(side * 0.62, 2.245, 0.66, 0.36, 0.04, 0.12, "glass")
        lb(side * 0.66, -2.265, 0.76, 0.36, 0.04, 0.12, "metal_red")
    lb(0, -2.36, 0.5, 0.5, 0.02, 0.12, "paint_white")
    M.collider((x, y, 0.7), (1.9, 4.4, 1.4), yaw, F_ALL)


def bus_wreck(x, y, yaw, color="metal_white", stripe="metal_blue"):
    """Stadtbus: Unterbau, Fensterband mit Holmen, schräge Frontscheibe mit Zielanzeige,
    Türen, Klimaanlage auf dem Dach, Räder mit Felgen und Radkästen."""
    length, width = 11.0, 2.55
    hl = length / 2

    def lb(lx, ly, lz, sx, sy, sz, mat, bevel=0.0):
        local_box(x, y, yaw, lx, ly, lz, sx, sy, sz, mat, collide=False, bevel=bevel)

    lb(0, -0.05, 0.75, width, length - 0.1, 0.8, color, bevel=0.05)  # Unterbau
    lb(0, 0, 0.42, width - 0.1, length - 0.3, 0.12, "metal_dark")  # Schweller
    lb(0, -0.05, 1.08, width + 0.01, length - 0.12, 0.16, stripe)  # Zierstreifen
    lb(0, -0.1, 1.95, width - 0.06, length - 0.4, 1.3, "window_dark")  # Fensterband
    for side in (-1, 1):
        for i in range(9):
            py = -hl + 0.45 + i * (length - 1.4) / 8
            lb(side * (width / 2 - 0.02), py, 1.95, 0.05, 0.14, 1.32, color)
        lb(side * (width / 2 - 0.02), -0.1, 1.27, 0.06, length - 0.4, 0.08, "chrome")
    lb(0, -0.05, 2.78, width, length - 0.1, 0.36, color, bevel=0.04)  # Oberband
    lb(0, -0.05, 2.98, width - 0.1, length - 0.3, 0.08, color, bevel=0.03)  # Dach
    lb(0, 1.6, 3.16, 1.6, 2.4, 0.3, "metal_grey", bevel=0.05)  # Klimaanlage
    lb(0, -4.4, 3.1, 1.2, 1.2, 0.18, "metal_grey")
    # Front: leicht geneigt, große Scheibe, Zielanzeige, Scheinwerfer, Stoßstange
    local_ext(x, y, yaw, [(hl - 0.1, 0.35), (hl, 0.35), (hl + 0.04, 1.2), (hl - 0.02, 2.98), (hl - 0.1, 2.98)], width, color)
    local_ext(x, y, yaw, [(hl + 0.035, 1.25), (hl + 0.07, 1.25), (hl + 0.02, 2.55), (hl - 0.01, 2.55)], width - 0.2, "window_dark")
    lb(0, hl + 0.0, 2.76, 1.7, 0.06, 0.26, "screen_black")
    for side in (-1, 1):
        lb(side * 0.95, hl + 0.06, 0.78, 0.32, 0.04, 0.14, "glass")
        lb(side * 1.38, hl - 0.25, 2.2, 0.08, 0.08, 0.5, "metal_dark")  # Spiegelarm
        lb(side * 1.42, hl - 0.25, 1.9, 0.06, 0.18, 0.32, "metal_dark")  # Spiegel
    lb(0, hl + 0.06, 0.45, width, 0.16, 0.22, "metal_dark", bevel=0.04)
    lb(0, -hl - 0.02, 0.45, width, 0.16, 0.22, "metal_dark", bevel=0.04)
    lb(0, -hl - 0.03, 2.0, width - 0.3, 0.04, 0.9, "window_dark")
    for side in (-1, 1):
        lb(side * 1.0, -hl - 0.04, 0.95, 0.24, 0.04, 0.3, "metal_red")
    # Türen rechts (Glasflügel mit Rahmen)
    for dy in (hl - 1.0, 0.4):
        lb(width / 2 + 0.005, dy, 1.6, 0.03, 1.15, 2.45, "metal_dark")
        for k in (-0.27, 0.27):
            lb(width / 2 + 0.02, dy + k, 1.75, 0.02, 0.48, 1.9, "window_dark")
    # Radkästen und Räder (hinten Zwillingsreifen)
    for ly in (hl - 1.95, -2.9):
        local_cyl(x, y, yaw, 0.0, ly, 0.5, 0.6, width + 0.004, "soot", seg=22)
        for lx in (-(width / 2 - 0.12), width / 2 - 0.12):
            wheel(x, y, yaw, lx, ly, 0.5, 0.32)
    M.collider((x, y, 1.5), (width + 0.1, length + 0.1, 3.0), yaw, F_ALL)


def bus_shelter(x, y, yaw):
    for lx in (-1.6, 1.6):
        for ly in (-0.6, 0.6):
            local_box(x, y, yaw, lx, ly, 1.25, 0.08, 0.08, 2.5, "metal_dark", collide=False)
            dx, dy = rot2(lx, ly, yaw)
            M.collider((x + dx, y + dy, 1.25), (0.12, 0.12, 2.5), yaw, F_PLAYER | F_ZOMBIE)
    local_box(x, y, yaw, 0, 0, 2.55, 3.5, 1.5, 0.08, "metal_grey", collide=False)
    local_box(x, y, yaw, 0, -0.6, 1.5, 3.2, 0.03, 1.6, "glass", collide=False)
    M.collider(
        (x + rot2(0, -0.6, yaw)[0], y + rot2(0, -0.6, yaw)[1], 1.25), (3.2, 0.1, 2.5), yaw, F_PLAYER | F_ZOMBIE
    )
    bench(x + rot2(0, -0.25, yaw)[0], y + rot2(0, -0.25, yaw)[1], yaw, 2.6)


def fence(a, b, h=2.6, posts=2.5, flags=F_PLAYER | F_ZOMBIE):
    ax, ay = a
    bx, by = b
    length = math.hypot(bx - ax, by - ay)
    yaw = math.degrees(math.atan2(by - ay, bx - ax))
    cx, cy = (ax + bx) / 2, (ay + by) / 2
    M.box((cx, cy, h / 2), (length, 0.02, h), "chainlink", yaw=yaw, collide=False)
    n = max(1, int(length / posts))
    for i in range(n + 1):
        t = i / n
        M.cyl((ax + (bx - ax) * t, ay + (by - ay) * t, h / 2 + 0.05), 0.04, h + 0.1, "metal_dark", seg=8, collide=False)
    M.box((cx, cy, h), (length, 0.04, 0.04), "metal_dark", yaw=yaw, collide=False)
    M.collider((cx, cy, h / 2), (length, 0.15, h), yaw, flags)


def rubble(x, y, r=1.5, h=0.8, n=10, flags=F_PLAYER | F_ZOMBIE, collide=True):
    b = M.mb("concrete_dark")
    for i in range(n):
        px = x + rnd.uniform(-r, r) * 0.8
        py = y + rnd.uniform(-r, r) * 0.8
        s = rnd.uniform(0.25, 0.6)
        b.ico((px, py, s * 0.4), s, L["concrete_dark"], subdiv=1, scale=(1.2, 1.0, 0.6), jitter=0.25, seed=i + int(x * 10))
    for i in range(n // 3):
        px = x + rnd.uniform(-r, r) * 0.7
        py = y + rnd.uniform(-r, r) * 0.7
        M.box((px, py, 0.15), (rnd.uniform(0.6, 1.4), 0.12, 0.08), "planks", yaw=rnd.uniform(0, 180), collide=False)
    if collide:
        M.collider((x, y, h / 2), (r * 1.6, r * 1.6, h), 0.0, flags)


def ceiling_lamp(x, y, z, kind="cold", power=False, flicker=0.0, intensity=60000.0, rng=10.0, needs_power=False):
    mat = {"cold": "lamp_cold", "warm": "lamp_warm", "red": "lamp_red"}[kind]
    M.box((x, y, z - 0.05), (1.2, 0.25, 0.08), "metal_grey", collide=False)
    M.box((x, y, z - 0.1), (1.1, 0.18, 0.03), mat, collide=False)
    col = {"cold": (0.82, 0.9, 1.0), "warm": (1.0, 0.78, 0.5), "red": (1.0, 0.15, 0.08)}[kind]
    M.add_light((x, y, z - 0.35), col, intensity, rng, power=power, flicker=flicker, off_without_power=needs_power)


def shelf(x, y, yaw, w=2.0, h=2.0, d=0.5, mat="metal_grey"):
    for lx in (-w / 2, w / 2):
        for ly in (-d / 2, d / 2):
            local_box(x, y, yaw, lx, ly, h / 2, 0.05, 0.05, h, mat, collide=False)
    for k in range(4):
        local_box(x, y, yaw, 0, 0, 0.1 + k * (h - 0.15) / 3, w, d, 0.03, mat, collide=False)
    for k in range(5):
        if rnd.random() < 0.7:
            lx = rnd.uniform(-w / 2 + 0.25, w / 2 - 0.25)
            lev = rnd.randint(0, 2)
            s = rnd.uniform(0.2, 0.35)
            local_box(x, y, yaw, lx, 0, 0.1 + lev * (h - 0.15) / 3 + s / 2 + 0.02, s * 1.3, s, s, "planks", collide=False)
    dx, dy = rot2(0, 0, yaw)
    M.collider((x, y, h / 2), (w + 0.1, d + 0.1, h), yaw, F_PLAYER | F_ZOMBIE)


def table(x, y, yaw=0.0, w=1.2, d=0.8, h=0.75, mat="wood_dark"):
    local_box(x, y, yaw, 0, 0, h, w, d, 0.05, mat, collide=False)
    for lx in (-w / 2 + 0.06, w / 2 - 0.06):
        for ly in (-d / 2 + 0.06, d / 2 - 0.06):
            local_box(x, y, yaw, lx, ly, h / 2, 0.05, 0.05, h, "metal_dark", collide=False)
    M.collider((x, y, h / 2), (w, d, h), yaw, F_PLAYER | F_ZOMBIE)


def locker_row(x, y, yaw, n=4):
    w = 0.45
    for i in range(n):
        lx = (i - (n - 1) / 2) * w
        local_box(x, y, yaw, lx, 0, 0.95, w - 0.02, 0.5, 1.9, "metal_grey", collide=False, bevel=0.01)
        local_box(x, y, yaw, lx, 0.255, 1.55, w - 0.12, 0.01, 0.12, "metal_dark", collide=False)
    M.collider((x, y, 0.95), (n * w, 0.5, 1.9), yaw, F_PLAYER | F_ZOMBIE)


def pallet(x, y, yaw=0.0, z=0.0):
    for k in range(5):
        local_box(x, y, yaw, 0, -0.5 + k * 0.25, z + 0.13, 1.2, 0.1, 0.025, "planks", collide=False)
    for lx in (-0.55, 0, 0.55):
        local_box(x, y, yaw, lx, 0, z + 0.06, 0.1, 1.0, 0.1, "planks", collide=False)


def dead_tree(x, y, h=6.0, seed=0):
    r = random.Random(seed)
    b = M.mb("wood_dark")
    b.cyl((x, y, h / 2), 0.22, h, L["wood_dark"], r2=0.06, seg=8)
    for i in range(5):
        z = h * r.uniform(0.4, 0.85)
        ang = r.uniform(0, 360)
        ln = r.uniform(1.0, 2.2)
        dx, dy = math.cos(math.radians(ang)) * ln, math.sin(math.radians(ang)) * ln
        b.cyl_between((x, y, z), (x + dx, y + dy, z + ln * 0.6), 0.07, L["wood_dark"], seg=6, r2=0.02)
    M.collider((x, y, h / 2), (0.5, 0.5, h), 0.0, F_PLAYER | F_ZOMBIE)


def power_pole(x, y, h=8.0):
    M.cyl((x, y, h / 2), 0.14, h, "wood_dark", seg=8, flags=F_PLAYER | F_ZOMBIE)
    M.box((x, y, h - 0.6), (2.4, 0.14, 0.14), "wood_dark", collide=False)
    for k in (-1.0, 0.0, 1.0):
        M.cyl((x + k, y, h - 0.45), 0.05, 0.2, "glass", seg=6, collide=False)


def transformer(x, y, yaw):
    local_box(x, y, yaw, 0, 0, 1.1, 2.2, 1.6, 2.2, "metal_green", collide=False, bevel=0.03)
    for k in range(7):
        local_box(x, y, yaw, -0.9 + k * 0.3, 0.88, 1.0, 0.04, 0.2, 1.8, "metal_green", collide=False)
        local_box(x, y, yaw, -0.9 + k * 0.3, -0.88, 1.0, 0.04, 0.2, 1.8, "metal_green", collide=False)
    for k in (-0.6, 0.0, 0.6):
        dx, dy = rot2(k, 0, yaw)
        M.cyl((x + dx, y + dy, 2.5), 0.09, 0.6, "glass", seg=8, collide=False)
    local_box(x, y, yaw, 0, -0.81, 1.5, 0.5, 0.01, 0.35, "paint_yellow", collide=False)
    M.collider((x, y, 1.1), (2.4, 2.0, 2.2), yaw, F_ALL)


def workbench(x, y, yaw):
    local_box(x, y, yaw, 0, 0, 0.9, 2.2, 0.8, 0.08, "wood_dark", collide=False)
    local_box(x, y, yaw, 0, 0, 0.45, 2.1, 0.7, 0.04, "wood_dark", collide=False)
    for lx in (-1.0, 1.0):
        for ly in (-0.32, 0.32):
            local_box(x, y, yaw, lx, ly, 0.45, 0.07, 0.07, 0.9, "metal_dark", collide=False)
    local_box(x, y, yaw, 0, -0.38, 1.5, 2.2, 0.04, 1.1, "planks", collide=False)  # Werkzeugwand
    local_box(x, y, yaw, -0.6, 0, 1.0, 0.4, 0.3, 0.15, "metal_red", collide=False)
    M.collider((x, y, 0.5), (2.2, 0.85, 1.0), yaw, F_PLAYER | F_ZOMBIE)


def car_lift(x, y, yaw):
    for lx in (-1.4, 1.4):
        local_box(x, y, yaw, lx, 0, 1.6, 0.3, 0.3, 3.2, "metal_yellow", collide=False)
        dx, dy = rot2(lx, 0, yaw)
        M.collider((x + dx, y + dy, 1.6), (0.35, 0.35, 3.2), yaw, F_ALL)
    local_box(x, y, yaw, 0, 0, 1.9, 2.4, 0.3, 0.1, "metal_yellow", collide=False)
    # Auto auf der Hebebühne (nicht begehbar darunter: Kollision bis 2.9 m)
    local_box(x, y, yaw, 0, 0, 2.3, 1.8, 4.2, 0.6, "metal_blue", collide=False, bevel=0.06)
    local_box(x, y, yaw, 0, -0.3, 2.85, 1.6, 2.0, 0.5, "metal_blue", collide=False, bevel=0.08)
    local_box(x, y, yaw, 0, -0.3, 2.85, 1.62, 1.8, 0.34, "glass", collide=False)
    M.collider((x, y, 2.5), (1.9, 4.3, 1.1), yaw, F_BULLET | F_PLAYER)


def booth(x, y, yaw):
    table(x, y, yaw, 1.0, 0.7, 0.75, "metal_white")
    for side in (-1, 1):
        local_box(x, y, yaw, 0, side * 0.7, 0.25, 1.1, 0.5, 0.5, "fabric_red", collide=False, bevel=0.03)
        local_box(x, y, yaw, 0, side * 0.92, 0.75, 1.1, 0.12, 0.9, "fabric_red", collide=False, bevel=0.03)
    c = abs(math.cos(math.radians(yaw)))
    M.collider((x, y, 0.6), (1.2, 2.0, 1.2), yaw, F_PLAYER | F_ZOMBIE)


def stool(x, y):
    M.cyl((x, y, 0.35), 0.04, 0.7, "metal_dark", seg=8, collide=False)
    M.cyl((x, y, 0.72), 0.19, 0.08, "fabric_red", seg=12, collide=False)


def window_spec(s, w=1.3, lo=0.75, hi=2.15):
    return dict(s=s, w=w, lo=lo, hi=hi, kind="window")
