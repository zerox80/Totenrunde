//! Munition (Magazin / Reserve), Waffenname und Granaten unten rechts.

use bevy::prelude::*;

use super::widgets::label;
use crate::base::AppState;
use crate::base::assets::GameAssets;
use crate::weapons::{Loadout, def};

#[derive(Component)]
pub struct MagText;

#[derive(Component)]
pub struct ReserveText;

#[derive(Component)]
pub struct WeaponName;

#[derive(Component)]
pub struct Grenade(pub u32);

/// Hinweis unter dem Fadenkreuz ("NACHLADEN", "WENIG MUNITION", "KEINE MUNITION")
#[derive(Component)]
pub struct AmmoHint;

pub fn spawn(mut commands: Commands, assets: Res<GameAssets>, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                top: percent(58),
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_child((
            AmmoHint,
            label(
                "",
                &assets.font_bold,
                20.0,
                Color::srgba(1.0, 1.0, 1.0, 0.9),
            ),
        ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(40),
                bottom: px(30),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((
                WeaponName,
                label(
                    "",
                    &assets.font_bold,
                    17.0,
                    Color::srgba(0.92, 0.9, 0.84, 0.9),
                ),
            ));
            // feine Linie unter dem Namen
            p.spawn((
                Node {
                    width: px(170),
                    height: px(2),
                    margin: UiRect::vertical(px(3)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.35)),
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                column_gap: px(8),
                ..default()
            })
            .with_children(|r| {
                r.spawn((MagText, label("8", &assets.font_black, 52.0, Color::WHITE)));
                r.spawn((
                    ReserveText,
                    label(
                        "/ 80",
                        &assets.font_bold,
                        24.0,
                        Color::srgba(1.0, 1.0, 1.0, 0.75),
                    ),
                    Node {
                        margin: UiRect::bottom(px(6)),
                        ..default()
                    },
                ));
            });
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(5),
                margin: UiRect::top(px(4)),
                ..default()
            })
            .with_children(|g| {
                for i in 0..4u32 {
                    g.spawn((
                        Grenade(i),
                        ImageNode {
                            image: asset_server.load("ui/grenade.png"),
                            color: Color::srgb(0.62, 0.68, 0.5),
                            ..default()
                        },
                        Node {
                            width: px(26),
                            height: px(26),
                            ..default()
                        },
                    ));
                }
            });
        });
}

#[allow(clippy::type_complexity)]
pub fn update(
    lo: Single<&Loadout>,
    mut mag: Query<
        (&mut Text, &mut TextColor),
        (With<MagText>, Without<ReserveText>, Without<WeaponName>),
    >,
    mut res: Query<&mut Text, (With<ReserveText>, Without<MagText>, Without<WeaponName>)>,
    mut name: Query<&mut Text, (With<WeaponName>, Without<MagText>, Without<ReserveText>)>,
    mut nades: Query<(&Grenade, &mut Visibility)>,
    mut hint: Query<
        &mut Text,
        (
            With<AmmoHint>,
            Without<MagText>,
            Without<ReserveText>,
            Without<WeaponName>,
        ),
    >,
) {
    let h = match lo.current_slot() {
        Some(s) if s.mag == 0 && s.reserve == 0 => "KEINE MUNITION",
        Some(s)
            if s.mag as f32 <= (def(s.weapon).mag_size(s.upgraded) as f32 * 0.25).max(1.0)
                && s.reserve > 0 =>
        {
            "NACHLADEN"
        }
        Some(s) if s.reserve <= def(s.weapon).mag_size(s.upgraded) && s.mag > 0 => "WENIG MUNITION",
        _ => "",
    };
    for mut t in hint.iter_mut() {
        if t.0 != h {
            t.0 = h.to_string();
        }
    }
    let (m, r, n, low) = match lo.current_slot() {
        Some(s) => {
            let d = def(s.weapon);
            let low = s.mag as f32 <= (d.mag_size(s.upgraded) as f32 * 0.25).max(1.0);
            (
                s.mag.to_string(),
                format!("/ {}", s.reserve),
                d.display_name(s.upgraded).to_uppercase(),
                low,
            )
        }
        None => (String::new(), String::new(), String::new(), false),
    };
    for (mut t, mut c) in mag.iter_mut() {
        if t.0 != m {
            t.0 = m.clone();
        }
        c.0 = if low {
            Color::srgb(1.0, 0.3, 0.25)
        } else {
            Color::WHITE
        };
    }
    for mut t in res.iter_mut() {
        if t.0 != r {
            t.0 = r.clone();
        }
    }
    for mut t in name.iter_mut() {
        if t.0 != n {
            t.0 = n.clone();
        }
    }
    for (g, mut v) in nades.iter_mut() {
        let want = if g.0 < lo.grenades {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
    }
}
