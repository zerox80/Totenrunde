"""Back-Shader der Ego-Arme (Hochpoly mit Etiketten als Vertex-Attribute t_<etikett>).

  mode "color":  Emission = Grundfarbe (linear)
  mode "rough":  Emission R = Rauheit, G = Metall
  mode "normal": Principled-BSDF mit Bump (Gewebe, Ripstop, Lederkorn, Nähte)
"""

from lib import NB, hexcol

C = hexcol

TAGS = ["skin", "glove", "leather", "tip", "pad", "strap", "sleeve", "cuff", "watch", "watchface", "watchband"]


def _compose(nb, base, items, A):
    out = base
    for tag, val in items:
        out = nb.mix(A[tag], out, val) if not isinstance(val, float) else nb.mixf(A[tag], out, val)
    return out


def _composef(nb, base, items, A):
    out = base
    for tag, val in items:
        out = nb.mixf(A[tag], out, val)
    return out


def graph(nb, mode):
    p = nb.obj_coord()
    x, y, z = nb.sep(p)
    A = {t: nb.attr("t_" + t) for t in TAGS}
    dirt = nb.attr("dirt")
    blood = nb.attr("blood")

    # Gewebe (Handschuh): feines Nylon-Kreuzgewebe
    weave = nb.mul(nb.add(nb.sin(nb.mul(nb.add(x, z), 2600.0)), 1.0), nb.add(nb.sin(nb.mul(nb.sub(y, z), 2600.0)), 1.0))
    weave = nb.mul(weave, 0.25)
    # Ripstop (Ärmel): Grundgewebe + verstärktes Karo alle 5 mm
    rip_a = nb.smooth(nb.absv(nb.sub(nb.fract(nb.mul(nb.add(y, nb.mul(x, 0.3)), 200.0)), 0.5)), 0.46, 0.5)
    rip_b = nb.smooth(nb.absv(nb.sub(nb.fract(nb.mul(nb.add(z, nb.mul(x, 0.3)), 200.0)), 0.5)), 0.46, 0.5)
    ripstop = nb.clamp01(nb.add(rip_a, rip_b))
    fine = nb.mul(nb.add(nb.sin(nb.mul(nb.add(y, z), 3000.0)), 1.0), 0.5)
    # Lederkorn
    grain, _ = nb.voronoi3(p, 1400.0)
    lg, _ = nb.noise3(p, 260.0, detail=3)
    # Klett / Gurtband: Längsrippen
    web = nb.mul(nb.add(nb.sin(nb.mul(nb.add(x, z), 3000.0)), 1.0), 0.5)
    n_lo, _ = nb.noise3(p, 14.0, detail=4)
    n_mid, _ = nb.noise3(p, 60.0, detail=4)
    wear, _ = nb.noise3(p, 40.0, detail=5, rough=0.6)

    if mode == "color":
        glove = nb.ramp(n_lo, [(0.3, C("#141516")), (0.7, C("#1c1d20"))])
        glove = nb.mix(nb.mul(weave, 0.12), glove, C("#2a2c30"))
        leather = nb.ramp(lg, [(0.3, C("#141110")), (0.7, C("#1f1a16"))])
        leather = nb.mix(nb.mul(nb.smooth(wear, 0.6, 0.8), 0.45), leather, C("#3a3029"))
        tip = nb.mix(nb.mul(nb.smooth(wear, 0.58, 0.78), 0.5), leather, C("#3e342c"))
        pad = nb.ramp(n_mid, [(0.3, C("#101112")), (0.7, C("#1a1b1d"))])
        pad = nb.mix(nb.mul(nb.smooth(wear, 0.6, 0.8), 0.8), pad, C("#45474b"))
        strap = nb.mix(nb.mul(web, 0.3), C("#16171a"), C("#24262a"))
        # Woodland-Tarnmuster (vier Farben, weiche Flecken mit ausgefransten Rändern)
        warp, _ = nb.noise3(p, 90.0, detail=3)
        pw = nb.vmath("ADD", p, nb.comb(nb.mul(warp, 0.004), nb.mul(warp, 0.004), 0.0))
        c1, _ = nb.noise3(pw, 22.0, detail=2, rough=0.4)
        c2, _ = nb.noise3(nb.vmath("ADD", pw, (3.3, 1.1, 7.7)), 26.0, detail=2, rough=0.4)
        c3, _ = nb.noise3(nb.vmath("ADD", pw, (8.1, 5.2, 2.6)), 34.0, detail=2, rough=0.4)
        sleeve = C("#4e5138")
        sleeve = nb.mix(nb.smooth(c1, 0.52, 0.55), sleeve, C("#2f3424"))
        sleeve = nb.mix(nb.smooth(c2, 0.56, 0.59), sleeve, C("#5a4630"))
        sleeve = nb.mix(nb.smooth(c3, 0.62, 0.65), sleeve, C("#16170f"))
        sleeve = nb.mix(nb.mul(ripstop, 0.18), sleeve, C("#22241a"))
        sleeve = nb.mix(nb.mul(nb.smooth(n_lo, 0.55, 0.8), 0.35), sleeve, C("#3a352a"))
        cuff = nb.mix(0.55, sleeve, C("#24261b"))
        watch = nb.ramp(n_mid, [(0.3, C("#141414")), (0.7, C("#1e1e1f"))])
        face = C("#07090b")
        band = nb.mix(nb.mul(web, 0.3), C("#1c1d1a"), C("#2a2b26"))
        col = _compose(nb, glove, [("leather", leather), ("tip", tip), ("pad", pad), ("strap", strap), ("sleeve", sleeve),
                                   ("cuff", cuff), ("watch", watch), ("watchface", face), ("watchband", band)], A)
        bvar, _ = nb.noise3(p, 40.0, detail=4)
        blood_col = nb.mix(nb.smooth(bvar, 0.4, 0.7), C("#3a0605"), C("#200604"))
        col = nb.mix(nb.clamp01(nb.mul(blood, 1.1)), col, blood_col)
        col = nb.mix(nb.clamp01(nb.mul(dirt, 0.7)), col, C("#4a4034"))
        return col
    if mode == "rough":
        r = _composef(nb, nb.add(0.78, nb.mul(weave, 0.1)), [
            ("leather", nb.add(0.5, nb.mul(wear, 0.25))), ("tip", nb.add(0.55, nb.mul(wear, 0.3))), ("pad", 0.42), ("strap", 0.85),
            ("sleeve", 0.9), ("cuff", 0.92), ("watch", 0.35), ("watchface", 0.06), ("watchband", 0.8)], A)
        r = nb.mixf(nb.clamp01(blood), r, 0.3)
        r = nb.mixf(nb.clamp01(dirt), r, 0.92)
        metal = nb.mul(A["watchface"], 0.0)
        return nb.comb(r, metal, 0.0)
    # Relief
    h = _composef(nb, nb.mul(weave, 0.18), [
        ("leather", nb.add(nb.mul(grain, 0.5), nb.mul(lg, 0.3))), ("tip", nb.add(nb.mul(grain, 0.6), nb.mul(lg, 0.3))),
        ("pad", nb.mul(n_mid, 0.25)), ("strap", nb.mul(web, 0.7)), ("sleeve", nb.add(nb.mul(ripstop, 0.6), nb.mul(fine, 0.25))),
        ("cuff", nb.mul(nb.add(nb.sin(nb.mul(y, 2500.0)), 1.0), 0.4)), ("watch", 0.0), ("watchface", 0.0), ("watchband", nb.mul(web, 0.5))], A)
    bump = nb.n("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 1.0
    bump.inputs["Distance"].default_value = 0.00035
    nb.set(bump.inputs["Height"], h)
    return bump.outputs["Normal"]


def make_material(name, mode):
    import bpy

    m = bpy.data.materials.new(name)
    try:
        m.use_nodes = True
    except Exception:
        pass
    nb = NB(m)
    out = nb.n("ShaderNodeOutputMaterial")
    if mode in ("color", "rough"):
        em = nb.n("ShaderNodeEmission")
        nb.set(em.inputs["Color"], graph(nb, mode))
        nb.link(em.outputs[0], out.inputs["Surface"])
    else:
        bsdf = nb.n("ShaderNodeBsdfPrincipled")
        nb.link(graph(nb, "normal"), bsdf.inputs["Normal"])
        nb.link(bsdf.outputs[0], out.inputs["Surface"])
    return m
