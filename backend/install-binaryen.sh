#!/bin/sh
# Installiert eine feste binaryen-Version nach /opt/binaryen und wasm-opt nach /usr/local/bin.
# Bewusst nicht das Distributionspaket: Das wasm-opt aus Debian bookworm (Version 108) lenkt
# den Export __wbindgen_externrefs auf die Funktionstabelle fester Größe um, worauf im Browser
# "WebAssembly.Table.grow(): failed to grow table" auftritt.
set -eu

VERSION=133
case "$(uname -m)" in
    x86_64)
        ARCH=x86_64
        SHA256=2dc9c7813f5375db93d96ead4b78222fcc3e2677bbb832297af4797782a37489
        ;;
    aarch64 | arm64)
        ARCH=aarch64
        SHA256=89c07ea56faf38d0fbecf36ca8ec0721756716185f265b568e133d427f299bf8
        ;;
    *)
        echo "binaryen: Architektur $(uname -m) wird nicht unterstützt" >&2
        exit 1
        ;;
esac

ARCHIVE=$(mktemp)
trap 'rm -f "$ARCHIVE"' EXIT
curl -fsSL -o "$ARCHIVE" \
    "https://github.com/WebAssembly/binaryen/releases/download/version_$VERSION/binaryen-version_$VERSION-$ARCH-linux.tar.gz"
echo "$SHA256  $ARCHIVE" | sha256sum -c -
rm -rf /opt/binaryen
mkdir -p /opt/binaryen
tar -xzf "$ARCHIVE" -C /opt/binaryen --strip-components=1
ln -sf /opt/binaryen/bin/wasm-opt /usr/local/bin/wasm-opt
wasm-opt --version
