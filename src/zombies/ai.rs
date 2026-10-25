//! Zombie-KI als Zustandsautomat: aus dem Boden steigen, zum Fenster laufen, Bretter
//! abreißen, durchklettern, dem Spieler folgen (Flow-Field), angreifen, sterben.

use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use super::{SpeedClass, ZState, Zombie, ATTACK_RANGE, ZOMBIE_RADIUS};
use crate::game::balance::Balance;
use crate::player::{Player, PlayerDamage};
use crate::world::barricades::Barricades;
use crate::world::collision::{CollisionWorld, F_ZOMBIE};
use crate::world::nav::{NavGrid, UNREACHABLE};

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn turn_towards(yaw: f32, target: f32, max: f32) -> f32 {
    let mut d = (target - yaw) % (2.0 * PI);
    if d > PI {
        d -= 2.0 * PI;
    } else if d < -PI {
        d += 2.0 * PI;
    }
    yaw + d.clamp(-max, max)
}

pub fn yaw_of(dir: Vec3) -> f32 {
    // Modell schaut nach -Z: yaw so, dass (-sin, -cos) == dir
    (-dir.x).atan2(-dir.z)
}

#[allow(clippy::too_many_arguments)]
pub fn zombie_ai(
    time: Res<Time>,
    col: Res<CollisionWorld>,
    nav: Res<NavGrid>,
    mut barricades: ResMut<Barricades>,
    bal: Res<Balance>,
    player: Single<(&Transform, &Player), Without<Zombie>>,
    mut zombies: Query<(Entity, &mut Zombie, &mut Transform)>,
    mut dmg: MessageWriter<PlayerDamage>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let (ptf, pl) = *player;
    let ppos = ptf.translation;
    let player_downed = pl.frozen;
    let mut rng = rand::rng();

    for (e, mut z, mut tf) in zombies.iter_mut() {
        z.attack_cd = (z.attack_cd - dt).max(0.0);
        z.flinch = (z.flinch - dt * 4.0).max(0.0);
        let mut pos = tf.translation;
        let state = z.state;
        match state {
            ZState::Rising { t } => {
                let t = t + dt / 1.8;
                pos.y = -1.65 * (1.0 - smoothstep(t).powf(0.7));
                if t >= 1.0 {
                    pos.y = 0.0;
                    z.state = ZState::Chasing;
                } else {
                    z.state = ZState::Rising { t };
                }
                let to = ppos - pos;
                z.yaw = turn_towards(z.yaw, yaw_of(Vec3::new(to.x, 0.0, to.z).normalize_or_zero()), dt * 1.5);
            }
            ZState::ToWindow { b } => {
                let bar = &mut barricades.list[b];
                let occupied = bar.occupant.is_some_and(|o| o != e);
                let outside = bar.outside;
                let away = (bar.outside - bar.inside).normalize_or_zero();
                let target = if occupied {
                    let side = Vec3::new(-away.z, 0.0, away.x) * ((e.index_u32() % 3) as f32 - 1.0) * 0.8;
                    outside + away * 1.4 + side
                } else {
                    outside
                };
                let to = Vec3::new(target.x - pos.x, 0.0, target.z - pos.z);
                let d = to.length();
                if d < 0.2 {
                    if !occupied {
                        bar.occupant = Some(e);
                        z.state = ZState::AtWindow {
                            b,
                            cd: rng.random_range(0.2..0.8),
                            tearing: 0.0,
                        };
                    }
                    z.vel = Vec3::ZERO;
                    z.yaw = turn_towards(z.yaw, yaw_of(-away), dt * 4.0);
                } else {
                    let dir = to / d;
                    let speed = z.speed.min(d * 3.0);
                    z.vel = dir * speed;
                    pos += z.vel * dt;
                    z.yaw = turn_towards(z.yaw, yaw_of(dir), dt * 6.0);
                }
            }
            ZState::AtWindow { b, cd, tearing } => {
                let bar = &mut barricades.list[b];
                let inward = (bar.inside - bar.outside).normalize_or_zero();
                z.yaw = turn_towards(z.yaw, yaw_of(inward), dt * 4.0);
                z.vel = Vec3::ZERO;
                // Spieler nah am Fenster? Dann durchs Fenster schlagen
                let pd = Vec3::new(ppos.x - bar.center.x, 0.0, ppos.z - bar.center.z).length();
                if pd < 1.6 && z.attack_cd <= 0.0 && tearing <= 0.0 && !player_downed {
                    z.state = ZState::Attacking {
                        t: 0.0,
                        hit: false,
                        window: Some(b),
                    };
                } else if bar.intact_boards() == 0 && tearing <= 0.0 {
                    z.state = ZState::Climbing { b, t: 0.0 };
                } else if tearing > 0.0 {
                    let nt = tearing + dt;
                    if tearing < 0.75 && nt >= 0.75 {
                        bar.tear_one();
                    }
                    if nt >= 1.3 {
                        z.state = ZState::AtWindow {
                            b,
                            cd: rng.random_range(0.6..1.4),
                            tearing: 0.0,
                        };
                    } else {
                        z.state = ZState::AtWindow { b, cd, tearing: nt };
                    }
                } else {
                    let ncd = cd - dt;
                    if ncd <= 0.0 && bar.intact_boards() > 0 {
                        z.state = ZState::AtWindow {
                            b,
                            cd: 0.0,
                            tearing: 0.001,
                        };
                    } else {
                        z.state = ZState::AtWindow { b, cd: ncd, tearing: 0.0 };
                    }
                }
            }
            ZState::Climbing { b, t } => {
                let bar = &mut barricades.list[b];
                let t = t + dt / 1.5;
                let s = smoothstep(t);
                let p = bar.outside.lerp(bar.inside, s);
                pos = Vec3::new(p.x, (t * PI).sin().max(0.0) * bar.sill * 0.9, p.z);
                let inward = (bar.inside - bar.outside).normalize_or_zero();
                z.yaw = turn_towards(z.yaw, yaw_of(inward), dt * 4.0);
                if t >= 1.0 {
                    pos.y = 0.0;
                    if bar.occupant == Some(e) {
                        bar.occupant = None;
                    }
                    z.state = ZState::Chasing;
                } else {
                    z.state = ZState::Climbing { b, t };
                }
            }
            ZState::Chasing => {
                let to = Vec3::new(ppos.x - pos.x, 0.0, ppos.z - pos.z);
                let dist = to.length();
                if dist < ATTACK_RANGE && (ppos.y - pos.y).abs() < 1.3 && z.attack_cd <= 0.0 && !player_downed {
                    z.state = ZState::Attacking {
                        t: 0.0,
                        hit: false,
                        window: None,
                    };
                    z.vel = Vec3::ZERO;
                } else {
                    let eye = Vec3::Y * 1.0;
                    let direct = dist < 3.5 && col.line_clear(pos + eye, ppos + eye, F_ZOMBIE);
                    let dir = if direct || player_downed {
                        to.normalize_or_zero()
                    } else if let Some(d) = nav.flow_dir(pos) {
                        d
                    } else {
                        to.normalize_or_zero()
                    };
                    let mut speed = z.speed * (1.0 - z.flinch * 0.7);
                    if player_downed {
                        // am Boden liegende Spieler werden ignoriert: herumschlurfen
                        speed *= 0.3;
                    }
                    if dist < 0.9 {
                        speed *= (dist - 0.6).max(0.0) / 0.3;
                    }
                    let want = dir * speed;
                    let k = (dt * 6.0).min(1.0);
                    z.vel = z.vel + (want - z.vel) * k;
                    pos += z.vel * dt;
                    let face = if dist < 2.0 { to.normalize_or_zero() } else { dir };
                    let turn = match z.class {
                        SpeedClass::Walk => 3.0,
                        SpeedClass::Run => 5.0,
                        SpeedClass::Sprint => 7.0,
                    };
                    z.yaw = turn_towards(z.yaw, yaw_of(face), dt * turn);
                    // Festhängen erkennen
                    if (pos - z.progress_pos).length() > 0.6 {
                        z.progress_pos = pos;
                        z.stuck = 0.0;
                    } else {
                        z.stuck += dt;
                        if z.stuck > 2.5 && nav.dist_at(pos) == UNREACHABLE {
                            // Notfall: in die nächste begehbare Zelle springen
                            if let Some(i) = nav.nearest_walkable(pos) {
                                let c = nav.center(i);
                                pos.x = c.x;
                                pos.z = c.z;
                            }
                            z.stuck = 0.0;
                        }
                    }
                }
            }
            ZState::Attacking { t, hit, window } => {
                let t = t + dt;
                let to = Vec3::new(ppos.x - pos.x, 0.0, ppos.z - pos.z);
                z.yaw = turn_towards(z.yaw, yaw_of(to.normalize_or_zero()), dt * 5.0);
                let mut hit = hit;
                if !hit && t >= 0.45 {
                    hit = true;
                    let reach = if let Some(b) = window {
                        let c = barricades.list[b].center;
                        Vec3::new(ppos.x - c.x, 0.0, ppos.z - c.z).length() < 1.8
                    } else {
                        to.length() < ATTACK_RANGE + 0.45 && (ppos.y - pos.y).abs() < 1.4
                    };
                    if reach && !player_downed {
                        dmg.write(PlayerDamage {
                            amount: bal.zombie_damage,
                            from: pos,
                        });
                    }
                }
                if t >= 0.9 {
                    z.attack_cd = 0.25;
                    z.state = match window {
                        Some(b) => ZState::AtWindow {
                            b,
                            cd: 0.4,
                            tearing: 0.0,
                        },
                        None => ZState::Chasing,
                    };
                } else {
                    z.state = ZState::Attacking { t, hit, window };
                }
            }
            ZState::Dying { t } => {
                let t = t + dt;
                if t > 3.2 {
                    pos.y -= dt * 0.6;
                }
                if t > 5.0 {
                    commands.entity(e).despawn();
                    continue;
                }
                z.state = ZState::Dying { t };
            }
        }

        // Kollision mit der Welt (nicht beim Klettern/Aufsteigen/Sterben)
        if matches!(z.state, ZState::Chasing | ZState::Attacking { .. } | ZState::ToWindow { .. }) {
            col.resolve_cylinder(&mut pos, ZOMBIE_RADIUS, 1.7, 0.35, F_ZOMBIE);
            pos.y = col.ground_height(pos, ZOMBIE_RADIUS, 0.35, F_ZOMBIE).max(0.0);
        }
        tf.translation = pos;
        let lean = if z.flinch > 0.0 { z.flinch * 0.25 } else { 0.0 };
        tf.rotation = Quat::from_rotation_y(z.yaw) * Quat::from_rotation_x(-lean);
    }
}
