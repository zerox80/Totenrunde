# Totenrunde-Web

Der Server stellt die Bevy-Wasm-Anwendung, die Spiel-Assets und `GET /api/health` über **HTTP/3** bereit. `h3` und `h3-quinn`/`quinn` übernehmen QUIC und TLS; der vorhandene Axum-Router kümmert sich weiterhin um Routen, MIME-Typen, Kompression und Header.

Auf demselben Port lauscht zusätzlich ein HTTPS-Zugang über TCP (HTTP/1.1 und HTTP/2, `hyper` und `tokio-rustls`). Browser verbinden sich zu einer unbekannten Seite immer zuerst per TCP; jede Antwort enthält `Alt-Svc: h3=":PORT"`, worauf der Browser für die folgenden Anfragen auf HTTP/3 wechselt. Ohne diesen Einstieg wäre der reine QUIC-Server im Browser nicht erreichbar. Ein Klartext-Zugang (HTTP auf Port 80) existiert nicht.

## Einfachste Installation ohne Vorkenntnisse

Das fertige Pull-Paket in `~/Dokumente/Projekte/Totenstille-compose` enthält einen geführten Installer für **Ubuntu 24.04+** und **Rocky Linux 10+**:

```sh
cd ~/Dokumente/Projekte/Totenstille-compose
python3 install.py
```

Der Installer fragt Domain und Let's-Encrypt-E-Mail ab und richtet Docker, Compose, Certbot, Firewall, TLS, automatische Zertifikatserneuerung und den Container ein. Er benötigt dafür `sudo`/root. Vorher muss der öffentliche DNS-Eintrag der Domain auf den Server zeigen. Router und Cloud-Firewall müssen `TCP/80` für die ACME-Prüfung sowie den gewählten Port per TCP und UDP (standardmäßig `443/tcp` und `443/udp`) freigeben.

Ubuntu verwendet UFW, Rocky firewalld. SSH (`22/tcp`), Certbot (`80/tcp`) und das Spiel (`443/tcp` und `443/udp`) werden automatisch erlaubt; bei einem anderen SSH-Port unbedingt `--ssh-port PORT` angeben. Ein nicht interaktiver Lauf und ein gefahrloser Dry-Run sind ebenfalls möglich:

```sh
python3 install.py --dry-run --non-interactive \
  --domain spiel.example.de --email admin@example.de
python3 install.py --non-interactive \
  --domain spiel.example.de --email admin@example.de
```

Anschließend läuft das Spiel unter `https://spiel.example.de` beziehungsweise `https://spiel.example.de:PORT`. Ein aktueller Desktop-Browser mit WebAssembly und WebGL 2 wird benötigt.

## Docker Compose selbst bauen

Der Docker-Build erzeugt optimiertes Release-Wasm und den Server in getrennten Stufen; der Build-Kontext ist der Projektroot. Für die Produktion zuerst `backend/.env.example` nach `backend/.env` kopieren und Domain, E-Mail und Port eintragen. Der geführte Installer erledigt dies automatisch und kann auch mit `backend/install.py` gestartet werden.

Für einen reinen Protokolltest mit selbstsigniertem Zertifikat:

```sh
mkdir -p backend/certs
openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
  -subj '/CN=localhost' -addext 'subjectAltName=DNS:localhost' \
  -addext 'basicConstraints=critical,CA:FALSE' \
  -keyout backend/certs/privkey.pem -out backend/certs/fullchain.pem
TLS_SERVER_NAME=localhost PORT=8443 CERT_DIR=./certs \
  docker compose -f backend/compose.yaml up --build -d
docker compose -f backend/compose.yaml ps
docker compose -f backend/compose.yaml logs -f web
```

Der Healthcheck im Image prüft `https://TLS_SERVER_NAME:8443/api/health` nacheinander als Quinn/H3-Client und über HTTPS/TCP, jeweils mit SNI und Zertifikatsvalidierung. Selbstsignierte Zertifikate eignen sich für automatisierte Tests, aber nicht zuverlässig für Browser-HTTP/3; für den Browsertest ein echtes Domain-Zertifikat verwenden. Zum Stoppen:

```sh
docker compose -f backend/compose.yaml down
```

Die `.env`-Datei wird von Compose gelesen; der direkt gestartete Server lädt sie nicht automatisch. `certs/`, `.env` und generierte Wasm-Dateien sind von Git und Docker-Build-Kontext ausgeschlossen.

## Certbot und Erneuerung

`backend/certbot-deploy.sh` kopiert `fullchain.pem` und `privkey.pem` aus `RENEWED_LINEAGE` nach `CERT_DIR`, setzt kontrollierte Leserechte für den unprivilegierten Container und startet den Dienst neu. Der Installer legt denselben Mechanismus als Certbot-Deploy-Hook an und aktiviert `certbot.timer` oder `certbot-renew.timer`.

Manuell kann das Zertifikat so erneuert und dessen automatische Verarbeitung geprüft werden:

```sh
sudo certbot renew --dry-run
sudo systemctl list-timers --all
```

Der Spieledienst selbst öffnet niemals TCP/80. Certbot Standalone verwendet diesen Port nur kurzzeitig zur Let's-Encrypt-Validierung.

## Lokaler Wasm-Build und Server

