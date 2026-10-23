//! Nahkampf mit dem Messer (150 Schaden, +130 Punkte beim Kill) und Granatenwurf.

use bevy::prelude::*;

use super::assets::WeaponAssets;
use super::defs::Projectile;
use super::loadout::{Loadout, WState, KNIFE_DAMAGE};
use super::projectiles::ProjectileBody;
use crate::audio::{Sfx, SfxEvent};
use crate::base::{AppState, InGame};
use crate::player::{Player, PlayerCamera};
use crate::zombies::{DamageKind, HitPart, Zombie, ZombieHit, ZombieRig};

#[allow(clippy::too_many_arguments)]
pub fn melee_and_throw(
    mut commands: Commands,
    time: Res<Time>,
    ingame: Res<InGame>,
    wa: Res<WeaponAssets>,
    mut q: Single<(&mut Loadout, &mut Player, &Transform)>,
    cam: Single<&GlobalTransform, With<PlayerCamera>>,
    zombies: Query<(Entity, &Zombie, &Transform, Option<&ZombieRig>)>,
    mut hits: MessageWriter<ZombieHit>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    if ingame.paused {
        return;
    }
    let (ref mut lo, ref mut pl, _ptf) = *q;
    let dt = time.delta_secs();
    let cam_tf = *cam;
    let origin = cam_tf.translation();
    let forward = cam_tf.forward().as_vec3();
    let up = cam_tf.up().as_vec3();
    let right = cam_tf.right().as_vec3();

    // --- Messer ---
    if let WState::Melee { t, hit } = lo.state {
        let t = t + dt;
        let mut hit = hit;
        if !hit && t >= 0.12 {
            hit = true;
            let mut best: Option<(Entity, f32, Vec3)> = None;
            for (e, z, tf, _) in zombies.iter() {
                if !z.alive() {
                    continue;
                }
                let c = tf.translation + Vec3::Y * 1.1;
                let d = c - origin;
                let dist = Vec3::new(d.x, 0.0, d.z).length();
                if dist > 1.9 || d.y.abs() > 1.6 {
                    continue;
                }
                let fwd_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
                if fwd_flat.dot(Vec3::new(d.x, 0.0, d.z).normalize_or_zero()) < 0.55 {
                    continue;
                }
                if best.is_none_or(|(_, bd, _)| dist < bd) {
                    best = Some((e, dist, c));
                }
            }
            if let Some((e, _, c)) = best {
                hits.write(ZombieHit {
                    zombie: e,
                    damage: KNIFE_DAMAGE,
                    part: HitPart::Torso,
                    kind: DamageKind::Melee,
                    point: c,
                    dir: forward,
                });
                sfx.write(SfxEvent::at(Sfx::KnifeHit, c));
            }
        }
        lo.state = if t >= 0.55 {
            if lo.slots.is_empty() {
                WState::Empty
            } else {
                WState::Ready
            }
        } else {
            WState::Melee { t, hit }
        };
        return;
    }

    // --- Granate werfen ---
    if let WState::Throw { t, thrown } = lo.state {
        let t = t + dt;
        let mut thrown = thrown;
        if !thrown && t >= 0.28 {
            thrown = true;
            lo.grenades = lo.grenades.saturating_sub(1);
            let vel = (forward + up * 0.25).normalize() * 15.0 + Vec3::new(pl.vel.x, 0.0, pl.vel.z) * 0.5;
            commands.spawn((
                ProjectileBody {
                    kind: Projectile::Grenade,
                    vel,
                    age: 0.0,
                    fuse: 2.6,
                    damage: 0.0,
                    splash: 6.0,
                    splash_damage: 550.0,
                    upgraded: false,
                    thrown: true,
                },
                Mesh3d(wa.grenade_mesh.clone()),
                MeshMaterial3d(wa.grenade_mat.clone()),
                Transform::from_translation(origin + forward * 0.4 + right * 0.15 - up * 0.1),
                DespawnOnExit(AppState::Playing),
            ));
        }
        lo.state = if t >= 0.6 {
            if lo.slots.is_empty() {
                WState::Empty
            } else {
                WState::Raising { t: 0.15 }
            }
        } else {
            WState::Throw { t, thrown }
        };
        return;
    }

}
