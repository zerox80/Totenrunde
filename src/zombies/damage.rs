//! Schaden auswerten: Punkte (+10 pro Treffer, 50–100 pro Kill, 130 mit dem Messer; solo
//! x1.5), Sofort-Tod, Kopfschüsse (Kopf platzt), Tod mit Fallrichtung.
//!
//! Läuft nur, wo die Welt berechnet wird. Treffer von Mitspielern kommen über das Netz; ihre
//! Punkte gehen als Nachricht an den Schützen zurück.

use bevy::prelude::*;
use totenrunde_mp_client::protocol::Target;

use super::{Attacker, DamageKind, HitPart, ZState, Zombie, ZombieHit, ZombieKilled, ZombieRig};
use crate::fx::FxEvent;
use crate::game::balance::Balance;
use crate::game::points::Score;
use crate::game::powerups::ActivePowerups;
use crate::net::GameOut;
use crate::net::msg::GameMsg;
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
    mut out: ResMut<GameOut>,
    session: Option<Res<crate::net::Session>>,
) {
    // Punkte an den Schützen: eigener Spieler direkt, Mitspieler per Nachricht
    let mut award = |by: Attacker, base: u32, kill: bool, headshot: bool, at: Vec3| match by {
        Attacker::Local => {
            if base > 0 {
                score.add(base, &powerups, Some(at));
            }
            if kill {
                score.kills += 1;
            }
            if headshot {
                score.headshots += 1;
            }
        }
        Attacker::Remote(slot) => {
            let amount = if powerups.double_points > 0.0 {
                base * 2
            } else {
                base
            };
            out.send(
                Target::Player(slot),
                GameMsg::Points {
                    amount,
                    kill,
                    headshot,
                },
            );
        }
        Attacker::World => {
            if kill && session.is_none() {
                score.kills += 1;
            }
        }
    };
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
                award(h.by, 10, false, false, h.point);
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
        let pts = (pts as f32 * bal.kill_points).round() as u32;
        let headshot = h.part == HitPart::Head && h.kind == DamageKind::Bullet;
        award(h.by, pts, true, headshot, h.point);
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
