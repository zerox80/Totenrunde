#!/usr/bin/env python3
"""Erzeugt verlustfreie, größenangepasste Texturvarianten für die Web-Auslieferung.

Das Spiel verkleinert große Texturen beim Laden auf der CPU (src/base/textures.rs).
Im Browser blockiert das den Hauptthread: Über 500 MB Texturen werden heruntergeladen,
dekodiert und danach oft auf 1024 oder 512 heruntergerechnet, obwohl die kleineren
Pixel nie auf dem Bildschirm landen. Dieses Werkzeug rechnet dieselbe Verkleinerung
vorab aus und legt sie unter textures/<größe>/ ab. Der Browser lädt dann nur noch die
Auflösung, die die Grafikstufe tatsächlich anzeigt.

Die Verkleinerung entspricht der Laufzeitlogik (2×2-Box-Filter, Farben in linearem
Licht bei sRGB-Texturen, Normal-/MR-Maps linear). Bei 1024/512 sieht das Ergebnis also
genauso aus wie bisher nach dem Herunterrechnen im Browser. Die Originale bleiben
unverändert, sofern --recompress-originals nicht gesetzt ist; auch dort wird nur
verlustfrei neu komprimiert.

Beispiel:
    python3 backend/optimize-textures.py --root assets --sizes 512 1024 \
        --recompress-originals

Voraussetzungen: Pillow und NumPy (pip install pillow numpy).
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    import numpy as np
    from PIL import Image
except ImportError as error:  # pragma: no cover - nur im Docker-/CI-Build relevant
    sys.exit(
        f"Fehlende Abhängigkeit: {error.name}. Bitte 'pip install pillow numpy' ausführen."
    )

# Lightmaps verkleinert die Laufzeit nicht; sie bekommen keine Varianten.
SKIP_PREFIX = ("lm_",)


def is_linear(name: str) -> bool:
    """Normal- und Metallic-Roughness-Maps lädt Bevy linear, alle anderen sRGB."""
    return name.endswith("_n.png") or name.endswith("_mr.png")


def srgb_tables() -> tuple[np.ndarray, np.ndarray]:
    """Tabellen wie in src/base/textures.rs::SrgbConv."""
    x = (np.arange(256, dtype=np.float32) / np.float32(255.0)).astype(np.float32)
    to_lin = np.where(
        x <= np.float32(0.04045),
        x / np.float32(12.92),
        ((x + np.float32(0.055)) / np.float32(1.055)) ** np.float32(2.4),
    ).astype(np.float32)
    k = (
        (np.arange(255, dtype=np.float32) + np.float32(0.5)) / np.float32(255.0)
    ).astype(np.float32)
    bounds = np.where(
        k <= np.float32(0.04045),
        k / np.float32(12.92),
        ((k + np.float32(0.055)) / np.float32(1.055)) ** np.float32(2.4),
    ).astype(np.float32)
    return to_lin, bounds


def halve(array: np.ndarray, linear: bool, to_lin: np.ndarray, bounds: np.ndarray) -> np.ndarray:
    """2×2-Box wie die Laufzeit: Farbe linear (bzw. roh bei linearen Texturen), Alpha ganzzahlig."""
    height, width = array.shape[:2]
    new_w = max(width // 2, 1)
    new_h = max(height // 2, 1)
    x0 = np.minimum(np.arange(new_w) * 2, width - 1)
    x1 = np.minimum(x0 + 1, width - 1)
    y0 = np.minimum(np.arange(new_h) * 2, height - 1)
    y1 = np.minimum(y0 + 1, height - 1)
    quads = (
        array[np.ix_(y0, x0)],
        array[np.ix_(y0, x1)],
        array[np.ix_(y1, x0)],
        array[np.ix_(y1, x1)],
    )
    channels = array.shape[2]
    out = np.empty((new_h, new_w, channels), dtype=np.uint8)

    color_channels = 3 if channels >= 3 else 1
    color = np.zeros((new_h, new_w, color_channels), dtype=np.float32)
    for quad in quads:
        values = quad[:, :, :color_channels].astype(np.float32)
        # Wie die Laufzeit: lineare Daten als 0..1 mitteln, sRGB über die Nachschlagetabelle.
        color += values / np.float32(255.0) if linear else to_lin[values.astype(np.uint8)]
    color *= np.float32(0.25)
    if linear:
        out[:, :, :color_channels] = (
            np.clip(color, 0.0, 1.0) * np.float32(255.0) + np.float32(0.5)
        ).astype(np.uint8)
    else:
        out[:, :, :color_channels] = np.searchsorted(bounds, color, side="right").astype(
            np.uint8
        )

    alpha_channel = 3 if channels == 4 else 1 if channels == 2 else None
    if alpha_channel is not None:
        alpha = np.zeros((new_h, new_w), dtype=np.uint32)
        for quad in quads:
            alpha += quad[:, :, alpha_channel].astype(np.uint32)
        out[:, :, alpha_channel] = ((alpha + 2) // 4).astype(np.uint8)
    return out


def shrink(
    array: np.ndarray,
    target: int,
    linear: bool,
    to_lin: np.ndarray,
    bounds: np.ndarray,
) -> np.ndarray:
    """Verkleinert wie die Laufzeit: solange eine Kante zu groß ist, halbieren."""
    while max(array.shape[0], array.shape[1]) > target:
        array = halve(array, linear, to_lin, bounds)
    return array


def has_opaque_alpha(array: np.ndarray) -> bool:
    return array.ndim == 3 and array.shape[2] == 4 and int(array[:, :, 3].min()) == 255


def write_png(array: np.ndarray, path: Path) -> int:
    """Schreibt PNG verlustfrei; gibt die Dateigröße zurück."""
    if array.ndim == 2:
        image = Image.fromarray(array, "L")
    elif array.shape[2] == 4:
        image = Image.fromarray(array, "RGBA")
        if has_opaque_alpha(array):
            image = image.convert("RGB")
    elif array.shape[2] == 2:
        image = Image.fromarray(array, "LA")
    else:
        image = Image.fromarray(array[:, :, :3], "RGB")
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path, format="PNG", optimize=True, compress_level=9)
    return path.stat().st_size


def load_pixels(path: Path) -> np.ndarray:
    image = Image.open(path)
    image.load()
    if image.mode not in ("L", "LA", "RGB", "RGBA"):
        image = image.convert("RGBA")
    # RGB mit Transparenzfarbe braucht den Alphakanal, sonst ginge sie verloren.
    if image.mode == "RGB" and "transparency" in image.info:
        image = image.convert("RGBA")
    return np.asarray(image, dtype=np.uint8)


def replace_if_smaller(array: np.ndarray, path: Path, current: int) -> int:
    temporary = path.with_name(path.name + ".tmp")
    written = write_png(array, temporary)
    if written < current:
        temporary.replace(path)
        return written
    temporary.unlink()
    return current


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=Path("assets"), help="Asset-Verzeichnis")
    parser.add_argument(
        "--sizes",
        nargs="+",
        type=int,
        default=[512, 1024],
        help="Zielauflösungen (Standard: 512 1024)",
    )
    parser.add_argument(
        "--recompress-originals",
        action="store_true",
        help="Originale zusätzlich verlustfrei neu komprimieren (nur wenn kleiner)",
    )
    parser.add_argument(
        "--force", action="store_true", help="vorhandene Varianten überschreiben"
    )
    args = parser.parse_args()

    textures = args.root / "textures"
    if not textures.is_dir():
        sys.exit(f"Texturverzeichnis nicht gefunden: {textures}")
    sizes = sorted({size for size in args.sizes if size > 0})
    to_lin, bounds = srgb_tables()

    # Bereits erzeugte Varianten (textures/512/…, textures/1024/…) nicht erneut verarbeiten.
    originals = sorted(
        path
        for path in textures.rglob("*.png")
        if path.is_file() and not path.relative_to(textures).parts[0].isdigit()
    )
    original_now = original_after = variants = variant_bytes = 0
    for index, path in enumerate(originals, 1):
        relative = path.relative_to(textures)
        array = load_pixels(path)
        size_now = path.stat().st_size
        original_now += size_now
        if args.recompress_originals:
            original_after += replace_if_smaller(array, path, size_now)
        else:
            original_after += size_now

        if relative.name.startswith(SKIP_PREFIX):
            continue
        linear = is_linear(relative.name)
        for size in sizes:
            if max(array.shape[0], array.shape[1]) > size:
                variant = shrink(array, size, linear, to_lin, bounds)
            else:
                variant = array
            target = textures / str(size) / relative
            if target.exists() and not args.force:
                variant_bytes += target.stat().st_size
            else:
                variant_bytes += write_png(variant, target)
            variants += 1

        if index % 20 == 0:
            print(f"  {index}/{len(originals)}: {relative}", flush=True)

    print(
        f"Originale: {original_now / 1e6:.0f} MB -> {original_after / 1e6:.0f} MB "
        f"({'neu komprimiert' if args.recompress_originals else 'unverändert'})"
    )
    print(
        f"Varianten: {variants} Dateien, {variant_bytes / 1e6:.0f} MB "
        f"(Größen: {', '.join(str(size) for size in sizes)})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
