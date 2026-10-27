//! Spawnt den Spieler mit Ego-Kamera (Bloom, Tonemapping, Nebel, Raumklang-Ohren).

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::curve::cubic_splines::LinearSpline;
use bevy::post_process::auto_exposure::{AutoExposure, AutoExposureCompensationCurve};
use bevy::post_process::bloom::Bloom;
use bevy::post_process::effect_stack::{ChromaticAberration, Vignette};
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection};

use super::{CameraFov, Health, Player, PlayerCamera, Stance};
use crate::base::AppState;
use crate::base::settings::Settings;
use crate::world::mapdata::MapFile;

pub fn spawn_player(
    mut commands: Commands,
    map: Res<MapFile>,
    settings: Res<Settings>,
    existing: Query<(), With<Player>>,
) {
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
            Transform::from_translation(start.pos())
                .with_rotation(Quat::from_rotation_y(start.yaw)),
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
                CameraFov {
                    target: fov,
                    current: fov,
                },
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

/// Grundbelichtung aller Grafikstufen (die Augen-Anpassung regelt nur sanft darum herum).
const BASE_EXPOSURE: f32 = 0.35;

/// Filmischer Look: etwas kühler und entsättigt. Kein Kontrast: Bevy rechnet ihn linear um
/// 0.5 herum vor der Belichtung, das setzt alle Werte unter ~0.024 auf Schwarz – nachts fast
/// das ganze Bild.
fn grading() -> ColorGrading {
    ColorGrading {
        global: ColorGradingGlobal {
            exposure: BASE_EXPOSURE,
            temperature: -0.01,
            post_saturation: 0.88,
            ..default()
        },
        shadows: ColorGradingSection {
            saturation: 0.8,
            ..default()
        },
        midtones: ColorGradingSection {
            saturation: 0.9,
            ..default()
        },
        highlights: ColorGradingSection {
            saturation: 0.95,
            ..default()
        },
    }
}

/// Messbereich der Augen-Anpassung (log2 der Leuchtdichte). Bevy wirft alles darunter aus der
/// Messung; die Untergrenze muss daher tief liegen, sonst misst es nachts nur die Lampen.
const AE_RANGE: (f32, f32) = (-10.0, 2.0);

/// Augen-Anpassung als Zusatz zur Grundbelichtung: dunkle Szenen werden etwas aufgehellt,
/// helle etwas abgedunkelt, höchstens um eine Blende. Ohne diese Kurve regelt Bevy voll auf
/// Mittelgrau; nachts sähe alles grau und flau aus.
fn exposure_curve() -> AutoExposureCompensationCurve {
    // mittlere Helligkeit, bei der die Grundbelichtung unverändert bleibt (zwischen beleuchteter
    // Wartehalle und dunklem Hof). Werte bewusst exakt darstellbar: Bevy vergleicht die
    // Stützpunkte der Kurve auf exakte Gleichheit.
    const REFERENCE: f32 = -2.5;
    const STRENGTH: f32 = 0.25;
    const MAX_EV: f32 = 1.0;
    // Bevy rechnet: Belichtung = Kurve(Helligkeit) - Helligkeit
    let target = |x: f32| (STRENGTH * (REFERENCE - x)).clamp(-MAX_EV, MAX_EV);
    let points = [
        AE_RANGE.0,
        REFERENCE - MAX_EV / STRENGTH,
        REFERENCE + MAX_EV / STRENGTH,
        AE_RANGE.1,
    ]
    .map(|x| Vec2::new(x, target(x) + x));
    AutoExposureCompensationCurve::from_curve(LinearSpline::new(points)).unwrap_or_else(|e| {
        warn!("Belichtungskurve ungültig ({e}), Augen-Anpassung ohne Kurve");
        default()
    })
}

/// Wendet die Grafik-Einstellungen auf die Kamera und das Fenster an (beim Start und bei
/// jeder Änderung im Grafik-Menü).
pub fn apply_graphics(
    mut commands: Commands,
    settings: Res<Settings>,
    cams: Query<(Entity, Ref<PlayerCamera>)>,
    mut windows: Query<&mut Window>,
    #[cfg(not(target_arch = "wasm32"))] mut curves: ResMut<Assets<AutoExposureCompensationCurve>>,
    #[cfg(not(target_arch = "wasm32"))] mut curve: Local<
        Option<Handle<AutoExposureCompensationCurve>>,
    >,
) {
    let changed = settings.is_changed();
    if changed {
        for mut w in windows.iter_mut() {
            let want = if settings.vsync {
                bevy::window::PresentMode::AutoVsync
            } else {
                bevy::window::PresentMode::AutoNoVsync
            };
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
        #[cfg(not(target_arch = "wasm32"))]
        if s.auto_exposure {
            // Augen-Anpassung: dunkle Bereiche werden langsam heller, helle Lampen blenden kurz
            let curve = curve
                .get_or_insert_with(|| curves.add(exposure_curve()))
                .clone();
            ec.insert(AutoExposure {
                range: AE_RANGE.0..=AE_RANGE.1,
                filter: 0.15..=0.92,
                speed_brighten: 1.6,
                speed_darken: 2.2,
                compensation_curve: curve,
                ..default()
            });
        } else {
            ec.remove::<AutoExposure>();
        }
        if s.film {
            ec.insert((
                grading(),
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
            g.global.exposure = BASE_EXPOSURE;
            ec.insert(g);
        }
    }
}
