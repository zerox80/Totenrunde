//! Einstellungen: Grafikstufen NIEDRIG / MITTEL / HOCH / EXTREM, einzelne Optionen und die
//! Sprache. Änderungen wirken sofort (bis auf die Texturqualität, die beim nächsten Start
//! greift) und werden in settings.json bzw. im Browser gespeichert.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};

use super::widgets::{BLOOD, HOVER, IDLE, PRESS, text};
use crate::base::assets::GameAssets;
use crate::base::i18n::{self, LANGUAGES, Translated, t};
use crate::base::settings::{GraphicsMenuOpen, QUALITY_NAMES, Settings};

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
    #[cfg(target_arch = "wasm32")]
    RenderScale,
    Msaa,
    Shadows,
    Bloom,
    Film,
    AutoExposure,
    Cones,
    Fov,
    Vsync,
    Fps,
    Language,
    Back,
}

/// Text mit dem aktuellen Wert einer Zeile
#[derive(Component)]
struct OptValue(Opt);

#[derive(Component)]
struct OpenGraphics;

/// Knopf "EINSTELLUNGEN" für Haupt- und Pausenmenü.
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
            b.spawn((
                text(t("menu.settings"), font, 24.0, Color::srgb(0.92, 0.9, 0.86)),
                Translated("menu.settings"),
            ));
        })
        .observe(|_: On<Activate>, mut open: ResMut<GraphicsMenuOpen>| open.0 = true);
}

/// Übersetzungsschlüssel der Zeilenbeschriftung
fn label(o: Opt) -> &'static str {
    match o {
        Opt::Preset => "settings.preset",
        Opt::Textures => "settings.textures",
        #[cfg(target_arch = "wasm32")]
        Opt::RenderScale => "settings.render_scale",
        Opt::Msaa => "settings.msaa",
        Opt::Shadows => "settings.shadows",
        Opt::Bloom => "settings.bloom",
        Opt::Film => "settings.film",
        Opt::AutoExposure => "settings.auto_exposure",
        Opt::Cones => "settings.cones",
        Opt::Fov => "settings.fov",
        Opt::Vsync => "settings.vsync",
        Opt::Fps => "settings.fps",
        Opt::Language => "settings.language",
        Opt::Back => "settings.back",
    }
}

fn onoff(b: bool) -> String {
    t(if b { "settings.on" } else { "settings.off" }).into()
}

fn value(o: Opt, s: &Settings) -> String {
    match o {
        Opt::Preset => t(QUALITY_NAMES[s.quality.min(4) as usize]).into(),
        Opt::Textures => match s.texture_max {
            0..=512 => t("settings.tex_low").into(),
            513..=1024 => t("settings.tex_mid").into(),
            _ => t("settings.tex_high").into(),
        },
        #[cfg(target_arch = "wasm32")]
        Opt::RenderScale => format!("{:.0} %", s.render_scale * 100.0),
        Opt::Msaa => match s.msaa {
            0 => t("settings.off").into(),
            1 => "FXAA".into(),
            2 if !cfg!(feature = "webgpu") => "MSAA 2×".into(),
            _ => "MSAA 4×".into(),
        },
        Opt::Shadows => onoff(s.shadows),
        Opt::Bloom => onoff(s.bloom),
        Opt::Film => onoff(s.film),
        Opt::AutoExposure => {
            if cfg!(feature = "webgpu") {
                t("settings.not_browser").into()
            } else if cfg!(target_arch = "wasm32") {
                t("settings.not_webgl2").into()
            } else {
                onoff(s.auto_exposure)
            }
        }
        Opt::Cones => onoff(s.light_cones),
        Opt::Fov => format!("{:.0}°", s.fov_deg),
        Opt::Vsync => onoff(s.vsync),
        Opt::Fps => onoff(s.show_fps),
        Opt::Language if s.language == "auto" => {
            format!("{} ({})", t("settings.auto"), i18n::current().name)
        }
        Opt::Language => i18n::current().name.into(),
        Opt::Back => String::new(),
    }
}

