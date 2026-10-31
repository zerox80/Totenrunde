//! Game-Over-Bildschirm wie in BO2: "Du hast N Runden überlebt" und die Statistik.

use bevy::prelude::*;

use super::widgets::{BLOOD, MenuAction, button, root, text};
use crate::base::AppState;
use crate::base::assets::GameAssets;
use crate::base::i18n::{t, tf};
use crate::game::points::Score;
use crate::game::rounds::Rounds;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameOver), spawn)
            .add_systems(Update, keys.run_if(in_state(AppState::GameOver)));
    }
}

fn spawn(
    mut commands: Commands,
    assets: Res<GameAssets>,
    score: Option<Res<Score>>,
    rounds: Option<Res<Rounds>>,
) {
    commands.spawn((Camera2d, DespawnOnExit(AppState::GameOver)));
    let round = rounds.map(|r| r.round).unwrap_or(1);
    let survived = if round == 1 {
        t("gameover.survived_one").to_string()
    } else {
        tf("gameover.survived", &[("rounds", &round)])
    };
    let stats: Vec<(&str, String)> = match score {
        Some(s) => vec![
            (t("stats.points"), s.earned.to_string()),
            (t("stats.kills"), s.kills.to_string()),
            (t("stats.headshots"), s.headshots.to_string()),
            (t("stats.downs"), s.downs.to_string()),
            (t("stats.revives"), s.revives.to_string()),
            (t("stats.doors"), s.doors.to_string()),
            (t("stats.boards"), s.boards.to_string()),
        ],
        None => vec![],
    };
    commands
        .spawn((
            root(Color::srgb(0.03, 0.0, 0.0)),
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|p| {
            p.spawn(text(t("gameover.title"), &assets.font_black, 84.0, BLOOD));
            p.spawn((
                text(
                    survived,
                    &assets.font_bold,
                    30.0,
                    Color::srgb(0.9, 0.88, 0.84),
                ),
                Node {
                    margin: UiRect::bottom(px(36)),
                    ..default()
                },
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(px(36)),
                row_gap: px(4),
                ..default()
            })
            .with_children(|t| {
                for (k, v) in stats {
                    t.spawn(Node {
                        width: px(420),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(text(
                            k,
                            &assets.font_regular,
                            22.0,
                            Color::srgba(1.0, 1.0, 1.0, 0.7),
                        ));
                        row.spawn(text(
                            v,
                            &assets.font_bold,
                            22.0,
                            Color::srgb(0.95, 0.93, 0.9),
                        ));
                    });
                }
            });
            button(p, "gameover.again", MenuAction::Play, &assets.font_bold);
            button(p, "menu.main_menu", MenuAction::MainMenu, &assets.font_bold);
        });
}

fn keys(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) {
        next.set(AppState::Playing);
    } else if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::Menu);
    }
}
