//! Gesundheit, Schaden, Regeneration und "Letztes Gefecht" (Solo mit Zweiter Atem:
//! automatische Wiederbelebung nach 10 Sekunden, Verlust aller Perks).

use bevy::prelude::*;

use super::{Player, Stance};
use crate::audio::{Sfx, SfxEvent};
use crate::base::debug::DebugConfig;
use crate::game::perks::{Perk, Perks};
use crate::game::points::Score;
use crate::weapons::{Loadout, enter_last_stand, leave_last_stand};

pub const REGEN_DELAY: f32 = 2.8;
pub const REVIVE_TIME: f32 = 10.0;

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
    /// Zeit bis zur Selbstwiederbelebung (Zweiter Atem, Solo)
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
) {
    let dt = time.delta_secs();
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
            if perks.has(Perk::QuickRevive) {
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
        if h.revive_left <= 0.0 {
            h.downed = false;
            h.hp = h.max;
            p.speed_mult = 1.0;
            p.stance = Stance::Stand;
            score.revives += 1;
            leave_last_stand(lo);
        }
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
