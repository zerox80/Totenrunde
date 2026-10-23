//! Grundlagen: Spielzustände, System-Reihenfolge, Einstellungen, Assets und Debug-Hilfen.

pub mod assets;
pub mod debug;
pub mod settings;
pub mod state;

use bevy::prelude::*;

pub use state::{set_paused, AppState, GameSet, InGame};

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(settings::Settings::load())
            .init_state::<AppState>()
            .configure_sets(
                Update,
                (GameSet::Input, GameSet::Movement, GameSet::Combat, GameSet::World, GameSet::Ui).chain(),
            )
            .add_plugins((assets::AssetsPlugin, debug::DebugPlugin, state::StatePlugin));
    }
}
