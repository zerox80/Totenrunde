//! Grundlagen: Spielzustände, System-Reihenfolge, Einstellungen, Assets und Debug-Hilfen.

pub mod assets;
pub mod debug;
pub mod platform;
pub mod settings;
pub mod state;
pub mod textures;
#[cfg(target_arch = "wasm32")]
mod web;

use bevy::prelude::*;

pub use state::{AppState, GameSet, InGame, set_paused};

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(settings::Settings::load())
            .insert_resource(settings::GraphicsMenuOpen(
                std::env::var("TR_GFXMENU").is_ok(),
            ))
            .init_state::<AppState>()
            .configure_sets(
                Update,
                (
                    GameSet::Input,
                    GameSet::Movement,
                    GameSet::Combat,
                    GameSet::World,
                    GameSet::Ui,
                )
                    .chain(),
            )
            .add_plugins((
                assets::AssetsPlugin,
                debug::DebugPlugin,
                state::StatePlugin,
                textures::TexturePlugin,
            ));
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, platform::set_window_icon);
        #[cfg(target_arch = "wasm32")]
        app.add_plugins(web::WebPlugin);
    }
}
