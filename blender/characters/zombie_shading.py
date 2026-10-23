"""Back-Shader des hochaufgelösten Zombies.

Die Skulptur trägt je Vertex Material-Masken (t_skin, t_shirt, ...) sowie Felder für Blut und
Schmutz. Daraus baut dieser Shader Farbe, Rauheit/Metall und Relief (Poren, Falten, Gewebe,
Lederkorn), die auf das Spielmodell gebacken werden.

  mode "color":  Emission = Grundfarbe (linear)
  mode "rough":  Emission R = Rauheit, G = Metall
  mode "normal": Principled-BSDF mit Bump (für den Normal-Bake)
"""

from lib import NB, hexcol

C = hexcol

TAGS = ["skin", "shirt", "pants", "shoes", "sole", "flesh", "bone", "teeth", "mouth", "nail", "belt", "buckle", "hair"]


def _skin(nb, p, sk, eyes, kind):
    x, y, z = nb.sep(p)
    n_lo, _ = nb.noise3(p, 2.2, detail=3)
    n_mid, _ = nb.noise3(p, 7.0, detail=6, rough=0.6)
    n_hi, _ = nb.noise3(p, 30.0, detail=4)
    # Grundton: fahl, grau-oliv mit gelblichen und bläulichen Zonen
    col = nb.ramp(n_lo, [(0.3, C("#596048")), (0.5, C("#71695a")), (0.68, C("#5b6364"))])
    col = nb.mix(nb.mul(nb.smooth(n_mid, 0.48, 0.72), 0.4), col, C("#8c8570"))
    col = nb.mix(nb.mul(nb.smooth(n_mid, 0.55, 0.3), 0.55), col, C("#3d4433"))
    col = nb.mix(nb.mul(nb.sub(n_hi, 0.5), 0.3), col, C("#938c76"))
    # Adern: verzweigte, geschlängelte Linien (Nulldurchgänge von verzerrtem Rauschen),
    # nur stellenweise (Schläfen, Hals, Arme) sichtbar
    veins = veins_mask(nb, p)
    col = nb.mix(nb.mul(veins, 0.55), col, C("#3a3150"))
    # Leichenflecken / Blutergüsse
    br, _ = nb.noise3(nb.vmath("ADD", p, (3.1, 1.7, 0.4)), 4.5, detail=4)
    col = nb.mix(nb.mul(nb.smooth(br, 0.58, 0.74), 0.7), col, C("#4a2a36"))
    # Verwesungsflecken (grünlich-schwarz)
    rot, _ = nb.noise3(nb.vmath("ADD", p, (9.2, 4.1, 2.2)), 6.0, detail=5, rough=0.65)
    col = nb.mix(nb.mul(nb.smooth(rot, 0.64, 0.74), 0.75), col, C("#2f3324"))
    # Augenringe
    ring = nb.clamp01(nb.add(nb.gauss(p, tuple(eyes[0]), 0.03), nb.gauss(p, tuple(eyes[1]), 0.03)))
    col = nb.mix(nb.mul(ring, 0.75), col, C("#2e2427"))
    # Lippen dunkel-violett
    hc = sk["hc"]
    hs = sk["hs"]
    mouth = (0.0, hc[1] + 0.095 * hs, hc[2] - 0.1 * hs)
    col = nb.mix(nb.mul(nb.gauss(p, mouth, 0.03 * hs), 0.35), col, C("#4a3236"))
    # Lippenrot (fahl-violett), scharf begrenzt
    mrel = nb.vmath("SUBTRACT", p, (mouth[0], mouth[1], mouth[2] + 0.0004 * hs))
    mx, my, mz = nb.sep(mrel)
    lip_d = nb.vmath("LENGTH", nb.comb(nb.div(mx, 0.0245 * hs), nb.div(mz, 0.0095 * hs), 0.0))
    lips = nb.mul(nb.smooth(lip_d, 1.0, 0.8), nb.smooth(my, -0.012 * hs, -0.004 * hs))
    col = nb.mix(nb.mul(lips, 0.85), col, C("#6a4446"))
    # Rötungen an Nase, Ohren, Augenlidern (dünne Haut, Kapillaren)
    rel = nb.vmath("SUBTRACT", p, tuple(hc))
    rx, ry, rz = nb.sep(rel)
    ax = nb.absv(rx)
    red = nb.clamp01(nb.add(nb.gauss(p, (0.0, hc[1] + 0.1 * hs, hc[2] - 0.065 * hs), 0.025 * hs),
                            nb.mul(nb.smooth(ax, 0.068 * hs, 0.08 * hs), nb.smooth(rz, 0.0, -0.03 * hs))))
    col = nb.mix(nb.mul(red, 0.45), col, C("#6d4440"))
    # Haare: kurz geschoren mit kahlen Stellen; Haaransatz vorn zurückgewichen
    front_line = nb.smooth(rz, (0.062 if kind == "male" else 0.05) * hs, (0.075 if kind == "male" else 0.06) * hs)
    back = nb.mul(nb.smooth(ry, 0.02 * hs, -0.01 * hs), nb.smooth(rz, -0.045 * hs, -0.02 * hs))
    sides = nb.mul(nb.smooth(ry, 0.035 * hs, 0.01 * hs), nb.mul(nb.smooth(rz, -0.03 * hs, -0.005 * hs), nb.smooth(ax, 0.05 * hs, 0.065 * hs)))
    scalp = nb.clamp01(nb.add(nb.add(front_line, back), sides))
    st, _ = nb.noise3(p, 520.0, detail=1)
    patch, _ = nb.noise3(p, 9.0, detail=3)
    hair = nb.mul(scalp, nb.smooth(patch, 0.34 if kind == "male" else 0.2, 0.46 if kind == "male" else 0.3))
    hair_col = C("#16120f") if kind == "male" else C("#22170f")
    col = nb.mix(nb.mul(hair, nb.add(0.5, nb.mul(st, 0.5))), col, hair_col)
    # Augenbrauen: Bogen über den Augen, Härchen schräg nach außen
    bx = nb.div(nb.sub(ax, 0.031 * hs), 0.022 * hs)
    bz = nb.div(nb.sub(rz, nb.add(-0.0035 * hs, nb.mul(nb.mul(bx, bx), -0.0045 * hs))), 0.0068 * hs)
    brow_d = nb.vmath("LENGTH", nb.comb(bx, bz, 0.0))
    front_y = nb.smooth(ry, 0.06 * hs, 0.075 * hs)
    hair_dir = nb.vmath("MULTIPLY", p, (300.0, 900.0, 1400.0))
    hs_n, _ = nb.noise3(hair_dir, 1.0, detail=2)
    brow = nb.mul(nb.mul(nb.smooth(brow_d, 1.0, 0.5), front_y), nb.smooth(hs_n, 0.36, 0.55))
    col = nb.mix(nb.mul(brow, 0.95), col, C("#18130f") if kind == "male" else C("#2a1c13"))
    beard = None
    if kind == "male":
        # Bartschatten: Kiefer, Kinn, Oberlippe, unter dem Kinn
        jaw = nb.mul(nb.smooth(rz, -0.045 * hs, -0.07 * hs), nb.smooth(ry, -0.02 * hs, 0.01 * hs))
        lipzone = nb.gauss(p, (0.0, hc[1] + 0.098 * hs, hc[2] - 0.1 * hs), 0.012 * hs)
        cheek_cut = nb.smooth(nb.sub(rz, nb.mul(ax, -0.6)), -0.03 * hs, -0.05 * hs)
        beard = nb.clamp01(nb.sub(nb.mul(jaw, cheek_cut), nb.mul(lipzone, 1.5)))
        neck_cut = nb.smooth(rz, -0.19 * hs, -0.15 * hs)
        beard = nb.mul(beard, neck_cut)
        dots, _ = nb.noise3(p, 1800.0, detail=1)
        col = nb.mix(nb.mul(beard, nb.add(0.25, nb.mul(dots, 0.3))), col, C("#1d1d22"))
    return col, n_hi, hair


