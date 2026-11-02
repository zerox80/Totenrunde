//! Spielregeln des Zombie-Modus: Runden, Spawnen, Punkte, Perks, Power-ups,
//! Interaktionen (Kaufen), Mysteriöse Kiste und Waffen-Verstärker. Dazu, wer die Welt
//! verändern darf (`requests`) und welche Spieler die Zombies jagen (`survivors`).

pub mod balance;
pub mod interact;
pub mod perks;
pub mod points;
pub mod powerups;
pub mod requests;
pub mod rounds;
pub mod spawning;
pub mod survivors;

use bevy::prelude::*;

use crate::base::{AppState, GameSet, InGame};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<survivors::Survivors>()
            .add_message::<survivors::SurvivorDamage>()
            .add_message::<requests::WorldRequest>()
            .add_message::<requests::WorldGrant>()
            .add_systems(
                Update,
                (
                    survivors::update_survivors
                        .in_set(GameSet::Movement)
                        .after(crate::player::movement::movement),
                    survivors::route_damage
                        .in_set(GameSet::World)
                        .after(crate::zombies::ZombieSystems)
                        .before(crate::player::health::player_health),
                )
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            )
            .add_systems(Startup, balance::setup)
            .add_plugins((
                rounds::RoundsPlugin,
                points::PointsPlugin,
                perks::PerksPlugin,
                powerups::PowerupPlugin,
                interact::InteractPlugin,
            ));
    }
}
