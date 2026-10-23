"""Alle Requisiten bauen: Perk-Automaten, Kiste, Deckel, Teddy, Verstärker, Stromkasten,
Power-ups. Aufruf: tools/blender.sh blender/props/build.py [-- perks|box|machines|powerups]"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from lib import script_args  # noqa: E402
from props import machines, mystery_box, perk_machines, powerups  # noqa: E402

ARGS = script_args()
parts = {"perks": perk_machines.build_all, "box": mystery_box.build_all, "machines": machines.build_all, "powerups": powerups.build_all}
for key, fn in parts.items():
    if not ARGS or key in ARGS:
        print("=== Baue", key)
        fn()
