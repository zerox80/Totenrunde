//! Partikel (Billboards und kleine Würfel mit Schwerkraft) und kurze Lichtblitze.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use rand::Rng;

use crate::base::AppState;
use crate::player::PlayerCamera;

#[derive(Component)]
pub struct Particle {
    pub vel: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub gravity: f32,
    pub size0: f32,
    pub size1: f32,
    pub billboard: bool,
    pub drag: f32,
    pub spin: f32,
}

#[derive(Component)]
pub struct Flash {
    pub life: f32,
    pub max: f32,
    pub intensity: f32,
}

pub fn spawn_particle(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    mat: &Handle<StandardMaterial>,
    pos: Vec3,
    p: Particle,
) {
    let size = p.size0;
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(pos).with_scale(Vec3::splat(size)),
        NotShadowCaster,
        p,
        DespawnOnExit(AppState::Playing),
    ));
}

pub fn rand_dir(rng: &mut impl Rng) -> Vec3 {
    loop {
        let v = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );
        let l = v.length_squared();
        if l > 0.01 && l <= 1.0 {
            return v / l.sqrt();
        }
    }
}

pub fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    cam: Query<&GlobalTransform, With<PlayerCamera>>,
    mut q: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let cam_pos = cam.single().map(|c| c.translation()).unwrap_or(Vec3::ZERO);
    for (e, mut p, mut tf) in q.iter_mut() {
        p.life += dt;
        if p.life >= p.max_life {
            commands.entity(e).despawn();
            continue;
        }
        let k = p.life / p.max_life;
        p.vel.y -= p.gravity * dt;
        let drag = (1.0 - p.drag * dt).max(0.0);
        p.vel *= drag;
        tf.translation += p.vel * dt;
        if tf.translation.y < 0.01 && p.gravity > 0.0 {
            tf.translation.y = 0.01;
            p.vel = Vec3::ZERO;
        }
        let s = p.size0 + (p.size1 - p.size0) * k;
        tf.scale = Vec3::splat(s.max(0.001));
        if p.billboard {
            let to_cam = (cam_pos - tf.translation).normalize_or(Vec3::Z);
            tf.rotation =
                Quat::from_rotation_arc(Vec3::Z, to_cam) * Quat::from_rotation_z(p.spin * p.life);
        }
    }
}

pub fn update_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Flash, &mut PointLight)>,
) {
    let dt = time.delta_secs();
    for (e, mut f, mut l) in q.iter_mut() {
        f.life += dt;
        let k = 1.0 - (f.life / f.max).min(1.0);
        l.intensity = f.intensity * k * k;
        if f.life >= f.max {
            commands.entity(e).despawn();
        }
    }
}
