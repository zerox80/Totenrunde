//! Alle Spieler der Partie aus Sicht der Welt: wo sie stehen und ob sie angreifbar sind.
//! Zombies, Spawner, Zonen und Power-ups richten sich danach – im Solospiel steht hier nur
//! der eigene Spieler, im Koop auch die Mitspieler.

use bevy::prelude::*;
use totenrunde_mp_client::protocol::Target;

use crate::net::msg::{GameMsg, Life};
use crate::net::{GameOut, Session, Team, local_slot};
use crate::player::{Health, Player, PlayerDamage};

#[derive(Clone, Copy, Debug)]
pub struct Survivor {
    pub slot: u8,
    pub pos: Vec3,
    /// Steht (nicht am Boden, nicht tot)
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct Survivors {
    pub list: Vec<Survivor>,
}

fn flat(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

impl Survivors {
    /// Nächster stehender Spieler (waagerechter Abstand).
    pub fn nearest_active(&self, p: Vec3) -> Option<Survivor> {
        self.list
            .iter()
            .filter(|s| s.active)
            .min_by(|a, b| flat(a.pos, p).total_cmp(&flat(b.pos, p)))
            .copied()
    }

    pub fn nearest(&self, p: Vec3) -> Option<Survivor> {
        self.list
            .iter()
            .min_by(|a, b| flat(a.pos, p).total_cmp(&flat(b.pos, p)))
            .copied()
    }

    /// Kleinster waagerechter Abstand zu irgendeinem Spieler.
    pub fn min_distance(&self, p: Vec3) -> f32 {
        self.list
            .iter()
            .map(|s| flat(s.pos, p))
            .fold(f32::INFINITY, f32::min)
    }

    /// Ziele für das Flow-Field: stehende Spieler, sonst alle.
    pub fn goals(&self) -> Vec<Vec3> {
        let active: Vec<Vec3> = self
            .list
            .iter()
            .filter(|s| s.active)
            .map(|s| s.pos)
            .collect();
        if active.is_empty() {
            self.list.iter().map(|s| s.pos).collect()
        } else {
            active
        }
    }

    /// Alle am Boden oder tot (Koop: Spiel vorbei).
    pub fn all_down(&self) -> bool {
        !self.list.is_empty() && self.list.iter().all(|s| !s.active)
    }
}

/// Schaden an einem Spieler; landet beim eigenen Spieler oder wird an den Mitspieler geschickt.
#[derive(Message, Clone, Copy)]
pub struct SurvivorDamage {
    pub slot: u8,
    pub amount: f32,
    pub from: Vec3,
}

pub fn update_survivors(
    player: Query<(&Transform, &Health), With<Player>>,
    team: Option<Res<Team>>,
    session: Option<Res<Session>>,
    mut survivors: ResMut<Survivors>,
) {
    survivors.list.clear();
    if let Ok((tf, h)) = player.single() {
        survivors.list.push(Survivor {
            slot: local_slot(session.as_deref()),
            pos: tf.translation,
            active: !h.downed && !h.dead,
        });
    }
    if let Some(team) = team.filter(|_| session.is_some()) {
        for m in team.members.iter().filter(|m| m.connected && m.seen) {
            survivors.list.push(Survivor {
                slot: m.slot,
                pos: m.motion.latest().unwrap_or(Vec3::from_array(m.state.pos)),
                active: m.state.life == Life::Alive,
            });
        }
    }
}

pub fn route_damage(
    mut events: MessageReader<SurvivorDamage>,
    session: Option<Res<Session>>,
    mut local: MessageWriter<PlayerDamage>,
    mut out: ResMut<GameOut>,
) {
    let me = local_slot(session.as_deref());
    for ev in events.read() {
        if ev.slot == me {
            local.write(PlayerDamage {
                amount: ev.amount,
                from: ev.from,
            });
        } else {
            out.send(
                Target::Player(ev.slot),
                GameMsg::Damage {
                    amount: ev.amount,
                    from: ev.from.to_array(),
                },
            );
        }
    }
}
