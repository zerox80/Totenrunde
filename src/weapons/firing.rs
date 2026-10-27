//! Schießen: Hitscan mit Streuung, Durchschlag, Schadensabfall und Trefferzonen,
//! Projektilwaffen, Rückstoß, Leuchtspuren und automatisches Nachladen.

use std::f32::consts::PI;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use rand::Rng;

use super::assets::WeaponAssets;
use super::defs::{FireMode, Projectile, WeaponClass, def};
use super::input::start_reload;
use super::loadout::{Loadout, WState};
use super::projectiles::{ProjectileBody, Tracer, explode};
use crate::audio::{Sfx, SfxEvent};
use crate::base::{AppState, InGame};
use crate::fx::FxEvent;
use crate::game::perks::{Perk, Perks};
use crate::player::{Player, PlayerCamera, PlayerDamage};
use crate::world::collision::{CollisionWorld, F_BULLET};
use crate::zombies::{DamageKind, HitPart, Zombie, ZombieHit, ZombieRig, raycast_zombies};

pub fn spread_dir(
    forward: Vec3,
    up: Vec3,
    right: Vec3,
    spread_deg: f32,
    rng: &mut impl Rng,
) -> Vec3 {
    if spread_deg <= 0.0 {
        return forward;
    }
    let r = (spread_deg.to_radians()) * rng.random::<f32>().sqrt();
    let a = rng.random_range(0.0..(2.0 * PI));
    (forward + (right * a.cos() + up * a.sin()) * r.tan()).normalize()
}

