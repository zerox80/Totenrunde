#!/bin/sh
set -eu

: "${CERT_DIR:?CERT_DIR muss auf das Zertifikatsziel zeigen}"
: "${RENEWED_LINEAGE:?RENEWED_LINEAGE wird von Certbot gesetzt}"

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
COMPOSE_FILE=${COMPOSE_FILE:-"$SCRIPT_DIR/compose.yaml"}
ENV_FILE=${ENV_FILE:-"$SCRIPT_DIR/.env"}

install -d -m 0755 "$CERT_DIR"
install -o 10001 -g 10001 -m 0444 "$RENEWED_LINEAGE/fullchain.pem" "$CERT_DIR/.fullchain.pem.tmp"
install -o 10001 -g 10001 -m 0400 "$RENEWED_LINEAGE/privkey.pem" "$CERT_DIR/.privkey.pem.tmp"
mv -f "$CERT_DIR/.fullchain.pem.tmp" "$CERT_DIR/fullchain.pem"
mv -f "$CERT_DIR/.privkey.pem.tmp" "$CERT_DIR/privkey.pem"

if [ "${SKIP_RESTART:-0}" != "1" ]; then
    docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" restart web
fi
