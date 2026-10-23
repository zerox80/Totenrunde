//! Power-ups fallen lassen (Punkteschwelle + Zufall), schweben, blinken, verschwinden.

use bevy::prelude::*;
use rand::Rng;

use super::{ActivePowerups, Powerup, PowerupKind, LIFETIME};
use crate::audio::{Sfx, SfxEvent};
use crate::base::AppState;
use crate::game::points::Score;
use crate::world::nav::NavGrid;
use crate::zombies::{DamageKind, ZombieKilled};

pub fn spawn_powerup(commands: &mut Commands, asset_server: &AssetServer, kind: PowerupKind, pos: Vec3) {
    let base = Vec3::new(pos.x, 0.0, pos.z);
    commands
        .spawn((
            Name::new(format!("Power-up {}", kind.name())),
            Powerup { kind, age: 0.0, base },
            Transform::from_translation(base + Vec3::Y * 0.9),
            Visibility::default(),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((
                WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(kind.model()))),
                Transform::IDENTITY,
            ));
            p.spawn((
                PointLight {
                    color: Color::srgb(0.35, 1.0, 0.35),
                    intensity: 40000.0,
                    range: 5.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
        });
}

pub fn drop_powerups(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut killed: MessageReader<ZombieKilled>,
    mut pu: ResMut<ActivePowerups>,
    score: Res<Score>,
    nav: Res<NavGrid>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let mut rng = rand::rng();
    for k in killed.read() {
        if k.kind == DamageKind::Nuke || pu.drops_this_round >= 4 {
            continue;
        }
        // Nur in begehbaren Bereichen (nicht hinter Fenstern)
        if !nav.walkable(k.pos) {
            continue;
        }
        let by_score = score.earned >= pu.next_drop_score;
        let by_chance = rng.random::<f32>() < 0.02;
        if !(by_score || by_chance) {
            continue;
        }
        if by_score {
            pu.increment *= 1.14;
            pu.next_drop_score = score.earned + pu.increment as u64;
        }
        pu.drops_this_round += 1;
        let kind = pu.next_kind();
        spawn_powerup(&mut commands, &asset_server, kind, k.pos);
        sfx.write(SfxEvent::at(Sfx::PowerupSpawn, k.pos + Vec3::Y));
    }
}

pub fn animate_powerups(
    time: Res<Time>,
    mut q: Query<(Entity, &mut Powerup, &mut Transform, &mut Visibility)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (e, mut p, mut tf, mut vis) in q.iter_mut() {
        p.age += dt;
        tf.translation = p.base + Vec3::Y * (0.85 + (p.age * 2.2).sin() * 0.08);
        tf.rotation = Quat::from_rotation_y(p.age * 1.6);
        // Blinken in den letzten 10 Sekunden, dann verschwinden
        let left = LIFETIME - p.age;
        if left < 10.0 {
            let rate = if left < 4.0 { 9.0 } else { 4.0 };
            let on = (p.age * rate).fract() < 0.6;
            *vis = if on { Visibility::Inherited } else { Visibility::Hidden };
        }
        if left <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}
