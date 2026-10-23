"""Baukasten für Shader-Node-Graphen (für Materialien und gebackene Texturen)."""

import bpy

from .paths import TAU

class NB:
    """Baut Shader-Node-Graphen aus Code. Werte können Floats, Tupel oder Sockets sein."""

    def __init__(self, mat, clear=True):
        self.mat = mat
        self.nt = mat.node_tree
        if clear:
            self.nt.nodes.clear()
        self._uv = None

    # -- Grundlagen --
    def n(self, type_, **props):
        node = self.nt.nodes.new(type_)
        for k, v in props.items():
            setattr(node, k, v)
        return node

    def link(self, out_sock, in_sock):
        self.nt.links.new(out_sock, in_sock)

    def set(self, sock, v):
        if isinstance(v, bpy.types.NodeSocket):
            self.link(v, sock)
        else:
            if isinstance(v, (int, float)) and hasattr(sock, "default_value"):
                dv = sock.default_value
                if hasattr(dv, "__len__"):
                    n = len(dv)
                    v = (float(v),) * n if n != 4 else (float(v), float(v), float(v), 1.0)
            sock.default_value = v

    @staticmethod
    def inp(node, ident):
        for s in node.inputs:
            if s.identifier == ident:
                return s
        return node.inputs[ident]

    @staticmethod
    def out(node, ident):
        for s in node.outputs:
            if s.identifier == ident:
                return s
        return node.outputs[ident]

    # -- Koordinaten --
    def uv(self):
        if self._uv is None:
            tc = self.n("ShaderNodeTexCoord")
            self._uv = tc.outputs["UV"]
        return self._uv

    def uv_xy(self):
        s = self.n("ShaderNodeSeparateXYZ")
        self.link(self.uv(), s.inputs[0])
        return s.outputs["X"], s.outputs["Y"]

    def sep(self, vec):
        s = self.n("ShaderNodeSeparateXYZ")
        self.set(s.inputs[0], vec)
        return s.outputs["X"], s.outputs["Y"], s.outputs["Z"]

    def comb(self, x, y, z):
        c = self.n("ShaderNodeCombineXYZ")
        self.set(c.inputs["X"], x)
        self.set(c.inputs["Y"], y)
        self.set(c.inputs["Z"], z)
        return c.outputs[0]

    # -- Mathe --
    def math(self, op, a, b=None, c=None, clamp=False):
        node = self.n("ShaderNodeMath", operation=op, use_clamp=clamp)
        self.set(node.inputs[0], a)
        if b is not None:
            self.set(node.inputs[1], b)
        if c is not None:
            self.set(node.inputs[2], c)
        return node.outputs[0]

    def add(self, a, b):
        return self.math("ADD", a, b)

    def sub(self, a, b):
        return self.math("SUBTRACT", a, b)

    def mul(self, a, b):
        return self.math("MULTIPLY", a, b)

    def div(self, a, b):
        return self.math("DIVIDE", a, b)

    def madd(self, a, b, c):
        return self.math("MULTIPLY_ADD", a, b, c)

    def pow(self, a, b):
        return self.math("POWER", a, b)

    def floor(self, a):
        return self.math("FLOOR", a)

    def fract(self, a):
        return self.math("FRACT", a)

    def mod(self, a, b):
        return self.math("FLOORED_MODULO", a, b)

    def absv(self, a):
        return self.math("ABSOLUTE", a)

    def minv(self, a, b):
        return self.math("MINIMUM", a, b)

    def maxv(self, a, b):
        return self.math("MAXIMUM", a, b)

    def sin(self, a):
        return self.math("SINE", a)

    def cos(self, a):
        return self.math("COSINE", a)

    def clamp01(self, a):
        return self.math("ADD", a, 0.0, clamp=True)

    def lt(self, a, b):
        return self.math("LESS_THAN", a, b)

    def gt(self, a, b):
        return self.math("GREATER_THAN", a, b)

    def maprange(self, v, a, b, c=0.0, d=1.0, interp="LINEAR", clamp=True):
        node = self.n("ShaderNodeMapRange", data_type="FLOAT", interpolation_type=interp, clamp=clamp)
        self.set(node.inputs["Value"], v)
        self.set(node.inputs["From Min"], a)
        self.set(node.inputs["From Max"], b)
        self.set(node.inputs["To Min"], c)
        self.set(node.inputs["To Max"], d)
        return node.outputs["Result"]

    def smooth(self, v, a, b):
        return self.maprange(v, a, b, 0.0, 1.0, interp="SMOOTHSTEP")

    # -- Farbe --
    def mix(self, fac, a, b, blend="MIX"):
        node = self.n("ShaderNodeMix", data_type="RGBA", blend_type=blend, clamp_result=True)
        self.set(self.inp(node, "Factor_Float"), fac)
        self.set(self.inp(node, "A_Color"), a)
        self.set(self.inp(node, "B_Color"), b)
        return self.out(node, "Result_Color")

    def mixf(self, fac, a, b):
        node = self.n("ShaderNodeMix", data_type="FLOAT")
        self.set(self.inp(node, "Factor_Float"), fac)
        self.set(self.inp(node, "A_Float"), a)
        self.set(self.inp(node, "B_Float"), b)
        return self.out(node, "Result_Float")

    def ramp(self, fac, stops, interp="LINEAR"):
        node = self.n("ShaderNodeValToRGB")
        cr = node.color_ramp
        cr.interpolation = interp
        els = cr.elements
        while len(els) > 1:
            els.remove(els[-1])
        els[0].position = stops[0][0]
        els[0].color = stops[0][1]
        for pos, col in stops[1:]:
            e = els.new(pos)
            e.color = col
        self.set(node.inputs["Fac"], fac)
        return node.outputs["Color"]

    def rgb(self, col):
        node = self.n("ShaderNodeRGB")
        node.outputs[0].default_value = col
        return node.outputs[0]

    def bw(self, col):
        node = self.n("ShaderNodeRGBToBW")
        self.set(node.inputs[0], col)
        return node.outputs[0]

    # -- Kachelbare Rauschfunktionen (4D-Torus-Abbildung der UV-Koordinaten) --
    def torus(self, u, v, fu, fv, off=0.0):
        """Bildet (u,v) in [0,1]^2 periodisch auf einen 4D-Torus ab.

        fu/fv = Anzahl "Rausch-Einheiten" über die Kachel (≈ Strukturgröße)."""
        a = self.mul(u, TAU)
        b = self.mul(v, TAU)
        ru, rv = fu / TAU, fv / TAU
        x = self.madd(self.cos(a), ru, off * 1.0 + 11.3)
        y = self.madd(self.sin(a), ru, off * 1.7 + 3.1)
        z = self.madd(self.cos(b), rv, off * 0.9 + 7.7)
        w = self.madd(self.sin(b), rv, off * 2.3 + 5.2)
        return self.comb(x, y, z), w

    def noise(self, fu, fv=None, detail=4.0, rough=0.55, lac=2.0, distortion=0.0, off=0.0, uv=None, ntype="FBM"):
        fv = fu if fv is None else fv
        u, v = uv if uv is not None else self.uv_xy()
        vec, w = self.torus(u, v, fu, fv, off)
        node = self.n("ShaderNodeTexNoise", noise_dimensions="4D")
        try:
            node.noise_type = ntype
        except Exception:
            pass
        self.set(node.inputs["Vector"], vec)
        self.set(node.inputs["W"], w)
        self.set(node.inputs["Scale"], 1.0)
        self.set(node.inputs["Detail"], detail)
        self.set(node.inputs["Roughness"], rough)
        self.set(node.inputs["Lacunarity"], lac)
        self.set(node.inputs["Distortion"], distortion)
        return node.outputs["Fac"], node.outputs["Color"]

    def voronoi(self, fu, fv=None, feature="F1", metric="EUCLIDEAN", rnd=1.0, off=0.0, uv=None):
        fv = fu if fv is None else fv
        u, v = uv if uv is not None else self.uv_xy()
        vec, w = self.torus(u, v, fu, fv, off)
        node = self.n("ShaderNodeTexVoronoi", voronoi_dimensions="4D", feature=feature, distance=metric)
        self.set(node.inputs["Vector"], vec)
        self.set(node.inputs["W"], w)
        self.set(node.inputs["Scale"], 1.0)
        self.set(node.inputs["Randomness"], rnd)
        outs = {s.identifier: s for s in node.outputs}
        return outs.get("Distance"), outs.get("Color")

    # -- 3D (Objektraum), für nicht kachelnde Texturen auf Modellen --
    def obj_coord(self):
        tc = self.n("ShaderNodeTexCoord")
        return tc.outputs["Object"]

    def vmath(self, op, a, b=None, scale=None):
        node = self.n("ShaderNodeVectorMath", operation=op)
        self.set(node.inputs[0], a)
        if b is not None:
            self.set(node.inputs[1], b)
        if scale is not None:
            self.set(node.inputs[3], scale)
        out = node.outputs["Value"] if op in ("DISTANCE", "LENGTH", "DOT_PRODUCT") else node.outputs["Vector"]
        return out

    def dist(self, a, b):
        return self.vmath("DISTANCE", a, b)

    def vec(self, x, y, z):
        node = self.n("ShaderNodeCombineXYZ")
        node.inputs["X"].default_value = x
        node.inputs["Y"].default_value = y
        node.inputs["Z"].default_value = z
        return node.outputs[0]

    def noise3(self, vec, scale, detail=4.0, rough=0.55, distortion=0.0, lac=2.0):
        node = self.n("ShaderNodeTexNoise", noise_dimensions="3D")
        self.set(node.inputs["Vector"], vec)
        self.set(node.inputs["Scale"], scale)
        self.set(node.inputs["Detail"], detail)
        self.set(node.inputs["Roughness"], rough)
        self.set(node.inputs["Lacunarity"], lac)
        self.set(node.inputs["Distortion"], distortion)
        return node.outputs["Fac"], node.outputs["Color"]

    def voronoi3(self, vec, scale, feature="F1", rnd=1.0):
        node = self.n("ShaderNodeTexVoronoi", voronoi_dimensions="3D", feature=feature)
        self.set(node.inputs["Vector"], vec)
        self.set(node.inputs["Scale"], scale)
        self.set(node.inputs["Randomness"], rnd)
        outs = {o.identifier: o for o in node.outputs}
        return outs.get("Distance"), outs.get("Color")

    def attr(self, name):
        """Float-Attribut des Meshes (z. B. Masken aus der Skulptur)."""
        node = self.n("ShaderNodeAttribute", attribute_type="GEOMETRY", attribute_name=name)
        return node.outputs["Fac"]

    def gauss(self, vec, center, radius):
        """1 im Zentrum, weicher Abfall bis `radius`."""
        d = self.dist(vec, self.vec(*center))
        return self.smooth(d, radius, 0.0)

    def white(self, a, b=0.0, c=0.0):
        node = self.n("ShaderNodeTexWhiteNoise", noise_dimensions="3D")
        self.set(node.inputs["Vector"], self.comb(a, b, c))
        return node.outputs["Value"], node.outputs["Color"]
