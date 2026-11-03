//! Zentrale Asset-Handles und Ladezustand.

use bevy::asset::{LoadState, RecursiveDependencyLoadState};
use bevy::pbr::Lightmap;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::world_serialization::WorldAsset;

use super::AppState;
use crate::world::level::MapReady;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadProgress>()
            .add_systems(Startup, load_assets)
            .add_systems(
                Update,
                (
                    check_loaded.run_if(not(resource_exists::<AssetsReady>)),
                    track_textures.run_if(not(resource_exists::<TexturesReady>)),
                ),
            );
    }
}

/// Wird eingefügt, sobald alle wichtigen Assets geladen sind.
#[derive(Resource)]
pub struct AssetsReady;

/// Wird eingefügt, sobald alle Texturen der bekannten Materialien und Lightmaps geladen sind.
/// Ohne sie zeichnet Bevy die betroffenen Flächen nicht; die Karte bliebe grau.
#[derive(Resource)]
pub struct TexturesReady;

/// Ladefortschritt für den Ladebildschirm.
#[derive(Resource, Default)]
pub struct LoadProgress {
    /// 0..=1, steigt nur an
    pub fraction: f32,
    pub textures_done: usize,
    pub textures_total: usize,
}

/// Zombie-Modelle (Varianten). Fehlende Dateien werden übersprungen.
pub const ZOMBIE_MODELS: &[&str] = &["models/zombie_m.gltf", "models/zombie_f.gltf"];

#[derive(Resource)]
pub struct GameAssets {
    pub zombies: Vec<Handle<Gltf>>,
    /// Alle Waffen der Mysteriösen Kiste. Schon beim Laden geholt und dauerhaft gehalten:
    /// sonst lädt die Kiste beim Rollen jedes Modell samt Texturen neu (Ruckeln).
    pub weapons: Vec<Handle<Gltf>>,
    /// Modelle, die sonst erst mitten im Spiel geladen würden (Perks, Power-ups, Verstärker,
    /// Wandwaffen). Im Browser würde das Dekodieren und Mipmaps-Bauen den Hauptthread
    /// blockieren – genau die kurzen Ruckler beim ersten Auftauchen. Nur dort befüllt.
    pub preload: Vec<Handle<Gltf>>,
    pub planks: Handle<Image>,
    pub font_bold: Handle<Font>,
    pub font_regular: Handle<Font>,
    pub font_black: Handle<Font>,
}

/// Ein Zombie-Modell mit Animationsgraph und den Knoten aller Clips.
#[derive(Clone)]
pub struct ZombieModel {
    pub scene: Handle<WorldAsset>,
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub sprint: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub tear: AnimationNodeIndex,
    pub climb: AnimationNodeIndex,
    pub rise: AnimationNodeIndex,
    pub death_fwd: AnimationNodeIndex,
    pub death_back: AnimationNodeIndex,
    pub crawl: AnimationNodeIndex,
}

/// Alle geladenen Zombie-Varianten.
#[derive(Resource, Clone)]
pub struct ZombieAnims {
    pub models: Vec<ZombieModel>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let zombies = ZOMBIE_MODELS
        .iter()
        .filter(|p| crate::world::mapdata::asset_exists(p))
        .map(|p| asset_server.load(*p))
        .collect();
    let weapons = crate::game::interact::mystery_box::box_weapons()
        .map(|w| crate::weapons::def(w).model())
        .filter(|p| crate::world::mapdata::asset_exists(p))
        .map(|p| asset_server.load(p))
        .collect();
    let preload = if cfg!(target_arch = "wasm32") {
        web_preload_paths()
            .into_iter()
            .map(|p| asset_server.load(p))
            .collect()
    } else {
        Vec::new()
    };
    commands.insert_resource(GameAssets {
        zombies,
        weapons,
        preload,
        planks: asset_server.load("textures/planks.png"),
        font_bold: asset_server.load("fonts/Montserrat-ExtraBold.otf"),
        font_regular: asset_server.load("fonts/Montserrat-SemiBold.otf"),
        font_black: asset_server.load("fonts/Montserrat-Black.otf"),
    });
}

