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
}

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
        }
    }
}

fn settings_path() -> std::path::PathBuf {
    crate::world::mapdata::asset_root().with_file_name("settings.json")
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(settings_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(settings_path(), s);
        }
    }
}
