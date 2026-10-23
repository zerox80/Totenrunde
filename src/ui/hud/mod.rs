//! HUD im Stil von BO2:
//!   * unten links: Rundenzahl in roter Kreide (Runde 1–5 als Strichliste), darüber Perks
//!   * unten rechts: Punkte mit aufsteigenden Änderungen, Munition, Granaten, Waffenname
//!   * unten Mitte: aktive Power-ups mit Restzeit, darüber der Hinweistext (Taste F)
//!   * roter Bildschirmrand bei Schaden, Fadenkreuz, FPS-Anzeige (F3)

mod ammo;
mod damage;
mod points;
mod powerups;
mod prompt;
mod round;
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
                    points::spawn,
                    ammo::spawn,
                    powerups::spawn,
                    prompt::spawn,
                    damage::spawn,
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
                for (w, h, x, y) in [(2.0, 8.0, 0.0, -10.0), (2.0, 8.0, 0.0, 10.0), (8.0, 2.0, -10.0, 0.0), (8.0, 2.0, 10.0, 0.0)] {
                    c.spawn((
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
fn crosshair_visibility(player: Single<&Player>, mut q: Query<&mut Visibility, With<Crosshair>>) {
    let show = player.ads < 0.3;
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
    cfg: Res<crate::base::debug::DebugConfig>,
    diag: Res<DiagnosticsStore>,
    player: Query<(&Transform, &Player)>,
    mut text: Query<&mut Text, With<DebugText>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        show.0 = !show.0;
    }
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