/// Modelle, die der Browser zusätzlich zu Zombies und Kistenwaffen vorab lädt.
fn web_preload_paths() -> Vec<String> {
    use crate::game::{perks::Perk, powerups::PowerupKind};
    let box_weapons: Vec<usize> = crate::game::interact::mystery_box::box_weapons().collect();
    crate::weapons::WEAPONS
        .iter()
        .enumerate()
        .filter(|(i, _)| !box_weapons.contains(i))
        .map(|(_, w)| w.model())
        .chain(Perk::ALL.map(Perk::model))
        .chain(PowerupKind::ALL.map(|kind| kind.model().to_string()))
        .chain(
            [
                "models/weapons/knife.gltf",
                "models/weapons/perk_bottle.gltf",
                "models/pap.gltf",
                "models/power_switch.gltf",
                "models/mystery_box.gltf",
                "models/mystery_box_lid.gltf",
                "models/teddy.gltf",
            ]
            .map(String::from),
        )
        .collect()
}

fn check_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    state: Res<State<AppState>>,
) {
    let handles = || {
        assets
            .zombies
            .iter()
            .chain(&assets.weapons)
            .chain(&assets.preload)
    };
    // Ein fehlgeschlagener Ladevorgang darf den Ladebildschirm nicht für immer blockieren
    // (ein Netzwerkfehler im Browser genügt): Das Spiel läuft ohne dieses Modell weiter.
    let load_state = |h: &Handle<Gltf>| asset_server.get_recursive_dependency_load_state(h.id());
    if handles().any(|h| {
        matches!(
            load_state(h),
            Some(RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading)
        )
    }) {
        return;
    }
    for handle in handles() {
        if let Some(RecursiveDependencyLoadState::Failed(error)) = load_state(handle) {
            warn!(
                "Asset {:?} nicht vollständig geladen: {error}",
                handle.path()
            );
        }
    }
    let names = [
        "Idle",
        "Walk",
        "Run",
        "Sprint",
        "Attack",
        "Tear",
        "Climb",
        "Rise",
        "DeathFwd",
        "DeathBack",
        "Crawl",
    ];
    let mut models = Vec::new();
    for handle in &assets.zombies {
        if matches!(
            asset_server.get_load_state(handle.id()),
            Some(LoadState::Failed(_))
        ) {
            continue;
        }
        let Some(z) = gltfs.get(handle) else { return };
        let clips: Vec<Handle<AnimationClip>> = names
            .iter()
            .map(|n| {
                z.named_animations
                    .get(*n)
                    .unwrap_or_else(|| panic!("Zombie-Animation '{n}' fehlt"))
                    .clone()
            })
            .collect();
        let (graph, idx) = AnimationGraph::from_clips(clips);
        models.push(ZombieModel {
            scene: z.scenes[0].clone(),
            graph: graphs.add(graph),
            idle: idx[0],
            walk: idx[1],
            run: idx[2],
            sprint: idx[3],
            attack: idx[4],
            tear: idx[5],
            climb: idx[6],
            rise: idx[7],
            death_fwd: idx[8],
            death_back: idx[9],
            crawl: idx[10],
        });
    }
    if models.is_empty() {
        panic!("Keine Zombie-Modelle gefunden (assets/models/zombie_*.gltf) – bitte Assets bauen");
    }
    info!(
        "Assets geladen: {} Zombie-Varianten (Zustand {:?})",
        models.len(),
        state.get()
    );
    commands.insert_resource(ZombieAnims { models });
    commands.insert_resource(AssetsReady);
}

/// Zählt die geladenen Texturen aller Materialien und Lightmaps. Erst wenn Karte und Zombies
/// bereit sind, kennt das Spiel alle großen Materialien; bis dahin zählen nur diese beiden
/// Schritte (die ersten 10 %).
fn track_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    materials: Res<Assets<StandardMaterial>>,
    lightmaps: Query<&Lightmap>,
    map: Option<Res<MapReady>>,
    models: Option<Res<AssetsReady>>,
    mut progress: ResMut<LoadProgress>,
) {
    let mut images = HashSet::new();
    for (_, material) in materials.iter() {
        images.extend(
            [
                &material.base_color_texture,
                &material.emissive_texture,
                &material.metallic_roughness_texture,
                &material.normal_map_texture,
                &material.occlusion_texture,
            ]
            .into_iter()
            .flatten()
            .map(Handle::id),
        );
    }
    images.extend(lightmaps.iter().map(|lightmap| lightmap.image.id()));
    // Selbst erzeugte Bilder kennt der AssetServer nicht (None); sie gelten als fertig.
    let pending = images
        .iter()
        .filter(|id| {
            matches!(
                asset_server.get_load_state(**id),
                Some(LoadState::NotLoaded | LoadState::Loading)
            )
        })
        .count();
    let steps = (map.is_some() as u8 + models.is_some() as u8) as f32 / 2.0;
    let all_known = map.is_some() && models.is_some();
    let textures = if all_known && !images.is_empty() {
        (images.len() - pending) as f32 / images.len() as f32
    } else {
        0.0
    };
    progress.textures_total = images.len();
    progress.textures_done = images.len() - pending;
    progress.fraction = progress.fraction.max(0.1 * steps + 0.9 * textures);
    if all_known && pending == 0 {
        progress.fraction = 1.0;
        info!("Alle {} Texturen geladen", images.len());
        commands.insert_resource(TexturesReady);
    }
}

