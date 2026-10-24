//! Grafik-Einstellungen: Stufen NIEDRIG / MITTEL / HOCH / EXTREM und einzelne Optionen.
//! Änderungen wirken sofort (bis auf die Texturqualität, die beim nächsten Start greift)
//! und werden in settings.json gespeichert.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};

use super::widgets::{text, BLOOD, HOVER, IDLE, PRESS};
use crate::base::assets::GameAssets;
use crate::base::settings::{GraphicsMenuOpen, Settings, QUALITY_NAMES};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync, values, visuals));
    }
}

#[derive(Component)]
struct GfxRoot;

/// Eine Zeile im Menü (Klick schaltet den Wert weiter).
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Opt {
    Preset,
    Textures,
    Msaa,
    Shadows,
    Bloom,
    Film,
    AutoExposure,
    Cones,
    Fov,
    Vsync,
    Fps,
    Back,
}

/// Text mit dem aktuellen Wert einer Zeile
#[derive(Component)]
struct OptValue(Opt);

#[derive(Component)]
struct OpenGraphics;

/// Knopf "GRAFIK" für Haupt- und Pausenmenü.
pub fn open_button(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            OpenGraphics,
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
            b.spawn(text("GRAFIK", font, 24.0, Color::srgb(0.92, 0.9, 0.86)));
        })
        .observe(|_: On<Activate>, mut open: ResMut<GraphicsMenuOpen>| open.0 = true);
}

fn label(o: Opt) -> &'static str {
    match o {
        Opt::Preset => "GRAFIKSTUFE",
        Opt::Textures => "TEXTUREN",
        Opt::Msaa => "KANTENGLÄTTUNG",
        Opt::Shadows => "ECHTZEIT-SCHATTEN",
        Opt::Bloom => "LEUCHTEFFEKT (BLOOM)",
        Opt::Film => "FILMLOOK",
        Opt::AutoExposure => "AUGEN-ANPASSUNG",
        Opt::Cones => "LICHTKEGEL",
        Opt::Fov => "SICHTFELD",
        Opt::Vsync => "VSYNC",
        Opt::Fps => "FPS ANZEIGEN",
        Opt::Back => "ZURÜCK",
    }
}

fn onoff(b: bool) -> String {
    if b { "AN".into() } else { "AUS".into() }
}

fn value(o: Opt, s: &Settings) -> String {
    match o {
        Opt::Preset => QUALITY_NAMES[s.quality.min(4) as usize].into(),
        Opt::Textures => match s.texture_max {
            0..=512 => "NIEDRIG (512)".into(),
            513..=1024 => "MITTEL (1024)".into(),
            _ => "HOCH (2048) *".into(),
        },
        Opt::Msaa => match s.msaa {
            0 => "AUS".into(),
            1 => "FXAA".into(),
            2 => "MSAA 2×".into(),
            _ => "MSAA 4×".into(),
        },
        Opt::Shadows => onoff(s.shadows),
        Opt::Bloom => onoff(s.bloom),
        Opt::Film => onoff(s.film),
        Opt::AutoExposure => onoff(s.auto_exposure),
        Opt::Cones => onoff(s.light_cones),
        Opt::Fov => format!("{:.0}°", s.fov_deg),
        Opt::Vsync => onoff(s.vsync),
        Opt::Fps => onoff(s.show_fps),
        Opt::Back => String::new(),
    }
}

const ROWS: [Opt; 12] = [
    Opt::Preset,
    Opt::Textures,
    Opt::Msaa,
    Opt::Shadows,
    Opt::Bloom,
    Opt::Film,
    Opt::AutoExposure,
    Opt::Cones,
    Opt::Fov,
    Opt::Vsync,
    Opt::Fps,
    Opt::Back,
];

