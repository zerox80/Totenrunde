//! Power-ups: Max. Munition, Sofort-Tod, Doppelte Punkte, Atombombe, Zimmermann, Ausverkauf.
//!
//! Drop-Logik wie im Original: Sobald die insgesamt verdienten Punkte eine steigende
//! Schwelle (2000, x1.14) überschreiten, lässt der nächste getötete Zombie ein Power-up
//! fallen; zusätzlich 2 % Zufall. Höchstens 4 Drops pro Runde. Solo: Schwelle 1500 (x1.1)
//! und 5 % Zufall (siehe `balance`).

pub mod drops;
pub mod effects;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::base::{AppState, GameSet, InGame};
use crate::game::balance::Balance;

pub use drops::spawn_powerup;

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePowerups>()
            .add_message::<PowerupCollected>()
            .add_systems(OnEnter(AppState::Playing), reset_powerups)
            .add_systems(
                Update,
                (
                    drops::drop_powerups,
                    drops::animate_powerups,
                    effects::collect_powerups,
                    effects::tick_powerups,
                    effects::nuke_sequence,
                )
                    .chain()
                    .in_set(GameSet::World)
                    .after(crate::zombies::ZombieSystems)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

pub const DURATION: f32 = 30.0;
pub const LIFETIME: f32 = 30.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PowerupKind {
    MaxAmmo,
    InstaKill,
    DoublePoints,
    Nuke,
    Carpenter,
    FireSale,
}

impl PowerupKind {
    pub fn name(self) -> &'static str {
        match self {
            PowerupKind::MaxAmmo => "Max. Munition",
            PowerupKind::InstaKill => "Sofort-Tod",
            PowerupKind::DoublePoints => "Doppelte Punkte",
            PowerupKind::Nuke => "Kaboom!",
            PowerupKind::Carpenter => "Zimmermann",
            PowerupKind::FireSale => "Ausverkauf",
        }
    }
    pub fn model(self) -> &'static str {
        match self {
            PowerupKind::MaxAmmo => "models/pu_maxammo.gltf",
            PowerupKind::InstaKill => "models/pu_instakill.gltf",
            PowerupKind::DoublePoints => "models/pu_double.gltf",
            PowerupKind::Nuke => "models/pu_nuke.gltf",
            PowerupKind::Carpenter => "models/pu_carpenter.gltf",
            PowerupKind::FireSale => "models/pu_firesale.gltf",
        }
    }
}

#[derive(Resource, Default)]
pub struct ActivePowerups {
    pub insta_kill: f32,
    pub double_points: f32,
    pub fire_sale: f32,
    pub drops_this_round: u32,
    pub next_drop_score: u64,
    pub increment: f64,
    pub cycle: Vec<PowerupKind>,
    pub cycle_index: usize,
    /// Darf "Ausverkauf" fallen (Kiste ist mindestens einmal umgezogen)?
    pub fire_sale_allowed: bool,
    pub nuke_queue: Vec<(f32, Entity)>,
}

#[derive(Component)]
pub struct Powerup {
    pub kind: PowerupKind,
    pub age: f32,
    pub base: Vec3,
}

#[derive(Message, Clone, Copy)]
pub struct PowerupCollected {
    pub kind: PowerupKind,
}

fn shuffled_cycle() -> Vec<PowerupKind> {
    let mut v = vec![
        PowerupKind::MaxAmmo,
        PowerupKind::InstaKill,
        PowerupKind::DoublePoints,
        PowerupKind::Nuke,
        PowerupKind::Carpenter,
        PowerupKind::FireSale,
    ];
    v.shuffle(&mut rand::rng());
    v
}

fn reset_powerups(mut commands: Commands, bal: Res<Balance>) {
    commands.insert_resource(ActivePowerups {
        next_drop_score: bal.drop_first,
        increment: bal.drop_first as f64,
        cycle: shuffled_cycle(),
        ..default()
    });
}

impl ActivePowerups {
    pub fn next_kind(&mut self) -> PowerupKind {
        for _ in 0..12 {
            if self.cycle_index >= self.cycle.len() {
                self.cycle = shuffled_cycle();
                self.cycle_index = 0;
            }
            let k = self.cycle[self.cycle_index];
            self.cycle_index += 1;
            if k == PowerupKind::FireSale && !self.fire_sale_allowed {
                continue;
            }
            return k;
        }
        PowerupKind::MaxAmmo
    }
}

pub fn new_round(pu: &mut ActivePowerups) {
    pu.drops_this_round = 0;
}
