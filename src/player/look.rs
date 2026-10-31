//! Umsehen mit der Maus (inkl. Rückstoß-Abbau) und Mauszeiger fangen.

use std::f32::consts::FRAC_PI_2;

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

use super::{CameraFov, Player};
use crate::base::InGame;
use crate::base::debug::DebugConfig;
use crate::base::settings::Settings;

/// Maus im Spiel fangen und verstecken, im Menü / in der Pause freigeben.
pub fn cursor_grab(
    mut cursor: Single<&mut CursorOptions>,
    ingame: Option<Res<InGame>>,
    windows: Query<&Window>,
) {
    let focused = windows.iter().any(|w| w.focused);
    let want = ingame.map(|g| !g.paused).unwrap_or(false) && focused;
    let mode = if want {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != mode {
        cursor.grab_mode = mode;
        cursor.visible = !want;
    }
}

/// Browser: Der Pointer-Lock kann enden, ohne dass Bevy davon erfährt. Esc fängt der Browser
/// selbst ab, dazu kommen Fokuswechsel und abgelehnte Anfragen. Der Modus bliebe dann auf
/// `Locked` und würde nie neu angefordert; die Maus stieße am Bildschirmrand an. Stattdessen
/// pausieren: Weiterspielen per Klick oder Esc fordert den Lock mit einer Nutzeraktion neu an.
#[cfg(target_arch = "wasm32")]
pub fn pause_on_lost_pointer_lock(
    cursor: Single<&CursorOptions>,
    game: Option<ResMut<InGame>>,
    mut vtime: ResMut<Time<Virtual>>,
    real: Res<Time<Real>>,
    // (Lock schon einmal bestätigt, seit wann angefordert)
    mut watch: Local<(bool, Option<f32>)>,
) {
    let Some(mut game) = game else {
        *watch = default();
        return;
    };
    if game.paused || cursor.grab_mode != CursorGrabMode::Locked {
        *watch = default();
        return;
    }
    let locked = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.pointer_lock_element())
        .is_some();
    if locked {
        *watch = (true, None);
        return;
    }
    // Die Anfrage wird asynchron bestätigt; eine abgelehnte erkennt man nur an der Dauer.
    let now = real.elapsed_secs();
    let requested = *watch.1.get_or_insert(now);
    if watch.0 || now - requested > 1.5 {
        crate::base::set_paused(&mut game, &mut vtime, true);
        *watch = default();
    }
}

pub fn mouse_look(
    motion: Res<AccumulatedMouseMotion>,
    cfg: Res<DebugConfig>,
    settings: Res<Settings>,
    ingame: Res<InGame>,
    time: Res<Time>,
    fov: Query<&CameraFov>,
    mut player: Single<(&mut Player, &mut Transform)>,
) {
    let (ref mut p, ref mut tf) = *player;
    if ingame.paused {
        return;
    }
    let dt = time.delta_secs();
    // Starke Vergrößerung (Zielfernrohr) zusätzlich mit dem Zoom verlangsamen; Kimme und
    // Korn (Zoom ab 0.75) behalten die eingestellte Ziel-Empfindlichkeit.
    let zoom = fov
        .iter()
        .next()
        .map(|f| f.current / settings.fov_deg.to_radians())
        .unwrap_or(1.0);
    let scope = (zoom / 0.75).min(1.0);
    let sens =
        settings.sensitivity * 0.0022 * (1.0 - p.ads * (1.0 - settings.ads_sensitivity)) * scope;
    let d = if cfg.no_input {
        Vec2::ZERO
    } else {
        motion.delta
    };
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
