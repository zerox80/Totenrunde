#!/bin/sh
# Legt für den aktuellen Benutzer einen Startmenü-Eintrag für Totenrunde an.
DIR="$(cd "$(dirname "$0")" && pwd)"
APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$APPS"
cat > "$APPS/totenrunde.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Totenrunde
Comment=Zombie-Rundenshooter
Exec="$DIR/Totenrunde"
Path=$DIR
Icon=$DIR/assets/ui/icon.png
Terminal=false
Categories=Game;ActionGame;
StartupWMClass=totenrunde
DESKTOP
echo "Startmenü-Eintrag angelegt: $APPS/totenrunde.desktop"
