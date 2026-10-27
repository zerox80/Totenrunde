//! Hauptmenü: Titel, "Spielen" und "Beenden" (Enter / Esc).

use bevy::prelude::*;

use super::widgets::{BLOOD, MenuAction, button, root, text};
use crate::base::AppState;
use crate::base::assets::GameAssets;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), spawn)
            .add_systems(Update, keys.run_if(in_state(AppState::Menu)));
    }
}

fn spawn(mut commands: Commands, assets: Res<GameAssets>, lib: Option<Res<crate::audio::SfxLib>>) {
    commands.spawn((Camera2d, DespawnOnExit(AppState::Menu)));
    if let Some(lib) = lib {
        commands.spawn((
            AudioPlayer::new(lib.music_menu.clone()),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.55)),
            DespawnOnExit(AppState::Menu),
        ));
    }
    commands
        .spawn((root(Color::srgb(0.025, 0.02, 0.02)), DespawnOnExit(AppState::Menu)))
        .with_children(|p| {
            p.spawn(text("TOTENRUNDE", &assets.font_black, 110.0, BLOOD));
            p.spawn((
                text("ENDSTATION", &assets.font_bold, 26.0, Color::srgb(0.7, 0.68, 0.64)),
                Node {
                    margin: UiRect::bottom(px(60)),
                    ..default()
                },
            ));
            button(p, "SPIELEN", MenuAction::Play, &assets.font_bold);
            super::graphics::open_button(p, &assets.font_bold);
            button(p, "BEENDEN", MenuAction::Quit, &assets.font_bold);
            p.spawn((
                text(
                    "Enter: Spielen   ·   Esc: Beenden\nWASD laufen · Maus zielen · Linksklick schießen · R nachladen · V Messer · G Granate · F benutzen",
                    &assets.font_regular,
                    16.0,
                    Color::srgba(1.0, 1.0, 1.0, 0.45),
                ),
                TextLayout::justify(Justify::Center),
                Node {
                    margin: UiRect::top(px(50)),
                    ..default()
                },
            ));
        });
}

fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    mut gfx: ResMut<crate::base::settings::GraphicsMenuOpen>,
) {
    if gfx.0 {
        if keys.just_pressed(KeyCode::Escape) {
            gfx.0 = false;
        }
        return;
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next.set(AppState::Playing);
    } else if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
