"""M1911A1 (Startpistole) – detailliert: Schlitten mit Riffelung und Auswurffenster, Rahmen
mit Abzugsbügel, Griffsicherung, Hahn, Sicherungen, Holzgriffschalen mit Fischhaut,
Laufbuchse und Magazin; beidhändiger Griff mit den Ego-Armen.

Maße an das Original angelehnt (Länge 216 mm), Visierlinie bei z = 0.
"""

from .kit import F, bake_weapon, cut, finish, finish_part, harden, join, load_arms, mb, pose_arm, remove_arms, uv_atlas

# Profilhöhen (z): Schlittenoberkante -0.0075, Schlittenunterkante -0.034, Rahmenunterkante -0.048
TOP = -0.0075
SLIDE_BOT = -0.034


def slide():
    blued = F("blued")
    # Grundkörper als ein geschlossenes Teil (Schnitte nur hier)
    s = mb("SlideBody")
    s.extrude([(-0.036, SLIDE_BOT), (0.172, SLIDE_BOT), (0.172, TOP), (-0.036, TOP)], 0.0236, blued)
    body = s.to_object(smooth_angle=None)
    cuts = mb("_cuts")
    # Fasen an den oberen Längskanten und vorne unten
    for sx in (-1, 1):
        cuts.box((sx * 0.0118, 0.068, TOP), (0.007, 0.26, 0.007), blued, rot=(0, 45, 0))
    cuts.box((0, 0.176, SLIDE_BOT + 0.002), (0.03, 0.012, 0.012), blued, rot=(45, 0, 0))
    # Auswurffenster rechts
    cuts.box((0.012, 0.052, -0.0175), (0.012, 0.042, 0.0105), blued)
    # Griffrillen hinten (beidseitig)
    for i in range(9):
        y = -0.031 + i * 0.0033
        for sx in (-1, 1):
            cuts.box((sx * 0.0122, y, -0.021), (0.0016, 0.0011, 0.022), blued)
    # Kimmenkerbe / Schwalbenschwanz
    cuts.box((0, -0.026, TOP), (0.03, 0.008, 0.0016), blued)
    cut(body, cuts)
    harden(body, width=0.0004)
    d = mb("SlideDetail")
    d.cyl((0, 0.1725, -0.0215), 0.0098, 0.006, blued, axis="Y", seg=28)
    d.cyl((0, 0.176, -0.0215), 0.0063, 0.002, F("steel"), axis="Y", seg=24)
    d.box((0, 0.161, TOP + 0.0028), (0.0032, 0.007, 0.0058), F("black"))
    d.extrude([(-0.032, TOP - 0.0012), (-0.019, TOP - 0.0012), (-0.021, TOP + 0.0048), (-0.032, TOP + 0.0048)], 0.0125, F("black"))
    d.box((0.0121, 0.03, -0.0105), (0.0012, 0.03, 0.004), blued)
    det = finish_part(d, width=0.0003)
    notch = mb("_notch")
    notch.box((0, -0.025, TOP + 0.005), (0.0028, 0.02, 0.0036), blued)
    cut(det, notch)
    return join([body, det], "Slide")


