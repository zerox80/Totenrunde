//! Decals: Blutflecken und Einschusslöcher, die langsam verblassen.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use rand::Rng;

use super::FxAssets;
use crate::base::AppState;

#[derive(Component)]
pub struct Decal {
    pub life: f32,
}

pub fn spawn_decal(
    commands: &mut Commands,
    fx: &FxAssets,
    mat: &Handle<StandardMaterial>,
    pos: Vec3,
    normal: Vec3,
    size: f32,
    life: f32,
) {
    let mut rng = rand::rng();
    let rot = Quat::from_rotation_arc(Vec3::Z, normal)
        * Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::TAU));
    commands.spawn((
        Mesh3d(fx.quad.clone()),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(pos + normal * (0.012 + rng.random_range(0.0..0.004)))
            .with_rotation(rot)
            .with_scale(Vec3::splat(size)),
        NotShadowCaster,
        Decal { life },
        DespawnOnExit(AppState::Playing),
    ));
}

pub fn update_decals(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Decal, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (e, mut d, mut tf) in q.iter_mut() {
        d.life -= dt;
        if d.life < 2.0 {
            tf.scale *= 1.0 - dt * 0.5;
        }
        if d.life <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}
