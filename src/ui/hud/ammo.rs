//! Munition (Magazin / Reserve), Waffenname und Granaten unten rechts.

use bevy::prelude::*;

use super::widgets::label;
use crate::base::assets::GameAssets;
use crate::base::AppState;
use crate::weapons::{def, Loadout};

#[derive(Component)]
pub struct MagText;

#[derive(Component)]
pub struct ReserveText;

#[derive(Component)]
pub struct WeaponName;

#[derive(Component)]
pub struct Grenade(pub u32);

pub fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
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
            p.spawn((WeaponName, label("", &assets.font_bold, 18.0, Color::srgba(1.0, 1.0, 1.0, 0.85))));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                column_gap: px(8),
                ..default()
            })
            .with_children(|r| {
                r.spawn((MagText, label("8", &assets.font_black, 46.0, Color::WHITE)));
                r.spawn((
                    ReserveText,
                    label("/ 80", &assets.font_bold, 24.0, Color::srgba(1.0, 1.0, 1.0, 0.75)),
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
                        Node {
                            width: px(12),
                            height: px(16),
                            border_radius: BorderRadius::all(px(6)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.45, 0.55, 0.35)),
                    ));
                }
            });
        });
}

#[allow(clippy::type_complexity)]
pub fn update(
    lo: Single<&Loadout>,
    mut mag: Query<(&mut Text, &mut TextColor), (With<MagText>, Without<ReserveText>, Without<WeaponName>)>,
    mut res: Query<&mut Text, (With<ReserveText>, Without<MagText>, Without<WeaponName>)>,
    mut name: Query<&mut Text, (With<WeaponName>, Without<MagText>, Without<ReserveText>)>,
    mut nades: Query<(&Grenade, &mut Visibility)>,
) {
    let (m, r, n, low) = match lo.current_slot() {
        Some(s) => {
            let d = def(s.weapon);
            let low = s.mag as f32 <= (d.mag_size(s.upgraded) as f32 * 0.25).max(1.0);
            (s.mag.to_string(), format!("/ {}", s.reserve), d.display_name(s.upgraded).to_string(), low)
        }
        None => (String::new(), String::new(), String::new(), false),
    };
    for (mut t, mut c) in mag.iter_mut() {
        if t.0 != m {
            t.0 = m.clone();
        }
        c.0 = if low { Color::srgb(1.0, 0.3, 0.25) } else { Color::WHITE };
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
        let want = if g.0 < lo.grenades { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}
