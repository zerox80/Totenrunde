"""Farben: Blender-Sockets erwarten lineare Werte, Farben werden hier als sRGB-Hex notiert."""

def srgb_to_linear(c):
    if c <= 0.04045:
        return c / 12.92
    return ((c + 0.055) / 1.055) ** 2.4


def hexcol(h, alpha=1.0):
    """'#rrggbb' (sRGB) -> lineares RGBA-Tupel für Blender-Sockets."""
    h = h.lstrip("#")
    r, g, b = (int(h[i : i + 2], 16) / 255.0 for i in (0, 2, 4))
    return (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), alpha)


def lin(r, g, b, a=1.0):
    return (r, g, b, a)
