import os, sys
import numpy as np
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "blender"))
from textures.synth import read_png, write_png
R = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
ui = os.path.join(R, "assets", "ui")
canvas = np.full((420, 1100, 3), 0.18, np.float32)
def paste(name, x, y, tint=(1, 1, 1), s=1):
    a = read_png(os.path.join(ui, name))[::s, ::s]
    h, w = a.shape[:2]
    rgb = a[..., :3] * np.array(tint)
    al = a[..., 3:4]
    canvas[y:y+h, x:x+w] = canvas[y:y+h, x:x+w] * (1 - al) + rgb * al
red = (0.75, 0.08, 0.06)
for i in range(5):
    paste(f"chalk_tally_{i+1}.png", 10 + i * 205, 10, red)
for d in range(10):
    paste(f"chalk_{d}.png", 10 + d * 105, 170, red)
paste("points_bar.png", 10, 330)
paste("grenade.png", 350, 330)
write_png(os.path.join(R, "tools", "preview", "hud_sheet.png"), canvas)
b = read_png(os.path.join(ui, "blood_overlay.png"))
bg = np.full(b.shape[:2] + (3,), 0.35, np.float32)
write_png(os.path.join(R, "tools", "preview", "blood_prev.png"), (b[..., :3] * b[..., 3:4] + bg * (1 - b[..., 3:4]))[::2, ::2])