fn sync(
    mut commands: Commands,
    open: Res<GraphicsMenuOpen>,
    roots: Query<Entity, With<GfxRoot>>,
    assets: Option<Res<GameAssets>>,
    settings: Res<Settings>,
) {
    let shown = !roots.is_empty();
    if open.0 && !shown {
        let Some(assets) = assets else { return };
        commands
            .spawn((
                GfxRoot,
                GlobalZIndex(80),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.015, 0.015, 0.99)),
            ))
            .with_children(|p| {
                p.spawn((
                    text("GRAFIK", &assets.font_black, 64.0, BLOOD),
                    Node {
                        margin: UiRect::bottom(px(18)),
                        ..default()
                    },
                ));
                for o in ROWS {
                    let back = o == Opt::Back;
                    p.spawn((
                        Button,
                        Hovered::default(),
                        o,
                        Node {
                            width: px(640),
                            height: px(if back { 50.0 } else { 42.0 }),
                            margin: UiRect::vertical(px(if back { 16.0 } else { 3.0 })),
                            padding: UiRect::horizontal(px(18)),
                            justify_content: if back { JustifyContent::Center } else { JustifyContent::SpaceBetween },
                            align_items: AlignItems::Center,
                            border: UiRect::left(px(if o == Opt::Preset { 6.0 } else { 3.0 })),
                            ..default()
                        },
                        BorderColor::all(BLOOD),
                        BackgroundColor(IDLE),
                    ))
                    .with_children(|r| {
                        r.spawn(text(label(o), &assets.font_bold, if o == Opt::Preset { 22.0 } else { 19.0 }, Color::srgb(0.92, 0.9, 0.86)));
                        if !back {
                            r.spawn((OptValue(o), text(value(o, &settings), &assets.font_black, 19.0, Color::srgb(1.0, 0.82, 0.55))));
                        }
                    })
                    .observe(on_click);
                }
                p.spawn(text(
                    "* Texturen ändern sich beim nächsten Start · Klick schaltet weiter · Esc zurück",
                    &assets.font_regular,
                    15.0,
                    Color::srgba(1.0, 1.0, 1.0, 0.45),
                ));
            });
    } else if !open.0 && shown {
        for e in roots.iter() {
            commands.entity(e).despawn();
        }
    }
}

fn on_click(ev: On<Activate>, opts: Query<&Opt>, mut settings: ResMut<Settings>, mut open: ResMut<GraphicsMenuOpen>) {
    let Ok(o) = opts.get(ev.entity) else { return };
    let s = &mut *settings;
    let mut custom = true;
    match o {
        Opt::Preset => {
            let next = if s.quality >= 3 { 0 } else { s.quality + 1 };
            s.apply_preset(next);
            custom = false;
        }
        Opt::Textures => {
            s.texture_max = match s.texture_max {
                0..=512 => 1024,
                513..=1024 => 2048,
                _ => 512,
            }
        }
        Opt::Msaa => {
            s.msaa = match s.msaa {
                0 => 1,
                1 => 2,
                2 => 4,
                _ => 0,
            }
        }
        Opt::Shadows => s.shadows = !s.shadows,
        Opt::Bloom => s.bloom = !s.bloom,
        Opt::Film => s.film = !s.film,
        Opt::AutoExposure => s.auto_exposure = !s.auto_exposure,
        Opt::Cones => s.light_cones = !s.light_cones,
        Opt::Fov => {
            s.fov_deg = if s.fov_deg >= 80.0 { 55.0 } else { s.fov_deg + 5.0 };
            custom = false;
        }
        Opt::Vsync => {
            s.vsync = !s.vsync;
            custom = false;
        }
        Opt::Fps => {
            s.show_fps = !s.show_fps;
            custom = false;
        }
        Opt::Back => {
            open.0 = false;
            custom = false;
        }
    }
    if custom {
        s.quality = 4;
    }
    s.save();
}

fn values(settings: Res<Settings>, mut q: Query<(&OptValue, &mut Text)>) {
    if !settings.is_changed() {
        return;
    }
    for (v, mut t) in q.iter_mut() {
        let s = value(v.0, &settings);
        if t.0 != s {
            t.0 = s;
        }
    }
}

#[allow(clippy::type_complexity)]
fn visuals(mut q: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), Or<(With<Opt>, With<OpenGraphics>)>>) {
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