def frame():
    blued = F("blued")
    parts = []
    # Rahmen unter dem Schlitten mit Staubschutz
    a = mb("FrameTop")
    a.extrude([(-0.03, -0.0485), (0.125, -0.0485), (0.125, SLIDE_BOT - 0.0005), (-0.03, SLIDE_BOT - 0.0005)], 0.0222, blued)
    parts.append(finish_part(a, width=0.0004))
    # Griffrahmen (geneigt)
    g = mb("GripFrame")
    g.extrude([(-0.034, -0.046), (0.011, -0.046), (-0.012, -0.142), (-0.058, -0.136), (-0.046, -0.08)], 0.0236, blued)
    parts.append(finish_part(g, width=0.0004))
    # Abzugsbügel: geschlossenes Teil mit Loch
    tg = mb("TriggerGuard")
    tg.extrude([(0.0, -0.0485), (0.052, -0.0485), (0.051, -0.0745), (0.004, -0.0765), (-0.002, -0.068)], 0.0075, blued)
    tgo = tg.to_object(smooth_angle=None)
    hole = mb("_tg")
    hole.extrude([(0.005, -0.052), (0.047, -0.052), (0.046, -0.0715), (0.007, -0.0725), (0.003, -0.066)], 0.02, blued)
    cut(tgo, hole)
    harden(tgo, width=0.0004)
    parts.append(tgo)
    # Kleinteile (ohne Schnitte)
    k = mb("Small")
    k.extrude([(-0.046, -0.04), (-0.034, -0.042), (-0.034, -0.062), (-0.05, -0.06), (-0.056, -0.05)], 0.02, blued)
    k.extrude([(-0.056, -0.085), (-0.046, -0.082), (-0.056, -0.136), (-0.062, -0.135)], 0.021, F("blued_checker"))
    k.extrude([(0.012, -0.05), (0.02, -0.05), (0.018, -0.068), (0.011, -0.068)], 0.006, F("blued_checker"))
    k.extrude([(-0.037, -0.02), (-0.031, -0.03), (-0.036, -0.036), (-0.048, -0.016), (-0.05, -0.008), (-0.044, -0.006)], 0.0062, F("steel"))
    k.extrude([(-0.03, -0.036), (-0.012, -0.034), (-0.014, -0.04), (-0.032, -0.042)], 0.004, blued, offset=-0.0128)
    k.extrude([(0.03, -0.036), (0.062, -0.035), (0.064, -0.041), (0.032, -0.044)], 0.004, blued, offset=-0.0128)
    k.cyl((-0.0128, 0.007, -0.054), 0.004, 0.003, blued, axis="X", seg=16)
    for (y, z) in ((0.034, -0.041), (-0.012, -0.039), (-0.026, -0.04), (-0.047, -0.11)):
        k.cyl((0, y, z), 0.0018, 0.0238, F("steel"), axis="X", seg=10)
    parts.append(finish_part(k, width=0.0003))
    return join(parts, "Frame")


def grips():
    g = mb("Grips")
    # Holzschalen mit Fischhaut (beidseitig), Schraubenköpfe
    pts = [(-0.031, -0.056), (0.006, -0.056), (-0.013, -0.128), (-0.051, -0.123), (-0.042, -0.07)]
    for sx in (-1, 1):
        g.extrude(pts, 0.004, F("wood_checker"), offset=sx * 0.0135)
        for z in (-0.068, -0.115):
            g.cyl((sx * 0.0157, -0.016 + (z + 0.068) * 0.25, z), 0.0026, 0.0012, F("steel"), axis="X", seg=12)
    return finish_part(g, width=0.0006, angle=40.0)


def barrel():
    b = mb("Barrel")
    b.cyl((0, 0.075, -0.0215), 0.0085, 0.2, F("steel"), axis="Y", seg=24)
    return finish_part(b, width=0.0003)


def magazine():
    m = mb("Mag")
    m.extrude([(-0.03, -0.06), (0.002, -0.06), (-0.016, -0.143), (-0.05, -0.138)], 0.0205, F("blued"))
    m.extrude([(-0.053, -0.137), (-0.012, -0.143), (-0.012, -0.149), (-0.054, -0.143)], 0.0225, F("black"))
    return finish_part(m, width=0.0004)


def build():
    sl = slide()
    fr = frame()
    gr = grips()
    br = barrel()
    mg = magazine()
    body = join([fr, gr, br], "Body")
    uv_atlas([body, sl, mg])
    arms = load_arms()
    # Rechte Hand: Handfläche an der rechten Griffseite, Handachse schräg nach vorn oben
    right = pose_arm(
        "R", arms,
        wrist=(0.024, -0.09, -0.074), a=(-0.12, 0.95, -0.3), n=(-1.0, 0.0, 0.0),
        elbow=(0.17, -0.4, -0.3), targets=[body, mg],
        curls={"index": (18, 30, 12)},
    )
    # Linke Hand: umschließt die rechte (Beidhandgriff), Daumen nach vorn
    left = pose_arm(
        "L", arms,
        wrist=(-0.056, -0.062, -0.158), a=(0.22, 0.85, 0.45), n=(0.85, 0.0, 0.5),
        elbow=(-0.22, -0.38, -0.34), targets=[body, mg, right],
    )
    remove_arms(arms)
    body = join([body, right], "Body")
    left.name = "ArmL"
    left.data.name = "ArmL"
    bake_weapon("m1911", [body, sl, mg, left])
    finish("m1911", (0, 0.18, -0.0215))
