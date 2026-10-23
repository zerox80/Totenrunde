#!/usr/bin/env python3
"""Zeigt den Inhalt einer .glb/.gltf-Datei (Szenen, Nodes, Meshes, Animationen)."""
import json, struct, sys

def load(path):
    data = open(path, 'rb').read()
    if path.endswith('.gltf'):
        return json.loads(data)
    magic, ver, length = struct.unpack_from('<III', data, 0)
    clen, ctype = struct.unpack_from('<II', data, 12)
    return json.loads(data[20:20 + clen])

g = load(sys.argv[1])
print('scenes:', [(s.get('name'), len(s.get('nodes', []))) for s in g.get('scenes', [])])
print('nodes:', len(g.get('nodes', [])), [n.get('name') for n in g.get('nodes', [])][:40])
print('meshes:', [(m.get('name'), len(m['primitives'])) for m in g.get('meshes', [])][:40])
print('materials:', [m.get('name') for m in g.get('materials', [])])
print('images:', [i.get('uri', i.get('name')) for i in g.get('images', [])])
print('skins:', [(s.get('name'), len(s['joints'])) for s in g.get('skins', [])])
for a in g.get('animations', []):
    acc = g['accessors'][a['samplers'][0]['input']]
    print('anim:', a.get('name'), 'channels', len(a['channels']), 'len', acc.get('max'))
