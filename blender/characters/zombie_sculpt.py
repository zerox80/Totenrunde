"""Setzt den Zombie als Distanzfeld zusammen: Körper, Kopf, Kleidung, Wunden.

sculpt(kind) -> (sk, sculpt, info); info enthält Augenpositionen und Wundstellen."""

from sculpt.sdf import Sculpt

from . import anatomy, body_sdf, head_sdf


def sculpt(kind="male", clothes=True, wounds=True):
    sk = anatomy.skeleton(kind)
    sc = Sculpt(pad=0.012)
    body_sdf.build(sc, sk)
    head_sdf.build(sc, sk, gaunt=0.65 if kind == "female" else 1.0)
    if kind == "female":
        from . import hair_sdf

        hair_sdf.long_hair(sc, sk)
    eyes, eye_r = head_sdf.eye_centers(sk)
    info = dict(eyes=eyes, eye_r=eye_r, wounds=[])
    if clothes:
        from . import clothing_sdf

        clothing_sdf.build(sc, sk, info)
    if wounds:
        from . import wounds_sdf

        wounds_sdf.build(sc, sk, info)
    return sk, sc, info
