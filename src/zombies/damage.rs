//! Schaden auswerten: Punkte (+10 pro Treffer, 50–100 pro Kill, 130 mit dem Messer; solo
//! x1.5), Sofort-Tod, Kopfschüsse (Kopf platzt), Tod mit Fallrichtung.

use bevy::prelude::*;

use super::{DamageKind, HitPart, ZState, Zombie, ZombieHit, ZombieKilled, ZombieRig};
use crate::fx::FxEvent;
use crate::game::balance::Balance;
use crate::game::points::Score;
use crate::game::powerups::ActivePowerups;
use crate::world::barricades::Barricades;

#[allow(clippy::too_many_arguments)]
pub fn apply_damage(
    mut hits: MessageReader<ZombieHit>,
    mut killed: MessageWriter<ZombieKilled>,
    mut zombies: Query<(&mut Zombie, &Transform, Option<&ZombieRig>)>,
    mut barricades: ResMut<Barricades>,
    mut commands: Commands,
    powerups: Res<ActivePowerups>,
    bal: Res<Balance>,
    mut score: ResMut<Score>,
    mut fx: MessageWriter<FxEvent>,
) {
    for h in hits.read() {
        let Ok((mut z, tf, rig)) = zombies.get_mut(h.zombie) else {
            continue;
        };
        if !z.alive() {
            continue;
        }
        let mut dmg = h.damage;
        if powerups.insta_kill > 0.0 && h.kind != DamageKind::Nuke {
            dmg = z.health + 1.0;
        }
        z.health -= dmg;
        z.flinch = (z.flinch + 0.5).min(1.0);
        fx.write(FxEvent::Blood {
            pos: h.point,
            dir: h.dir,
            amount: if h.part == HitPart::Head { 2.0 } else { 1.0 },
        });
        if z.health > 0.0 {
            if matches!(h.kind, DamageKind::Bullet | DamageKind::Melee) {
                score.add(10, &powerups, Some(h.point));
            }
            continue;
        }
        // Tod
        let pts = match h.kind {
            DamageKind::Melee => 130,
            DamageKind::Nuke => 0,
            DamageKind::Explosive => 60,
            DamageKind::Bullet => match h.part {
                HitPart::Head => 100,
                HitPart::Neck => 70,
                HitPart::Torso => 60,
                HitPart::Limb => 50,
            },
        };
        if pts > 0 {
            let pts = (pts as f32 * bal.kill_points).round() as u32;
            score.add(pts, &powerups, Some(h.point));
        }
        score.kills += 1;
        if h.part == HitPart::Head && h.kind == DamageKind::Bullet {
            score.headshots += 1;
        }
        // Blickrichtung des Zombies vs. Schussrichtung -> nach hinten oder vorne fallen
        let fwd = tf.rotation * Vec3::NEG_Z;
        z.death_back = fwd.dot(h.dir) < 0.0;
        z.state = ZState::Dying { t: 0.0 };
        z.vel = Vec3::ZERO;
        if h.part == HitPart::Head
            && (h.kind == DamageKind::Bullet || h.kind == DamageKind::Explosive)
        {
            z.headless = true;
            if let Some(rig) = rig {
                if let Some(hm) = rig.head_mesh {
                    commands.entity(hm).insert(Visibility::Hidden);
                }
                if let Some(e) = rig.eyes_mesh {
                    commands.entity(e).insert(Visibility::Hidden);
                }
                if let Some(s) = rig.stump_mesh {
                    commands.entity(s).insert(Visibility::Inherited);
                }
            }
            fx.write(FxEvent::Gib { pos: h.point });
        }
        barricades.release(h.zombie);
        killed.write(ZombieKilled {
            zombie: h.zombie,
            pos: tf.translation,
            part: h.part,
            kind: h.kind,
        });
    }
}
