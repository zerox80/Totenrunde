//! Spielereinstellungen (werden in `settings.json` neben den Assets gespeichert).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Maus-Empfindlichkeit (1.0 = Standard)
    pub sensitivity: f32,
    /// Faktor beim Zielen
    pub ads_sensitivity: f32,
    pub invert_y: bool,
    /// Vertikales Sichtfeld in Grad
    pub fov_deg: f32,
    pub master_volume: f32,
    pub music_volume: f32,
    pub head_bob: f32,
    /// Echtzeit-Schatten der flackernden Lampen und Feuer; statische Schatten (auch die des
    /// Mondes) stecken in den Lightmaps
    pub shadows: bool,
    /// Kantenglättung: 0 = aus, 1 = FXAA, 2/4 = MSAA
    pub msaa: u32,
    /// Größte Texturkante in Pixeln (2048 = hoch, 1024 = mittel, 512 = niedrig)
    pub texture_max: u32,
    /// Grafikstufe: 0 Niedrig, 1 Mittel, 2 Hoch, 3 Extrem, 4 Benutzerdefiniert
    pub quality: u8,
    /// Auflösung des 3D-Renderpuffers relativ zur physischen Fenstergröße
    pub render_scale: f32,
    pub bloom: bool,
    /// Filmlook: Farbkorrektur, Vignette, leichte Farbsäume
    pub film: bool,
    /// Augen-Anpassung (automatische Belichtung)
    pub auto_exposure: bool,
    /// Sichtbare Lichtkegel unter Lampen
    pub light_cones: bool,
    pub vsync: bool,
    pub show_fps: bool,
}

/// Namen der Grafikstufen
pub const QUALITY_NAMES: [&str; 5] = ["NIEDRIG", "MITTEL", "HOCH", "EXTREM", "EIGENE"];

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            ads_sensitivity: 0.75,
            invert_y: false,
            fov_deg: 62.0,
            master_volume: 0.8,
            music_volume: 0.7,
            head_bob: 1.0,
            shadows: false,
            msaa: 4,
            texture_max: 1024,
            quality: 2,
            render_scale: 1.0,
            bloom: true,
            film: true,
            auto_exposure: cfg!(not(target_arch = "wasm32")),
            light_cones: true,
            vsync: true,
            show_fps: false,
        }
    }
}

impl Settings {
    /// Setzt alle Grafikwerte einer Stufe (0–3).
    pub fn apply_preset(&mut self, q: u8) {
        self.quality = q.min(3);
        let (tex, msaa, scale, shadows, bloom, film, ae, cones) = match self.quality {
            0 => (512, 0, 0.5, false, false, false, false, false),
            1 => (1024, 1, 0.75, false, true, true, true, true),
            2 => (1024, 4, 1.0, false, true, true, true, true),
            _ => (2048, 4, 1.0, true, true, true, true, true),
        };
        self.texture_max = tex;
        self.msaa = msaa;
        self.render_scale = scale;
        self.shadows = shadows;
        self.bloom = bloom;
        self.film = film;
        self.auto_exposure = ae && cfg!(not(target_arch = "wasm32"));
        self.light_cones = cones;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn settings_path() -> std::path::PathBuf {
    crate::world::mapdata::asset_root().with_file_name("settings.json")
}

#[cfg(target_arch = "wasm32")]
fn browser_default_quality() -> u8 {
    let cores = web_sys::window()
        .map(|window| window.navigator().hardware_concurrency())
        .unwrap_or(0.0);
    match cores {
        1.0..=4.0 => 0,
        4.5..=8.0 => 1,
        8.5.. => 2,
        _ => 1,
    }
}

impl Settings {
    pub fn load() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let mut s: Self = std::fs::read_to_string(settings_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        #[cfg(target_arch = "wasm32")]
        let mut s: Self = match web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("totenrunde.settings").ok().flatten())
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(settings) => settings,
            None => {
                let mut settings = Self::default();
                settings.apply_preset(browser_default_quality());
                settings
            }
        };
        #[cfg(target_arch = "wasm32")]
        {
            s.auto_exposure = false;
            s.render_scale = s.render_scale.clamp(0.5, 1.0);
        }
        // Überschreiben für Tests: TR_QUALITY=0..3, TR_SHADOWS=0/1, TR_MSAA=0/1/2/4
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(q) = std::env::var("TR_QUALITY")
                .ok()
                .and_then(|v| v.parse().ok())
            {
                s.apply_preset(q);
            }
            if let Ok(v) = std::env::var("TR_SHADOWS") {
                s.shadows = v == "1";
            }
            if let Some(v) = std::env::var("TR_MSAA").ok().and_then(|v| v.parse().ok()) {
                s.msaa = v;
            }
        }
        s
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            #[cfg(not(target_arch = "wasm32"))]
            let _ = std::fs::write(settings_path(), s);
            #[cfg(target_arch = "wasm32")]
            if let Some(storage) =
                web_sys::window().and_then(|window| window.local_storage().ok().flatten())
            {
                let _ = storage.set_item("totenrunde.settings", &s);
            }
        }
    }
}

/// Grafik-Menü ist offen (Esc schließt dann zuerst das Menü).
#[derive(Resource, Default)]
pub struct GraphicsMenuOpen(pub bool);
