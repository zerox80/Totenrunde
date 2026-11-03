FROM rust:1.97-bookworm AS wasm-builder
WORKDIR /build
COPY backend/install-binaryen.sh /tmp/install-binaryen.sh
RUN sh /tmp/install-binaryen.sh \
    && rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.129 --locked
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY assets/maps/endstation.json ./assets/maps/endstation.json
# Der Browser lädt damit vorab verkleinerte Texturen statt der 2048er-Originale.
ENV TR_TEXTURE_VARIANTS=1
# Zwei Builds: WebGPU für moderne Browser, WebGL2 als Rückfall; main.js wählt beim Laden.
RUN cargo build --release --locked --target wasm32-unknown-unknown --bin totenrunde \
    && wasm-bindgen --target web --out-name totenrunde \
        --out-dir /web/pkg target/wasm32-unknown-unknown/release/totenrunde.wasm \
    && wasm-opt -Oz /web/pkg/totenrunde_bg.wasm -o /web/pkg/totenrunde_bg.wasm
RUN cargo build --release --locked --target wasm32-unknown-unknown --bin totenrunde \
        --no-default-features --features webgpu --target-dir target/webgpu \
    && wasm-bindgen --target web --out-name totenrunde_webgpu \
        --out-dir /web/pkg target/webgpu/wasm32-unknown-unknown/release/totenrunde.wasm \
    && wasm-opt -Oz /web/pkg/totenrunde_webgpu_bg.wasm -o /web/pkg/totenrunde_webgpu_bg.wasm

# Verkleinerte Texturvarianten (512/1024) und verlustfreie Rekompression der Originale.
# Der Browser lädt danach nur die Auflösung, die die Grafikstufe wirklich anzeigt.
FROM python:3.13-slim AS asset-optimizer
WORKDIR /build
COPY backend/optimize-textures.requirements.txt ./
RUN pip install --no-cache-dir --quiet -r optimize-textures.requirements.txt
COPY assets ./assets
COPY backend/optimize-textures.py backend/texture-golden.json ./
RUN python3 optimize-textures.py --root assets --sizes 512 1024 --recompress-originals

FROM rust:1.97-bookworm AS backend-builder
WORKDIR /build/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --no-create-home app \
    && mkdir /data && chown app:app /data
WORKDIR /app
COPY --from=backend-builder --chown=app:app /build/backend/target/release/totenrunde-web /app/totenrunde-web
COPY --from=wasm-builder --chown=app:app /web/pkg /app/web/pkg
COPY --chown=app:app backend/web/ /app/web/
COPY --from=asset-optimizer --chown=app:app /build/assets/ /app/assets/
COPY --chown=app:app LICENSE /app/LICENSE
USER app
# Konten und Statistik (SQLite) liegen in /data, das als Volume erhalten bleibt
VOLUME /data
ENV HOST=0.0.0.0 PORT=8443 WEB_ROOT=/app/web ASSET_ROOT=/app/assets DATA_DIR=/data \
    TLS_CERT_PATH=/certs/fullchain.pem TLS_KEY_PATH=/certs/privkey.pem \
    TLS_SERVER_NAME=localhost
EXPOSE 8443/tcp 8443/udp
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD ["/app/totenrunde-web", "--healthcheck"]
ENTRYPOINT ["/app/totenrunde-web"]
