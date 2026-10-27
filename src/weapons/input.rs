//! Waffen-Eingaben: Zielen, Nachladen, Wechseln, Messer, Granate, Zustandsautomat.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use super::defs::def;
use super::loadout::{Loadout, WState};
use crate::audio::{Sfx, SfxEvent};
use crate::base::InGame;
use crate::base::settings::Settings;
use crate::game::perks::{Perk, Perks};
use crate::player::{CameraFov, Player};

#[allow(clippy::too_many_arguments)]
pub fn weapon_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    ingame: Res<InGame>,
    perks: Res<Perks>,
    settings: Res<Settings>,
    mut q: Single<(&mut Loadout, &mut Player)>,
    mut fov: Single<&mut CameraFov>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let (ref mut lo, ref mut pl) = *q;
    let dt = time.delta_secs();
    if ingame.paused {
        return;
    }
    let speed_cola = perks.has(Perk::SpeedCola);
    // Zielen
    let wants_ads = mouse.pressed(MouseButton::Right)
        && matches!(lo.state, WState::Ready | WState::ShellReload { .. })
        && !pl.sprinting
        && lo.current_slot().is_some();
    let ads_speed = 1.0 / 0.22;
    lo.ads = if wants_ads {
        (lo.ads + dt * ads_speed).min(1.0)
    } else {
        (lo.ads - dt * ads_speed).max(0.0)
    };
    pl.ads = lo.ads;
    let zoom = lo
        .current_slot()
        .map(|s| def(s.weapon).ads_zoom)
        .unwrap_or(1.0);
    let base_fov = settings.fov_deg.to_radians();
    fov.target = base_fov * (1.0 - (1.0 - zoom) * lo.ads);
    pl.weapon_mobility = lo
        .current_slot()
        .map(|s| def(s.weapon).mobility)
        .unwrap_or(1.0);

    if pl.sprinting {
        lo.sprint_out = 0.22;
        lo.burst_left = 0;
    } else {
        lo.sprint_out = (lo.sprint_out - dt).max(0.0);
    }
    lo.fire_cd = (lo.fire_cd - dt).max(0.0);
    lo.bloom = (lo.bloom - dt * 3.0).max(0.0);

    // Zustandsautomat
    let state = lo.state;
    match state {
        WState::Reloading { t, dur } => {
            let t = t + dt;
            if t >= dur {
                if let Some(s) = lo.current_slot_mut() {
                    let d = def(s.weapon);
                    let need = d.mag_size(s.upgraded) - s.mag;
                    let take = need.min(s.reserve);
                    s.mag += take;
                    s.reserve -= take;
                }
                lo.state = WState::Ready;
            } else {
                if t >= dur * 0.25 && t - dt < dur * 0.25 {
                    sfx.write(SfxEvent::ui(Sfx::MagOut).vol(0.6));
                }
                if t >= dur * 0.7 && t - dt < dur * 0.7 {
                    sfx.write(SfxEvent::ui(Sfx::MagIn).vol(0.7));
                }
                lo.state = WState::Reloading { t, dur };
            }
        }
        WState::ShellReload { t, per } => {
            let t = t + dt;
            if t >= per {
                let mut done = true;
                if let Some(s) = lo.current_slot_mut() {
                    let d = def(s.weapon);
                    if s.mag < d.mag_size(s.upgraded) && s.reserve > 0 {
                        s.mag += 1;
                        s.reserve -= 1;
                        done = s.mag >= d.mag_size(s.upgraded) || s.reserve == 0;
                    }
                }
                sfx.write(SfxEvent::ui(Sfx::Shell).vol(0.6));
                if done {
                    sfx.write(SfxEvent::ui(Sfx::Bolt).vol(0.6));
                    lo.state = WState::Ready;
                } else {
                    lo.state = WState::ShellReload { t: 0.0, per };
                }
            } else {
                lo.state = WState::ShellReload { t, per };
            }
        }
        WState::Lowering { t, to } => {
            let t = t + dt;
            if t >= 0.22 {
                if let Some(to) = to {
                    if to < lo.slots.len() {
                        lo.current = to;
                    }
                }
                lo.state = WState::Raising { t: 0.0 };
                sfx.write(SfxEvent::ui(Sfx::Switch).vol(0.5));
            } else {
                lo.state = WState::Lowering { t, to };
            }
        }
        WState::Raising { t } => {
            let t = t + dt;
            lo.state = if t >= 0.35 {
                if lo.slots.is_empty() {
                    WState::Empty
                } else {
                    WState::Ready
                }
            } else {
                WState::Raising { t }
            };
        }
        WState::Busy { t, dur } => {
            let t = t + dt;
            lo.state = if t >= dur {
                WState::Raising { t: 0.0 }
            } else {
                WState::Busy { t, dur }
            };
        }
        WState::Melee { .. } | WState::Throw { .. } => {}
        WState::Ready | WState::Empty => {}
    }

    if pl.frozen {
        return;
    }
    let can = matches!(lo.state, WState::Ready | WState::ShellReload { .. });
    // Waffenwechsel
    let n = lo.slots.len();
    let mut switch_to = None;
    if n > 1 && can {
        if keys.just_pressed(KeyCode::Digit1) {
            switch_to = Some(0);
        }
        if keys.just_pressed(KeyCode::Digit2) {
            switch_to = Some(1);
        }
        if keys.just_pressed(KeyCode::Digit3) && n > 2 {
            switch_to = Some(2);
        }
        if scroll.delta.y.abs() > 0.1 || keys.just_pressed(KeyCode::KeyQ) {
            let dir: i32 = if scroll.delta.y < 0.0 { 1 } else { -1 };
            switch_to = Some(((lo.current as i32 + dir).rem_euclid(n as i32)) as usize);
        }
    }
    if let Some(to) = switch_to {
        if to != lo.current && to < n {
            lo.state = WState::Lowering {
                t: 0.0,
                to: Some(to),
            };
            lo.burst_left = 0;
            return;
        }
    }
    // Nachladen
    if keys.just_pressed(KeyCode::KeyR) && matches!(lo.state, WState::Ready) {
        start_reload(lo, speed_cola, &mut sfx);
    }
    // Messer
    if keys.just_pressed(KeyCode::KeyV)
        && matches!(
            lo.state,
            WState::Ready | WState::ShellReload { .. } | WState::Empty
        )
    {
        lo.state = WState::Melee { t: 0.0, hit: false };
        sfx.write(SfxEvent::ui(Sfx::KnifeSwing));
    }
    // Granate
    if keys.just_pressed(KeyCode::KeyG)
        && lo.grenades > 0
        && matches!(lo.state, WState::Ready | WState::Empty)
    {
        lo.state = WState::Throw {
            t: 0.0,
            thrown: false,
        };
        sfx.write(SfxEvent::ui(Sfx::GrenadePin).vol(0.7));
    }
}

pub fn start_reload(lo: &mut Loadout, speed_cola: bool, sfx: &mut MessageWriter<SfxEvent>) {
    let Some(s) = lo.current_slot().copied() else {
        return;
    };
    let d = def(s.weapon);
    if s.mag >= d.mag_size(s.upgraded) || s.reserve == 0 {
        return;
    }
    let mult = if speed_cola { 0.5 } else { 1.0 };
    lo.ads = lo.ads.min(0.5);
    if d.shell_reload {
        lo.state = WState::ShellReload {
            t: -0.25 * mult,
            per: d.reload * mult,
        };
    } else {
        let dur = if s.mag == 0 { d.reload_empty } else { d.reload } * mult;
        lo.state = WState::Reloading { t: 0.0, dur };
    }
    let _ = sfx;
}
