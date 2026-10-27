# Totenrunde-Web

Der Axum-Server stellt die Bevy-Wasm-Anwendung, die Spiel-Assets und `GET /api/health` bereit. Der Docker-Build erzeugt den Wasm-Port und den Server in getrennten Build-Stufen. Der Build-Kontext ist der Projektroot.

## Mit Docker Compose

Vom Projektroot aus:

```sh
docker compose -f backend/compose.yaml up --build -d
docker compose -f backend/compose.yaml ps
docker compose -f backend/compose.yaml logs -f web
```

Die Anwendung läuft anschließend auf <http://localhost:8080>. Zum Stoppen:

```sh
docker compose -f backend/compose.yaml down
```

`backend/.env.example` enthält sichere lokale Vorgaben. Lege daraus optional deine Compose-Konfiguration an. Port, Image und Tag lassen sich darin oder direkt über Umgebungsvariablen ändern:

```sh
cp backend/.env.example backend/.env
PORT=9090 TAG=dev docker compose -f backend/compose.yaml up --build -d
```

Die `.env`-Datei wird von Compose gelesen; der lokale Server lädt sie nicht automatisch.

## Lokaler Lauf

Vorausgesetzt werden Rust, das Wasm-Ziel, `wasm-bindgen-cli` in exakt der zur Root-Crate passenden Version und `wasm-opt` aus Binaryen. Unter Debian/Ubuntu stellt das Paket `binaryen` letzteres bereit. Vom Projektroot aus:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cargo build --release --locked --target wasm32-unknown-unknown --bin totenrunde
wasm-bindgen --target web --out-name totenrunde \
  --out-dir backend/web/pkg \
  target/wasm32-unknown-unknown/release/totenrunde.wasm
wasm-opt -Oz backend/web/pkg/totenrunde_bg.wasm \
  -o backend/web/pkg/totenrunde_bg.wasm
cargo run --release --manifest-path backend/Cargo.toml
```

Nur das so erzeugte Release-Wasm ist für Leistungstests und die Auslieferung geeignet; Debug-Wasm ist wesentlich größer und langsamer. Der Docker-Builder führt denselben Optimierungsschritt aus.

## Grafik und GPU

WebGL 2 rendert auf der GPU, sofern der Browser die Hardware-Beschleunigung nicht deaktiviert hat oder auf Software-Rendering zurückfällt. Beim ersten Start dienen die gemeldeten CPU-Kerne als grobe Starthilfe: bis 4 Niedrig, bis 8 Mittel, darüber Hoch; ohne Angabe Mittel. Gespeicherte Einstellungen werden nie automatisch überschrieben.

Die Presets wählen 50 % (Niedrig), 75 % (Mittel) oder 100 % (Hoch/Extrem) Renderauflösung. Im Grafikmenü sind diese drei Werte auch einzeln einstellbar. Die Anzeige bleibt bildschirmfüllend; ein kleinerer Renderpuffer reduziert vor allem die GPU-Last. Texturänderungen greifen nach einem Neustart. Auto Exposure bleibt unter WebGL2 deaktiviert, da dafür Compute-Shader erforderlich sind. Mit **F3** lässt sich die FPS-Anzeige einblenden.

Ohne weitere Konfiguration verwendet der lokale Server `127.0.0.1:8080`, `backend/web` und das vorhandene Verzeichnis `assets`. Falls `/app/web` beziehungsweise `/app/assets` existiert, werden stattdessen diese Containerpfade verwendet.

Alle Einstellungen können per Umgebungsvariable überschrieben werden:

```sh
HOST=0.0.0.0 PORT=8080 \
WEB_ROOT="$PWD/backend/web" ASSET_ROOT="$PWD/assets" \
cargo run --release --manifest-path backend/Cargo.toml
```

- `HOST`: IP-Adresse, an die der Server gebunden wird (Standard lokal: `127.0.0.1`)
- `PORT`: TCP-Port (Standard: `8080`)
- `WEB_ROOT`: Verzeichnis mit `index.html` und `pkg/`
- `ASSET_ROOT`: Verzeichnis mit den unter `/assets` veröffentlichten Spiel-Assets

## Tests

```sh
cargo fmt --manifest-path backend/Cargo.toml --check
cargo test --manifest-path backend/Cargo.toml
```

Unbekannte `/api/*`-Routen liefern JSON mit Status 404. HTML wird ohne Cache ausgeliefert; versionierte Dateien können langfristig zwischengespeichert werden. Gzip- und Brotli-Kompression sind aktiv. Es werden bewusst keine COOP-/COEP-Header gesetzt, damit der single-threaded Wasm-Port keine Cross-Origin-Isolation benötigt.
