"""Kontaktbogen aus Vorschaubildern: python3 tools/inspect/sheet.py out.png h a.png b.png ..."""
import sys
from PIL import Image
out, h = sys.argv[1], int(sys.argv[2])
ims = [Image.open(p).convert("RGB") for p in sys.argv[3:]]
ims = [i.resize((int(i.width * h / i.height), h)) for i in ims]
sheet = Image.new("RGB", (sum(i.width for i in ims), h))
x = 0
for i in ims:
    sheet.paste(i, (x, 0))
    x += i.width
sheet.save(out)
