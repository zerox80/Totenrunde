"""Erzeugt die PBR-Texturen der Karte (numpy) nach assets/textures:
<name>.png (Farbe, sRGB), <name>_n.png (Normalen), <name>_mr.png (R=Kavität, G=Rauheit, B=Metall)
und eine Übersicht tools/preview/pbr_sheet.png.

Aufruf: tools/blender.sh blender/textures/build_pbr.py [-- name1 name2 ...]
"""

import os
import sys
import zlib
import time

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import numpy as np  # noqa: E402
from lib.paths import ROOT, TEX_DIR  # noqa: E402
from textures.pbr import RECIPES  # noqa: E402
from textures.synth import Canvas, cavity, normal_from_height, write_png  # noqa: E402


def args():
    if "--" in sys.argv:
        return sys.argv[sys.argv.index("--") + 1 :]
    return []


def build(name, spec, quick=False):
    size = 512 if quick else spec["size"]
    cv = Canvas(size, seed=zlib.crc32(name.encode()) % 10000)
    r = spec["fn"](cv)
    c = np.clip(r["color"], 0, 1)
    h = r["height"].astype(np.float32)
    h = (h - h.min()) / max(h.max() - h.min(), 1e-6)
    cav = np.clip(cavity(cv, h, size / 256.0) * 6.0, 0, 1)
    c = c * (1.0 - 0.35 * cav[..., None])
    rough = np.clip(np.broadcast_to(r["rough"], h.shape), 0.03, 1.0)
    metal = np.clip(np.broadcast_to(r["metal"], h.shape), 0.0, 1.0)
    nrm = normal_from_height(h, r.get("normal", 4.0) * size / 512.0)
    alpha = r.get("alpha")
    if quick:
        return c, nrm, rough
    rgba = np.dstack([c, alpha if alpha is not None else np.ones_like(h)])
    write_png(os.path.join(TEX_DIR, name + ".png"), rgba if alpha is not None else c, srgb=True)
    write_png(os.path.join(TEX_DIR, name + "_n.png"), nrm)
    write_png(os.path.join(TEX_DIR, name + "_mr.png"), np.dstack([1.0 - cav, rough, metal]))
    return c, nrm, rough


def shade(c, nrm, rough):
    """Schnelle Vorschau: Streiflicht von links oben."""
    n = nrm * 2 - 1
    L = np.array([-0.55, 0.55, 0.63])
    L /= np.linalg.norm(L)
    d = np.clip((n * L).sum(-1), 0, 1)
    spec = np.clip(n[..., 2], 0, 1) ** (2 + (1 - rough) * 60) * (1 - rough) * 0.6
    return np.clip(c * (0.25 + 0.95 * d[..., None]) + spec[..., None], 0, 1)


def main():
    wanted = [a for a in args() if not a.startswith("-")]
    quick = "--quick" in args() or "quick" in wanted
    wanted = [w for w in wanted if w != "quick"]
    tiles = []
    for name, spec in RECIPES.items():
        if wanted and name not in wanted:
            continue
        t0 = time.time()
        c, nrm, rough = build(name, spec, quick)
        s = c.shape[0] // 256
        th = shade(c, nrm, rough)[::s, ::s][:256, :256]
        tiles.append(th)
        print(f"PBR {name}: {time.time() - t0:.1f}s", flush=True)
    if tiles:
        cols = 6
        rows = (len(tiles) + cols - 1) // cols
        sheet = np.zeros((rows * 260, cols * 260, 3), np.float32)
        for i, t in enumerate(tiles):
            y, x = divmod(i, cols)
            sheet[y * 260 : y * 260 + 256, x * 260 : x * 260 + 256] = t
        out = os.path.join(ROOT, "tools", "preview")
        os.makedirs(out, exist_ok=True)
        write_png(os.path.join(out, "pbr_sheet.png"), sheet, srgb=True)


main()
