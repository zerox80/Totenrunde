//! Power-ups einsammeln und ihre Wirkung: Ansage, Timer, Atombombe, Zimmermann.

use bevy::prelude::*;
use rand::Rng;

use super::{ActivePowerups, DURATION, Powerup, PowerupCollected, PowerupKind};
use crate::audio::{Sfx, SfxEvent};
use crate::game::points::Score;
use crate::game::survivors::Survivors;
use crate::world::barricades::Barricades;
use crate::zombies::{Attacker, DamageKind, HitPart, Zombie, ZombieHit};

/// Einsammeln (nur wo die Welt berechnet wird): der nächste stehende Spieler in Reichweite.
#[allow(clippy::too_many_arguments)]
pub fn collect_powerups(
    mut commands: Commands,
    q: Query<(Entity, &Powerup, &Transform)>,
    survivors: Res<Survivors>,
    mut pu: ResMut<ActivePowerups>,
    mut barricades: ResMut<Barricades>,
    zombies: Query<(Entity, &Zombie, &Transform)>,
    mut collected: MessageWriter<PowerupCollected>,
) {
    for (e, p, tf) in q.iter() {
        let Some(by) = survivors.list.iter().find(|s| {
            let d = Vec3::new(tf.translation.x - s.pos.x, 0.0, tf.translation.z - s.pos.z);
            s.active && d.length() <= 1.3 && (tf.translation.y - s.pos.y).abs() <= 2.2
        }) else {
            continue;
        };
        commands.entity(e).despawn();
        collected.write(PowerupCollected {
            kind: p.kind,
            by: by.slot,
        });
        match p.kind {
            PowerupKind::MaxAmmo => {}
            PowerupKind::InstaKill => pu.insta_kill = DURATION,
            PowerupKind::DoublePoints => pu.double_points = DURATION,
            PowerupKind::FireSale => pu.fire_sale = DURATION,
            PowerupKind::Carpenter => {
                for b in barricades.list.iter_mut() {
                    b.repair_all();
                }
            }
            PowerupKind::Nuke => {
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

/// Bei jedem Spieler: Ansage, Geräusch und die Punkte für Zimmermann (200) und Atombombe (400).
pub fn powerup_feedback(
    mut collected: MessageReader<PowerupCollected>,
    pu: Res<ActivePowerups>,
    mut score: ResMut<Score>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    for c in collected.read() {
        sfx.write(SfxEvent::ui(Sfx::PowerupGrab));
        let voice = match c.kind {
            PowerupKind::MaxAmmo => Sfx::VoiceMaxAmmo,
            PowerupKind::InstaKill => Sfx::VoiceInstaKill,
            PowerupKind::DoublePoints => Sfx::VoiceDoublePoints,
            PowerupKind::Nuke => Sfx::VoiceNuke,
            PowerupKind::Carpenter => Sfx::VoiceCarpenter,
            PowerupKind::FireSale => Sfx::VoiceFireSale,
        };
        sfx.write(SfxEvent::ui(voice));
        // Nur doppelte Punkte zählen hier, nicht ein gerade erst eingesammeltes Doppelt
        let pus = ActivePowerups {
            double_points: pu.double_points,
            ..default()
        };
        match c.kind {
            PowerupKind::Carpenter => {
                score.add(200, &pus, None);
                sfx.write(SfxEvent::ui(Sfx::Carpenter));
            }
            PowerupKind::Nuke => {
                score.add(400, &pus, None);
                sfx.write(SfxEvent::ui(Sfx::Nuke));
            }
            _ => {}
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
                    by: Attacker::World,
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
