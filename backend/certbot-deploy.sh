#!/bin/sh
set -eu

: "${CERT_DIR:?CERT_DIR muss auf das Zertifikatsziel zeigen}"
: "${RENEWED_LINEAGE:?RENEWED_LINEAGE wird von Certbot gesetzt}"

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
COMPOSE_FILE=${COMPOSE_FILE:-"$SCRIPT_DIR/compose.yaml"}
ENV_FILE=${ENV_FILE:-"$SCRIPT_DIR/.env"}

# UID/GID des Container-Benutzers "app". Auf dem Host existiert er nicht, deshalb
# chown statt install -o/-g: Die Rust-coreutils (Ubuntu 25.10+) lehnen dort
# numerische IDs ohne passwd-Eintrag ab.
APP_ID=10001

stage() {
    rm -f "$CERT_DIR/.$1.tmp"
    cp "$RENEWED_LINEAGE/$1" "$CERT_DIR/.$1.tmp"
    chown "$APP_ID:$APP_ID" "$CERT_DIR/.$1.tmp"
    chmod "$2" "$CERT_DIR/.$1.tmp"
}

install -d -m 0755 "$CERT_DIR"
umask 077
stage fullchain.pem 0444
stage privkey.pem 0400
mv -f "$CERT_DIR/.fullchain.pem.tmp" "$CERT_DIR/fullchain.pem"
mv -f "$CERT_DIR/.privkey.pem.tmp" "$CERT_DIR/privkey.pem"

if [ "${SKIP_RESTART:-0}" != "1" ]; then
    docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" restart web
fi
