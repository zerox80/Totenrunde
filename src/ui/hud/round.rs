//! Rundenzähler in roter Kreide (unten links) und Perk-Symbole darüber.
//! Runde 1–5 als Strichliste wie im Original, ab Runde 6 als Zahl. Beim Rundenwechsel
//! blinkt der Zähler weiß.

use bevy::prelude::*;
use bevy::ui::UiTransform;

use super::widgets::{label, CHALK_RED};
use crate::base::assets::GameAssets;
use crate::base::{AppState, InGame};
use crate::game::perks::{Perk, Perks};
use crate::game::rounds::{RoundPhase, Rounds};
use crate::world::mapdata::asset_root;

#[derive(Component)]
pub struct RoundNumber;

#[derive(Component)]
pub struct Tally(pub u32);

#[derive(Component)]
pub struct PerkRow;

#[derive(Component)]
pub struct PerkIcon(pub Perk);

pub fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(36),
                bottom: px(26),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(10),
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((
                PerkRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8),
                    ..default()
                },
            ));
            // Strichliste (5 Striche, der fünfte quer)
            p.spawn(Node {
                width: px(150),
                height: px(96),
                ..default()
            })
            .with_children(|t| {
                for i in 0..5u32 {
                    let (left, rot, h) = if i < 4 {
                        (8.0 + i as f32 * 24.0, -0.08 + (i as f32 * 0.07).sin() * 0.05, 86.0)
                    } else {
                        (52.0, 1.05, 124.0)
                    };
                    t.spawn((
                        Tally(i + 1),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(left),
                            top: px(if i < 4 { 4.0 } else { -14.0 }),
                            width: px(9),
                            height: px(h),
                            border_radius: BorderRadius::all(px(4)),
                            ..default()
                        },
                        UiTransform::from_rotation(Rot2::radians(rot)),
                        BackgroundColor(CHALK_RED),
                        Visibility::Hidden,
                    ));
                }
                t.spawn((
                    RoundNumber,
                    label("", &assets.font_black, 96.0, CHALK_RED),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(-14),
                        ..default()
                    },
                ));
            });
        });
}

pub fn update(
    time: Res<Time>,
    game: Res<InGame>,
    rounds: Option<Res<Rounds>>,
    mut num: Query<(&mut Text, &mut TextColor), With<RoundNumber>>,
    mut tallies: Query<(&Tally, &mut Visibility, &mut BackgroundColor)>,
) {
    let Some(r) = rounds else { return };
    let round = r.round;
    // weißes Pulsieren beim Rundenwechsel
    let pulse = match r.phase {
        RoundPhase::Intro { .. } | RoundPhase::Intermission { .. } => (time.elapsed_secs() * 4.0).sin() * 0.5 + 0.5,
        RoundPhase::Active => 0.0,
    };
    let since = game.time - r.changed_at;
    let flash = if since < 3.0 { pulse } else { pulse * 0.6 };
    let col = CHALK_RED.mix(&Color::srgb(0.95, 0.92, 0.88), flash);
    for (mut t, mut c) in num.iter_mut() {
        let s = if round > 5 { round.to_string() } else { String::new() };
        if t.0 != s {
            t.0 = s;
        }
        c.0 = col;
    }
    for (tally, mut v, mut bg) in tallies.iter_mut() {
        let want = if round <= 5 && tally.0 <= round { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
        bg.0 = col;
    }
}

pub fn perk_icons(
    mut commands: Commands,
    perks: Res<Perks>,
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    row: Query<Entity, With<PerkRow>>,
    icons: Query<(Entity, &PerkIcon)>,
) {
    if !perks.is_changed() {
        return;
    }
    let Ok(row) = row.single() else { return };
    for (e, _) in icons.iter() {
        commands.entity(e).despawn();
    }
    for perk in &perks.owned {
        let icon = perk.icon();
        let mut e = commands.spawn((
            PerkIcon(*perk),
            Node {
                width: px(48),
                height: px(48),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5)),
            ChildOf(row),
        ));
        if asset_root().join(&icon).exists() {
            e.insert(ImageNode::new(asset_server.load(icon)));
        } else {
            let c = perk.color().to_srgba();
            e.insert(BackgroundColor(Color::srgba(c.red * 0.55, c.green * 0.55, c.blue * 0.55, 0.92)));
            let short: String = perk.name().split(['-', ' ']).filter_map(|w| w.chars().next()).take(2).collect();
            e.with_child(label(short, &assets.font_black, 20.0, Color::WHITE));
        }
    }
}
