//! Kleine Hilfen für HUD-Texte.

use bevy::prelude::*;
use bevy::text::FontSize;

pub fn label(s: impl Into<String>, font: &Handle<Font>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::new(2.0, 2.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.8),
        },
    )
}

pub const CHALK_RED: Color = Color::srgb(0.72, 0.06, 0.05);
