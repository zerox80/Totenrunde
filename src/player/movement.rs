//! Bewegung: Gehen, Sprinten (mit Ausdauer), Ducken, Hinlegen, Springen, Stufen,
//! Kollision mit der Welt und mit Zombies.

use bevy::prelude::*;

use super::{GRAVITY, JUMP_VEL, Player, RADIUS, SPRINT_MULT, STEP, Stance, WALK_SPEED};
use crate::base::InGame;
use crate::world::collision::{CollisionWorld, F_PLAYER};
use crate::zombies::{ZOMBIE_RADIUS, ZState, Zombie};

pub fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    col: Res<CollisionWorld>,
    ingame: Res<InGame>,
    mut player: Single<(&mut Player, &mut Transform)>,
) {
    let (ref mut p, ref mut tf) = *player;
    let dt = time.delta_secs().min(0.05);
    if dt <= 0.0 {
        return;
    }
    let active = !ingame.paused && !p.frozen;

    // --- Haltung ---
    if active && keys.just_pressed(KeyCode::KeyC) {
        p.stance = match p.stance {
            Stance::Crouch => Stance::Stand,
            _ => Stance::Crouch,
        };
    }
    if active && (keys.just_pressed(KeyCode::ControlLeft) || keys.just_pressed(KeyCode::KeyZ)) {
        p.stance = match p.stance {
            Stance::Prone => Stance::Crouch,
            _ => Stance::Prone,
        };
    }
    if active
        && keys.just_pressed(KeyCode::Space)
        && p.stance != Stance::Stand
        && p.speed_mult > 0.5
    {
        // Leertaste stellt auf
        p.stance = match p.stance {
            Stance::Prone => Stance::Crouch,
            _ => Stance::Stand,
        };
    }
    // Aufstehen nur, wenn Platz ist
    let ceiling = col.ceiling_height(tf.translation, RADIUS * 0.9, F_PLAYER);
    while p.stance.height() > ceiling - tf.translation.y + 0.02 && p.stance != Stance::Prone {
        p.stance = match p.stance {
            Stance::Stand => Stance::Crouch,
            _ => Stance::Prone,
        };
    }

    // --- Eingabe ---
    let mut input = Vec2::ZERO;
    if active {
        if keys.pressed(KeyCode::KeyW) {
            input.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            input.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            input.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            input.x -= 1.0;
        }
    }
    let input = input.normalize_or_zero();
    let forward = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    let wish = forward * input.y + right * input.x;

    // --- Sprinten ---
    let want_sprint = active
        && keys.pressed(KeyCode::ShiftLeft)
        && input.y > 0.5
        && p.ads < 0.3
        && p.speed_mult > 0.5;
    if want_sprint && p.stamina > 0.0 && p.grounded {
        if !p.sprinting {
            p.stance = Stance::Stand;
        }
        p.sprinting = true;
    } else if !want_sprint || p.stamina <= 0.0 {
        if p.sprinting {
            p.stamina_cooldown = 0.6;
        }
        p.sprinting = false;
    }
    if p.sprinting {
        p.stamina = (p.stamina - dt).max(0.0);
    } else if p.stamina_cooldown > 0.0 {
        p.stamina_cooldown -= dt;
    } else {
        p.stamina = (p.stamina + dt * 1.2).min(p.stamina_max);
    }

    // --- Geschwindigkeit ---
    let mut speed = WALK_SPEED * p.stance.speed() * p.weapon_mobility * p.speed_mult;
    if p.sprinting {
        speed *= SPRINT_MULT;
    }
    speed *= 1.0 - 0.45 * p.ads;
    if input.y < -0.1 {
        speed *= 0.85;
    }
    let target = wish * speed;
    let hv = Vec3::new(p.vel.x, 0.0, p.vel.z);
    let accel = if p.grounded { 38.0 } else { 7.0 };
    let diff = target - hv;
    let max_dv = accel * dt;
    let hv = if diff.length() <= max_dv {
        target
    } else {
        hv + diff.normalize() * max_dv
    };
    p.vel.x = hv.x;
    p.vel.z = hv.z;

    // --- Springen / Schwerkraft ---
    if active && keys.just_pressed(KeyCode::Space) && p.grounded && p.stance == Stance::Stand {
        p.vel.y = JUMP_VEL;
        p.grounded = false;
    }
    p.vel.y -= GRAVITY * dt;

    // --- Integration mit Kollision (in Teilschritten gegen Tunneln) ---
    let height = p.stance.height();
    let mut pos = tf.translation;
    let total = p.vel * dt;
    let steps = ((total.length() / (RADIUS * 0.5)).ceil() as usize).clamp(1, 8);
    let step_v = total / steps as f32;
    let mut grounded = false;
    for _ in 0..steps {
        pos.x += step_v.x;
        pos.z += step_v.z;
        col.resolve_cylinder(&mut pos, RADIUS, height, STEP, F_PLAYER);
        pos.y += step_v.y;
        let g = col.ground_height(pos, RADIUS, STEP, F_PLAYER);
        if pos.y <= g + 0.001 {
            if pos.y < g - 0.001 {
                // Stufe hinauf: Kamera weich nachziehen
                p.land_kick += (g - pos.y) * 0.5;
            }
            pos.y = g;
            if p.vel.y < 0.0 {
                if p.vel.y < -6.0 {
                    p.land_kick += (-p.vel.y - 6.0) * 0.02;
                }
                p.vel.y = 0.0;
            }
            grounded = true;
        } else if p.grounded && p.vel.y <= 0.0 && pos.y - g < STEP {
            // an der Stufe abwärts kleben bleiben
            pos.y = g;
            p.vel.y = 0.0;
            grounded = true;
        }
        // Kopf stößt an
        let ceil = col.ceiling_height(pos, RADIUS * 0.9, F_PLAYER);
        if pos.y + height > ceil && p.vel.y > 0.0 {
            p.vel.y = 0.0;
            pos.y = (ceil - height).max(pos.y - 0.05);
        }
    }
    p.grounded = grounded;
    tf.translation = pos;

    // --- Kopfwippen ---
    let hspeed = Vec2::new(p.vel.x, p.vel.z).length();
    p.moving = hspeed > 0.5;
    let bob_target = if p.grounded {
        (hspeed / WALK_SPEED).min(1.6)
    } else {
        0.0
    };
    p.bob += (bob_target - p.bob) * (dt * 8.0).min(1.0);
    let freq = if p.sprinting { 13.5 } else { 10.0 };
    p.bob_phase += dt * freq * (0.4 + 0.6 * (hspeed / WALK_SPEED).min(1.5));
    let target_eye = p.stance.eye();
    p.eye += (target_eye - p.eye) * (dt * 10.0).min(1.0);
    p.land_kick *= 1.0 - (dt * 9.0).min(1.0);
}

/// Zombies blockieren den Spieler (man kann eingekesselt werden wie im Original).
pub fn push_from_zombies(
    mut player: Single<&mut Transform, With<Player>>,
    zombies: Query<(&Zombie, &Transform), Without<Player>>,
    col: Res<CollisionWorld>,
) {
    let mut pos = player.translation;
    let mut moved = false;
    for (z, ztf) in zombies.iter() {
        if !matches!(z.state, ZState::Chasing | ZState::Attacking { .. }) {
            continue;
        }
        let d = Vec3::new(pos.x - ztf.translation.x, 0.0, pos.z - ztf.translation.z);
        let l = d.length();
        let min = RADIUS + ZOMBIE_RADIUS - 0.02;
        if l < min && l > 1e-4 {
            pos += d / l * (min - l) * 0.6;
            moved = true;
        }
    }
    if moved {
        col.resolve_cylinder(&mut pos, RADIUS, 1.2, STEP, F_PLAYER);
        player.translation = pos;
    }
}
