# Totenrunde

Ein rundenbasierter Zombie-Shooter im Stil von *Black Ops 2*, geschrieben in **Rust** mit
**Bevy 0.20**. Die Karte „Endstation“ ist ein verlassener Busbahnhof bei Nacht.

Das Spiel gibt es für **Windows**, **Linux** und den **Browser**. Der Browser-Port verwendet
WebAssembly und **WebGPU**, in älteren Browsern WebGL 2. Der Webserver liefert es per **HTTP/3 über QUIC/UDP** aus; der erste
Aufruf läuft über HTTPS/TCP, das den Browser auf HTTP/3 umleitet.

Gespielt wird **solo** oder im **Koop mit bis zu vier Spielern** (Menü → *Multiplayer*). Für den
Koop braucht jeder ein kostenloses Konto auf dem Server; Desktop- und Browser-Spieler können
gemeinsam spielen.

Sprachen: Deutsch, Englisch, Französisch, Spanisch, Italienisch, Portugiesisch, Niederländisch,
Polnisch, Tschechisch, Türkisch, Russisch und Ukrainisch. Die Sprache folgt der Browser- bzw.
Systemsprache und lässt sich unter **Einstellungen → Sprache** umstellen. Die Texte stehen in
`src/base/lang/`; eine neue Sprache braucht dort eine Datei und einen Eintrag in
`src/base/i18n.rs`.

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

Einen aktuellen Desktop-Browser mit WebAssembly, WebGPU oder WebGL 2 und aktivierter
Hardware-Beschleunigung verwenden. Ist sie ausgeschaltet, rendert der Browser auf der CPU; die
Seite weist dann darauf hin.

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

### Einstellungen (`backend/.env`)

Der Installer legt `backend/.env` an; die Vorlage ist `.env.example`. Aktive Zeilen stehen ohne
`#`, optionale sind auskommentiert und lassen sich durch Entfernen des `#` einschalten:

| Einstellung | Bedeutung |
|---|---|
| `DOMAIN`, `TLS_SERVER_NAME`, `CERTBOT_EMAIL`, `CERT_DIR` | Domain und Let’s-Encrypt-Zertifikat |
| `PORT` | Öffentlicher Port für HTTPS (TCP) und HTTP/3 (UDP), Standard 443 |
| `IMAGE`, `TAG` | Container-Image und Version |
| `#MULTIPLAYER=0` | Koop-Multiplayer abschalten (Standard: an) |
| `#DATA_DIR=./data` | Konten in einem Ordner statt im Docker-Volume speichern |

Änderungen gelten nach `docker compose … up -d`. Ein erneuter Lauf des Installers übernimmt
eingeschaltete optionale Werte.

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

## Multiplayer

1. Im Hauptmenü **Multiplayer** wählen.
2. Ohne Konto auf **Konto erstellen / Anmelden** klicken. Im Browser öffnet sich die Konto-Seite
   (`/konto/`) mit einem Code. Dort registrieren oder anmelden, den Code mit dem eigenen Spiel
   vergleichen und **Dieses Spiel verbinden** bestätigen. Im Browser-Spiel genügt die Anmeldung
   auf der Konto-Seite derselben Domain.
3. **Lobby erstellen** (öffentlich oder privat) oder einer offenen Lobby bzw. per
   fünfstelligem Code beitreten. Sind alle **bereit**, startet der Host die Partie.

Im Koop gilt die Abstimmung des Originals für mehrere Spieler (mehr und stärkere Zombies,
Zweiter Atem kostet 1500). Wer fällt, liegt 45 Sekunden am Boden und kann von Mitspielern mit
**F** wiederbelebt werden; wer ausblutet, kehrt zur nächsten Runde zurück. Liegen alle am
Boden, ist die Partie vorbei. Die Konto-Seite zeigt eigene Statistik und eine Bestenliste.

Die Desktop-Version verbindet sich mit dem Server aus `settings.json` (`"server":
"https://spiel.example.de"`). Release-Builds können ihn über die Repository-Variable
`TR_SERVER` fest eintragen; ohne Angabe gilt der lokale Entwicklungsserver
`http://127.0.0.1:8080`.

