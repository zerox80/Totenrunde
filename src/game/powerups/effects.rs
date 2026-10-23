//! Power-ups einsammeln und ihre Wirkung: Ansage, Timer, Atombombe, Zimmermann.

use bevy::prelude::*;
use rand::Rng;

use super::{ActivePowerups, Powerup, PowerupCollected, PowerupKind, DURATION};
use crate::audio::{Sfx, SfxEvent};
use crate::game::points::Score;
use crate::player::Player;
use crate::world::barricades::Barricades;
use crate::zombies::{DamageKind, HitPart, Zombie, ZombieHit};

pub fn collect_powerups(
    mut commands: Commands,
    q: Query<(Entity, &Powerup, &Transform)>,
    player: Single<&Transform, With<Player>>,
    mut pu: ResMut<ActivePowerups>,
    mut score: ResMut<Score>,
    mut barricades: ResMut<Barricades>,
    zombies: Query<(Entity, &Zombie, &Transform), Without<Player>>,
    mut collected: MessageWriter<PowerupCollected>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let ppos = player.translation;
    for (e, p, tf) in q.iter() {
        let d = Vec3::new(tf.translation.x - ppos.x, 0.0, tf.translation.z - ppos.z).length();
        if d > 1.3 || (tf.translation.y - ppos.y).abs() > 2.2 {
            continue;
        }
        commands.entity(e).despawn();
        collected.write(PowerupCollected { kind: p.kind });
        sfx.write(SfxEvent::ui(Sfx::PowerupGrab));
        let voice = match p.kind {
            PowerupKind::MaxAmmo => Sfx::VoiceMaxAmmo,
            PowerupKind::InstaKill => Sfx::VoiceInstaKill,
            PowerupKind::DoublePoints => Sfx::VoiceDoublePoints,
            PowerupKind::Nuke => Sfx::VoiceNuke,
            PowerupKind::Carpenter => Sfx::VoiceCarpenter,
            PowerupKind::FireSale => Sfx::VoiceFireSale,
        };
        sfx.write(SfxEvent::ui(voice));
        match p.kind {
            PowerupKind::MaxAmmo => {}
            PowerupKind::InstaKill => pu.insta_kill = DURATION,
            PowerupKind::DoublePoints => pu.double_points = DURATION,
            PowerupKind::FireSale => pu.fire_sale = DURATION,
            PowerupKind::Carpenter => {
                for b in barricades.list.iter_mut() {
                    b.repair_all();
                }
                let pus = ActivePowerups {
                    double_points: pu.double_points,
                    ..default()
                };
                score.add(200, &pus, None);
                sfx.write(SfxEvent::ui(Sfx::Carpenter));
            }
            PowerupKind::Nuke => {
                sfx.write(SfxEvent::ui(Sfx::Nuke));
                let pus = ActivePowerups {
                    double_points: pu.double_points,
                    ..default()
                };
                score.add(400, &pus, None);
                let mut rng = rand::rng();
                for (ze, z, _) in zombies.iter() {
                    if z.alive() {
                        let delay = rng.random_range(0.3..1.6);
                        pu.nuke_queue.push((delay, ze));
                    }
                }
            }
        }
    }
}

pub fn tick_powerups(time: Res<Time>, mut pu: ResMut<ActivePowerups>) {
    let dt = time.delta_secs();
    pu.insta_kill = (pu.insta_kill - dt).max(0.0);
    pu.double_points = (pu.double_points - dt).max(0.0);
    pu.fire_sale = (pu.fire_sale - dt).max(0.0);
}

pub fn nuke_sequence(
    time: Res<Time>,
    mut pu: ResMut<ActivePowerups>,
    zombies: Query<&Transform, With<Zombie>>,
    mut hits: MessageWriter<ZombieHit>,
) {
    if pu.nuke_queue.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    let mut keep = Vec::new();
    for (t, e) in pu.nuke_queue.drain(..) {
        let t = t - dt;
        if t <= 0.0 {
            if let Ok(tf) = zombies.get(e) {
                hits.write(ZombieHit {
                    zombie: e,
                    damage: 1.0e9,
                    part: HitPart::Torso,
                    kind: DamageKind::Nuke,
                    point: tf.translation + Vec3::Y * 1.2,
                    dir: Vec3::Y,
                });
            }
        } else {
            keep.push((t, e));
        }
    }
    pu.nuke_queue = keep;
}
