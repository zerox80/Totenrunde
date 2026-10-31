//! Zentrale Asset-Handles und Ladezustand.

use bevy::asset::LoadState;
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
    commands.insert_resource(GameAssets {
        zombies,
        planks: asset_server.load("textures/planks.png"),
        font_bold: asset_server.load("fonts/Montserrat-ExtraBold.otf"),
        font_regular: asset_server.load("fonts/Montserrat-SemiBold.otf"),
        font_black: asset_server.load("fonts/Montserrat-Black.otf"),
    });
}

fn check_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    state: Res<State<AppState>>,
) {
    if assets
        .zombies
        .iter()
        .any(|z| !asset_server.is_loaded_with_dependencies(z))
    {
        return;
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
