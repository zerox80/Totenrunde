"""Pfade und Kommandozeilen-Argumente der Asset-Skripte."""

import math
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
ASSETS = os.path.join(ROOT, "assets")
TEX_DIR = os.path.join(ASSETS, "textures")
MODEL_DIR = os.path.join(ASSETS, "models")
UI_DIR = os.path.join(ASSETS, "ui")
MAP_DIR = os.path.join(ASSETS, "maps")
PREVIEW_DIR = os.path.join(ROOT, "tools", "preview")

TAU = math.tau


def script_args():
    if "--" in sys.argv:
        return sys.argv[sys.argv.index("--") + 1 :]
    return []
