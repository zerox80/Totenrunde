"""Register der Texturen: jede Textur ist eine Funktion, die einen Shader-Node-Graphen
baut und ein dict mit `color`, optional `height` und `alpha` (Sockets) zurückgibt."""

TEXTURES = {}


def texture(name, size=1024, normal_strength=4.0, samples=4):
    def deco(fn):
        TEXTURES[name] = dict(fn=fn, size=size, normal=normal_strength, samples=samples)
        return fn

    return deco
