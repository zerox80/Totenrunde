//! Ladebildschirm mit Fortschrittsbalken, bis Karte, Modelle und alle Texturen geladen sind.
//! Im Browser kommen die Texturen erst nach und nach über das Netz; ohne ihn stünde man
//! so lange vor einer grauen Fläche.

use bevy::prelude::*;

use super::widgets::{BLOOD, IDLE, root, text};
use crate::base::AppState;
use crate::base::assets::LoadProgress;
use crate::base::i18n::tf;

const BACKGROUND: Color = Color::srgb(0.025, 0.02, 0.02);

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), spawn)
            .add_systems(Update, update.run_if(in_state(AppState::Loading)));
    }
}

#[derive(Component)]
struct ProgressBar;

#[derive(Component)]
struct ProgressLabel;

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    // dieselben Pfade wie GameAssets, also dieselben (schnell geladenen) Schriften
    let font_black = asset_server.load("fonts/Montserrat-Black.otf");
    let font_bold = asset_server.load("fonts/Montserrat-ExtraBold.otf");
    let font_regular = asset_server.load("fonts/Montserrat-SemiBold.otf");
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(BACKGROUND),
            ..default()
        },
        DespawnOnExit(AppState::Loading),
    ));
    commands
        .spawn((root(BACKGROUND), DespawnOnExit(AppState::Loading)))
        .with_children(|p| {
            p.spawn(text("TOTENRUNDE", &font_black, 110.0, BLOOD));
            p.spawn((
                text("ENDSTATION", &font_bold, 26.0, Color::srgb(0.7, 0.68, 0.64)),
                Node {
                    margin: UiRect::bottom(px(60)),
                    ..default()
                },
            ));
            p.spawn((
                Node {
                    width: px(520),
                    max_width: percent(80),
                    height: px(10),
                    ..default()
                },
                BackgroundColor(IDLE),
            ))
            .with_children(|track| {
                track.spawn((
                    ProgressBar,
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(BLOOD),
                ));
            });
            p.spawn((
                ProgressLabel,
                text("", &font_regular, 16.0, Color::srgba(1.0, 1.0, 1.0, 0.55)),
                Node {
                    margin: UiRect::top(px(16)),
                    ..default()
                },
            ));
        });
}

fn update(
    progress: Res<LoadProgress>,
    mut bars: Query<&mut Node, With<ProgressBar>>,
    mut labels: Query<&mut Text, With<ProgressLabel>>,
) {
    let percent_done = (progress.fraction * 100.0).floor();
    for mut bar in &mut bars {
        bar.width = percent(percent_done);
    }
    let label = if progress.textures_total > 0 && progress.fraction > 0.1 {
        tf(
            "loading.textures",
            &[
                ("done", &progress.textures_done),
                ("total", &progress.textures_total),
                ("percent", &percent_done),
            ],
        )
    } else {
        tf("loading.models", &[("percent", &percent_done)])
    };
    for mut text in &mut labels {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
}
