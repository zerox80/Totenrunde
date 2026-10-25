//! Spielregeln des Zombie-Modus: Runden, Spawnen, Punkte, Perks, Power-ups,
//! Interaktionen (Kaufen), Mysteriöse Kiste und Waffen-Verstärker.

pub mod balance;
pub mod interact;
pub mod perks;
pub mod points;
pub mod powerups;
pub mod rounds;
pub mod spawning;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, balance::setup).add_plugins((
            rounds::RoundsPlugin,
            points::PointsPlugin,
            perks::PerksPlugin,
            powerups::PowerupPlugin,
            interact::InteractPlugin,
        ));
    }
}