#[allow(clippy::too_many_arguments)]
pub fn fire_weapons(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    ingame: Res<InGame>,
    perks: Res<Perks>,
    col: Res<CollisionWorld>,
    wa: Res<WeaponAssets>,
    mut q: Single<(&mut Loadout, &mut Player, &Transform)>,
    cam: Single<&GlobalTransform, With<PlayerCamera>>,
    zombies: Query<(Entity, &Zombie, &Transform, Option<&ZombieRig>)>,
    bones: Query<&GlobalTransform>,
    mut hits: MessageWriter<ZombieHit>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxEvent>,
    mut dmg: MessageWriter<PlayerDamage>,
) {
    if ingame.paused {
        return;
    }
    let (ref mut lo, ref mut pl, ptf) = *q;
    let dt = time.delta_secs();
    let cam_tf = *cam;
    let origin = cam_tf.translation();
    let forward = cam_tf.forward().as_vec3();
    let up = cam_tf.up().as_vec3();
    let right = cam_tf.right().as_vec3();
    let mut rng = rand::rng();
    lo.flash = (lo.flash - dt).max(0.0);
    lo.kick = (lo.kick - dt * 6.0).max(0.0);

    // --- Schießen ---
    let Some(slot) = lo.current_slot().copied() else {
        return;
    };
    let d = def(slot.weapon);
    let can_fire = matches!(lo.state, WState::Ready | WState::ShellReload { .. })
        && lo.sprint_out <= 0.0
        && !pl.sprinting;
    let trigger = match d.fire {
        FireMode::Auto => mouse.pressed(MouseButton::Left),
        FireMode::Semi => mouse.just_pressed(MouseButton::Left),
        FireMode::Burst(n) => {
            if mouse.just_pressed(MouseButton::Left) && lo.burst_left == 0 && lo.fire_cd <= 0.0 {
                lo.burst_left = n;
            }
            lo.burst_left > 0
        }
    };
    if !trigger || !can_fire || lo.fire_cd > 0.0 {
        return;
    }
    if slot.mag == 0 {
        lo.burst_left = 0;
        if mouse.just_pressed(MouseButton::Left) {
            sfx.write(SfxEvent::ui(Sfx::DryFire));
        }
        if slot.reserve > 0 {
            start_reload(lo, perks.has(Perk::SpeedCola), &mut sfx);
        }
        return;
    }
    // Schrotflinten-Nachladen unterbrechen
    if matches!(lo.state, WState::ShellReload { .. }) {
        lo.state = WState::Ready;
    }
    let double_tap = perks.has(Perk::DoubleTap);
    let rpm = d.rpm
        * if slot.upgraded { d.pap.rpm_mult } else { 1.0 }
        * if double_tap { 1.33 } else { 1.0 };
    lo.fire_cd = 60.0 / rpm;
    if let FireMode::Burst(_) = d.fire {
        lo.burst_left -= 1;
        if lo.burst_left == 0 {
            lo.fire_cd = 0.32;
        }
    }
    if let Some(s) = lo.current_slot_mut() {
        s.mag -= 1;
    }
    lo.shots += 1;
    lo.flash = 0.05;
    lo.kick = 1.0;

    let dmg_mult = (if slot.upgraded {
        d.pap.damage_mult
    } else {
        1.0
    }) * if double_tap { 2.0 } else { 1.0 };
    let crouch_bonus = match pl.stance {
        crate::player::Stance::Stand => 1.0,
        crate::player::Stance::Crouch => 0.75,
        crate::player::Stance::Prone => 0.55,
    };
    let move_pen = if pl.moving { 1.25 } else { 1.0 };
    let spread = (d.spread_hip * (1.0 - lo.ads) + d.spread_ads * lo.ads) * crouch_bonus * move_pen
        + lo.bloom * (1.0 - lo.ads * 0.8);
    lo.bloom = (lo.bloom + d.recoil * 0.25).min(3.0);

    let splash = if slot.upgraded && d.pap.splash > 0.0 {
        d.pap.splash
    } else {
        d.splash
    };
    let splash_damage = if slot.upgraded && d.pap.splash > 0.0 {
        d.pap.splash_damage
    } else {
        d.splash_damage
            * if slot.upgraded {
                d.pap.damage_mult
            } else {
                1.0
            }
    };

    let sound_pitch = if slot.upgraded { 0.92 } else { 1.0 };
    sfx.write(SfxEvent::ui(d.sound).pitch(sound_pitch));

    match d.projectile {
        Projectile::Hitscan => {
            for _ in 0..d.pellets {
                let dir = spread_dir(forward, up, right, spread, &mut rng);
                let world = col.raycast(origin, dir, 250.0, F_BULLET);
                let max_t = world.map(|h| h.t).unwrap_or(250.0);
                let zh = raycast_zombies(origin, dir, max_t, &zombies, &bones);
                let mut pen_mult = 1.0;
                let max_hits = 1 + d.penetration as usize + if slot.upgraded { 1 } else { 0 };
                let mut splash_at: Option<Vec3> = None;
                for (i, h) in zh.iter().take(max_hits).enumerate() {
                    let falloff = if h.t > d.range {
                        (1.0 - (h.t - d.range) / d.range * 0.5).max(0.5)
                    } else {
                        1.0
                    };
                    let part_mult = match h.part {
                        HitPart::Head => d.head_mult,
                        HitPart::Neck => 2.0,
                        HitPart::Torso => 1.0,
                        HitPart::Limb => 0.8,
                    };
                    let point = origin + dir * h.t;
                    hits.write(ZombieHit {
                        zombie: h.zombie,
                        damage: d.damage * dmg_mult * part_mult * falloff * pen_mult,
                        part: h.part,
                        kind: DamageKind::Bullet,
                        point,
                        dir,
                    });
                    sfx.write(
                        SfxEvent::at(
                            if h.part == HitPart::Head {
                                Sfx::Headshot
                            } else {
                                Sfx::HitFlesh
                            },
                            point,
                        )
                        .vol(0.8),
                    );
                    pen_mult *= 0.7;
                    if i == 0 && splash > 0.0 {
                        splash_at = Some(point);
                    }
                }
                if zh.len() < max_hits {
                    if let Some(w) = world {
                        fx.write(FxEvent::Impact {
                            pos: w.point,
                            normal: w.normal,
                        });
                        if splash > 0.0 && splash_at.is_none() {
                            splash_at = Some(w.point + w.normal * 0.1);
                        }
                    }
                }
                if let Some(p) = splash_at {
                    explode(
                        p,
                        splash,
                        splash_damage,
                        &zombies,
                        &col,
                        ptf.translation,
                        &mut hits,
                        &mut fx,
                        &mut sfx,
                        &mut dmg,
                        false,
                    );
                }
                // Leuchtspur (jede dritte Kugel)
                if lo.shots % 3 == 0 && d.pellets == 1 {
                    let end = origin + dir * max_t.min(60.0);
                    let start = origin + forward * 0.8 + right * 0.08 - up * 0.08;
                    let mid = (start + end) / 2.0;
                    let len = (end - start).length();
                    commands.spawn((
                        Mesh3d(wa.bolt_mesh.clone()),
                        MeshMaterial3d(wa.tracer_mat.clone()),
                        Transform::from_translation(mid)
                            .looking_to(end - start, Vec3::Y)
                            .with_scale(Vec3::new(0.12, 0.12, len)),
                        Tracer { life: 0.04 },
                        NotShadowCaster,
                        DespawnOnExit(AppState::Playing),
                    ));
                }
            }
        }
        Projectile::RayBolt => {
            let dir = spread_dir(forward, up, right, spread, &mut rng);
            commands.spawn((
                ProjectileBody {
                    kind: Projectile::RayBolt,
                    vel: dir * 55.0,
                    age: 0.0,
                    fuse: 3.0,
                    damage: d.damage * dmg_mult,
                    splash,
                    splash_damage,
                    upgraded: slot.upgraded,
                    thrown: false,
                },
                Mesh3d(wa.bolt_mesh.clone()),
                MeshMaterial3d(if slot.upgraded {
                    wa.bolt_red.clone()
                } else {
                    wa.bolt_green.clone()
                }),
                Transform::from_translation(origin + dir * 0.6 + right * 0.05 - up * 0.05)
                    .looking_to(dir, Vec3::Y)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, dir)),
                NotShadowCaster,
                PointLight {
                    color: if slot.upgraded {
                        Color::srgb(1.0, 0.3, 0.3)
                    } else {
                        Color::srgb(0.3, 1.0, 0.3)
                    },
                    intensity: 300000.0,
                    range: 6.0,
                    ..default()
                },
                DespawnOnExit(AppState::Playing),
            ));
        }
        Projectile::Grenade => {
            let dir = spread_dir(forward, up, right, spread, &mut rng);
            commands.spawn((
                ProjectileBody {
                    kind: Projectile::Grenade,
                    vel: (dir + up * 0.06) * 32.0,
                    age: 0.0,
                    fuse: 6.0,
                    damage: d.damage * dmg_mult,
                    splash,
                    splash_damage,
                    upgraded: slot.upgraded,
                    thrown: false,
                },
                Mesh3d(wa.grenade_mesh.clone()),
                MeshMaterial3d(wa.grenade_mat.clone()),
                Transform::from_translation(origin + dir * 0.5),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }

    // Rückstoß
    let side = rng.random_range(-1.0..1.0) * d.recoil_side;
    let ads_k = 1.0 - lo.ads * 0.35;
    pl.recoil_vel += Vec2::new(
        d.recoil.to_radians() * 14.0 * ads_k,
        side.to_radians() * 14.0 * ads_k,
    );
    if d.class == WeaponClass::Sniper || d.class == WeaponClass::Shotgun {
        pl.shake = (pl.shake + 0.3).min(1.0);
    }
    // Automatisch nachladen, wenn leer
    if let Some(s) = lo.current_slot() {
        if s.mag == 0 && s.reserve > 0 && lo.burst_left == 0 {
            let sc = perks.has(Perk::SpeedCola);
            start_reload(lo, sc, &mut sfx);
        }
    }
}