def graph(nb, mode, sk, eyes, kind):
    p = nb.obj_coord()
    x, y, z = nb.sep(p)
    A = {t: nb.attr("t_" + t) for t in TAGS}
    blood = nb.attr("blood")
    dirt = nb.attr("dirt")
    skin_col, skin_n, hair = _skin(nb, p, sk, eyes, kind)

    # --- Stoffe ---
    weave = nb.mul(nb.add(nb.sin(nb.mul(nb.add(x, z), 2400.0)), 1.0), nb.add(nb.sin(nb.mul(nb.sub(y, z), 2400.0)), 1.0))
    weave = nb.mul(weave, 0.25)
    c1, _ = nb.noise3(p, 5.0, detail=5)
    c2, _ = nb.noise3(p, 1.6, detail=4)
    shirt = nb.ramp(c1, [(0.3, C("#b3ada1")), (0.7, C("#cdc7ba"))])
    shirt = nb.mix(nb.mul(weave, 0.18), shirt, C("#8f897d"))
    sweat, _ = nb.noise3(nb.vmath("ADD", p, (1.0, 2.0, 3.0)), 3.0, detail=4)
    shirt = nb.mix(nb.mul(nb.smooth(sweat, 0.58, 0.7), 0.45), shirt, C("#a08f5c"))
    grime = nb.mul(nb.smooth(c2, 0.42, 0.75), nb.add(0.3, nb.mul(nb.smooth(z, 1.3, 0.6), 0.7)))
    shirt = nb.mix(nb.mul(grime, 0.6), shirt, C("#4a4236"))
    # Jeans: Köperbindung (diagonal), ausgeblichen an Knien und Oberschenkeln
    twill = nb.add(nb.mul(nb.sin(nb.mul(nb.add(nb.mul(x, 0.7), z), 2800.0)), 0.5), 0.5)
    pants = nb.mix(nb.mul(twill, 0.25), C("#c4c8cf"), C("#8e939c"))
    fade, _ = nb.noise3(p, 3.0, detail=3)
    pants = nb.mix(nb.mul(nb.smooth(fade, 0.5, 0.75), 0.4), pants, C("#e2e4e6"))
    pants = nb.mix(nb.mul(grime, 0.55), pants, C("#4b453a"))
    # Leder (Stiefel, Gürtel) mit Abrieb
    lg, _ = nb.noise3(p, 220.0, detail=2)
    scuff, _ = nb.noise3(p, 14.0, detail=5)
    leather = nb.ramp(lg, [(0.3, C("#2b1c12")), (0.7, C("#3a271a"))])
    leather = nb.mix(nb.mul(nb.smooth(scuff, 0.58, 0.72), 0.7), leather, C("#6b5038"))
    belt_c = nb.mix(0.4, leather, C("#140d08"))
    sole = C("#151414")
    buckle = C("#8d8a84")
    # Fleisch, Knochen, Zähne, Mundhöhle, Nägel
    fiber_p = nb.vmath("MULTIPLY", p, (60.0, 60.0, 260.0))
    fn, _ = nb.noise3(fiber_p, 1.0, detail=5, rough=0.6, distortion=0.8)
    fn2, _ = nb.noise3(p, 45.0, detail=4)
    flesh = nb.ramp(fn, [(0.25, C("#1e0302")), (0.45, C("#4e0a08")), (0.62, C("#7a1d16")), (0.8, C("#9e3a2a"))])
    flesh = nb.mix(nb.mul(nb.smooth(fn2, 0.66, 0.78), 0.8), flesh, C("#c8a070"))
    bn, _ = nb.noise3(p, 60.0, detail=4)
    bone = nb.ramp(bn, [(0.3, C("#9d8f72")), (0.65, C("#d8ccad"))])
    teeth = nb.ramp(bn, [(0.3, C("#6d5a3c")), (0.6, C("#c9b78f"))])
    mouthc = C("#1c0504")
    nail = nb.ramp(c1, [(0.3, C("#5b5341")), (0.7, C("#9a8c6b"))])
    strand_p = nb.vmath("MULTIPLY", p, (900.0, 900.0, 60.0))
    hs_n, _ = nb.noise3(strand_p, 1.0, detail=3, distortion=0.5)
    hair_c = nb.ramp(hs_n, [(0.3, C("#120c08")), (0.55, C("#2c1d13")), (0.75, C("#4a3423"))])
    hair_c = nb.mix(nb.mul(nb.smooth(c2, 0.5, 0.75), 0.5), hair_c, C("#3a3a30"))

    def compose(base, items):
        out = base
        for tag, val in items:
            out = nb.mix(A[tag], out, val) if mode == "color" else nb.mixf(A[tag], out, val)
        return out

    if mode == "color":
        col = compose(
            skin_col,
            [("shirt", shirt), ("pants", pants), ("shoes", leather), ("sole", sole), ("belt", belt_c), ("buckle", buckle),
             ("flesh", flesh), ("bone", bone), ("teeth", teeth), ("mouth", mouthc), ("nail", nail), ("hair", hair_c)],
        )
        # Blut (frisch dunkelrot, eingetrocknet bräunlich) und Schmutz
        bvar, _ = nb.noise3(p, 18.0, detail=4)
        blood_col = nb.mix(nb.smooth(bvar, 0.4, 0.7), C("#4a0806"), C("#2b0a06"))
        col = nb.mix(nb.clamp01(nb.mul(blood, 1.2)), col, blood_col)
        col = nb.mix(nb.clamp01(nb.mul(dirt, 0.85)), col, C("#3c3127"))
        return col
    if mode == "rough":
        skin_r = nb.add(0.55, nb.mul(nb.sub(skin_n, 0.5), 0.2))
        skin_r = nb.mixf(hair, skin_r, 0.75)
        r = compose(
            skin_r,
            [("shirt", 0.92), ("pants", 0.86), ("shoes", nb.add(0.42, nb.mul(scuff, 0.3))), ("sole", 0.8), ("belt", 0.5),
             ("buckle", 0.35), ("flesh", 0.22), ("bone", 0.45), ("teeth", 0.32), ("mouth", 0.18), ("nail", 0.38),
             ("hair", nb.add(0.55, nb.mul(hs_n, 0.25)))],
        )
        r = nb.mixf(nb.clamp01(blood), r, 0.24)
        r = nb.mixf(nb.clamp01(dirt), r, 0.88)
        metal = nb.mul(A["buckle"], 1.0)
        return nb.comb(r, metal, 0.0)
    # --- Relief für den Normal-Bake ---
    pores, _ = nb.voronoi3(p, 900.0)
    wr, _ = nb.noise3(nb.vmath("MULTIPLY", p, (60.0, 60.0, 260.0)), 1.0, detail=3)  # feine Querfalten
    skin_h = nb.add(nb.mul(pores, 0.35), nb.mul(wr, 0.5))
    skin_h = nb.add(skin_h, nb.mul(wrinkles(nb, p, sk), 1.0 if kind == "male" else 0.35))
    skin_h = nb.add(skin_h, nb.mul(veins_bump(nb, p), 0.5))
    cloth_h = nb.mul(weave, 0.6)
    denim_h = nb.mul(twill, 0.6)
    leather_h = nb.add(nb.mul(lg, 0.5), nb.mul(scuff, 0.4))
    flesh_h = nb.add(nb.mul(fn, 0.8), nb.mul(fn2, 0.3))
    h = compose(
        skin_h,
        [("shirt", cloth_h), ("pants", denim_h), ("shoes", leather_h), ("belt", leather_h), ("flesh", nb.mul(flesh_h, 2.4)),
         ("bone", nb.mul(bn, 0.4)), ("teeth", 0.3), ("mouth", 0.2), ("nail", nb.mul(c1, 0.3)), ("sole", nb.mul(lg, 0.3)),
         ("hair", nb.mul(hs_n, 1.6))],
    )
    bump = nb.n("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 1.0
    bump.inputs["Distance"].default_value = 0.0006
    nb.set(bump.inputs["Height"], h)
    return bump.outputs["Normal"]


def wrinkles(nb, p, sk):
    """Stirnfalten, Krähenfüße und Halsfalten (Höhenwert für den Bump)."""
    hc, hs = sk["hc"], sk["hs"]
    rel = nb.vmath("SUBTRACT", p, tuple(hc))
    rx, ry, rz = nb.sep(rel)
    ax = nb.absv(rx)
    wob, _ = nb.noise3(p, 30.0, detail=2)
    # Stirn: waagerechte, leicht gewellte Linien
    fz = nb.add(rz, nb.mul(nb.sub(wob, 0.5), 0.006))
    fl = nb.mul(nb.add(nb.sin(nb.mul(fz, 2 * 3.14159 / 0.0085)), 1.0), 0.5)
    fmask = nb.mul(nb.mul(nb.smooth(rz, 0.012 * hs, 0.022 * hs), nb.smooth(rz, 0.062 * hs, 0.045 * hs)),
                   nb.mul(nb.smooth(ry, 0.06 * hs, 0.075 * hs), nb.smooth(ax, 0.045 * hs, 0.025 * hs)))
    h = nb.mul(nb.mul(fl, fmask), 1.4)
    # Krähenfüße: Strahlen vom äußeren Augenwinkel
    cx = nb.sub(ax, 0.048 * hs)
    cz = nb.sub(rz, -0.018 * hs)
    ang = nb.math("ARCTAN2", cz, cx)
    rays = nb.mul(nb.add(nb.sin(nb.add(nb.mul(ang, 11.0), nb.mul(wob, 3.0))), 1.0), 0.5)
    rr = nb.vmath("LENGTH", nb.comb(cx, cz, 0.0))
    cmask = nb.mul(nb.mul(nb.smooth(rr, 0.018 * hs, 0.009 * hs), nb.smooth(rr, 0.003 * hs, 0.007 * hs)), nb.smooth(cx, 0.0, 0.006))
    h = nb.add(h, nb.mul(nb.mul(rays, cmask), 0.6))
    # Hals: waagerechte Ringe
    nl = nb.mul(nb.add(nb.sin(nb.mul(nb.add(rz, nb.mul(wob, 0.01)), 2 * 3.14159 / 0.012)), 1.0), 0.5)
    nmask = nb.mul(nb.smooth(rz, -0.2 * hs, -0.17 * hs), nb.smooth(rz, -0.12 * hs, -0.15 * hs))
    return nb.add(h, nb.mul(nb.mul(nl, nmask), 0.9))


def veins_mask(nb, p):
    n, _ = nb.noise3(p, 16.0, detail=3, rough=0.5, distortion=0.6)
    line = nb.smooth(nb.absv(nb.sub(n, 0.5)), 0.02, 0.0)
    n2, _ = nb.noise3(nb.vmath("ADD", p, (5.3, 2.1, 7.7)), 40.0, detail=2, distortion=0.4)
    fine = nb.mul(nb.smooth(nb.absv(nb.sub(n2, 0.5)), 0.015, 0.0), 0.6)
    vm, _ = nb.noise3(p, 3.0, detail=2)
    return nb.mul(nb.clamp01(nb.add(line, fine)), nb.smooth(vm, 0.5, 0.64))


def veins_bump(nb, p):
    return veins_mask(nb, p)


def make_material(name, mode, sk, eyes, kind):
    import bpy

    m = bpy.data.materials.new(name)
    try:
        m.use_nodes = True
    except Exception:
        pass
    nb = NB(m)
    out = nb.n("ShaderNodeOutputMaterial")
    if mode in ("color", "rough"):
        sock = graph(nb, mode, sk, eyes, kind)
        em = nb.n("ShaderNodeEmission")
        nb.set(em.inputs["Color"], sock)
        nb.link(em.outputs[0], out.inputs["Surface"])
    else:
        nrm = graph(nb, "normal", sk, eyes, kind)
        bsdf = nb.n("ShaderNodeBsdfPrincipled")
        nb.link(nrm, bsdf.inputs["Normal"])
        nb.link(bsdf.outputs[0], out.inputs["Surface"])
    return m


def make_direct_material(name, sk, eyes, kind):
    """Principled-Material mit Farbe, Rauheit und Relief direkt aus den Masken (Vorschau)."""
    import bpy

    m = bpy.data.materials.new(name)
    try:
        m.use_nodes = True
    except Exception:
        pass
    nb = NB(m)
    out = nb.n("ShaderNodeOutputMaterial")
    bsdf = nb.n("ShaderNodeBsdfPrincipled")
    nb.set(bsdf.inputs["Base Color"], graph(nb, "color", sk, eyes, kind))
    rm = graph(nb, "rough", sk, eyes, kind)
    r, metal, _ = nb.sep(rm)
    nb.set(bsdf.inputs["Roughness"], r)
    nb.set(bsdf.inputs["Metallic"], metal)
    nb.link(graph(nb, "normal", sk, eyes, kind), bsdf.inputs["Normal"])
    nb.link(bsdf.outputs[0], out.inputs["Surface"])
    return m