/// Pfad einer vorab verkleinerten Texturvariante (`textures/<größe>/…`), die
/// `backend/optimize-textures.py` erzeugt. Nur im Browser relevant: Dort lädt der
/// AssetReader sonst die Originaldatei, obwohl die Laufzeit sie gleich wieder
/// herunterrechnet. Lightmaps bleiben unangetastet, ab 2048 wird das Original geladen.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn texture_variant(path: &str, texture_max: u32) -> Option<String> {
    let size = match texture_max {
        0..=512 => "512",
        513..=1024 => "1024",
        _ => return None,
    };
    let (prefix, rest) = path.split_once("textures/")?;
    let name = rest.rsplit('/').next()?;
    if !name.ends_with(".png") || name.starts_with("lm_") {
        return None;
    }
    Some(format!("{prefix}textures/{size}/{rest}"))
}

/// Lädt Texturen aus den Varianten-Ordnern und fällt auf das Original zurück, wenn eine
/// Variante auf dem Server fehlt (Wasm-Build und ausgelieferte Assets passen nicht zusammen).
#[cfg(target_arch = "wasm32")]
pub struct VariantAssetReader {
    variants: bevy::asset::io::wasm::HttpWasmAssetReader,
    originals: bevy::asset::io::wasm::HttpWasmAssetReader,
    texture_max: u32,
}

#[cfg(target_arch = "wasm32")]
impl VariantAssetReader {
    pub fn new(root: &str, texture_max: u32) -> Self {
        use bevy::asset::io::wasm::HttpWasmAssetReader;
        use std::borrow::Cow;
        Self {
            variants: HttpWasmAssetReader::new(root).with_request_mapper(move |path| {
                texture_variant(path, texture_max).map_or(Cow::Borrowed(path), Cow::Owned)
            }),
            originals: HttpWasmAssetReader::new(root),
            texture_max,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl bevy::asset::io::AssetReader for VariantAssetReader {
    async fn read<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<impl bevy::asset::io::Reader + 'a, bevy::asset::io::AssetReaderError> {
        use bevy::asset::io::AssetReaderError;
        if path
            .to_str()
            .is_some_and(|p| texture_variant(p, self.texture_max).is_some())
        {
            match self.variants.read(path).await {
                Err(AssetReaderError::NotFound(missing)) => {
                    warn!("Texturvariante {missing:?} fehlt, lade das Original");
                }
                result => return result,
            }
        }
        self.originals.read(path).await
    }

    async fn read_meta<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<impl bevy::asset::io::Reader + 'a, bevy::asset::io::AssetReaderError> {
        self.originals.read_meta(path).await
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<Box<bevy::asset::io::PathStream>, bevy::asset::io::AssetReaderError> {
        self.originals.read_directory(path).await
    }

    async fn is_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<bool, bevy::asset::io::AssetReaderError> {
        self.originals.is_directory(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::texture_variant;

    #[test]
    fn texture_variants_only_for_known_textures() {
        assert_eq!(
            texture_variant("assets/textures/concrete.png", 1024).as_deref(),
            Some("assets/textures/1024/concrete.png")
        );
        assert_eq!(
            texture_variant("textures/concrete_n.png", 512).as_deref(),
            Some("textures/512/concrete_n.png")
        );
        assert_eq!(
            texture_variant("textures/lm_endstation_main.png", 1024),
            None
        );
        assert_eq!(texture_variant("textures/concrete.png", 2048), None);
        assert_eq!(texture_variant("ui/concrete.png", 1024), None);
    }
}