So funktioniert es: Alle Spiele halten eine WebSocket-Verbindung zum Server (`/api/mp/ws`). Der
Server prüft Konten und Lobbys und reicht die Spieldaten weiter. Das Spiel des Hosts berechnet
die Welt (Zombies, Runden, Türen, Strom, Kiste, Power-ups) und schickt 20-mal pro Sekunde einen
Schnappschuss; jedes Spiel steuert seinen eigenen Spieler. Treffer melden die Spieler dem Host,
der Schaden und Punkte berechnet. Käufe, die die Welt verändern, sind Anfragen an den Host –
bezahlt wird erst mit seiner Zusage. Geht der Host, endet die Partie für alle.

## Steuerung

| Eingabe | Aktion |
|---|---|
| WASD | Laufen |
| Maus | Zielen |
| Linksklick / Rechtsklick | Schießen / Visier |
| R / V / G | Nachladen / Messer / Granate |
| F | Benutzen und kaufen, im Koop Mitspieler wiederbeleben |
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
  |-- /konto/          Konto-Seite und Bestenliste
  |-- /api/account/... Registrieren, Anmelden, Gerät verknüpfen
  |-- /api/mp/ws       WebSocket: Lobbys und Spieldaten
  `-- /api/health

Das Spiel wird im Browser ausgeführt und rendert über WebGPU (sonst WebGL 2) auf der GPU des
Spielers.
```

Der Server stellt also das Spiel bereit; er berechnet nicht die 3D-Grafik für die Spieler.
Für den Multiplayer verwaltet er zusätzlich Konten (SQLite in `/data`, Passwörter als
Argon2-Hash) und Lobbys und leitet die Spieldaten weiter. Die Datenbank liegt im
Compose-Volume `totenrunde-data` und bleibt bei Updates erhalten. Konten, Lobbys und
Weiterleitung kommen aus dem [Multiplayer-SDK](https://github.com/zerox80/totenrunde-mp-sdk)
(`totenrunde-mp-server`), die Verbindung im Spiel aus `totenrunde-mp-client`.
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
  net/               Koop-Multiplayer: Sitzung, Abgleich, Mitspieler-Figuren (Verbindung: SDK)
assets/              Fertige Modelle, Texturen, Sounds und Karten
backend/
  src/http3.rs       HTTP/3-Server über QUIC
  src/tcp.rs         HTTPS-Einstieg über TCP mit Alt-Svc
  src/tls.rs         Zertifikate laden
  src/main.rs        Axum-Routen und Dateiauslieferung
  web/               Browser-Einstieg und Konto-Seite (web/konto/)
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

Multiplayer lokal testen – Server ohne TLS auf Port 8080 starten (ohne Zertifikate läuft dann
nur dieser Zugang; die Datenbank landet in `backend/data/`):

```bash
HTTP_PORT=8080 cargo run --manifest-path backend/Cargo.toml
```

Danach zwei Spiele starten, im Menü *Multiplayer* wählen und die Konto-Seite unter
`http://127.0.0.1:8080/konto/` benutzen. Für automatische Tests gibt es `TR_TOKEN`, `TR_MP=host`
bzw. `TR_MP=join` und `TR_SERVER` (siehe `src/base/debug.rs`).

Spiel und Server binden das Multiplayer-SDK per Git-Tag ein. Um das SDK gleichzeitig lokal zu
ändern, es neben dieses Repository klonen und vorübergehend (nicht einchecken) in `Cargo.toml`
und `backend/Cargo.toml` umlenken:

```toml
[patch."https://github.com/zerox80/totenrunde-mp-sdk"]
totenrunde-mp-client = { path = "../totenrunde-mp-sdk/crates/client" }
totenrunde-mp-protocol = { path = "../totenrunde-mp-sdk/crates/protocol" }
```

Anleitungen für den manuellen Wasm-Build, lokale Protokolltests und Assets stehen in
`backend/README.md`.

## Lizenz

MIT, siehe `LICENSE`. Die Montserrat-Schriften stehen unter der SIL Open Font License
(`assets/fonts/Montserrat-OFL.txt`).
