//! Menüs: Hauptmenü, Pause und Game-Over-Bildschirm.

mod game_over;
mod graphics;
mod main_menu;
mod pause;
mod widgets;

use bevy::prelude::*;

pub use widgets::MenuAction;

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((main_menu::MainMenuPlugin, pause::PausePlugin, game_over::GameOverPlugin, graphics::GraphicsPlugin))
            .add_systems(Update, widgets::button_visuals);
    }
}
