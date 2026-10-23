//! Übersetzt FxEvents (Blut, Kopfschuss, Einschlag, Explosion, ...) in Partikel und Decals.

use bevy::prelude::*;
use rand::Rng;

use super::decals::{spawn_decal, Decal};
use super::particles::{rand_dir, spawn_particle, Flash, Particle};
use super::{FxAssets, FxEvent};
use crate::base::AppState;
use crate::world::collision::{CollisionWorld, F_BULLET};

#[allow(clippy::too_many_arguments)]
pub fn handle_fx(
    mut commands: Commands,
    mut events: MessageReader<FxEvent>,
    fx: Res<FxAssets>,
    col: Res<CollisionWorld>,
    decals: Query<Entity, With<Decal>>,
) {
    let mut rng = rand::rng();
    let mut decal_count = decals.iter().count();
    for ev in events.read() {
        match *ev {
            FxEvent::Blood { pos, dir, amount } => {
                let n = (6.0 * amount) as usize;
                for _ in 0..n {
                    let v = (dir * rng.random_range(1.0..3.5) + rand_dir(&mut rng) * 1.8 + Vec3::Y * 0.8) * amount.sqrt();
                    spawn_particle(
                        &mut commands,
                        &fx.cube,
                        &fx.blood_drop,
                        pos,
                        Particle {
                            vel: v,
                            life: 0.0,
                            max_life: rng.random_range(0.4..0.9),
                            gravity: 9.0,
                            size0: rng.random_range(0.018..0.04),
                            size1: 0.01,
                            billboard: false,
                            drag: 0.5,
                            spin: 0.0,
                        },
                    );
                }
                // Blutnebel
                for _ in 0..(2.0 * amount) as usize {
                    spawn_particle(
                        &mut commands,
                        &fx.quad,
                        &fx.blood,
                        pos + rand_dir(&mut rng) * 0.05,
                        Particle {
                            vel: dir * 0.8 + rand_dir(&mut rng) * 0.4,
                            life: 0.0,
                            max_life: 0.35,
                            gravity: 0.5,
                            size0: 0.12 * amount,
                            size1: 0.45 * amount,
                            billboard: true,
                            drag: 3.0,
                            spin: rng.random_range(-3.0..3.0),
                        },
                    );
                }
                // Blutfleck auf dem Boden / der Wand dahinter
                if decal_count < 60 {
                    let o = pos;
                    let d = (dir + Vec3::NEG_Y * 0.7).normalize();
                    if let Some(hit) = col.raycast(o, d, 3.0, F_BULLET) {
                        spawn_decal(&mut commands, &fx, &fx.decal_blood, hit.point, hit.normal, rng.random_range(0.35..0.8), 40.0);
                        decal_count += 1;
                    } else if pos.y < 3.0 {
                        let ground = Vec3::new(pos.x + dir.x * 0.8, 0.0, pos.z + dir.z * 0.8);
                        spawn_decal(&mut commands, &fx, &fx.decal_blood, ground, Vec3::Y, rng.random_range(0.4..0.9), 40.0);
                        decal_count += 1;
                    }
                }
            }
            FxEvent::Gib { pos } => {
                for _ in 0..14 {
                    let v = rand_dir(&mut rng) * rng.random_range(1.0..3.5) + Vec3::Y * 2.5;
                    spawn_particle(
                        &mut commands,
                        &fx.cube,
                        &fx.blood_drop,
                        pos,
                        Particle {
                            vel: v,
                            life: 0.0,
                            max_life: rng.random_range(0.5..1.1),
                            gravity: 9.0,
                            size0: rng.random_range(0.025..0.06),
                            size1: 0.02,
                            billboard: false,
                            drag: 0.3,
                            spin: 0.0,
                        },
                    );
                }
                for _ in 0..4 {
                    spawn_particle(
                        &mut commands,
                        &fx.quad,
                        &fx.blood,
                        pos,
                        Particle {
                            vel: rand_dir(&mut rng) * 0.6 + Vec3::Y * 0.5,
                            life: 0.0,
                            max_life: 0.5,
                            gravity: 0.3,
                            size0: 0.25,
                            size1: 0.9,
                            billboard: true,
                            drag: 2.5,
                            spin: rng.random_range(-2.0..2.0),
                        },
                    );
                }
            }
            FxEvent::Impact { pos, normal } => {
                for _ in 0..4 {
                    spawn_particle(
                        &mut commands,
                        &fx.quad,
                        &fx.spark,
                        pos + normal * 0.02,
                        Particle {
                            vel: (normal + rand_dir(&mut rng) * 0.8).normalize() * rng.random_range(2.0..5.0),
                            life: 0.0,
                            max_life: rng.random_range(0.08..0.2),
                            gravity: 6.0,
                            size0: 0.05,
                            size1: 0.02,
                            billboard: true,
                            drag: 1.0,
                            spin: 0.0,
                        },
                    );
                }
                spawn_particle(
                    &mut commands,
                    &fx.quad,
                    &fx.dust,
                    pos + normal * 0.05,
                    Particle {
                        vel: normal * 0.5,
                        life: 0.0,
                        max_life: 0.6,
                        gravity: -0.2,
                        size0: 0.08,
                        size1: 0.4,
                        billboard: true,
                        drag: 2.0,
                        spin: rng.random_range(-1.0..1.0),
                    },
                );
                if decal_count < 60 {
                    spawn_decal(&mut commands, &fx, &fx.decal_hole, pos, normal, rng.random_range(0.04..0.07), 15.0);
                    decal_count += 1;
                }
            }
            FxEvent::Explosion { pos, radius } => {
                commands.spawn((
                    PointLight {
                        color: Color::srgb(1.0, 0.6, 0.3),
                        intensity: 4.0e6,
                        range: radius * 4.0,
                        ..default()
                    },
                    Transform::from_translation(pos + Vec3::Y * 0.5),
                    Flash {
                        life: 0.0,
                        max: 0.35,
                        intensity: 4.0e6,
                    },
                    DespawnOnExit(AppState::Playing),
                ));
                for i in 0..10 {
                    spawn_particle(
                        &mut commands,
                        &fx.quad,
                        if i < 5 { &fx.fire } else { &fx.smoke },
                        pos + rand_dir(&mut rng) * 0.3,
                        Particle {
                            vel: rand_dir(&mut rng) * radius * 0.6 + Vec3::Y * 1.5,
                            life: 0.0,
                            max_life: if i < 5 { 0.35 } else { 1.6 },
                            gravity: -0.6,
                            size0: radius * 0.25,
                            size1: radius * if i < 5 { 0.8 } else { 1.0 },
                            billboard: true,
                            drag: 2.5,
                            spin: rng.random_range(-1.5..1.5),
                        },
                    );
                }
                for _ in 0..12 {
                    spawn_particle(
                        &mut commands,
                        &fx.quad,
                        &fx.spark,
                        pos,
                        Particle {
                            vel: rand_dir(&mut rng) * rng.random_range(4.0..10.0) + Vec3::Y * 3.0,
                            life: 0.0,
                            max_life: rng.random_range(0.3..0.7),
                            gravity: 9.0,
                            size0: 0.08,
                            size1: 0.03,
                            billboard: true,
                            drag: 0.5,
                            spin: 0.0,
                        },
                    );
                }
                if decal_count < 60 {
                    let g = Vec3::new(pos.x, 0.0, pos.z);
                    spawn_decal(&mut commands, &fx, &fx.decal_hole, g, Vec3::Y, radius * 0.5, 30.0);
                    decal_count += 1;
                }
            }
            FxEvent::RayImpact { pos, upgraded } => {
                commands.spawn((
                    PointLight {
                        color: if upgraded { Color::srgb(1.0, 0.3, 0.3) } else { Color::srgb(0.3, 1.0, 0.3) },
                        intensity: 1.5e6,
                        range: 8.0,
                        ..default()
                    },
                    Transform::from_translation(pos),
                    Flash {
                        life: 0.0,
                        max: 0.25,
                        intensity: 1.5e6,
                    },
                    DespawnOnExit(AppState::Playing),
                ));
                for _ in 0..8 {
                    spawn_particle(
                        &mut commands,
                        &fx.quad,
                        if upgraded { &fx.glow_red } else { &fx.glow_green },
                        pos,
                        Particle {
                            vel: rand_dir(&mut rng) * rng.random_range(1.0..4.0),
                            life: 0.0,
                            max_life: rng.random_range(0.2..0.5),
                            gravity: 0.0,
                            size0: 0.25,
                            size1: 0.05,
                            billboard: true,
                            drag: 3.0,
                            spin: 0.0,
                        },
                    );
                }
            }
            FxEvent::Dirt { pos } => {
                for _ in 0..6 {
                    spawn_particle(
                        &mut commands,
                        &fx.cube,
                        &fx.dirt,
                        pos + Vec3::new(rng.random_range(-0.3..0.3), 0.05, rng.random_range(-0.3..0.3)),
                        Particle {
                            vel: Vec3::new(rng.random_range(-1.0..1.0), rng.random_range(2.0..3.5), rng.random_range(-1.0..1.0)),
                            life: 0.0,
                            max_life: 0.7,
                            gravity: 9.0,
                            size0: rng.random_range(0.04..0.09),
                            size1: 0.03,
                            billboard: false,
                            drag: 0.3,
                            spin: 0.0,
                        },
                    );
                }
                spawn_particle(
                    &mut commands,
                    &fx.quad,
                    &fx.dust,
                    pos + Vec3::Y * 0.2,
                    Particle {
                        vel: Vec3::Y * 0.4,
                        life: 0.0,
                        max_life: 1.4,
                        gravity: 0.0,
                        size0: 0.5,
                        size1: 1.5,
                        billboard: true,
                        drag: 1.0,
                        spin: rng.random_range(-0.5..0.5),
                    },
                );
            }
            FxEvent::Sparks { pos } => {
                for _ in 0..10 {
                    spawn_particle(
                        &mut commands,
                        &fx.quad,
                        &fx.spark,
                        pos,
                        Particle {
                            vel: rand_dir(&mut rng) * rng.random_range(1.5..4.0) + Vec3::Y,
                            life: 0.0,
                            max_life: rng.random_range(0.2..0.6),
                            gravity: 8.0,
                            size0: 0.05,
                            size1: 0.02,
                            billboard: true,
                            drag: 0.5,
                            spin: 0.0,
                        },
                    );
                }
            }
        }
    }
}
