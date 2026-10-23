//! Hinweistext für die Taste F ("Halte F für M14 [Kosten: 500]").

use bevy::prelude::*;

use super::widgets::label;
use crate::base::assets::GameAssets;
use crate::base::AppState;
use crate::game::interact::Prompt;

#[derive(Component)]
pub struct PromptText;

pub fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                top: percent(64),
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((PromptText, label("", &assets.font_bold, 22.0, Color::srgb(0.96, 0.95, 0.9))));
        });
}

pub fn update(prompt: Res<Prompt>, mut q: Query<&mut Text, With<PromptText>>) {
    let s = prompt.text.clone().unwrap_or_default();
    for mut t in q.iter_mut() {
        if t.0 != s {
            t.0 = s.clone();
        }
    }
}
