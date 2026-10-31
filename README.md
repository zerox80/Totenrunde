# Totenrunde

Ein rundenbasierter Zombie-Shooter im Stil von *Black Ops 2*, geschrieben in **Rust** mit
**Bevy 0.20**. Die Karte „Endstation“ ist ein verlassener Busbahnhof bei Nacht.

Das Spiel gibt es für **Windows**, **Linux** und den **Browser**. Der Browser-Port verwendet
WebAssembly und WebGL 2. Der Webserver liefert es per **HTTP/3 über QUIC/UDP** aus; der erste
Aufruf läuft über HTTPS/TCP, das den Browser auf HTTP/3 umleitet.

**Aktuell ist das Spiel Solo; Online-Multiplayer ist nicht enthalten.**

## Windows oder Linux spielen

1. Das passende Paket unter [Releases](https://github.com/zerox80/Totenrunde/releases/latest)
   herunterladen.
2. Das Archiv vollständig entpacken.
3. `Totenrunde.exe` (Windows) beziehungsweise `Totenrunde` (Linux) starten.

Der Ordner `assets` muss neben der Programmdatei bleiben. Benötigt wird eine Grafikkarte mit
Vulkan oder DirectX 12.

## Browser-Version auf einem eigenen Server installieren

Dieser Teil ist nur für einen **öffentlichen Server mit eigener Domain** gedacht. Für die
Desktop-Version wird er nicht benötigt.

### Voraussetzungen

- Ubuntu 24.04+ oder Rocky Linux 10+
- eine Domain wie `spiel.example.de`, deren DNS auf den Server zeigt
- ein sudo-/root-Zugang
- `TCP/80` für das Let’s-Encrypt-Zertifikat sowie `TCP/443` und `UDP/443` für das Spiel freigegeben

Der Installer richtet Docker, Compose, Certbot, UFW/firewalld und das fertige Container-Image
automatisch ein. Rust und Blender werden dafür nicht benötigt.

### Installation

Git und Python installieren:

```bash
# Ubuntu
sudo apt update && sudo apt install -y git python3

# Rocky Linux
sudo dnf install -y git python3
```

Danach Repository herunterladen und Installer starten:

```bash
git clone https://github.com/zerox80/Totenrunde.git
cd Totenrunde
python3 backend/install.py
```

Der Installer fragt nur nach der Domain, einer E-Mail-Adresse für Let’s Encrypt und einer
Bestätigung. Anschließend ist das Spiel unter folgender Adresse erreichbar:

```text
https://DEINE_DOMAIN
```

Einen aktuellen Desktop-Browser mit WebAssembly, WebGL 2 und aktivierter
Hardware-Beschleunigung verwenden.

Bei einem abweichenden SSH-Port diesen angeben, damit die Firewall ihn offen lässt:

```bash
python3 backend/install.py --ssh-port 2222
```

Die geplanten Schritte lassen sich vorher ohne Änderungen anzeigen:

```bash
python3 backend/install.py --dry-run --non-interactive \
  --domain spiel.example.de --email admin@example.de
```

Ausführliche Hilfe zu DNS, Ports, Certbot und Fehlern steht in `backend/README.md`.

### Server verwalten

Diese Befehle werden im Repository-Stammverzeichnis ausgeführt:

```bash
# Status
docker compose --env-file backend/.env -f backend/compose.yaml ps

# Logs; mit Strg+C verlassen
docker compose --env-file backend/.env -f backend/compose.yaml logs -f web

# Aktualisieren
git pull
docker compose --env-file backend/.env -f backend/compose.yaml pull
docker compose --env-file backend/.env -f backend/compose.yaml up -d

# Stoppen
docker compose --env-file backend/.env -f backend/compose.yaml down
```

## Steuerung

| Eingabe | Aktion |
|---|---|
| WASD | Laufen |
| Maus | Zielen |
| Linksklick / Rechtsklick | Schießen / Visier |
| R / V / G | Nachladen / Messer / Granate |
| F | Benutzen und kaufen |
| Shift / Leertaste | Sprinten / Springen |
| C / Strg oder Z | Ducken / Hinlegen |
| 1–3, Mausrad oder Q | Waffe wechseln |
| Esc | Pause |
| F3 | FPS-Anzeige |

Bei niedrigen FPS im Grafikmenü **Niedrig** und **50 % Renderauflösung** wählen. Auf stärkeren
Systemen sind höhere Qualität und 100 % Auflösung verfügbar. Browser-Einstellungen werden in
`localStorage`, Desktop-Einstellungen in `settings.json` gespeichert.

## Wie der Browser-Port funktioniert

```text
Browser
  | 1. HTTPS über TCP (HTTP/1.1 oder HTTP/2) -> Antwort mit Alt-Svc: h3
  | 2. danach HTTP/3 über QUIC/UDP
  v
hyper + rustls (TCP) / Quinn + h3 (UDP)
  v
Axum-Server
  |-- Wasm und JavaScript
  |-- Spiel-Assets
  `-- /api/health

Das Spiel wird im Browser ausgeführt und rendert über WebGL 2 auf der GPU des Spielers.
```

Der Server stellt also das Spiel bereit; er berechnet nicht die 3D-Grafik für die Spieler.
Certbot verwaltet das TLS-Zertifikat auf dem Host. Docker startet den Server als
unprivilegierter Benutzer. Nginx wird nicht benötigt.

## Projektaufbau

```text
src/                 Rust-/Bevy-Spielcode
  base/              Einstellungen, Assets und WebGL-Anpassungen
  world/             Karte, Licht, Kollision und Navigation
  player/            Spieler und Kamera
  weapons/           Waffen und Animationen
  zombies/           Zombie-KI und Treffer
  game/              Runden, Punkte, Perks und Power-ups
  ui/                HUD und Menüs
assets/              Fertige Modelle, Texturen, Sounds und Karten
backend/
  src/http3.rs       HTTP/3-Server über QUIC
  src/tcp.rs         HTTPS-Einstieg über TCP mit Alt-Svc
  src/tls.rs         Zertifikate laden
  src/main.rs        Axum-Routen und Dateiauslieferung
  web/               Browser-Einstieg
  install.py         Geführter Server-Installer
  compose.yaml       Container-Konfiguration
  Dockerfile         Optimierter Wasm- und Server-Build
blender/             Quellen und Skripte für die Spielinhalte
.github/workflows/   Tests, Releases und Container-Images
```

## Für Entwickler

Desktop-Version bauen:

```bash
cargo run --release
```

Backend prüfen:

```bash
cargo clippy --locked --manifest-path backend/Cargo.toml -- -D warnings
cargo test --locked --manifest-path backend/Cargo.toml
```

Anleitungen für den manuellen Wasm-Build, lokale Protokolltests und Assets stehen in
`backend/README.md`.

## Lizenz

MIT, siehe `LICENSE`. Die Montserrat-Schriften stehen unter der SIL Open Font License
(`assets/fonts/Montserrat-OFL.txt`).
