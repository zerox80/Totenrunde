//! Menüs: Ladebildschirm, Hauptmenü, Multiplayer, Pause und Game-Over-Bildschirm.

mod game_over;
mod graphics;
mod loading;
mod main_menu;
mod multiplayer;
mod pause;
mod widgets;

use bevy::prelude::*;

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            loading::LoadingPlugin,
            main_menu::MainMenuPlugin,
            multiplayer::MultiplayerPlugin,
            pause::PausePlugin,
            game_over::GameOverPlugin,
            graphics::GraphicsPlugin,
        ))
        .add_systems(Update, widgets::button_visuals);
    }
}
