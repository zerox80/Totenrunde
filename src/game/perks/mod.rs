//! Perks (eigene Namen, Mechanik wie im Original): Automaten, Trinken, Effekte.

pub mod effects;
pub mod machines;

use bevy::prelude::*;

use crate::audio::Sfx;
use crate::base::{AppState, GameSet, InGame};

pub struct PerksPlugin;

impl Plugin for PerksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Perks>()
            .add_systems(OnEnter(AppState::Playing), (reset_perks, machines::spawn_machines))
            .add_systems(
                Update,
                (
                    effects::apply_perk_effects,
                    effects::finish_drinking,
                    machines::machine_lights,
                    machines::machine_jingles,
                )
                    .in_set(GameSet::World)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

pub const PERK_LIMIT: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Perk {
    QuickRevive,
    Juggernog,
    SpeedCola,
    DoubleTap,
    StaminUp,
    MuleKick,
}

impl Perk {
    pub const ALL: [Perk; 6] = [
        Perk::QuickRevive,
        Perk::Juggernog,
        Perk::SpeedCola,
        Perk::DoubleTap,
        Perk::StaminUp,
        Perk::MuleKick,
    ];

    pub fn from_id(id: &str) -> Option<Perk> {
        Some(match id {
            "revive" => Perk::QuickRevive,
            "jugg" => Perk::Juggernog,
            "speed" => Perk::SpeedCola,
            "doubletap" => Perk::DoubleTap,
            "staminup" => Perk::StaminUp,
            "mulekick" => Perk::MuleKick,
            _ => return None,
        })
    }

    pub fn id(self) -> &'static str {
        match self {
            Perk::QuickRevive => "revive",
            Perk::Juggernog => "jugg",
            Perk::SpeedCola => "speed",
            Perk::DoubleTap => "doubletap",
            Perk::StaminUp => "staminup",
            Perk::MuleKick => "mulekick",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Perk::QuickRevive => "Zweiter Atem",
            Perk::Juggernog => "Panzer-Tonikum",
            Perk::SpeedCola => "Turbo-Cola",
            Perk::DoubleTap => "Doppelschuss-Bräu",
            Perk::StaminUp => "Ausdauer-Limo",
            Perk::MuleKick => "Maultier-Malz",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Perk::QuickRevive => "Selbstwiederbelebung (solo, 3x) und schnellere Heilung",
            Perk::Juggernog => "Hält 5 statt 2 Treffer aus",
            Perk::SpeedCola => "Doppelt so schnell nachladen und Barrikaden reparieren",
            Perk::DoubleTap => "Schnellere Feuerrate, doppelter Schaden",
            Perk::StaminUp => "Schneller und länger sprinten",
            Perk::MuleKick => "Eine dritte Waffe tragen",
        }
    }

    /// Solo-Preis (Zweiter Atem kostet solo nur 500 wie im Original)
    pub fn cost(self) -> u32 {
        match self {
            Perk::QuickRevive => 500,
            Perk::Juggernog => 2500,
            Perk::SpeedCola => 3000,
            Perk::DoubleTap => 2000,
            Perk::StaminUp => 2000,
            Perk::MuleKick => 4000,
        }
    }

    pub fn needs_power(self) -> bool {
        !matches!(self, Perk::QuickRevive)
    }

    pub fn jingle(self) -> Sfx {
        match self {
            Perk::QuickRevive => Sfx::PerkJingleRevive,
            Perk::Juggernog => Sfx::PerkJingleJugg,
            Perk::SpeedCola => Sfx::PerkJingleSpeed,
            Perk::DoubleTap => Sfx::PerkJingleDoubleTap,
            Perk::StaminUp => Sfx::PerkJingleStamin,
            Perk::MuleKick => Sfx::PerkJingleMule,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Perk::QuickRevive => Color::srgb(0.35, 0.75, 1.0),
            Perk::Juggernog => Color::srgb(1.0, 0.25, 0.2),
            Perk::SpeedCola => Color::srgb(0.35, 1.0, 0.35),
            Perk::DoubleTap => Color::srgb(1.0, 0.8, 0.25),
            Perk::StaminUp => Color::srgb(1.0, 0.55, 0.15),
            Perk::MuleKick => Color::srgb(0.6, 0.85, 0.35),
        }
    }

    pub fn model(self) -> String {
        format!("models/perk_{}.gltf", self.id())
    }

    pub fn icon(self) -> String {
        format!("ui/perk_{}.png", self.id())
    }
}

#[derive(Resource, Default)]
pub struct Perks {
    pub owned: Vec<Perk>,
    pub revive_uses: u32,
    /// gerade getrunkener Perk (wird nach der Trink-Animation aktiv)
    pub drinking: Option<(Perk, f32)>,
}

impl Perks {
    pub fn has(&self, p: Perk) -> bool {
        self.owned.contains(&p)
    }
    pub fn full(&self) -> bool {
        self.owned.len() + self.drinking.is_some() as usize >= PERK_LIMIT
    }
    /// Beim Zu-Boden-Gehen verliert man alle Perks.
    pub fn on_down(&mut self) {
        if self.has(Perk::QuickRevive) {
            self.revive_uses += 1;
        }
        self.owned.clear();
        self.drinking = None;
    }
}

fn reset_perks(mut commands: Commands) {
    commands.insert_resource(Perks::default());
}