Vorausgesetzt werden Rust, OpenSSL, das Wasm-Ziel, `wasm-bindgen-cli` in exakt der zur Root-Crate passenden Version und `wasm-opt` aus Binaryen. Nicht das Paket aus Debian bookworm verwenden: Dessen Version 108 lenkt den Tabellen-Export von wasm-bindgen um, im Browser erscheint dann `WebAssembly.Table.grow(): failed to grow table`. `sudo sh backend/install-binaryen.sh` installiert die auch von Docker und CI genutzte geprüfte Version. Vom Projektroot aus:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
CARGO_BUILD_JOBS=1 cargo build --release --locked --target wasm32-unknown-unknown --bin totenrunde
wasm-bindgen --target web --out-name totenrunde \
  --out-dir backend/web/pkg \
  target/wasm32-unknown-unknown/release/totenrunde.wasm
wasm-opt -Oz backend/web/pkg/totenrunde_bg.wasm \
  -o backend/web/pkg/totenrunde_bg.wasm
python3 backend/check-wasm.py backend/web/pkg/totenrunde_bg.wasm
TLS_CERT_PATH="$PWD/backend/certs/fullchain.pem" \
TLS_KEY_PATH="$PWD/backend/certs/privkey.pem" \
TLS_SERVER_NAME=localhost PORT=8443 \
cargo run --release --locked --manifest-path backend/Cargo.toml
```

Nur das so erzeugte Release-Wasm ist für Leistungstests und die Auslieferung geeignet; Debug-Wasm ist wesentlich größer und langsamer. Der Docker-Builder führt denselben Optimierungsschritt aus.

Den laufenden Server und dessen Healthcheck direkt prüfen:

```sh
TLS_CERT_PATH="$PWD/backend/certs/fullchain.pem" \
TLS_SERVER_NAME=localhost PORT=8443 \
cargo run --release --locked --manifest-path backend/Cargo.toml -- --healthcheck
```

## Konfiguration

- `HOST`: Adresse, an die der UDP- und der TCP-Zugang gebunden werden (Standard: `127.0.0.1`, im Container `0.0.0.0`)
- `PORT`: Port für UDP und TCP (Standard: `8443`)
- `PUBLIC_PORT`: von außen erreichbarer Port, der im `Alt-Svc`-Header angekündigt wird (Standard: `PORT`; Compose setzt ihn auf den veröffentlichten Port)
- `TLS_CERT_PATH`: PEM-Zertifikatskette (Standard: `/certs/fullchain.pem`)
- `TLS_KEY_PATH`: privater PEM-Schlüssel (Standard: `/certs/privkey.pem`)
- `TLS_SERVER_NAME`: DNS-Name für SNI und Zertifikatsprüfung im Healthcheck
- `WEB_ROOT`: Verzeichnis mit `index.html` und `pkg/`
- `ASSET_ROOT`: Verzeichnis mit den unter `/assets` veröffentlichten Spiel-Assets
- Compose zusätzlich: `CERT_DIR`, `IMAGE`, `TAG`, `DOMAIN` und `CERTBOT_EMAIL`

Optional kann beim DNS-Anbieter zusätzlich ein HTTPS/SVCB-Eintrag mit `alpn="h3"` gesetzt werden; Browser, die ihn auswerten, nutzen HTTP/3 dann schon ab der ersten Anfrage.

## Grafik und GPU

WebGL 2 rendert auf der GPU, sofern der Browser die Hardware-Beschleunigung nicht deaktiviert hat oder auf Software-Rendering zurückfällt. Beim ersten Start dienen die gemeldeten CPU-Kerne als grobe Starthilfe: bis 4 Niedrig, bis 8 Mittel, darüber Hoch; ohne Angabe Mittel. Gespeicherte Einstellungen werden nie automatisch überschrieben.

Die Presets wählen 50 % (Niedrig), 75 % (Mittel) oder 100 % (Hoch/Extrem) Renderauflösung. Im Grafikmenü sind diese drei Werte auch einzeln einstellbar. Die Anzeige bleibt bildschirmfüllend; ein kleinerer Renderpuffer reduziert vor allem die GPU-Last. Texturänderungen greifen nach einem Neustart. Auto Exposure bleibt unter WebGL2 deaktiviert, da dafür Compute-Shader erforderlich sind. Mit **F3** lässt sich die FPS-Anzeige einblenden.

## Tests

```sh
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --locked --manifest-path backend/Cargo.toml -- -D warnings
cargo test --locked --manifest-path backend/Cargo.toml
python3 backend/install.py --dry-run --non-interactive \
  --domain spiel.example.de --email admin@example.de
```

Die Tests enthalten echte QUIC/HTTP/3- und HTTPS/TCP-Verbindungen mit temporärem `rcgen`-Zertifikat und prüfen Health-API, statische Wasm-Auslieferung und den `Alt-Svc`-Header. Unbekannte `/api/*`-Routen liefern JSON mit Status 404. HTML wird ohne Cache ausgeliefert; versionierte Dateien können langfristig zwischengespeichert werden. Gzip- und Brotli-Kompression sind aktiv. Es werden bewusst keine COOP-/COEP-Header gesetzt, damit der single-threaded Wasm-Port keine Cross-Origin-Isolation benötigt.
