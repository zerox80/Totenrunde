//! Windows: Programmsymbol und Versionsangaben in die .exe einbetten.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=packaging/totenrunde.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ico = Path::new(&dir).join("packaging").join("totenrunde.ico");
    let ver = std::env::var("CARGO_PKG_VERSION").unwrap();
    let num: Vec<&str> = ver.split(['.', '-']).take(3).collect();
    let num = format!("{},{},{},0", num[0], num[1], num[2]);
    let rc = format!(
        r#"1 ICON "{ico}"
1 VERSIONINFO
FILEVERSION {num}
PRODUCTVERSION {num}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040704B0"
    BEGIN
      VALUE "FileDescription", "Totenrunde"
      VALUE "FileVersion", "{ver}"
      VALUE "InternalName", "Totenrunde"
      VALUE "OriginalFilename", "Totenrunde.exe"
      VALUE "ProductName", "Totenrunde"
      VALUE "ProductVersion", "{ver}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x407, 1200
  END
END
"#,
        ico = ico.display().to_string().replace('\\', "\\\\"),
    );
    let out = Path::new(&std::env::var("OUT_DIR").unwrap()).join("totenrunde.rc");
    std::fs::write(&out, rc).unwrap();
    if let Err(e) = embed_resource::compile(&out, embed_resource::NONE).manifest_optional() {
        println!("cargo:warning=Windows-Ressourcen nicht eingebettet: {e}");
    }
}
