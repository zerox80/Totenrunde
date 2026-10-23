//! Aktive Power-ups unten in der Mitte (Symbol mit Restzeit, blinkt am Ende).

use bevy::prelude::*;

use super::widgets::label;
use crate::base::assets::GameAssets;
use crate::base::AppState;
use crate::game::powerups::ActivePowerups;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum PuBadge {
    Insta,
    Double,
    FireSale,
}

#[derive(Component)]
pub struct PuTime(pub PuBadge);

pub fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                bottom: px(40),
                justify_content: JustifyContent::Center,
                column_gap: px(18),
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            for (b, txt) in [(PuBadge::Insta, "TOD"), (PuBadge::Double, "×2"), (PuBadge::FireSale, "%")] {
                p.spawn((
                    b,
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|c| {
                    c.spawn((
                        Node {
                            width: px(58),
                            height: px(58),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(px(3)),
                            border_radius: BorderRadius::all(px(29)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.4, 1.0, 0.35)),
                        BackgroundColor(Color::srgba(0.05, 0.25, 0.05, 0.75)),
                    ))
                    .with_child(label(txt, &assets.font_black, 22.0, Color::srgb(0.75, 1.0, 0.7)));
                    c.spawn((PuTime(b), label("", &assets.font_bold, 16.0, Color::WHITE)));
                });
            }
        });
}

pub fn update(
    time: Res<Time>,
    pu: Res<ActivePowerups>,
    mut badges: Query<(&PuBadge, &mut Visibility)>,
    mut times: Query<(&PuTime, &mut Text)>,
) {
    let left = |b: PuBadge| match b {
        PuBadge::Insta => pu.insta_kill,
        PuBadge::Double => pu.double_points,
        PuBadge::FireSale => pu.fire_sale,
    };
    for (b, mut v) in badges.iter_mut() {
        let l = left(*b);
        let blink = l < 5.0 && (time.elapsed_secs() * 6.0).fract() < 0.4;
        let want = if l > 0.0 && !blink { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
    for (t, mut txt) in times.iter_mut() {
        let s = format!("{:.0}", left(t.0).ceil());
        if txt.0 != s {
            txt.0 = s;
        }
    }
}
