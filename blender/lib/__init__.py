"""Gemeinsame Bibliothek der Blender-Asset-Skripte von Totenrunde.

Konventionen:
  * Blender: Z ist oben. Modelle schauen nach +Y (wird beim glTF-Export zu Bevy -Z = "vorne").
  * Die linke Körperseite einer Figur liegt deshalb bei -X.
  * 1 Blender-Einheit = 1 Meter.
"""

from .colors import hexcol, lin, srgb_to_linear
from .export import export_gltf
from .materials import load_image, material
from .mesh import MB, _mat3
from .nodes import NB
from .paths import ASSETS, MAP_DIR, MODEL_DIR, PREVIEW_DIR, ROOT, TAU, TEX_DIR, UI_DIR, script_args
from .scene import empty, parent_keep, reset_scene, set_active
