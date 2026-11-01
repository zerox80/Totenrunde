//! Game-Over-Bildschirm wie in BO2: "Ihr habt N Runden überlebt" und die Statistik –
//! im Koop als Tabelle aller Spieler.

use bevy::prelude::*;

use super::widgets::{BLOOD, MenuAction, button, root, text};
use crate::base::AppState;
use crate::base::assets::GameAssets;
use crate::base::i18n::{t, tf};
use crate::game::points::Score;
use crate::game::rounds::Rounds;
use crate::net::{Session, Team, slot_color};

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
    session: Option<Res<Session>>,
    team: Res<Team>,
) {
    commands.spawn((Camera2d, DespawnOnExit(AppState::GameOver)));
    let round = rounds.map(|r| r.round).unwrap_or(1);
    if let (Some(session), Some(score)) = (session, score.as_deref()) {
        spawn_team(&mut commands, &assets, round, &session, score, &team);
        return;
    }
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

/// Koop: Tabelle aller Spieler, zurück in die Lobby oder ins Hauptmenü.
fn spawn_team(
    commands: &mut Commands,
    assets: &GameAssets,
    round: u32,
    session: &Session,
    score: &Score,
    team: &Team,
) {
    let survived = if round == 1 {
        t("gameover.team_survived_one").to_string()
    } else {
        tf("gameover.team_survived", &[("rounds", &round)])
    };
    // (Platz, Name, Punkte, Kills, Kopfschüsse, Niedergestreckt, Wiederbelebt)
    let mut rows = vec![(
        session.you,
        session.name(session.you).to_owned(),
        score.earned,
        score.kills,
        score.headshots,
        score.downs,
        score.revives,
    )];
    rows.extend(team.members.iter().filter(|m| m.seen).map(|m| {
        (
            m.slot,
            m.name.clone(),
            m.state.earned,
            m.state.kills,
            m.state.headshots,
            m.state.downs,
            m.state.revives,
        )
    }));
    rows.sort_by_key(|r| r.0);
    let columns = [
        t("stats.points"),
        t("stats.kills"),
        t("stats.headshots"),
        t("stats.downs"),
        t("stats.revives"),
    ];
    let dim = Color::srgba(1.0, 1.0, 1.0, 0.6);
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
                    margin: UiRect::bottom(px(30)),
                    ..default()
                },
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(px(30)),
                row_gap: px(6),
                ..default()
            })
            .with_children(|table| {
                let cell = |w: f32| Node {
                    width: px(w),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                };
                table
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    })
                    .with_children(|r| {
                        r.spawn(Node {
                            width: px(220),
                            ..default()
                        });
                        for c in columns {
                            r.spawn(cell(130.0)).with_child(text(
                                c,
                                &assets.font_regular,
                                16.0,
                                dim,
                            ));
                        }
                    });
                for (slot, name, points, kills, heads, downs, revives) in rows {
                    table
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(px(10), px(6)),
                                border: UiRect::left(px(4)),
                                ..default()
                            },
                            BorderColor::all(slot_color(slot)),
                            BackgroundColor(Color::srgba(0.1, 0.06, 0.06, 0.8)),
                        ))
                        .with_children(|r| {
                            r.spawn(Node {
                                width: px(210),
                                ..default()
                            })
                            .with_child(text(
                                name,
                                &assets.font_bold,
                                22.0,
                                slot_color(slot),
                            ));
                            for v in [
                                points.to_string(),
                                kills.to_string(),
                                heads.to_string(),
                                downs.to_string(),
                                revives.to_string(),
                            ] {
                                r.spawn(cell(130.0)).with_child(text(
                                    v,
                                    &assets.font_bold,
                                    22.0,
                                    Color::srgb(0.95, 0.93, 0.9),
                                ));
                            }
                        });
                }
            });
            button(
                p,
                "gameover.lobby",
                MenuAction::Multiplayer,
                &assets.font_bold,
            );
            button(p, "menu.main_menu", MenuAction::MainMenu, &assets.font_bold);
        });
}

fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    session: Option<Res<Session>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        next.set(if session.is_some() {
            AppState::Multiplayer
        } else {
            AppState::Playing
        });
    } else if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::Menu);
    }
}
