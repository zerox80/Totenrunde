//! Lädt die Karte (JSON + Modell), Mond, Umgebungslicht und Nebelfarbe und baut daraus
//! Kollisionswelt und Navigationsgitter.

use std::collections::HashMap;

use bevy::light::CascadeShadowConfigBuilder;
use bevy::pbr::Lightmap;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use super::collision::CollisionWorld;
use super::lighting::spawn_map_lights;
use super::mapdata::{self, MapFile};
use super::nav::NavGrid;
use crate::base::AppState;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), load_level)
            .add_systems(OnEnter(AppState::Playing), reset_level);
    }
}

/// Wird eingefügt, sobald das Kartenmodell gespawnt ist.
#[derive(Resource)]
pub struct MapReady;

#[derive(Component)]
pub struct MapRoot;

/// Bewegliches Teil einer Tür (aus dem Kartenmodell, per Name gefunden).
#[derive(Component)]
pub struct DoorPart {
    pub door: u16,
    pub closed: Transform,
    pub open: Transform,
}

fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<crate::base::settings::Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let map = mapdata::load_map("endstation");
    let col = CollisionWorld::from_map(&map);
    let nav = NavGrid::build(&map, &col);
    info!(
        "Karte '{}' geladen: {} Kollider, Navigationsgitter {}x{}",
        map.name,
        col.boxes.len(),
        nav.w,
        nav.h
    );

    commands
        .spawn((
            MapRoot,
            Name::new("Karte"),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(map.model.clone()))),
        ))
        .observe(on_map_ready);

    // Mondlicht mit Schatten
    let sun_dir = Vec3::from_array(map.sun.dir).normalize();
    commands.spawn((
        Name::new("Mond"),
        DirectionalLight {
            color: Color::linear_rgb(map.sun.color[0], map.sun.color[1], map.sun.color[2]),
            illuminance: map.sun.illuminance,
            shadow_maps_enabled: settings.shadows || map.lightmaps.is_empty(),
            // Mondlicht steckt in den Lightmaps; auf gebackenen Flächen nur noch Glanz
            affects_lightmapped_mesh_diffuse: map.lightmaps.is_empty(),
            ..default()
        },
        Transform::from_translation(-sun_dir * 50.0).looking_to(sun_dir, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            first_cascade_far_bound: 14.0,
            maximum_distance: 60.0,
            ..default()
        }
        .build(),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::linear_rgb(map.ambient.color[0], map.ambient.color[1], map.ambient.color[2]),
        // Das Umgebungslicht trifft nur noch bewegte Figuren (Zombies, Waffe, Türen); die
        // Karte selbst hat es in den Lightmaps. Gedämpft, damit Figuren nicht ausgewaschen wirken.
        brightness: map.ambient.brightness * if map.lightmaps.is_empty() { 1.0 } else { 0.4 },
        affects_lightmapped_meshes: map.lightmaps.is_empty(),
    });
    commands.insert_resource(ClearColor(Color::linear_rgb(
        map.fog.color[0],
        map.fog.color[1],
        map.fog.color[2],
    )));

    spawn_map_lights(&mut commands, &map, settings.shadows || map.lightmaps.is_empty(), &mut meshes, &mut mats);
    commands.insert_resource(col);
    commands.insert_resource(nav);
    commands.insert_resource(map);
}

fn on_map_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    map: Res<MapFile>,
    asset_server: Res<AssetServer>,
    children: Query<&Children>,
    names: Query<(&Name, &Transform)>,
    meshes: Query<(&ChildOf, &MeshMaterial3d<StandardMaterial>), With<Mesh3d>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Gebackenes Licht: Objektname -> (Lightmap, Belichtung)
    let mut lm_of: HashMap<&str, (Handle<Image>, f32)> = HashMap::new();
    for lm in &map.lightmaps {
        let img: Handle<Image> = asset_server.load(lm.image.clone());
        for o in &lm.objects {
            lm_of.insert(o.as_str(), (img.clone(), lm.exposure));
        }
    }
    let mut lit = 0;
    for e in children.iter_descendants(ready.entity) {
        if let Ok((parent, mat)) = meshes.get(e) {
            let node = names.get(e).ok().map(|(n, _)| n.as_str()).filter(|n| lm_of.contains_key(n));
            let node = node.or_else(|| names.get(parent.parent()).ok().map(|(n, _)| n.as_str()));
            if let Some((img, exposure)) = node.and_then(|n| lm_of.get(n)) {
                commands.entity(e).insert(Lightmap {
                    image: img.clone(),
                    uv_rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                    bicubic_sampling: false,
                });
                if let Some(mut m) = materials.get_mut(&mat.0) {
                    m.lightmap_exposure = *exposure;
                }
                lit += 1;
            }
        }
    }
    if lit > 0 {
        info!("Lightmaps auf {lit} Meshes angewendet");
    }
    for e in children.iter_descendants(ready.entity) {
        let Ok((name, tf)) = names.get(e) else { continue };
        for door in &map.doors {
            for part in &door.parts {
                if part.node == name.as_str() {
                    let open = Transform::from_translation(Vec3::from_array(part.to_pos))
                        .with_rotation(Quat::from_rotation_y(part.to_yaw))
                        .with_scale(tf.scale);
                    commands.entity(e).insert(DoorPart {
                        door: door.id,
                        closed: *tf,
                        open,
                    });
                }
            }
        }
    }
    info!("Kartenmodell bereit");
    commands.insert_resource(MapReady);
}

/// Neue Partie: Kollision und Navigation frisch aufbauen, Türen schließen.
fn reset_level(
    mut commands: Commands,
    map: Res<MapFile>,
    mut parts: Query<(&DoorPart, &mut Transform, &mut Visibility)>,
) {
    let col = CollisionWorld::from_map(&map);
    let nav = NavGrid::build(&map, &col);
    commands.insert_resource(col);
    commands.insert_resource(nav);
    for (p, mut tf, mut vis) in parts.iter_mut() {
        *tf = p.closed;
        *vis = Visibility::Inherited;
    }
}
