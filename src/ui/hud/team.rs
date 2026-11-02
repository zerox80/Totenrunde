//! Koop: Punkte der Mitspieler über den eigenen (wie in BO2, in der Spielerfarbe), kurze
//! Meldungen oben in der Mitte und der Hinweis, wenn man selbst am Boden liegt.

use bevy::prelude::*;

use super::widgets::label;
use crate::base::AppState;
use crate::base::assets::GameAssets;
use crate::base::i18n::{t, tf};
use crate::net::msg::Life;
use crate::net::{Notices, Session, Team, slot_color};
use crate::player::Health;

/// Spalte mit den Punkten der Mitspieler (im Rundenzähler-Block über den eigenen Punkten).
#[derive(Component)]
pub struct TeamList;

#[derive(Component)]
pub struct NoticeList;

#[derive(Component)]
pub struct DownedText;

const NOTICE_TIME: f32 = 5.0;

pub fn spawn(mut commands: Commands, assets: Res<GameAssets>, session: Option<Res<Session>>) {
    if session.is_none() {
        return;
    }
    commands.spawn((
        NoticeList,
        Node {
            position_type: PositionType::Absolute,
            top: px(70),
            width: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        DespawnOnExit(AppState::Playing),
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: percent(36),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_child((
            DownedText,
            label("", &assets.font_black, 30.0, Color::srgb(1.0, 0.3, 0.25)),
            TextLayout::justify(Justify::Center),
        ));
}

pub fn update_team(
    mut commands: Commands,
    team: Res<Team>,
    assets: Res<GameAssets>,
    list: Query<Entity, With<TeamList>>,
    mut shown: Local<String>,
) {
    let Ok(list) = list.single() else { return };
    let rows: Vec<(u8, String, u32, Life)> = team
        .members
        .iter()
        .filter(|m| m.connected && m.seen)
        .map(|m| (m.slot, m.name.clone(), m.state.points, m.state.life))
        .collect();
    let sig = format!("{rows:?}");
    if *shown == sig {
        return;
    }
    *shown = sig;
    commands.entity(list).despawn_children();
    commands.entity(list).with_children(|p| {
        for (slot, name, points, life) in rows {
            let color = if life == Life::Alive {
                slot_color(slot)
            } else {
                Color::srgb(1.0, 0.3, 0.25)
            };
            p.spawn((
                Node {
                    width: px(230),
                    padding: UiRect::new(px(12), px(14), px(2), px(2)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    border: UiRect::left(px(4)),
                    ..default()
                },
                BorderColor::all(slot_color(slot)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ))
            .with_children(|r| {
                r.spawn(label(name, &assets.font_bold, 17.0, color));
                r.spawn(label(points.to_string(), &assets.font_black, 22.0, color));
            });
        }
    });
}

pub fn update_notices(
    mut commands: Commands,
    time: Res<Time>,
    mut notices: ResMut<Notices>,
    assets: Res<GameAssets>,
    list: Query<Entity, With<NoticeList>>,
    mut shown: Local<usize>,
) {
    let Ok(list) = list.single() else { return };
    let dt = time.delta_secs();
    for n in notices.list.iter_mut() {
        n.1 += dt;
    }
    notices.list.retain(|n| n.1 < NOTICE_TIME);
    // neu aufbauen, wenn Meldungen dazukommen oder gehen
    let sig = notices.list.len() + notices.list.iter().map(|n| n.0.len()).sum::<usize>() * 8;
    if *shown == sig {
        return;
    }
    *shown = sig;
    commands.entity(list).despawn_children();
    commands.entity(list).with_children(|p| {
        for (text, _) in &notices.list {
            p.spawn(label(
                text.clone(),
                &assets.font_bold,
                20.0,
                Color::srgb(0.95, 0.92, 0.86),
            ));
        }
    });
}

/// Eigener Zustand im Koop: am Boden (mit Restzeit) oder tot bis zur nächsten Runde.
pub fn update_downed(health: Query<&Health>, mut text: Query<&mut Text, With<DownedText>>) {
    let (Ok(h), Ok(mut text)) = (health.single(), text.single_mut()) else {
        return;
    };
    let want = if h.dead {
        t("hud.coop_dead").to_owned()
    } else if h.downed {
        tf(
            "hud.coop_downed",
            &[("seconds", &(h.revive_left.max(0.0).ceil() as u32))],
        )
    } else {
        String::new()
    };
    if text.0 != want {
        text.0 = want;
    }
}
