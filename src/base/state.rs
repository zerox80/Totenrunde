//! Spielzustände (Laden, Menü, Spiel, Game Over), Pause und Spielzeit.

use bevy::prelude::*;

use crate::world::level::MapReady;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    Playing,
    GameOver,
}

/// Reihenfolge der Spiel-Systeme innerhalb eines Frames.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Movement,
    Combat,
    World,
    Ui,
}

/// Existiert, solange eine Partie läuft.
#[derive(Resource, Default)]
pub struct InGame {
    pub paused: bool,
    /// Spielzeit in Sekunden (ohne Pausen)
    pub time: f32,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, finish_loading.run_if(in_state(AppState::Loading)))
            .add_systems(OnEnter(AppState::Playing), start_game)
            .add_systems(OnExit(AppState::Playing), end_game)
            .add_systems(
                Update,
                (tick_game, toggle_pause).run_if(in_state(AppState::Playing)),
            );
    }
}

fn finish_loading(
    map: Option<Res<MapReady>>,
    assets: Option<Res<super::assets::AssetsReady>>,
    cfg: Res<super::debug::DebugConfig>,
    mut next: ResMut<NextState<AppState>>,
) {
    if map.is_some() && assets.is_some() {
        // Für automatische Tests direkt ins Spiel, sonst ins Hauptmenü
        next.set(if cfg.skip_menu {
            AppState::Playing
        } else {
            AppState::Menu
        });
    }
}

fn start_game(mut commands: Commands, mut vtime: ResMut<Time<Virtual>>) {
    commands.insert_resource(InGame::default());
    vtime.unpause();
}

fn end_game(mut commands: Commands, mut vtime: ResMut<Time<Virtual>>) {
    commands.remove_resource::<InGame>();
    vtime.unpause();
}

fn tick_game(time: Res<Time>, mut game: ResMut<InGame>) {
    if !game.paused {
        game.time += time.delta_secs();
    }
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<InGame>,
    mut vtime: ResMut<Time<Virtual>>,
    mut gfx: ResMut<super::settings::GraphicsMenuOpen>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if gfx.0 {
            gfx.0 = false;
            return;
        }
        let paused = !game.paused;
        set_paused(&mut game, &mut vtime, paused);
    }
}

pub fn set_paused(game: &mut InGame, vtime: &mut Time<Virtual>, paused: bool) {
    game.paused = paused;
    if paused {
        vtime.pause();
    } else {
        vtime.unpause();
    }
}
