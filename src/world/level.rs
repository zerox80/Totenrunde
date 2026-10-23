//! Lädt die Karte (JSON + Modell), Mond, Umgebungslicht und Nebelfarbe und baut daraus
//! Kollisionswelt und Navigationsgitter.

use bevy::light::CascadeShadowConfigBuilder;
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

fn load_level(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            shadow_maps_enabled: true,
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
        brightness: map.ambient.brightness,
        ..default()
    });
    commands.insert_resource(ClearColor(Color::linear_rgb(
        map.fog.color[0],
        map.fog.color[1],
        map.fog.color[2],
    )));

    spawn_map_lights(&mut commands, &map);
    commands.insert_resource(col);
    commands.insert_resource(nav);
    commands.insert_resource(map);
}

fn on_map_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    map: Res<MapFile>,
    children: Query<&Children>,
    names: Query<(&Name, &Transform)>,
) {
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
