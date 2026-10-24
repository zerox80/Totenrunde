//! Pausenmenü (Esc im Spiel): Weiter, Hauptmenü, Beenden.

use bevy::prelude::*;

use super::widgets::{button, root, text, MenuAction};
use crate::base::assets::GameAssets;
use crate::base::{AppState, InGame};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync.run_if(in_state(AppState::Playing)));
    }
}

#[derive(Component)]
struct PauseRoot;

fn sync(mut commands: Commands, game: Res<InGame>, roots: Query<Entity, With<PauseRoot>>, assets: Res<GameAssets>) {
    let shown = !roots.is_empty();
    if game.paused && !shown {
        commands
            .spawn((root(Color::srgba(0.0, 0.0, 0.0, 0.72)), PauseRoot, GlobalZIndex(50), DespawnOnExit(AppState::Playing)))
            .with_children(|p| {
                p.spawn((
                    text("PAUSE", &assets.font_black, 72.0, Color::srgb(0.9, 0.88, 0.84)),
                    Node {
                        margin: UiRect::bottom(px(40)),
                        ..default()
                    },
                ));
                button(p, "WEITER", MenuAction::Resume, &assets.font_bold);
                super::graphics::open_button(p, &assets.font_bold);
                button(p, "HAUPTMENÜ", MenuAction::MainMenu, &assets.font_bold);
                button(p, "BEENDEN", MenuAction::Quit, &assets.font_bold);
            });
    } else if !game.paused && shown {
        for e in roots.iter() {
            commands.entity(e).despawn();
        }
    }
}
