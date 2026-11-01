//! Gesundheit, Schaden, Regeneration und "Letztes Gefecht".
//!
//! * Solo mit Zweiter Atem: automatische Wiederbelebung nach 10 Sekunden, Verlust aller Perks.
//! * Koop: Wer fällt, liegt 45 Sekunden am Boden und kann mit der Pistole weiterschießen, bis
//!   ein Mitspieler ihn wiederbelebt. Sonst blutet er aus und kehrt zu Beginn der nächsten
//!   Runde zurück. Liegen alle am Boden, ist das Spiel vorbei (entscheidet der Host).

use bevy::prelude::*;

use super::{Player, Stance};
use crate::audio::{Sfx, SfxEvent};
use crate::base::InGame;
use crate::base::debug::DebugConfig;
use crate::game::perks::{Perk, Perks};
use crate::game::points::Score;
use crate::game::rounds::RoundStarted;
use crate::net::{Session, local_slot};
use crate::weapons::{Loadout, WState, enter_last_stand, leave_last_stand, starting_loadout};
use crate::world::mapdata::MapFile;

pub const REGEN_DELAY: f32 = 2.8;
pub const REVIVE_TIME: f32 = 10.0;
/// Koop: so lange kann man am Boden auf Hilfe warten
pub const BLEED_TIME: f32 = 45.0;

/// Koop: Ein Mitspieler hat uns wiederbelebt.
#[derive(Message, Clone, Copy)]
pub struct Revived;

#[derive(Message, Clone, Copy)]
pub struct PlayerDamage {
    pub amount: f32,
    pub from: Vec3,
}

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub max: f32,
    /// Sekunden seit dem letzten Treffer
    pub since_hit: f32,
    /// Rot-Effekt am Bildschirmrand (0..1)
    pub hurt_flash: f32,
    /// Richtung des letzten Treffers (für die Anzeige)
    pub hit_dir: Vec3,
    pub downed: bool,
    pub down_time: f32,
    /// Zeit bis zur Selbstwiederbelebung (Zweiter Atem, Solo) bzw. bis zum Ausbluten (Koop)
    pub revive_left: f32,
    pub dead: bool,
    pub heartbeat_cd: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            hp: 100.0,
            max: 100.0,
            since_hit: 100.0,
            hurt_flash: 0.0,
            hit_dir: Vec3::ZERO,
            downed: false,
            down_time: 0.0,
            revive_left: 0.0,
            dead: false,
            heartbeat_cd: 0.0,
        }
    }
}

/// Gesetzt, wenn der Spieler endgültig gestorben ist.
#[derive(Resource, Default)]
pub struct GameOverFlag(pub bool);

#[allow(clippy::too_many_arguments)]
pub fn player_health(
    time: Res<Time>,
    mut events: MessageReader<PlayerDamage>,
    mut q: Single<(&mut Player, &mut Health, &Transform, &mut Loadout)>,
    mut perks: ResMut<Perks>,
    mut score: ResMut<Score>,
    mut sfx: MessageWriter<SfxEvent>,
    mut over: ResMut<GameOverFlag>,
    cfg: Res<DebugConfig>,
    game: Res<InGame>,
    mut revived: MessageReader<Revived>,
) {
    let dt = time.delta_secs();
    let coop = game.multiplayer;
    let was_revived = revived.read().count() > 0;
    let (ref mut p, ref mut h, tf, ref mut lo) = *q;
    h.since_hit += dt;
    h.hurt_flash = (h.hurt_flash - dt * 0.9).max(0.0);
    h.max = if perks.has(Perk::Juggernog) {
        250.0
    } else {
        100.0
    };

    for ev in events.read() {
        if h.downed || h.dead || cfg.god {
            continue;
        }
        h.hp -= ev.amount;
        h.since_hit = 0.0;
        h.hurt_flash = (h.hurt_flash + 0.6).min(1.0);
        h.hit_dir = (ev.from - tf.translation).normalize_or_zero();
        p.shake = (p.shake + 0.25).min(1.0);
        p.recoil_vel += Vec2::new(-0.6, 0.0);
        sfx.write(SfxEvent::ui(Sfx::PlayerHurt));
        if h.hp <= 0.0 {
            h.hp = 0.0;
            score.downs += 1;
            sfx.write(SfxEvent::ui(Sfx::PlayerDown));
            if coop {
                // Am Boden, bis ein Mitspieler hilft oder die Zeit abläuft
                h.downed = true;
                h.down_time = 0.0;
                h.revive_left = BLEED_TIME;
                perks.on_down();
                p.stance = Stance::Prone;
                enter_last_stand(lo);
            } else if perks.has(Perk::QuickRevive) {
                h.downed = true;
                h.down_time = 0.0;
                h.revive_left = REVIVE_TIME;
                perks.on_down();
                p.stance = Stance::Prone;
                enter_last_stand(lo);
            } else {
                h.dead = true;
                over.0 = true;
                p.frozen = true;
            }
        }
    }

    if h.downed {
        h.down_time += dt;
        h.revive_left -= dt;
        p.stance = Stance::Prone;
        p.speed_mult = 0.35;
        let stand_up = if coop {
            was_revived
        } else {
            h.revive_left <= 0.0
        };
        if stand_up {
            h.downed = false;
            h.hp = h.max;
            h.since_hit = 0.0;
            p.speed_mult = 1.0;
            p.stance = Stance::Stand;
            if !coop {
                score.revives += 1;
            }
            leave_last_stand(lo);
        } else if coop && h.revive_left <= 0.0 {
            // Ausgeblutet: bis zur nächsten Runde zuschauen
            h.downed = false;
            h.dead = true;
            p.frozen = true;
            p.speed_mult = 1.0;
            leave_last_stand(lo);
            lo.state = WState::Busy { t: 0.0, dur: 1e9 };
        }
        return;
    }
    if h.dead {
        return;
    }
    // Regeneration
    let regen_delay = if perks.has(Perk::QuickRevive) {
        REGEN_DELAY * 0.5
    } else {
        REGEN_DELAY
    };
    if h.since_hit > regen_delay && h.hp < h.max {
        h.hp = (h.hp + dt * 110.0).min(h.max);
    }
    if h.hp > h.max {
        h.hp = h.max;
    }
    // Herzschlag bei wenig Leben
    h.heartbeat_cd -= dt;
    if h.hp < h.max * 0.5 && h.heartbeat_cd <= 0.0 {
        h.heartbeat_cd = 1.0;
        sfx.write(SfxEvent::ui(Sfx::Heartbeat));
    }
}

/// Koop: Tote Spieler kehren zu Beginn der nächsten Runde mit der Startausrüstung zurück
/// (ihre Punkte behalten sie).
#[allow(clippy::too_many_arguments)]
pub fn respawn_on_round(
    mut events: MessageReader<RoundStarted>,
    game: Res<InGame>,
    map: Res<MapFile>,
    session: Option<Res<Session>>,
    mut q: Single<(&mut Player, &mut Health, &mut Transform, &mut Loadout)>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    if events.read().count() == 0 || !game.multiplayer {
        return;
    }
    let (ref mut p, ref mut h, ref mut tf, ref mut lo) = *q;
    if !h.dead {
        return;
    }
    **h = Health::default();
    p.frozen = false;
    p.stance = Stance::Stand;
    p.vel = Vec3::ZERO;
    p.speed_mult = 1.0;
    p.yaw = map.player_start.yaw;
    tf.translation = super::spawn::spawn_point(&map, local_slot(session.as_deref()));
    **lo = starting_loadout();
    sfx.write(SfxEvent::ui(Sfx::Switch));
}
