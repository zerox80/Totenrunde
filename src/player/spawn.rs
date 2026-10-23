//! Spawnt den Spieler mit Ego-Kamera (Bloom, Tonemapping, Nebel, Raumklang-Ohren).

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

use super::{CameraFov, Health, Player, PlayerCamera, Stance};
use crate::base::settings::Settings;
use crate::base::AppState;
use crate::world::mapdata::MapFile;

pub fn spawn_player(mut commands: Commands, map: Res<MapFile>, settings: Res<Settings>, existing: Query<(), With<Player>>) {
    if !existing.is_empty() {
        return;
    }
    let start = map.player_start.clone();
    let fov = settings.fov_deg.to_radians();
    commands
        .spawn((
            Name::new("Spieler"),
            Player {
                yaw: start.yaw,
                ..default()
            },
            Health::default(),
            Transform::from_translation(start.pos()).with_rotation(Quat::from_rotation_y(start.yaw)),
            Visibility::default(),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((
                Name::new("Kamera"),
                PlayerCamera,
                Camera3d::default(),
                Projection::from(PerspectiveProjection {
                    fov,
                    near: 0.01,
                    ..default()
                }),
                CameraFov { target: fov, current: fov },
                Tonemapping::TonyMcMapface,
                Bloom {
                    intensity: 0.12,
                    ..Bloom::NATURAL
                },
                DistanceFog {
                    color: Color::linear_rgb(map.fog.color[0], map.fog.color[1], map.fog.color[2]),
                    falloff: FogFalloff::Linear {
                        start: map.fog.start,
                        end: map.fog.end,
                    },
                    ..default()
                },
                Msaa::Sample4,
                Transform::from_xyz(0.0, Stance::Stand.eye(), 0.0),
                SpatialListener::new(2.0),
            ));
        });
}
