# Totenrunde

Ein rundenbasierter Zombie-Überlebensmodus im Stil von *Black Ops 2*, geschrieben in **Rust** mit
**Bevy 0.20**. Karte „Endstation“: ein verlassener Busbahnhof bei Nacht mit Wartehalle, Hof,
Werkstatt, Diner und Umspannwerk.

Alle Inhalte sind selbst erzeugt. Es gibt keine gekauften oder heruntergeladenen Assets
(einzige Ausnahme: die Schrift Montserrat, OFL-Lizenz).

- **Modelle** entstehen per Python-Skript in Blender. Die Zombies und die Ego-Arme sind per
  SDF modelliert, dann gerigged und gebacken. Waffen und Requisiten sind Hard-Surface-Modelle
  mit Fasen und gebackenen PBR-Texturen.
- **Texturen** werden prozedural mit numpy erzeugt (Farbe, Normalen, Rauheit/Metall), dazu ein
  Decal-Atlas mit Blut, Dreck, Papier und Plakaten.
- **Licht** ist mit Cycles in Lightmaps vorberechnet. Flackernde Lampen und Feuer sind
  Echtzeit-Lichter, statische Schatten und indirektes Licht sind gebacken.
- **Geräusche und Musik** sind synthetisiert (`src/bin/sfxgen`), der Ansager nutzt espeak-ng.

## Herunterladen

Fertige Pakete für **Windows** und **Linux** gibt es unter
[Releases](https://github.com/zerox80/Totenrunde/releases/latest): Archiv entpacken, dann
`Totenrunde.exe` (Windows) bzw. `Totenrunde` (Linux) doppelklicken. Der Ordner `assets` muss neben
der Programmdatei liegen. Benötigt wird eine Grafikkarte mit Vulkan oder DirectX 12.

## Selbst bauen und spielen

```bash
cargo run --release
```

Steuerung: **WASD** laufen · **Maus** zielen · **Linksklick** schießen · **Rechtsklick** über Kimme
und Korn · **R** nachladen · **V** Messer · **G** Granate · **F** benutzen (kaufen, Türen, Barrikaden) ·
**Shift** sprinten · **Leertaste** springen · **C** ducken · **Strg/Z** hinlegen ·
**1–3 / Mausrad / Q** Waffe wechseln · **Esc** Pause · **F3** FPS.

Im Haupt- oder Pausenmenü unter **GRAFIK**: Stufen *Niedrig / Mittel / Hoch / Extrem* sowie
einzeln Texturqualität, Kantenglättung, Echtzeit-Schatten, Bloom, Filmlook, Augen-Anpassung,
Lichtkegel, Sichtfeld, VSync und FPS-Anzeige. Die Einstellungen landen in `settings.json`.

## Projektaufbau

```
src/
  base/      Zustände, Einstellungen, Assets, Textur-Mipmaps, Debug-Hilfen
  world/     Karte, Lightmaps, Licht, Kollision, Navigation, Türen, Barrikaden
  player/    Ego-Kamera, Bewegung, Gesundheit, Grafik-Effekte
  weapons/   Waffendaten, Schießen, Ego-Waffe mit Animationen
  zombies/   KI, Animation, Trefferzonen, Kontaktschatten
  game/      Runden, Punkte, Perks, Power-ups, Kiste, Verstärker, Kaufen
  ui/        HUD (Kreide-Rundenzähler, Punkte, Munition) und Menüs
  bin/sfxgen Geräusch- und Musik-Synthese
blender/
  textures/  PBR-Texturen (numpy) und Decal-Atlas
  maps/      Karte, Detail-Bausteine, Lightmap-Backen
  fparms/    Ego-Arme (Skulptur, Bake, Skelett)
  weapons/   Waffen-Baukasten, Modelle, automatisches Greifen der Finger
  characters/ Zombies (Skulptur, Bake, Rig, Animationen)
  props/     Perk-Automaten, Kiste, Verstärker, Power-ups
  ui/        HUD-Grafiken (Kreide-Ziffern, Blut-Overlay)
```

## Assets neu erzeugen

Benötigt Blender 5.x (nativ oder als Flatpak). `tools/blender.sh` startet es headless.

```bash
tools/blender.sh blender/textures/build_pbr.py          # Oberflächen-Texturen + Decals
tools/blender.sh blender/fparms/arms_build.py           # Ego-Arme (vor den Waffen)
tools/blender.sh blender/weapons/build.py               # alle Waffen (-- m1911 mp5 ... einzeln)
tools/blender.sh blender/maps/endstation.py             # Karte + Lightmaps (ca. 15 min)
tools/blender.sh blender/ui/hud_textures.py             # HUD-Grafiken
cargo run --release --bin sfxgen                        # Geräusche und Musik
```

Bei Karte und Waffen beschleunigt `-- nobake` den Durchlauf (ohne Lightmaps bzw. Texturen).

## Release erstellen

Ein Tag `v*` startet den Workflow `.github/workflows/release.yml`. Er baut auf GitHub für Windows und
Linux, packt Programm, `assets`, `LIESMICH.txt` und Lizenz zusammen und veröffentlicht das Release:

```bash
git tag v0.1.0 && git push origin v0.1.0
```

Das Programmsymbol (`assets/ui/icon.png`, `packaging/totenrunde.ico`) erzeugt
`tools/blender.sh blender/ui/hud_textures.py -- icon`.

## Lizenz

MIT, siehe `LICENSE`. Die Montserrat-Schriften stehen unter der SIL Open Font License
(`assets/fonts/Montserrat-OFL.txt`).
