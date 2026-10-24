//! Spawnt den Spieler mit Ego-Kamera (Bloom, Tonemapping, Nebel, Raumklang-Ohren).

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::auto_exposure::AutoExposure;
use bevy::post_process::bloom::Bloom;
use bevy::post_process::effect_stack::{ChromaticAberration, Vignette};
use bevy::render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection};
use bevy::prelude::*;

use super::{CameraFov, Health, Player, PlayerCamera, Stance};
use crate::base::settings::Settings;
use crate::base::AppState;
use crate::world::mapdata::MapFile;

pub fn spawn_player(mut commands: Commands, map: Res<MapFile>, settings: Res<Settings>, existing: Query<(), With<Player>>) {
    if !existing.is_empty() {
        return;
    }
    let start = map.player_start.clone();
    let fov = settings.fov_deg.to_radians();
    commands
        .spawn((
            Name::new("Spieler"),
            Player {
                yaw: start.yaw,
                ..default()
            },
            Health::default(),
            Transform::from_translation(start.pos()).with_rotation(Quat::from_rotation_y(start.yaw)),
            Visibility::default(),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn((
                Name::new("Kamera"),
                PlayerCamera,
                Camera3d::default(),
                Projection::from(PerspectiveProjection {
                    fov,
                    near: 0.01,
                    ..default()
                }),
                CameraFov { target: fov, current: fov },
                Tonemapping::TonyMcMapface,
                DistanceFog {
                    color: Color::linear_rgb(map.fog.color[0], map.fog.color[1], map.fog.color[2]),
                    falloff: FogFalloff::Linear {
                        start: map.fog.start,
                        end: map.fog.end,
                    },
                    ..default()
                },
                msaa(&settings),
                Transform::from_xyz(0.0, Stance::Stand.eye(), 0.0),
                SpatialListener::new(2.0),
            ));
        });
}

fn msaa(s: &Settings) -> Msaa {
    match s.msaa {
        0 | 1 => Msaa::Off,
        2 => Msaa::Sample2,
        _ => Msaa::Sample4,
    }
}

fn bloom() -> Bloom {
    Bloom {
        intensity: 0.12,
        ..Bloom::NATURAL
    }
}

/// Filmischer Look: etwas kühler, entsättigt, mehr Kontrast in den Mitten
fn grading(auto_exposure: bool) -> ColorGrading {
    ColorGrading {
        global: ColorGradingGlobal {
            // Mit Augen-Anpassung wird der Mittelwert auf 18 % Grau geregelt -> dunkler kompensieren
            exposure: if auto_exposure { -1.1 } else { 0.35 },
            temperature: -0.01,
            post_saturation: 0.88,
            ..default()
        },
        shadows: ColorGradingSection {
            saturation: 0.8,
            contrast: 1.05,
            ..default()
        },
        midtones: ColorGradingSection {
            saturation: 0.9,
            contrast: 1.06,
            ..default()
        },
        highlights: ColorGradingSection {
            saturation: 0.95,
            ..default()
        },
    }
}

/// Wendet die Grafik-Einstellungen auf die Kamera und das Fenster an (beim Start und bei
/// jeder Änderung im Grafik-Menü).
pub fn apply_graphics(
    mut commands: Commands,
    settings: Res<Settings>,
    cams: Query<(Entity, Ref<PlayerCamera>)>,
    mut windows: Query<&mut Window>,
) {
    let changed = settings.is_changed();
    if changed {
        for mut w in windows.iter_mut() {
            let want = if settings.vsync { bevy::window::PresentMode::AutoVsync } else { bevy::window::PresentMode::AutoNoVsync };
            if w.present_mode != want {
                w.present_mode = want;
            }
        }
    }
    for (e, cam) in cams.iter() {
        if !changed && !cam.is_added() {
            continue;
        }
        let s = &*settings;
        let mut ec = commands.entity(e);
        ec.insert(msaa(s));
        // 1 = FXAA: günstige Kantenglättung als Nachbearbeitung
        if s.msaa == 1 {
            ec.insert(bevy::anti_alias::fxaa::Fxaa::default());
        } else {
            ec.remove::<bevy::anti_alias::fxaa::Fxaa>();
        }
        if s.bloom {
            ec.insert(bloom());
        } else {
            ec.remove::<Bloom>();
        }
        if s.auto_exposure {
            // Augen-Anpassung: dunkle Bereiche werden langsam heller, helle Lampen blenden kurz
            ec.insert(AutoExposure {
                range: -3.5..=3.0,
                filter: 0.15..=0.92,
                speed_brighten: 1.6,
                speed_darken: 2.2,
                ..default()
            });
        } else {
            ec.remove::<AutoExposure>();
        }
        if s.film {
            ec.insert((
                grading(s.auto_exposure),
                Vignette {
                    intensity: 0.3,
                    radius: 0.95,
                    smoothness: 2.2,
                    ..default()
                },
                ChromaticAberration {
                    intensity: 0.0035,
                    max_samples: 6,
                    ..default()
                },
            ));
        } else {
            ec.remove::<(Vignette, ChromaticAberration)>();
            let mut g = ColorGrading::default();
            g.global.exposure = if s.auto_exposure { -1.1 } else { 0.35 };
            ec.insert(g);
        }
    }
}
