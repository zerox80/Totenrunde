"""Vernetzer: wertet ein Sculpt-Objekt dünn besetzt aus (nur Blöcke nahe der Oberfläche)
und erzeugt per Surface Nets ein geschlossenes Quad-Mesh.

Gitter: Knoten im Abstand h; je 8×8×8 Zellen bilden einen Block. Zuerst wird jede
Blockmitte ausgewertet; nur Blöcke, deren Mitte nahe der Oberfläche liegt, werden fein
berechnet. Blöcke werden räumlich gruppiert, damit pro Gruppe nur die Operationen laufen,
die dort wirken können."""

import time

import numpy as np

B = 8  # Zellen pro Block und Achse

# Ecken einer Zelle und ihre 12 Kanten
_CORNERS = np.array([[i, j, k] for i in (0, 1) for j in (0, 1) for k in (0, 1)])
_EDGES = [
    (a, b)
    for a in range(8)
    for b in range(a + 1, 8)
    if np.abs(_CORNERS[a] - _CORNERS[b]).sum() == 1
]


def _lin(g, dims):
    return (g[:, 0] * dims[1] + g[:, 1]) * dims[2] + g[:, 2]


def _group(ab, group):
    sk = ab // group
    order = np.lexsort((sk[:, 2], sk[:, 1], sk[:, 0]))
    sk = sk[order]
    starts = np.nonzero(np.any(np.diff(sk, axis=0) != 0, axis=1))[0] + 1
    return order, np.split(np.arange(len(ab)), starts)


def _evaluate(sculpt, ab, lo, h, group):
    """Knotenwerte (n, B+1, B+1, B+1) für die Blöcke ab."""
    off = np.stack(np.meshgrid(*[np.arange(B + 1)] * 3, indexing="ij"), -1).reshape(-1, 3)
    order, groups = _group(ab, group)
    D = np.empty((len(ab), B + 1, B + 1, B + 1), dtype=np.float32)
    for gi in groups:
        idx = order[gi]
        g = (ab[idx][:, None, :] * B + off[None]).reshape(-1, 3)
        D[idx] = sculpt.eval(lo + g * h).reshape(len(idx), B + 1, B + 1, B + 1)
    return D


