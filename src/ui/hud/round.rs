//! Rundenzähler in roter Kreide (unten links) und Perk-Symbole darüber.
//! Runde 1–5 als Strichliste wie im Original, ab Runde 6 als Zahl. Beim Rundenwechsel
//! blinkt der Zähler weiß.

use bevy::prelude::*;
use bevy::ui::UiTransform;

use super::widgets::{CHALK_RED, label};
use crate::base::assets::GameAssets;
use crate::base::{AppState, InGame};
use crate::game::perks::{Perk, Perks};
use crate::game::rounds::{RoundPhase, Rounds};

#[derive(Component)]
pub struct RoundNumber;

#[derive(Component)]
pub struct Tally(pub u32);

#[derive(Component)]
pub struct PerkRow;

#[derive(Component)]
pub struct PerkIcon(pub Perk);

pub fn spawn(mut commands: Commands, asset_server: Res<AssetServer>, assets: Res<GameAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(30),
                bottom: px(18),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(6),
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((
                PerkRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(6),
                    margin: UiRect::bottom(px(4)),
                    ..default()
                },
            ));
            p.spawn((
                super::points::PointsAnchor,
                Node {
                    width: px(260),
                    height: px(52),
                    padding: UiRect::new(px(18), px(30), px(4), px(4)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode::new(asset_server.load("ui/points_bar.png")),
            ))
            .with_children(|b| {
                b.spawn((
                    super::points::PointsText,
                    label("500", &assets.font_black, 34.0, Color::WHITE),
                ));
            });
            // Kreide: Strichliste (Runde 1–5) bzw. Ziffern
            p.spawn(Node {
                width: px(240),
                height: px(150),
                ..default()
            })
            .with_children(|t| {
                for i in 1..=5u32 {
                    t.spawn((
                        Tally(i),
                        ImageNode {
                            image: asset_server.load(format!("ui/chalk_tally_{i}.png")),
                            color: CHALK_RED,
                            ..default()
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            width: px(200),
                            height: px(150),
                            ..default()
                        },
                        Visibility::Hidden,
                    ));
                }
                t.spawn((
                    RoundNumber,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                ));
            });
        });
}

/// Ziffer in Kreide
#[derive(Component)]
pub struct ChalkDigit;

#[allow(clippy::too_many_arguments)]
pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<InGame>,
    rounds: Option<Res<Rounds>>,
    asset_server: Res<AssetServer>,
    num: Query<(Entity, Option<&Children>), With<RoundNumber>>,
    mut digits: Query<&mut ImageNode, (With<ChalkDigit>, Without<Tally>)>,
    mut tallies: Query<(&Tally, &mut Visibility, &mut ImageNode), Without<ChalkDigit>>,
    mut shown: Local<u32>,
) {
    let Some(r) = rounds else { return };
    let round = r.round;
    // weißes Pulsieren beim Rundenwechsel
    let pulse = match r.phase {
        RoundPhase::Intro { .. } | RoundPhase::Intermission { .. } => {
            (time.elapsed_secs() * 4.0).sin() * 0.5 + 0.5
        }
        RoundPhase::Active => 0.0,
    };
    let since = game.time - r.changed_at;
    let flash = if since < 3.0 { pulse } else { pulse * 0.6 };
    let col = CHALK_RED.mix(&Color::srgb(0.95, 0.92, 0.88), flash);
    if let Ok((e, children)) = num.single() {
        if *shown != round {
            *shown = round;
            if let Some(ch) = children {
                for c in ch.iter() {
                    commands.entity(c).despawn();
                }
            }
            if round > 5 {
                for (i, d) in round.to_string().chars().enumerate() {
                    commands.spawn((
                        ChalkDigit,
                        ImageNode {
                            image: asset_server.load(format!("ui/chalk_{d}.png")),
                            color: col,
                            ..default()
                        },
                        Node {
                            width: px(96),
                            height: px(120),
                            margin: UiRect::left(px(if i == 0 { 0.0 } else { -22.0 })),
                            ..default()
                        },
                        UiTransform::from_rotation(Rot2::radians(-0.04 + i as f32 * 0.05)),
                        ChildOf(e),
                    ));
                }
            }
        }
    }
    for mut img in digits.iter_mut() {
        img.color = col;
    }
    for (tally, mut v, mut img) in tallies.iter_mut() {
        let want = if round <= 5 && tally.0 == round {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
        img.color = col;
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
                width: px(44),
                height: px(44),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5)),
            ChildOf(row),
        ));
        if crate::world::mapdata::asset_exists(&icon) {
            e.insert(ImageNode::new(asset_server.load(icon)));
        } else {
            let c = perk.color().to_srgba();
            e.insert(BackgroundColor(Color::srgba(
                c.red * 0.55,
                c.green * 0.55,
                c.blue * 0.55,
                0.92,
            )));
            let short: String = perk
                .name()
                .split(['-', ' '])
                .filter_map(|w| w.chars().next())
                .take(2)
                .collect();
            e.with_child(label(short, &assets.font_black, 20.0, Color::WHITE));
        }
    }
}
