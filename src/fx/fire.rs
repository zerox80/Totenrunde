//! Flackernde Flammen-Billboards an den Feuerstellen der Karte.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::FxAssets;
use crate::base::AppState;
use crate::player::PlayerCamera;
use crate::world::mapdata::MapFile;

#[derive(Component)]
pub struct Fire {
    pub seed: f32,
}

pub fn spawn_fires(mut commands: Commands, map: Res<MapFile>, fx: Res<FxAssets>, lib: Option<Res<crate::audio::SfxLib>>) {
    for (i, f) in map.fires.iter().enumerate() {
        let pos = Vec3::from_array(f.pos);
        // Knistern als räumliche Schleife
        if let Some(lib) = &lib {
            commands.spawn((
                AudioPlayer::new(lib.fire_loop.clone()),
                PlaybackSettings::LOOP
                    .with_volume(bevy::audio::Volume::Linear(0.55 * f.scale))
                    .with_spatial(true),
                Transform::from_translation(pos),
                DespawnOnExit(AppState::Playing),
            ));
        }
        for k in 0..4 {
            commands.spawn((
                Mesh3d(fx.quad.clone()),
                MeshMaterial3d(fx.flame.clone()),
                Transform::from_translation(pos + Vec3::new((k as f32 - 1.5) * 0.25 * f.scale, 0.0, 0.0))
                    .with_scale(Vec3::new(0.7, 1.1, 1.0) * f.scale),
                NotShadowCaster,
                Fire {
                    seed: i as f32 * 3.7 + k as f32 * 1.3,
                },
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

pub fn animate_fires(time: Res<Time>, cam: Query<&GlobalTransform, With<PlayerCamera>>, mut q: Query<(&Fire, &mut Transform)>) {
    let t = time.elapsed_secs();
    let cam_pos = cam.single().map(|c| c.translation()).unwrap_or(Vec3::ZERO);
    for (f, mut tf) in q.iter_mut() {
        let to_cam = cam_pos - tf.translation;
        let yaw = to_cam.x.atan2(to_cam.z);
        let flick = 1.0 + (t * 9.0 + f.seed).sin() * 0.12 + (t * 23.0 + f.seed * 2.0).sin() * 0.06;
        tf.rotation = Quat::from_rotation_y(yaw);
        let base = tf.scale.x.max(0.01);
        tf.scale = Vec3::new(base, base * 1.55 * flick, 1.0);
    }
}
