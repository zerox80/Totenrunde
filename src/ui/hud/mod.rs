//! HUD im Stil von BO2:
//!   * unten links: Rundenzahl in roter Kreide (Runde 1–5 als Strichliste), darüber Perks
//!   * unten rechts: Punkte mit aufsteigenden Änderungen, Munition, Granaten, Waffenname
//!   * unten Mitte: aktive Power-ups mit Restzeit, darüber der Hinweistext (Taste F)
//!   * roter Bildschirmrand bei Schaden, Fadenkreuz, Fernrohrbild, FPS-Anzeige (F3)

mod ammo;
mod damage;
mod points;
mod powerups;
mod prompt;
mod round;
mod scope;
mod widgets;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::text::FontSize;

use crate::base::{AppState, GameSet, InGame};
use crate::player::Player;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<ShowFps>()
            .add_systems(
                OnEnter(AppState::Playing),
                (
                    spawn_crosshair,
                    round::spawn,
                    ammo::spawn,
                    powerups::spawn,
                    prompt::spawn,
                    damage::spawn,
                    scope::spawn,
                ),
            )
            .add_systems(
                Update,
                (
                    debug_text,
                    round::update,
                    round::perk_icons,
                    points::update,
                    points::popups,
                    ammo::update,
                    powerups::update,
                    prompt::update,
                    damage::update,
                    scope::update,
                    crosshair_visibility,
                )
                    .in_set(GameSet::Ui)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

#[derive(Resource, Default)]
struct ShowFps(bool);

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct Crosshair;

/// Ein Strich des Fadenkreuzes (Richtung x, y)
#[derive(Component)]
struct CrossLine(Vec2);

fn spawn_crosshair(mut commands: Commands) {
    commands
        .spawn((
            Crosshair,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            // Nullgroßer Ankerpunkt in der Bildschirmmitte; die Striche liegen relativ dazu
            p.spawn(Node {
                width: px(0),
                height: px(0),
                ..default()
            })
            .with_children(|c| {
                for (w, h, x, y) in [(2.0, 9.0, 0.0, -10.0), (2.0, 9.0, 0.0, 10.0), (9.0, 2.0, -10.0, 0.0), (9.0, 2.0, 10.0, 0.0)] {
                    c.spawn((
                        CrossLine(Vec2::new(x, y) / 10.0),
                        BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.6), Val::Px(0.0), Val::Px(0.0), Val::Px(1.0), Val::Px(1.5)),
                        Node {
                            position_type: PositionType::Absolute,
                            width: px(w),
                            height: px(h),
                            left: Val::Px(x - w / 2.0),
                            top: Val::Px(y - h / 2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                    ));
                }
            });
        });
    commands.spawn((
        DebugText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: px(6),
            left: px(8),
            ..default()
        },
        DespawnOnExit(AppState::Playing),
    ));
}

/// Beim Zielen (Kimme und Korn) und während Aktionen kein Fadenkreuz – wie in CoD.
/// Die Striche wandern mit der Streuung (Laufen, Dauerfeuer) nach außen.
fn crosshair_visibility(
    time: Res<Time>,
    player: Single<&Player>,
    lo: Single<&crate::weapons::Loadout>,
    mut q: Query<&mut Visibility, With<Crosshair>>,
    mut lines: Query<(&CrossLine, &mut Node)>,
    mut gap: Local<f32>,
) {
    let show = player.ads < 0.3;
    let base = lo.current_slot().map(|s| crate::weapons::def(s.weapon).spread_hip).unwrap_or(2.0);
    let want = 6.0 + base * 3.0 + player.bob * 14.0 + lo.bloom * 6.0;
    *gap += (want - *gap) * (time.delta_secs() * 12.0).min(1.0);
    for (l, mut n) in lines.iter_mut() {
        let (w, h) = if l.0.x == 0.0 { (2.0, 9.0) } else { (9.0, 2.0) };
        let x = l.0.x * (*gap + 4.5);
        let y = l.0.y * (*gap + 4.5);
        n.left = Val::Px(x - w / 2.0);
        n.top = Val::Px(y - h / 2.0);
    }
    for mut v in q.iter_mut() {
        let want = if show { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}

fn debug_text(
    keys: Res<ButtonInput<KeyCode>>,
    mut show: ResMut<ShowFps>,
    mut settings: ResMut<crate::base::settings::Settings>,
    cfg: Res<crate::base::debug::DebugConfig>,
    diag: Res<DiagnosticsStore>,
    player: Query<(&Transform, &Player)>,
    mut text: Query<&mut Text, With<DebugText>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        settings.show_fps = !settings.show_fps;
    }
    show.0 = settings.show_fps;
    let fps = diag
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let Ok((tf, p)) = player.single() else { return };
    for mut t in text.iter_mut() {
        t.0 = if show.0 || cfg.active {
            format!(
                "{fps:.0} FPS  pos {:.1} {:.1} {:.1}  yaw {:.0}",
                tf.translation.x,
                tf.translation.y,
                tf.translation.z,
                p.yaw.to_degrees()
            )
        } else {
            String::new()
        };
    }
}
