//! Gemeinsame Menü-Bausteine: Titel, Knöpfe mit Hover-Effekt, Aktionen.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};

use crate::base::{set_paused, AppState, InGame};

/// Was ein Menüknopf auslöst.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction {
    Play,
    Resume,
    MainMenu,
    Quit,
}

pub const BLOOD: Color = Color::srgb(0.62, 0.05, 0.04);
pub const IDLE: Color = Color::srgba(0.08, 0.06, 0.06, 0.85);
pub const HOVER: Color = Color::srgba(0.38, 0.04, 0.03, 0.95);
pub const PRESS: Color = Color::srgba(0.55, 0.06, 0.04, 1.0);

pub fn text(s: impl Into<String>, font: &Handle<Font>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

/// Knopf mit Beschriftung; Aktion wird beim Loslassen ausgeführt.
pub fn button(parent: &mut ChildSpawnerCommands, label: &str, action: MenuAction, font: &Handle<Font>) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            action,
            Node {
                width: px(320),
                height: px(54),
                margin: UiRect::vertical(px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::left(px(4)),
                ..default()
            },
            BorderColor::all(BLOOD),
            BackgroundColor(IDLE),
        ))
        .with_children(|b| {
            b.spawn(text(label, font, 24.0, Color::srgb(0.92, 0.9, 0.86)));
        })
        .observe(on_activate);
}

pub fn button_visuals(mut q: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<MenuAction>>) {
    for (hovered, pressed, mut bg) in q.iter_mut() {
        let c = if pressed {
            PRESS
        } else if hovered.0 {
            HOVER
        } else {
            IDLE
        };
        if bg.0 != c {
            bg.0 = c;
        }
    }
}

fn on_activate(
    ev: On<Activate>,
    actions: Query<&MenuAction>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    mut game: Option<ResMut<InGame>>,
    mut vtime: ResMut<Time<Virtual>>,
) {
    if let Ok(action) = actions.get(ev.entity) {
        apply(*action, &mut next, &mut exit, game.as_deref_mut(), &mut vtime);
    }
}

pub fn apply(
    action: MenuAction,
    next: &mut NextState<AppState>,
    exit: &mut MessageWriter<AppExit>,
    game: Option<&mut InGame>,
    vtime: &mut Time<Virtual>,
) {
    match action {
        MenuAction::Play => next.set(AppState::Playing),
        MenuAction::Resume => {
            if let Some(g) = game {
                set_paused(g, vtime, false);
            }
        }
        MenuAction::MainMenu => next.set(AppState::Menu),
        MenuAction::Quit => {
            exit.write(AppExit::Success);
        }
    }
}

/// Vollbild-Wurzel für ein Menü (zentrierte Spalte).
pub fn root(bg: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(bg),
    )
}