const ROWS: [Opt; 13 + cfg!(target_arch = "wasm32") as usize] = [
    Opt::Preset,
    Opt::Textures,
    #[cfg(target_arch = "wasm32")]
    Opt::RenderScale,
    Opt::Msaa,
    Opt::Shadows,
    Opt::Bloom,
    Opt::Film,
    Opt::AutoExposure,
    Opt::Cones,
    Opt::Fov,
    Opt::Vsync,
    Opt::Fps,
    Opt::Language,
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
                    text(t("settings.title"), &assets.font_black, 64.0, BLOOD),
                    Translated("settings.title"),
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
                            height: px(if back { 50.0 } else { 38.0 }),
                            margin: UiRect::vertical(px(if back { 14.0 } else { 2.0 })),
                            padding: UiRect::horizontal(px(18)),
                            justify_content: if back {
                                JustifyContent::Center
                            } else {
                                JustifyContent::SpaceBetween
                            },
                            align_items: AlignItems::Center,
                            border: UiRect::left(px(if o == Opt::Preset { 6.0 } else { 3.0 })),
                            ..default()
                        },
                        BorderColor::all(BLOOD),
                        BackgroundColor(IDLE),
                    ))
                    .with_children(|r| {
                        r.spawn((
                            text(
                                t(label(o)),
                                &assets.font_bold,
                                if o == Opt::Preset { 22.0 } else { 19.0 },
                                Color::srgb(0.92, 0.9, 0.86),
                            ),
                            Translated(label(o)),
                        ));
                        if !back {
                            r.spawn((
                                OptValue(o),
                                text(
                                    value(o, &settings),
                                    &assets.font_black,
                                    19.0,
                                    Color::srgb(1.0, 0.82, 0.55),
                                ),
                            ));
                        }
                    })
                    .observe(on_click);
                }
                p.spawn((
                    Translated("settings.footer"),
                    text(
                        t("settings.footer"),
                        &assets.font_regular,
                        15.0,
                        Color::srgba(1.0, 1.0, 1.0, 0.45),
                    ),
                ));
            });
    } else if !open.0 && shown {
        for e in roots.iter() {
            commands.entity(e).despawn();
        }
    }
}

fn on_click(
    ev: On<Activate>,
    opts: Query<&Opt>,
    mut settings: ResMut<Settings>,
    mut open: ResMut<GraphicsMenuOpen>,
) {
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
        #[cfg(target_arch = "wasm32")]
        Opt::RenderScale => {
            s.render_scale = match s.render_scale {
                x if x < 0.6 => 0.75,
                x if x < 0.9 => 1.0,
                _ => 0.5,
            };
        }
        Opt::Msaa => {
            s.msaa = match s.msaa {
                0 => 1,
                // WebGPU kennt kein MSAA 2×
                1 if cfg!(feature = "webgpu") => 4,
                1 => 2,
                2 => 4,
                _ => 0,
            }
        }
        Opt::Shadows => s.shadows = !s.shadows,
        Opt::Bloom => s.bloom = !s.bloom,
        Opt::Film => s.film = !s.film,
        Opt::AutoExposure => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                s.auto_exposure = !s.auto_exposure;
            }
            custom = false;
        }
        Opt::Cones => s.light_cones = !s.light_cones,
        Opt::Fov => {
            s.fov_deg = if s.fov_deg >= 80.0 {
                55.0
            } else {
                s.fov_deg + 5.0
            };
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
        Opt::Language => {
            // AUTOMATISCH → alle Sprachen der Reihe nach → wieder AUTOMATISCH
            let next = match LANGUAGES.iter().position(|l| l.code == s.language) {
                None => 0,
                Some(i) if i + 1 < LANGUAGES.len() => i + 1,
                Some(_) => usize::MAX,
            };
            s.language = LANGUAGES
                .get(next)
                .map_or_else(|| "auto".into(), |l| l.code.into());
            i18n::set(&s.language);
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
fn visuals(
    mut q: Query<
        (&Hovered, Has<Pressed>, &mut BackgroundColor),
        Or<(With<Opt>, With<OpenGraphics>)>,
    >,
) {
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
