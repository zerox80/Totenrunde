#!/usr/bin/env python3
"""Prüft, ob __wbindgen_externrefs auf eine wachsende externref-Tabelle zeigt.

wasm-bindgen ruft beim Start table.grow() auf genau diesem Export auf. Ein fehlerhaftes
wasm-opt kann den Export auf die Funktionstabelle fester Größe umlenken; der Build läuft
dann durch, das Spiel scheitert aber erst im Browser.
"""

import sys

EXPORT = "__wbindgen_externrefs"
EXTERNREF = 0x6F


def leb128(data, offset):
    result = shift = 0
    while True:
        byte = data[offset]
        offset += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if byte < 0x80:
            return result, offset


def name(data, offset):
    length, offset = leb128(data, offset)
    return data[offset : offset + length].decode("utf-8"), offset + length


def limits(data, offset):
    flags = data[offset]
    minimum, offset = leb128(data, offset + 1)
    maximum = None
    if flags & 1:
        maximum, offset = leb128(data, offset)
    return (minimum, maximum), offset


def tables_and_exports(data):
    if data[:4] != b"\0asm":
        raise ValueError("keine WebAssembly-Datei")
    tables, exports = [], {}
    offset = 8
    while offset < len(data):
        section = data[offset]
        size, offset = leb128(data, offset + 1)
        end = offset + size
        count, cursor = leb128(data, offset) if section in (2, 4, 7) else (0, offset)
        for _ in range(count):
            if section == 2:  # Importe
                _, cursor = name(data, cursor)
                _, cursor = name(data, cursor)
                kind = data[cursor]
                cursor += 1
                if kind == 0:
                    _, cursor = leb128(data, cursor)
                elif kind == 1:
                    element = data[cursor]
                    bounds, cursor = limits(data, cursor + 1)
                    tables.append((element, bounds))
                elif kind == 2:
                    _, cursor = limits(data, cursor)
                elif kind == 3:
                    cursor += 2
                else:
                    raise ValueError(f"unbekannte Importart {kind}")
            elif section == 4:  # Tabellen
                element = data[cursor]
                bounds, cursor = limits(data, cursor + 1)
                tables.append((element, bounds))
            elif section == 7:  # Exporte
                export, cursor = name(data, cursor)
                kind = data[cursor]
                index, cursor = leb128(data, cursor + 1)
                if kind == 1:
                    exports[export] = index
        offset = end
    return tables, exports


def main():
    path = sys.argv[1]
    with open(path, "rb") as file:
        tables, exports = tables_and_exports(file.read())
    if EXPORT not in exports:
        sys.exit(f"{path}: Export {EXPORT} fehlt")
    index = exports[EXPORT]
    element, (minimum, maximum) = tables[index]
    if element != EXTERNREF or (maximum is not None and maximum <= minimum):
        sys.exit(
            f"{path}: {EXPORT} zeigt auf Tabelle {index} (Typ 0x{element:02x}, "
            f"min={minimum}, max={maximum}); erwartet wird eine wachsende externref-Tabelle"
        )
    print(f"{path}: {EXPORT} -> Tabelle {index} (externref, min={minimum}, max={maximum})")


if __name__ == "__main__":
    main()
