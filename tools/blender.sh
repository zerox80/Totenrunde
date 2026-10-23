#!/usr/bin/env bash
# Startet Blender headless (Flatpak oder native Installation) mit einem Python-Skript.
# Nutzung: tools/blender.sh blender/skript.py [-- argumente...]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$1"; shift || true

if command -v blender >/dev/null 2>&1; then
    exec blender -b --factory-startup --python "$SCRIPT" "$@"
fi

# Das Flatpak-Blender 5.1 bringt eine OCIO-Konfiguration (v2.5) mit, die seine eigene
# OCIO-Bibliothek (2.4) nicht laden kann. Wir legen eine gepatchte Kopie an.
OCIO_DIR="$ROOT/tools/ocio"
if [ ! -f "$OCIO_DIR/config.ocio" ]; then
    mkdir -p "$OCIO_DIR"
    flatpak run --command=sh org.blender.Blender -c \
        "cp -r /app/share/blender/*/datafiles/colormanagement/* '$OCIO_DIR/'"
    sed -i 's/^ocio_profile_version: 2\.5/ocio_profile_version: 2.4/' "$OCIO_DIR/config.ocio"
fi
exec flatpak run --env=OCIO="$OCIO_DIR/config.ocio" org.blender.Blender \
    -b --factory-startup --python "$SCRIPT" "$@"
