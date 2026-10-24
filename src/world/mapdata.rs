//! Datenformat der Karten (wird vom Blender-Skript `blender/map_endstation.py` erzeugt).
//! Alle Koordinaten sind bereits in Bevy-Koordinaten (Y oben, -Z = Norden).

use bevy::prelude::*;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone, Debug)]
pub struct NavSpec {
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub cell: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Placement {
    pub pos: [f32; 3],
    pub yaw: f32,
}

impl Placement {
    pub fn pos(&self) -> Vec3 {
        Vec3::from_array(self.pos)
    }
    pub fn transform(&self) -> Transform {
        Transform::from_translation(self.pos()).with_rotation(Quat::from_rotation_y(self.yaw))
    }
    /// Richtung, in die die Vorderseite des Objekts zeigt.
    pub fn forward(&self) -> Vec3 {
        Quat::from_rotation_y(self.yaw) * Vec3::NEG_Z
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ColliderSpec {
    pub c: [f32; 3],
    pub h: [f32; 3],
    pub yaw: f32,
    pub f: u8,
    pub kind: String,
    #[serde(default)]
    pub door: Option<u16>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ZoneSpec {
    pub id: usize,
    pub name: String,
    pub open: bool,
    /// Rechtecke in Bevy-XZ: [x0, z0, x1, z1]
    pub rects: Vec<[f32; 4]>,
}

impl ZoneSpec {
    pub fn contains(&self, p: Vec3) -> bool {
        self.rects
            .iter()
            .any(|r| p.x >= r[0] && p.x <= r[2] && p.z >= r[1] && p.z <= r[3])
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct DoorPartSpec {
    pub node: String,
    pub to_pos: [f32; 3],
    pub to_yaw: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DoorSpec {
    pub id: u16,
    pub cost: u32,
    pub kind: String,
    pub zones: Vec<usize>,
    pub prompt: [f32; 3],
    pub parts: Vec<DoorPartSpec>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BarricadeSpec {
    pub id: usize,
    pub zone: usize,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub width: f32,
    pub sill: f32,
    pub top: f32,
    pub inside: [f32; 3],
    pub outside: [f32; 3],
}

#[derive(Deserialize, Clone, Debug)]
pub struct SpawnerSpec {
    pub zone: usize,
    pub kind: String,
    #[serde(default)]
    pub barricade: Option<usize>,
    pub pos: [f32; 3],
}

#[derive(Deserialize, Clone, Debug)]
pub struct WallBuySpec {
    pub weapon: String,
    pub pos: [f32; 3],
    pub yaw: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PerkSpec {
    pub perk: String,
    pub pos: [f32; 3],
    pub yaw: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BoxSpotSpec {
    pub pos: [f32; 3],
    pub yaw: f32,
    pub start: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LightSpec {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    #[serde(default)]
    pub shadows: bool,
    #[serde(default)]
    pub flicker: f32,
    #[serde(default)]
    pub needs_power: bool,
    /// Licht ist in die Lightmaps gebacken (wirkt dort nur noch spekular)
    #[serde(default)]
    pub baked: bool,
}

/// Gebackene Beleuchtung für eine Gruppe statischer Objekte (ein Atlas).
#[derive(Deserialize, Clone, Debug)]
pub struct LightmapSpec {
    pub image: String,
    pub exposure: f32,
    pub objects: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FireSpec {
    pub pos: [f32; 3],
    pub scale: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SunSpec {
    pub dir: [f32; 3],
    pub color: [f32; 3],
    pub illuminance: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FogSpec {
    pub color: [f32; 3],
    pub start: f32,
    pub end: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AmbientSpec {
    pub color: [f32; 3],
    pub brightness: f32,
}

#[derive(Deserialize, Clone, Debug, Resource)]
pub struct MapFile {
    pub name: String,
    pub model: String,
    pub nav: NavSpec,
    pub player_start: Placement,
    pub colliders: Vec<ColliderSpec>,
    pub zones: Vec<ZoneSpec>,
    pub doors: Vec<DoorSpec>,
    pub barricades: Vec<BarricadeSpec>,
    pub spawners: Vec<SpawnerSpec>,
    pub wallbuys: Vec<WallBuySpec>,
    pub perks: Vec<PerkSpec>,
    pub box_spots: Vec<BoxSpotSpec>,
    pub pap: Option<Placement>,
    pub power: Option<Placement>,
    pub lights: Vec<LightSpec>,
    pub fires: Vec<FireSpec>,
    pub sun: SunSpec,
    pub fog: FogSpec,
    pub ambient: AmbientSpec,
    #[serde(default)]
    pub lightmaps: Vec<LightmapSpec>,
}

impl MapFile {
    pub fn zone_at(&self, p: Vec3) -> Option<usize> {
        self.zones.iter().find(|z| z.contains(p)).map(|z| z.id)
    }
}

/// Verzeichnis der Assets (wie Bevys Dateisystem-Reader: BEVY_ASSET_ROOT, Cargo-Manifest
/// oder Ordner der ausführbaren Datei).
pub fn asset_root() -> PathBuf {
    if let Ok(p) = std::env::var("BEVY_ASSET_ROOT") {
        return PathBuf::from(p).join("assets");
    }
    if let Ok(p) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(p).join("assets");
    }
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default()
        .join("assets")
}

pub fn load_map(name: &str) -> MapFile {
    let path = asset_root().join("maps").join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Karte {} konnte nicht gelesen werden: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("Karte {} ist fehlerhaft: {e}", path.display()))
}
