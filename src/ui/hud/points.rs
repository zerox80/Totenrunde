//! Punkte unten rechts und aufsteigende Änderungen ("+10", "-950").

use bevy::prelude::*;

use super::widgets::label;
use crate::base::assets::GameAssets;
use crate::base::AppState;
use crate::game::points::Score;

#[derive(Component)]
pub struct PointsText;

#[derive(Component)]
pub struct PointsAnchor;

#[derive(Component)]
pub struct Popup {
    pub t: f32,
    pub dx: f32,
}

pub fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            PointsAnchor,
            Node {
                position_type: PositionType::Absolute,
                right: px(40),
                bottom: px(150),
                min_width: px(150),
                padding: UiRect::axes(px(16), px(6)),
                justify_content: JustifyContent::FlexEnd,
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.85, 0.66, 0.12, 0.85)),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((PointsText, label("500", &assets.font_black, 30.0, Color::WHITE)));
        });
}

pub fn update(score: Res<Score>, mut q: Query<&mut Text, With<PointsText>>) {
    if !score.is_changed() {
        return;
    }
    for mut t in q.iter_mut() {
        let s = score.points.to_string();
        if t.0 != s {
            t.0 = s;
        }
    }
}

pub fn popups(
    mut commands: Commands,
    time: Res<Time>,
    mut score: ResMut<Score>,
    assets: Res<GameAssets>,
    anchor: Query<Entity, With<PointsAnchor>>,
    mut q: Query<(Entity, &mut Popup, &mut Node, &mut TextColor)>,
) {
    let Ok(anchor) = anchor.single() else { return };
    let mut rng_off = 0.0;
    for amount in score.pending_popups.drain(..) {
        let (txt, col) = if amount >= 0 {
            (format!("+{amount}"), Color::srgb(1.0, 0.86, 0.25))
        } else {
            (format!("{amount}"), Color::srgb(1.0, 0.25, 0.2))
        };
        commands.spawn((
            Popup { t: 0.0, dx: rng_off },
            label(txt, &assets.font_black, 24.0, col),
            Node {
                position_type: PositionType::Absolute,
                right: px(170),
                bottom: px(4),
                ..default()
            },
            ChildOf(anchor),
        ));
        rng_off += 18.0;
    }
    for (e, mut p, mut n, mut c) in q.iter_mut() {
        p.t += time.delta_secs();
        let k = p.t / 1.1;
        n.bottom = px(4.0 + k * 70.0);
        n.right = px(170.0 + p.dx + k * 30.0);
        c.0 = c.0.with_alpha((1.0 - k).clamp(0.0, 1.0));
        if k >= 1.0 {
            commands.entity(e).despawn();
        }
    }
}
