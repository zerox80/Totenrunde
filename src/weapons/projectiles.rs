//! Projektile (Strahlenpistolen-Bolzen, Granatwerfer, Handgranaten mit Abprallern)
//! und Explosionen mit Flächenschaden.

use bevy::prelude::*;

use super::defs::Projectile;
use crate::audio::{Sfx, SfxEvent};
use crate::fx::FxEvent;
use crate::player::{Player, PlayerDamage};
use crate::world::collision::{CollisionWorld, F_BULLET};
use crate::zombies::{DamageKind, HitPart, Zombie, ZombieHit, ZombieRig, raycast_zombies};

#[derive(Component)]
pub struct ProjectileBody {
    pub kind: Projectile,
    pub vel: Vec3,
    pub age: f32,
    pub fuse: f32,
    pub damage: f32,
    pub splash: f32,
    pub splash_damage: f32,
    pub upgraded: bool,
    pub thrown: bool,
}

#[derive(Component)]
pub struct Tracer {
    pub life: f32,
}

/// Explosion: Flächenschaden an Zombies (und am Spieler).
#[allow(clippy::too_many_arguments)]
pub fn explode(
    pos: Vec3,
    radius: f32,
    damage: f32,
    zombies: &Query<(Entity, &Zombie, &Transform, Option<&ZombieRig>)>,
    col: &CollisionWorld,
    player_pos: Vec3,
    hits: &mut MessageWriter<ZombieHit>,
    fx: &mut MessageWriter<FxEvent>,
    sfx: &mut MessageWriter<SfxEvent>,
    dmg: &mut MessageWriter<PlayerDamage>,
    big: bool,
) {
    fx.write(FxEvent::Explosion { pos, radius });
    sfx.write(SfxEvent::at(Sfx::Explosion, pos).vol(if big { 1.0 } else { 0.6 }));
    for (e, z, tf, _) in zombies.iter() {
        if !z.alive() {
            continue;
        }
        let c = tf.translation + Vec3::Y * 0.9;
        let d = (c - pos).length();
        if d > radius {
            continue;
        }
        if !col.line_clear(pos + Vec3::Y * 0.2, c, F_BULLET) {
            continue;
        }
        let k = 1.0 - (d / radius) * 0.7;
        hits.write(ZombieHit {
            zombie: e,
            damage: damage * k,
            part: if d < radius * 0.3 {
                HitPart::Head
            } else {
                HitPart::Torso
            },
            kind: DamageKind::Explosive,
            point: c,
            dir: (c - pos).normalize_or(Vec3::Y),
        });
    }
    let pd = (player_pos + Vec3::Y * 0.9 - pos).length();
    if pd < radius * 0.75
        && col.line_clear(pos + Vec3::Y * 0.2, player_pos + Vec3::Y * 1.0, F_BULLET)
    {
        let amount = (1.0 - pd / (radius * 0.75)) * 70.0;
        if amount > 5.0 {
            dmg.write(PlayerDamage { amount, from: pos });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    col: Res<CollisionWorld>,
    mut q: Query<(Entity, &mut ProjectileBody, &mut Transform), (Without<Player>, Without<Zombie>)>,
    mut tracers: Query<(Entity, &mut Tracer)>,
    zombies: Query<(Entity, &Zombie, &Transform, Option<&ZombieRig>)>,
    bones: Query<&GlobalTransform>,
    player: Single<&Transform, With<Player>>,
    mut hits: MessageWriter<ZombieHit>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxEvent>,
    mut dmg: MessageWriter<PlayerDamage>,
) {
    let dt = time.delta_secs();
    for (e, mut tr) in tracers.iter_mut() {
        tr.life -= dt;
        if tr.life <= 0.0 {
            commands.entity(e).despawn();
        }
    }
    let ppos = player.translation;
    for (e, mut p, mut tf) in q.iter_mut() {
        p.age += dt;
        let gravity = match p.kind {
            Projectile::RayBolt => 0.0,
            Projectile::Grenade => 9.8,
            Projectile::Hitscan => 0.0,
        };
        p.vel.y -= gravity * dt;
        let step = p.vel * dt;
        let len = step.length();
        if len < 1e-6 {
            continue;
        }
        let dir = step / len;
        let start = tf.translation;
        let world = col.raycast(start, dir, len, F_BULLET);
        let max_t = world.map(|w| w.t).unwrap_or(len);
        let mut explode_at: Option<Vec3> = None;
        if !p.thrown {
            let zh = raycast_zombies(start, dir, max_t, &zombies, &bones);
            if let Some(h) = zh.first() {
                let point = start + dir * h.t;
                hits.write(ZombieHit {
                    zombie: h.zombie,
                    damage: p.damage,
                    part: h.part,
                    kind: DamageKind::Bullet,
                    point,
                    dir,
                });
                explode_at = Some(point);
            }
        }
        let ground_hit = start.y + step.y <= 0.02;
        if explode_at.is_none() {
            if let Some(w) = world {
                if p.thrown {
                    // abprallen
                    let n = w.normal;
                    tf.translation = w.point + n * 0.06;
                    p.vel = (p.vel - 2.0 * p.vel.dot(n) * n) * 0.45;
                    if p.vel.length() > 2.0 {
                        sfx.write(SfxEvent::at(Sfx::GrenadeBounce, w.point).vol(0.5));
                    }
                    continue;
                }
                explode_at = Some(w.point + w.normal * 0.1);
            } else if ground_hit {
                if p.thrown {
                    tf.translation = Vec3::new(start.x + step.x, 0.06, start.z + step.z);
                    if p.vel.y < -1.5 {
                        sfx.write(SfxEvent::at(Sfx::GrenadeBounce, tf.translation).vol(0.4));
                    }
                    p.vel.y = -p.vel.y * 0.35;
                    p.vel.x *= 0.7;
                    p.vel.z *= 0.7;
                    continue;
                }
                explode_at = Some(Vec3::new(start.x + step.x, 0.1, start.z + step.z));
            }
        }
        if explode_at.is_none() && p.age >= p.fuse {
            explode_at = Some(tf.translation);
        }
        if p.thrown && p.age < p.fuse {
            explode_at = None;
        }
        if let Some(at) = explode_at {
            match p.kind {
                Projectile::RayBolt => {
                    fx.write(FxEvent::RayImpact {
                        pos: at,
                        upgraded: p.upgraded,
                    });
                    sfx.write(SfxEvent::at(Sfx::Explosion, at).vol(0.35).pitch(1.6));
                    for (ze, z, ztf, _) in zombies.iter() {
                        if !z.alive() {
                            continue;
                        }
                        let c = ztf.translation + Vec3::Y * 0.9;
                        let d = (c - at).length();
                        if d < p.splash {
                            hits.write(ZombieHit {
                                zombie: ze,
                                damage: p.splash_damage * (1.0 - d / p.splash * 0.6),
                                part: HitPart::Torso,
                                kind: DamageKind::Explosive,
                                point: c,
                                dir: (c - at).normalize_or(Vec3::Y),
                            });
                        }
                    }
                    let pd = (ppos + Vec3::Y - at).length();
                    if pd < p.splash * 0.8 {
                        dmg.write(PlayerDamage {
                            amount: 25.0,
                            from: at,
                        });
                    }
                }
                _ => {
                    explode(
                        at,
                        p.splash,
                        p.splash_damage,
                        &zombies,
                        &col,
                        ppos,
                        &mut hits,
                        &mut fx,
                        &mut sfx,
                        &mut dmg,
                        true,
                    );
                }
            }
            commands.entity(e).despawn();
            continue;
        }
        tf.translation = start + step;
        if p.kind == Projectile::RayBolt {
            tf.rotation = Quat::from_rotation_arc(Vec3::Y, dir);
        }
        if p.age > 8.0 {
            commands.entity(e).despawn();
        }
    }
}
