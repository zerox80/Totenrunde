//! Umsehen mit der Maus (inkl. Rückstoß-Abbau) und Mauszeiger fangen.

use std::f32::consts::FRAC_PI_2;

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

use super::Player;
use crate::base::debug::DebugConfig;
use crate::base::settings::Settings;
use crate::base::InGame;

/// Maus im Spiel fangen und verstecken, im Menü / in der Pause freigeben.
pub fn cursor_grab(mut cursor: Single<&mut CursorOptions>, ingame: Option<Res<InGame>>, windows: Query<&Window>) {
    let focused = windows.iter().any(|w| w.focused);
    let want = ingame.map(|g| !g.paused).unwrap_or(false) && focused;
    let mode = if want { CursorGrabMode::Locked } else { CursorGrabMode::None };
    if cursor.grab_mode != mode {
        cursor.grab_mode = mode;
        cursor.visible = !want;
    }
}

pub fn mouse_look(
    motion: Res<AccumulatedMouseMotion>,
    cfg: Res<DebugConfig>,
    settings: Res<Settings>,
    ingame: Res<InGame>,
    time: Res<Time>,
    mut player: Single<(&mut Player, &mut Transform)>,
) {
    let (ref mut p, ref mut tf) = *player;
    if ingame.paused {
        return;
    }
    let dt = time.delta_secs();
    let sens = settings.sensitivity * 0.0022 * (1.0 - p.ads * (1.0 - settings.ads_sensitivity));
    let d = if cfg.no_input { Vec2::ZERO } else { motion.delta };
    p.yaw -= d.x * sens;
    p.pitch -= d.y * sens * if settings.invert_y { -1.0 } else { 1.0 };
    // Rückstoß abbauen
    let rv = p.recoil_vel;
    p.pitch += rv.x * dt;
    p.yaw += rv.y * dt;
    p.recoil_vel = rv * (1.0 - (dt * 14.0).min(1.0));
    p.pitch = p.pitch.clamp(-FRAC_PI_2 + 0.02, FRAC_PI_2 - 0.02);
    tf.rotation = Quat::from_rotation_y(p.yaw);
}
