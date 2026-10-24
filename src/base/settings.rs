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
    /// Echtzeit-Schatten (Mond, flackernde Lampen); statische Schatten stecken in den Lightmaps
    pub shadows: bool,
    /// Kantenglättung: 0 = aus, 1 = FXAA, 2/4 = MSAA
    pub msaa: u32,
    /// Größte Texturkante in Pixeln (2048 = hoch, 1024 = mittel, 512 = niedrig)
    pub texture_max: u32,
    /// Grafikstufe: 0 Niedrig, 1 Mittel, 2 Hoch, 3 Extrem, 4 Benutzerdefiniert
    pub quality: u8,
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
            bloom: true,
            film: true,
            auto_exposure: true,
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
        let (tex, msaa, shadows, bloom, film, ae, cones) = match self.quality {
            0 => (512, 0, false, false, false, false, false),
            1 => (1024, 1, false, true, true, true, true),
            2 => (1024, 4, false, true, true, true, true),
            _ => (2048, 4, true, true, true, true, true),
        };
        self.texture_max = tex;
        self.msaa = msaa;
        self.shadows = shadows;
        self.bloom = bloom;
        self.film = film;
        self.auto_exposure = ae;
        self.light_cones = cones;
    }
}

fn settings_path() -> std::path::PathBuf {
    crate::world::mapdata::asset_root().with_file_name("settings.json")
}

impl Settings {
    pub fn load() -> Self {
        let mut s: Self = std::fs::read_to_string(settings_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        // Überschreiben für Tests: TR_QUALITY=0..3, TR_SHADOWS=0/1, TR_MSAA=0/1/2/4
        if let Some(q) = std::env::var("TR_QUALITY").ok().and_then(|v| v.parse().ok()) {
            s.apply_preset(q);
        }
        if let Ok(v) = std::env::var("TR_SHADOWS") {
            s.shadows = v == "1";
        }
        if let Some(v) = std::env::var("TR_MSAA").ok().and_then(|v| v.parse().ok()) {
            s.msaa = v;
        }
        s
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(settings_path(), s);
        }
    }
}

/// Grafik-Menü ist offen (Esc schließt dann zuerst das Menü).
#[derive(Resource, Default)]
pub struct GraphicsMenuOpen(pub bool);