def _nets(ab, D, lo, h, dims):
    """Surface Nets über die ausgewerteten Blöcke. Liefert verts, quads und die Blöcke,
    die für verlorene Quads fehlen."""
    inside = D < 0
    cnt = np.zeros((len(ab), B, B, B), dtype=np.int8)
    for c in _CORNERS:
        cnt += inside[:, c[0] : c[0] + B, c[1] : c[1] + B, c[2] : c[2] + B]
    mixed = (cnt > 0) & (cnt < 8)
    bidx, ci, cj, ck = np.nonzero(mixed)
    acc = np.zeros((len(bidx), 3))
    num = np.zeros(len(bidx))
    for a, b in _EDGES:
        ca, cb = _CORNERS[a], _CORNERS[b]
        da = D[bidx, ci + ca[0], cj + ca[1], ck + ca[2]].astype(np.float64)
        db = D[bidx, ci + cb[0], cj + cb[1], ck + cb[2]].astype(np.float64)
        cross = (da < 0) != (db < 0)
        t = np.where(cross, da / np.where(cross, da - db, 1.0), 0.0)
        p = ca[None, :] + t[:, None] * (cb - ca)[None, :]
        acc += np.where(cross[:, None], p, 0.0)
        num += cross
    vloc = acc / np.maximum(num, 1)[:, None]
    cg = ab[bidx] * B + np.stack([ci, cj, ck], axis=1)
    verts = lo + (cg + vloc) * h
    cid = _lin(cg, dims)
    sort = np.argsort(cid)
    cid_sorted = cid[sort]
    # Zellen-Offsets je Achse in der Reihenfolge, die eine nach außen zeigende Normale ergibt
    cell_off = {
        0: [(0, -1, -1), (0, 0, -1), (0, 0, 0), (0, -1, 0)],
        1: [(-1, 0, -1), (-1, 0, 0), (0, 0, 0), (0, 0, -1)],
        2: [(-1, -1, 0), (0, -1, 0), (0, 0, 0), (-1, 0, 0)],
    }
    quads, missing = [], []
    for axis in range(3):
        sl1 = [slice(None), slice(0, B), slice(0, B), slice(0, B)]
        sl1[axis + 1] = slice(1, B + 1)
        n0 = D[:, :B, :B, :B]
        n1 = D[tuple(sl1)]
        cross = (n0 < 0) != (n1 < 0)
        b2, i2, j2, k2 = np.nonzero(cross)
        if len(b2) == 0:
            continue
        ins0 = n0[b2, i2, j2, k2] < 0
        g = ab[b2] * B + np.stack([i2, j2, k2], axis=1)
        ids = []
        ok = np.ones(len(b2), dtype=bool)
        for o in cell_off[axis]:
            c = g + np.array(o)
            lid = _lin(c, dims)
            pos = np.minimum(np.searchsorted(cid_sorted, lid), len(cid_sorted) - 1)
            found = cid_sorted[pos] == lid
            if (~found).any():
                missing.append(c[~found] // B)
            ok &= found
            ids.append(sort[pos])
        q = np.stack(ids, axis=1)[ok]
        ins0 = ins0[ok]
        q[~ins0] = q[~ins0][:, ::-1]
        quads.append(q)
    quads = np.concatenate(quads) if quads else np.zeros((0, 4), dtype=np.int64)
    missing = np.unique(np.concatenate(missing), axis=0) if missing else np.zeros((0, 3), dtype=np.int64)
    return verts, quads, missing


def surface_nets(sculpt, lo, hi, h, log=print, group=4, margin=1.6):
    """Liefert (verts (N,3) float64, quads (M,4) int64). lo/hi: Hüllbox in Metern."""
    t0 = time.time()
    lo = np.asarray(lo, dtype=np.float64)
    hi = np.asarray(hi, dtype=np.float64)
    nb = np.ceil((hi - lo) / (h * B)).astype(np.int64) + 1
    dims = nb * B + 1  # Knoten
    # 1) Blockmitten
    bi = np.stack(np.meshgrid(*[np.arange(n) for n in nb], indexing="ij"), -1).reshape(-1, 3)
    centers = lo + (bi + 0.5) * h * B
    dc = sculpt.eval_chunked(centers)
    thr = h * B * np.sqrt(3) / 2 * margin + 0.004
    ab = bi[np.abs(dc) < thr]
    # 2) fein auswerten; fehlende Nachbarblöcke nachladen, bis nichts mehr verloren geht
    D = _evaluate(sculpt, ab, lo, h, group)
    for it in range(4):
        verts, quads, missing = _nets(ab, D, lo, h, dims)
        if len(missing) == 0:
            break
        valid = np.all((missing >= 0) & (missing < nb), axis=1)
        missing = missing[valid]
        known = set(map(tuple, ab))
        missing = np.array([m for m in map(tuple, missing) if m not in known], dtype=np.int64).reshape(-1, 3)
        if len(missing) == 0:
            break
        log(f"  +{len(missing)} Nachbarblöcke")
        D = np.concatenate([D, _evaluate(sculpt, missing, lo, h, group)])
        ab = np.concatenate([ab, missing])
    log(
        f"  Gitter {tuple(int(x) for x in dims)} h={h * 1000:.2f} mm, {len(ab)} Blöcke, "
        f"{len(verts)} Vertices, {len(quads)} Quads in {time.time() - t0:.1f} s"
    )
    return verts, quads


def compact(verts, faces):
    """Entfernt unbenutzte Vertices."""
    used = np.zeros(len(verts), dtype=bool)
    used[faces.ravel()] = True
    remap = np.cumsum(used) - 1
    return verts[used], remap[faces]


def relax(verts, quads, sculpt, iters=2, lam=0.5):
    """Laplace-Glättung mit Rückprojektion auf die Fläche (gleichmäßigere Quads)."""
    n = len(verts)
    for _ in range(iters):
        acc = np.zeros_like(verts)
        cnt = np.zeros(n)
        for a in range(4):
            b = (a + 1) % 4
            np.add.at(acc, quads[:, a], verts[quads[:, b]])
            np.add.at(acc, quads[:, b], verts[quads[:, a]])
            np.add.at(cnt, quads[:, a], 1)
            np.add.at(cnt, quads[:, b], 1)
        avg = acc / np.maximum(cnt, 1)[:, None]
        verts = verts + lam * (avg - verts)
        d = sculpt.eval_chunked(verts)
        g = sculpt.gradient(verts)
        verts = verts - d[:, None] * g
    return verts


def to_blender(name, verts, faces, collection=None):
    """Erzeugt ein Blender-Mesh-Objekt aus numpy-Arrays (Quads oder Dreiecke)."""
    import bpy

    me = bpy.data.meshes.new(name)
    nv, nf, k = len(verts), len(faces), faces.shape[1]
    me.vertices.add(nv)
    me.vertices.foreach_set("co", verts.astype(np.float32).ravel())
    me.loops.add(nf * k)
    me.loops.foreach_set("vertex_index", faces.astype(np.int32).ravel())
    me.polygons.add(nf)
    me.polygons.foreach_set("loop_start", np.arange(0, nf * k, k, dtype=np.int32))
    try:
        me.polygons.foreach_set("loop_total", np.full(nf, k, dtype=np.int32))
    except (AttributeError, RuntimeError, TypeError):
        pass
    me.update(calc_edges=True)
    me.validate(clean_customdata=False)
    me.shade_smooth()
    obj = bpy.data.objects.new(name, me)
    (collection or bpy.context.scene.collection).objects.link(obj)
    return obj


def set_attr(obj, name, values, domain="POINT"):
    """Float-Attribut (je Vertex) setzen – für Masken im Back-Shader."""
    me = obj.data
    if name in me.attributes:
        me.attributes.remove(me.attributes[name])
    a = me.attributes.new(name, "FLOAT", domain)
    a.data.foreach_set("value", np.asarray(values, dtype=np.float32))
