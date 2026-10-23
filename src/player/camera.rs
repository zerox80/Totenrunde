//! Kamera-Effekte: Kopfwippen, Wackeln, Landen, weiches Zoomen beim Zielen.

use bevy::prelude::*;

use super::{CameraFov, Player, PlayerCamera};
use crate::base::settings::Settings;

pub fn camera_effects(
    time: Res<Time>,
    settings: Res<Settings>,
    mut player: Single<&mut Player>,
    mut cam: Single<(&mut Transform, &mut Projection, &mut CameraFov), With<PlayerCamera>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let p = &mut *player;
    let (ref mut tf, ref mut proj, ref mut fov) = *cam;
    let bob_y = (p.bob_phase * 2.0).sin() * 0.025 * p.bob * (1.0 - p.ads * 0.8) * settings.head_bob;
    let bob_x = p.bob_phase.sin() * 0.015 * p.bob * (1.0 - p.ads * 0.8) * settings.head_bob;
    let shake = p.shake;
    let sx = (t * 47.0).sin() * shake * 0.02;
    let sy = (t * 53.0).cos() * shake * 0.02;
    p.shake = (p.shake - dt * 2.5).max(0.0);
    tf.translation = Vec3::new(bob_x + sx, p.eye + bob_y - p.land_kick + sy, 0.0);
    let roll = bob_x * 0.6 + sx * 0.5;
    tf.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, p.pitch, roll);
    fov.current += (fov.target - fov.current) * (dt * 14.0).min(1.0);
    if let Projection::Perspective(pp) = proj.as_mut() {
        pp.fov = fov.current;
    }
}
