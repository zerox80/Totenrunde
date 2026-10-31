FROM rust:1.97-bookworm AS wasm-builder
WORKDIR /build
RUN apt-get update \
    && apt-get install --yes --no-install-recommends binaryen \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.129 --locked
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY assets/maps/endstation.json ./assets/maps/endstation.json
RUN cargo build --release --locked --target wasm32-unknown-unknown --bin totenrunde \
    && wasm-bindgen --target web --out-name totenrunde \
        --out-dir /web/pkg target/wasm32-unknown-unknown/release/totenrunde.wasm \
    && wasm-opt -Oz /web/pkg/totenrunde_bg.wasm -o /web/pkg/totenrunde_bg.wasm

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
    && useradd --uid 10001 --gid app --no-create-home app
WORKDIR /app
COPY --from=backend-builder --chown=app:app /build/backend/target/release/totenrunde-web /app/totenrunde-web
COPY --from=wasm-builder --chown=app:app /web/pkg /app/web/pkg
COPY --chown=app:app backend/web/ /app/web/
COPY --chown=app:app assets/ /app/assets/
COPY --chown=app:app LICENSE /app/LICENSE
USER app
ENV HOST=0.0.0.0 PORT=8443 WEB_ROOT=/app/web ASSET_ROOT=/app/assets \
    TLS_CERT_PATH=/certs/fullchain.pem TLS_KEY_PATH=/certs/privkey.pem \
    TLS_SERVER_NAME=localhost
EXPOSE 8443/tcp 8443/udp
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD ["/app/totenrunde-web", "--healthcheck"]
ENTRYPOINT ["/app/totenrunde-web"]
