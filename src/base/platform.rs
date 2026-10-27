//! Start per Doppelklick: Fehler erscheinen als Meldungsfenster (es gibt kein Terminal) und
//! landen in `totenrunde-absturz.txt` neben dem Spiel; dazu Prüfung der Assets und Fenster-Icon.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(not(target_arch = "wasm32"))]
use bevy::asset::RenderAssetUsages;
#[cfg(not(target_arch = "wasm32"))]
use bevy::ecs::system::NonSendMarker;
#[cfg(not(target_arch = "wasm32"))]
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
#[cfg(not(target_arch = "wasm32"))]
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::winit::WINIT_WINDOWS;

#[cfg(not(target_arch = "wasm32"))]
use crate::world::mapdata::asset_root;

/// Meldungsfenster mit Fehlertext (Windows: MessageBox, Linux: zenity/kdialog, sonst stderr).
pub fn show_error(text: &str) {
    eprintln!("{text}");
    #[cfg(windows)]
    {
        use std::ffi::c_void;
        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBoxW(
                hwnd: *mut c_void,
                text: *const u16,
                caption: *const u16,
                kind: u32,
            ) -> i32;
        }
        let wide = |s: &str| {
            s.encode_utf16()
                .chain(std::iter::once(0))
                .collect::<Vec<u16>>()
        };
        let (t, c) = (wide(text), wide("Totenrunde"));
        // MB_ICONERROR | MB_SETFOREGROUND | MB_TOPMOST
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                t.as_ptr(),
                c.as_ptr(),
                0x10 | 0x1_0000 | 0x4_0000,
            )
        };
    }
    #[cfg(all(not(windows), not(target_arch = "wasm32")))]
    {
        use std::process::Command;
        let ok = Command::new("zenity")
            .args(["--error", "--no-markup", "--title=Totenrunde", "--text"])
            .arg(text)
            .status()
            .is_ok_and(|s| s.code().is_some());
        if !ok {
            let _ = Command::new("kdialog")
                .args(["--title", "Totenrunde", "--error"])
                .arg(text)
                .status();
        }
    }
}

/// Prüft vor dem Start, ob der Ordner `assets` neben dem Spiel liegt.
#[cfg(not(target_arch = "wasm32"))]
pub fn check_assets() -> bool {
    let root = asset_root();
    if root.join("maps").is_dir() {
        return true;
    }
    show_error(&format!(
        "Der Ordner „assets“ wurde nicht gefunden:\n{}\n\nBitte das heruntergeladene Archiv vollständig \
         entpacken und Totenrunde aus dem entpackten Ordner starten.",
        root.display()
    ));
    false
}

#[cfg(target_arch = "wasm32")]
pub fn check_assets() -> bool {
    true
}

/// Bei einem Absturz: Protokoll schreiben und Meldung zeigen (nur einmal; weitere Threads
/// warten, bis die Meldung geschlossen ist, damit das Programm sie nicht vorher beendet).
#[cfg(not(target_arch = "wasm32"))]
pub fn install_crash_handler() {
    static STATE: AtomicU8 = AtomicU8::new(0);
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default(info);
        if STATE
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            while STATE.load(Ordering::SeqCst) == 1 {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            return;
        }
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unbekannter Fehler".into());
        let at = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let log = format!(
            "Totenrunde {} ist abgestürzt.\n\n{msg}\n{at}\n\n{}\n",
            env!("CARGO_PKG_VERSION"),
            std::backtrace::Backtrace::force_capture()
        );
        let mut path = asset_root().with_file_name("totenrunde-absturz.txt");
        if std::fs::write(&path, &log).is_err() {
            path = std::env::temp_dir().join("totenrunde-absturz.txt");
            let _ = std::fs::write(&path, &log);
        }
        show_error(&format!(
            "Totenrunde ist abgestürzt:\n\n{msg}\n\nDetails stehen in:\n{}",
            path.display()
        ));
        STATE.store(2, Ordering::SeqCst);
    }));
}

#[cfg(target_arch = "wasm32")]
pub fn install_crash_handler() {}

/// Setzt das Fenster-Icon, sobald das Fenster existiert (Windows/X11; unter Wayland kommt
/// das Icon aus dem Desktop-Eintrag).
#[cfg(not(target_arch = "wasm32"))]
pub fn set_window_icon(mut done: Local<bool>, _main: NonSendMarker) {
    if *done {
        return;
    }
    WINIT_WINDOWS.with_borrow(|ww| {
        if ww.windows.is_empty() {
            return;
        }
        *done = true;
        let Ok(img) = Image::from_buffer(
            include_bytes!("../../assets/ui/icon.png"),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        ) else {
            return;
        };
        let (w, h) = (img.width(), img.height());
        let Some(icon) = img
            .data
            .and_then(|d| winit::window::Icon::from_rgba(d, w, h).ok())
        else {
            return;
        };
        for win in ww.windows.values() {
            win.set_window_icon(Some(icon.clone()));
        }
    });
}
