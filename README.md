# Totenrunde

Ein rundenbasierter Zombie-Shooter im Stil von *Black Ops 2*, geschrieben in **Rust** mit
**Bevy 0.20**. Die Karte „Endstation“ ist ein verlassener Busbahnhof bei Nacht mit Wartehalle,
Hof, Werkstatt, Diner und Umspannwerk.

Totenrunde läuft als normales Windows-/Linux-Spiel und als **WebAssembly-Anwendung im Browser**.
Der Webserver verwendet ausschließlich **HTTP/3 über QUIC/UDP mit TLS** – ohne Nginx und ohne
HTTP/1.1- oder HTTP/2-Fallback.

> **Aktueller Spielmodus:** Solo. Der Webserver liefert das Spiel und eine Health-API aus;
> Matchmaking oder Online-Multiplayer sind derzeit nicht enthalten.

## Inhalt

- [Nur spielen, ohne eigenen Server](#nur-spielen-ohne-eigenen-server)
- [Webserver installieren – Schritt für Schritt](#webserver-installieren--schritt-für-schritt)
- [Nach der Installation](#nach-der-installation)
- [Wie der Web-Port aufgebaut ist](#wie-der-web-port-aufbaut-ist)
- [Konfiguration](#konfiguration)
- [Fehlersuche](#fehlersuche)
- [Steuerung und Grafik](#steuerung-und-grafik)
- [Projektaufbau](#projektaufbau)
- [Für Entwickler](#für-entwickler)

## Nur spielen, ohne eigenen Server

Fertige Pakete für **Windows** und **Linux** stehen unter
[Releases](https://github.com/zerox80/Totenrunde/releases/latest):

1. Das passende Archiv herunterladen und vollständig entpacken.
2. Unter Windows `Totenrunde.exe`, unter Linux `Totenrunde` starten.
3. Den Ordner `assets` neben der Programmdatei lassen.

Für die Desktop-Version wird eine Grafikkarte mit Vulkan oder DirectX 12 benötigt.

## Webserver installieren – Schritt für Schritt

Die geführte Installation richtet das fertige Docker-Image ein. **Rust, Blender und die
Entwicklerwerkzeuge werden dafür nicht benötigt.** Docker, Compose, Certbot und die Firewall
installiert das Skript bei Bedarf selbst.

### 1. Voraussetzungen

Benötigt werden:

- **Ubuntu 24.04 oder neuer** beziehungsweise **Rocky Linux 10 oder neuer**
- ein Server mit root-/sudo-Zugang
- eine **öffentliche Domain**, zum Beispiel `spiel.example.de`
- Python 3 und Git; deren Installation wird im nächsten Schritt gezeigt

Eine reine IP-Adresse oder `localhost` reicht für den öffentlichen Installer nicht aus, weil
Let's Encrypt dafür kein normales Browser-Zertifikat ausstellt.

Der Server rendert das Spiel nicht. Er liefert hauptsächlich Dateien aus; die eigentliche
Spielberechnung und 3D-Darstellung erfolgen auf dem Computer des Spielers.

### 2. Domain und Firewall vorbereiten

Beim DNS-Anbieter der Domain einen **A-Eintrag** auf die IPv4-Adresse des Servers setzen. Bei
IPv6 zusätzlich einen **AAAA-Eintrag** auf die IPv6-Adresse setzen.

Folgende Verbindungen müssen den Server erreichen können:

| Port | Protokoll | Wofür? |
|---|---|---|
| `80` | TCP | Let's-Encrypt-Prüfung und Zertifikatserneuerung durch Certbot |
| `443` | UDP | Totenrunde über HTTP/3 |
| `22` oder eigener Port | TCP | SSH-Verwaltung des Servers |

Das gilt sowohl für eine Cloud-Firewall als auch für einen Router mit Portweiterleitung.
**Der Spieledienst selbst benötigt kein TCP/443.** TCP/80 wird nur kurzzeitig von Certbot
verwendet. Dort darf während der Zertifikatsprüfung kein anderer Webserver den Port belegen.

Für eine besonders zuverlässige erste HTTP/3-Verbindung kann der DNS-Anbieter zusätzlich einen
**HTTPS/SVCB-Eintrag** mit `alpn="h3"` anbieten. Das ist optional, aber hilfreich, weil es keinen
Alt-Svc-Eintrag von einem früheren Besuch gibt und der Server bewusst kein HTTP/1.1 oder HTTP/2
anbietet.

### 3. Git und Python installieren

Auf **Ubuntu**:

```bash
sudo apt update
sudo apt install -y git python3
```

Auf **Rocky Linux**:

```bash
sudo dnf install -y git python3
```

### 4. Repository herunterladen

```bash
git clone https://github.com/zerox80/Totenrunde.git
cd Totenrunde
```

Wenn das Repository bereits vorhanden ist, stattdessen in dessen Ordner wechseln und aktualisieren:

```bash
git pull
```

### 5. Installation zuerst ohne Änderungen ansehen

Dieser Dry-Run zeigt alle geplanten Schritte an, installiert aber nichts und verändert keine
Dateien. Domain und E-Mail durch die eigenen Werte ersetzen:

```bash
python3 backend/install.py --dry-run --non-interactive \
  --domain spiel.example.de --email admin@example.de
```

### 6. Geführten Installer starten

```bash
python3 backend/install.py
```

Der Installer fragt nach:

1. **Domainname:** nur den Namen eingeben, zum Beispiel `spiel.example.de` – ohne `https://`.
2. **E-Mail-Adresse:** für Let's-Encrypt-Hinweise und Ablaufwarnungen.
3. **Bestätigung:** mit `j` bestätigen, danach gegebenenfalls das sudo-Passwort eingeben.

Der Installer erledigt anschließend automatisch:

- Prüfung von Betriebssystem, Domain und Certbot-Port
- Installation und Start von Docker und Compose v2
- Installation von Certbot
- Einrichtung von **UFW** auf Ubuntu beziehungsweise **firewalld** auf Rocky
- Freigabe von SSH, `TCP/80` und `UDP/443`
- Erstellung der lokalen Datei `backend/.env`
- Ausstellung eines echten Let's-Encrypt-Zertifikats
- Einrichtung der automatischen Zertifikatserneuerung
- Download des fertigen GHCR-Images
- Start des Containers und echter HTTP/3-Healthcheck

Bei einem abweichenden SSH-Port diesen unbedingt angeben, damit die Firewall den Zugang nicht
sperrt:

```bash
python3 backend/install.py --ssh-port 2222
```

Für einen anderen öffentlichen Spiele-Port beispielsweise:

```bash
python3 backend/install.py --port 8443
```

Dann muss auch im Router beziehungsweise der Cloud-Firewall **UDP/8443** statt UDP/443 freigegeben
werden. Die Adresse lautet anschließend `https://spiel.example.de:8443`.

Eine Installation ohne Rückfragen ist ebenfalls möglich:

```bash
python3 backend/install.py --non-interactive \
  --domain spiel.example.de --email admin@example.de
```

### 7. Spiel im Browser öffnen

Nach `Fertig` im Terminal im aktuellen Desktop-Browser öffnen:

```text
https://spiel.example.de
```

Wichtig:

- **`https://` verwenden, nicht `http://`.**
- HTTP/3, WebAssembly und WebGL 2 müssen unterstützt werden.
- Hardware-Beschleunigung im Browser aktivieren.
- Für Maussteuerung und Audio in die Spielansicht klicken.

Der Healthcheck ist ebenfalls im Browser erreichbar:

```text
https://spiel.example.de/api/health
```

Eine Antwort mit `"status":"ok"` zeigt, dass der Server funktioniert.

## Nach der Installation

Die folgenden Befehle werden aus dem Repository-Stammverzeichnis ausgeführt. Da die Compose-Datei
in `backend/` liegt, wird sie ausdrücklich angegeben.

### Status und Logs ansehen

```bash
docker compose --env-file backend/.env -f backend/compose.yaml ps
docker compose --env-file backend/.env -f backend/compose.yaml logs -f web
```

Die Logs mit **Strg+C** verlassen; der Server läuft dabei weiter.

### Server stoppen und starten

```bash
docker compose --env-file backend/.env -f backend/compose.yaml down
docker compose --env-file backend/.env -f backend/compose.yaml up -d
```

### Auf eine neue Version aktualisieren

```bash
git pull
docker compose --env-file backend/.env -f backend/compose.yaml pull
docker compose --env-file backend/.env -f backend/compose.yaml up -d
```

### Zertifikat und automatische Erneuerung prüfen

```bash
sudo certbot certificates
sudo certbot renew --dry-run
sudo systemctl list-timers --all
```

Nach einer erneuerten Zertifikatsdatei kopiert der Deploy-Hook diese mit passenden Rechten nach
`backend/certs/` und startet den Container neu. `.env`, `certs/` und private Schlüssel sind von Git
und vom Docker-Build-Kontext ausgeschlossen.

## Wie der Web-Port aufgebaut ist

### Der Weg einer Anfrage

```text
Aktueller Browser
  |
  | HTTPS mit HTTP/3, QUIC, UDP und TLS
  v
Quinn-Endpoint (nur UDP, ALPN "h3")
  |
  v
h3-Adapter
  |
  v
Axum-Router
  |-- /api/health       -> JSON-Status
  |-- /assets/...       -> Modelle, Texturen, Sounds, Karten
  |-- /pkg/...          -> JavaScript und optimiertes Wasm
  `-- übrige Pfade      -> index.html für die Web-Anwendung
```

Das heruntergeladene Wasm enthält denselben Bevy-Spielcode wie die Desktop-Version. Es läuft im
Browser und zeichnet über **WebGL 2 auf der GPU des Spielers**. Der Server benötigt dafür keine
Grafikkarte.

`backend/src/http3.rs` verbindet Quinn/H3 mit dem Axum-Router. Anfragen haben ein Body-Limit von
1 MiB; große Antworten wie Wasm und Assets werden gestreamt. TLS-Zertifikate kommen vom Host aus
Certbot und werden read-only in den Container eingebunden.

### Docker-Aufbau

Das Image wird in `backend/Dockerfile` in drei Stufen gebaut:

1. **Wasm-Builder:** kompiliert den Bevy-Spielcode als Release-Wasm, erzeugt die JavaScript-Bindings
   mit `wasm-bindgen` und optimiert die Datei mit `wasm-opt -Oz`.
2. **Backend-Builder:** kompiliert den Rust-Server mit Axum, H3 und Quinn.
3. **Runtime:** enthält nur Server, Web-Dateien, Assets und Laufzeitbibliotheken. Der Prozess läuft
   als unprivilegierter Benutzer mit read-only-Dateisystem und ohne Linux-Capabilities.

Nur UDP/8443 wird im Container geöffnet. Compose verbindet standardmäßig den öffentlichen Port
UDP/443 damit. Der Healthcheck ist selbst ein kleiner Quinn/H3-Client und prüft `/api/health` mit
SNI und Zertifikatsvalidierung.

### Certbot-Aufbau

```text
Let's Encrypt
  |
  | Zertifikat
  v
Certbot auf dem Host
  |
  | Deploy-Hook nach Ausstellung/Erneuerung
  v
backend/certs/fullchain.pem und privkey.pem
  |
  | read-only eingebunden als /certs
  v
Totenrunde-Container
```

## Konfiguration

Der Installer schreibt die Konfiguration nach `backend/.env`. Die Datei nicht veröffentlichen oder
mit echten Zugangsdaten committen. `backend/.env.example` zeigt alle verfügbaren Werte.

| Variable | Bedeutung | Standard |
|---|---|---|
| `DOMAIN` | Öffentlicher Domainname | vom Installer |
| `TLS_SERVER_NAME` | Name für TLS/SNI und Healthcheck; ohne `https://` | wie `DOMAIN` |
| `PORT` | Öffentlicher UDP-Port | `443` |
| `CERT_DIR` | Host-Ordner mit Zertifikat und privatem Schlüssel | `./certs` |
| `IMAGE` | Container-Image | `ghcr.io/zerox80/totenrunde-web` |
| `TAG` | Image-Version | `latest` |
| `CERTBOT_EMAIL` | E-Mail für Let's Encrypt | vom Installer |

Der interne Container-Port bleibt **UDP/8443**. Nach manuellen Änderungen an `.env`:

```bash
docker compose --env-file backend/.env -f backend/compose.yaml up -d
```

## Fehlersuche

### Die Seite ist nicht erreichbar oder meldet einen QUIC-Fehler

- Wirklich `https://` verwenden.
- Prüfen, ob der DNS-Eintrag auf den richtigen Server zeigt.
- UDP/443 beziehungsweise den gewählten UDP-Port in Router und Cloud-Firewall öffnen.
- Containerstatus und Logs mit den Befehlen aus [Nach der Installation](#nach-der-installation)
  kontrollieren.
- Bei Problemen mit der ersten H3-Erkennung einen HTTPS/SVCB-Eintrag mit `alpn="h3"` setzen.

Ein Test mit gewöhnlichem `curl http://...` schlägt absichtlich fehl: Der Server besitzt keinen
TCP-/HTTP-Fallback.

### Certbot kann kein Zertifikat ausstellen oder erneuern

- Domain muss öffentlich auf diesen Server zeigen.
- TCP/80 muss von außen erreichbar sein.
- Ein anderer Dienst darf TCP/80 während der Prüfung nicht belegen.
- `sudo certbot certificates` und die Ausgabe von `sudo certbot renew --dry-run` lesen.

### SSH ist nach der Einrichtung nicht erreichbar

Der Installer lässt standardmäßig TCP/22 offen. Bei einem eigenen SSH-Port diesen beim nächsten
Lauf mit `--ssh-port DEIN_PORT` freigeben. Auf einem entfernten Server vor Aktivierung einer
Firewall immer sicherstellen, dass der tatsächlich verwendete SSH-Port erlaubt ist.

### Das Spiel startet, ist aber langsam

- Hardware-Beschleunigung des Browsers aktivieren.
- Im Grafikmenü zunächst **Niedrig** und **50 % Renderauflösung** wählen.
- Mit **F3** die FPS-Anzeige einblenden.
- Keinen Debug-Wasm-Build ausliefern; Docker und CI verwenden bereits optimiertes Release-Wasm.
- Die Leistung des Spieler-Computers und dessen GPU ist wichtiger als die Serverhardware.

Weitere technische Details und lokale Tests stehen in `backend/README.md`.

## Steuerung und Grafik

**WASD** laufen · **Maus** zielen · **Linksklick** schießen · **Rechtsklick** über Kimme und Korn ·
**R** nachladen · **V** Messer · **G** Granate · **F** benutzen · **Shift** sprinten ·
**Leertaste** springen · **C** ducken · **Strg/Z** hinlegen · **1–3 / Mausrad / Q** Waffe wechseln ·
**Esc** Pause · **F3** FPS.

Im Haupt- oder Pausenmenü stehen die Stufen *Niedrig / Mittel / Hoch / Extrem* sowie einzelne
Einstellungen für Texturqualität, Kantenglättung, Schatten, Bloom, Filmlook, Augen-Anpassung,
Lichtkegel, Sichtfeld, VSync und FPS-Anzeige zur Verfügung.

Im Browser werden Einstellungen in `localStorage` gespeichert, auf dem Desktop in `settings.json`.
**Niedrig** rendert mit 50 %, **Mittel** mit 75 % und **Hoch/Extrem** mit 100 % der physischen
Auflösung. Die CPU-Erkennung ist nur eine Starthilfe und keine GPU-Messung. Auto Exposure ist im
WebGL2-Port deaktiviert, weil dieses Bevy-Feature Compute-Shader benötigt.

## Projektaufbau

```text
src/
  base/       Zustände, Einstellungen, WebGL-Anpassungen und Asset-Verwaltung
  world/      Karte, Lightmaps, Licht, Kollision, Navigation und Türen
  player/     Ego-Kamera, Bewegung, Gesundheit und Grafik-Effekte
  weapons/    Waffendaten, Schießen und Ego-Waffe mit Animationen
  zombies/    KI, Animation, Trefferzonen und Kontaktschatten
  game/       Runden, Punkte, Perks, Power-ups, Kiste und Verstärker
  ui/         HUD und Menüs
  bin/sfxgen  Geräusch- und Musik-Synthese
backend/
  src/main.rs        Axum-Routen, Health-API, Cache- und Security-Header
  src/http3.rs       Reiner H3-/Quinn-Transport und echter H3-Healthcheck
  web/index.html     Einstiegspunkt der Browser-Anwendung
  Dockerfile         Mehrstufiger Wasm-/Server-Build und schlanke Runtime
  compose.yaml       UDP-Port, TLS-Mount und Container-Härtung
  install.py         Geführter Installer für Ubuntu und Rocky
  certbot-deploy.sh  Zertifikate installieren und Dienst neu starten
assets/             Fertige Modelle, Texturen, Karten, Sounds und Schriften
blender/            Quellen und Skripte zur Erzeugung der Spielinhalte
packaging/          Desktop-Paketierung und Programmsymbole
.github/workflows/  CI, Release-Pakete und GHCR-Image-Veröffentlichung
```

Alle Inhalte sind selbst erzeugt; einzige Fremdausnahme ist die Schrift Montserrat (OFL).
Modelle werden per Python in Blender erstellt, Texturen prozedural mit numpy. Licht wird mit
Cycles in Lightmaps vorberechnet. Geräusche und Musik werden synthetisiert, der Ansager nutzt
espeak-ng.

## Für Entwickler

### Desktop selbst bauen

```bash
cargo run --release
```

Die benötigten nativen Linux-Bibliotheken und weitere Hinweise stehen in den CI-Workflows.

### Assets neu erzeugen

Benötigt Blender 5.x (nativ oder als Flatpak). `tools/blender.sh` startet es headless:

```bash
tools/blender.sh blender/textures/build_pbr.py
tools/blender.sh blender/fparms/arms_build.py
tools/blender.sh blender/weapons/build.py
tools/blender.sh blender/maps/endstation.py
tools/blender.sh blender/ui/hud_textures.py
cargo run --release --bin sfxgen
```

Bei Karte und Waffen beschleunigt `-- nobake` den Durchlauf ohne Lightmaps beziehungsweise
Texturen. Das Programmsymbol erzeugt `tools/blender.sh blender/ui/hud_textures.py -- icon`.

### Web-Backend und Wasm prüfen

```bash
cargo fmt --all --check
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --locked --manifest-path backend/Cargo.toml -- -D warnings
cargo test --locked --manifest-path backend/Cargo.toml
python3 backend/install.py --dry-run --non-interactive \
  --domain spiel.example.de --email admin@example.de
```

Die Backend-Tests enthalten eine echte QUIC/HTTP/3-Verbindung mit temporärem Testzertifikat.
Anleitungen für den manuellen Wasm-Build und selbstsignierte Protokolltests stehen in
`backend/README.md`.

### CI und Releases

Der Workflow `.github/workflows/web.yml` prüft Desktop, Wasm, Backend, Installer und Compose. Bei
`main` und Versions-Tags wird das optimierte Image in GHCR veröffentlicht.

Ein Tag `v*` startet zusätzlich `.github/workflows/release.yml` und erstellt die Desktop-Pakete:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Lizenz

MIT, siehe `LICENSE`. Die Montserrat-Schriften stehen unter der SIL Open Font License
(`assets/fonts/Montserrat-OFL.txt`).
