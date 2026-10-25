//! Entwicklungshilfen, gesteuert über Umgebungsvariablen:
//!
//! * `TR_SHOTS="2.5:/pfad/a.png,6:/pfad/b.png"` – Screenshots zu Zeitpunkten (Sekunden im Spiel)
//! * `TR_EXIT=8`        – Spiel nach N Sekunden beenden
//! * `TR_POSE="x,y,z,yaw_grad,pitch_grad"` – Spieler beim Start platzieren
//! * `TR_SCRIPT="1:key=W;3:key=-W"` – einfache Tasten-Skripte (siehe `ScriptStep`)
//! * `TR_GOD=1` – Spieler unverwundbar
//! * `TR_ROUND=10` – mit Runde N starten, `TR_POINTS=50000` – Startpunkte
//! * `TR_PLAYERS=2` – Schwierigkeit für N Spieler (siehe `game::balance`)
//! * `TR_SKIPMENU=1` – Hauptmenü überspringen, `TR_NOINPUT=1` – echte Eingaben ignorieren
//! * `TR_POWER=1` – Strom ist von Anfang an eingeschaltet
//! * `TR_MENUSHOT=/pfad/menu.png` – Screenshot des Hauptmenüs nach 1,5 s
//! * `TR_MENUPLAY=1` – im Hauptmenü nach 3 s "Spielen" drücken (Enter)
//! * `TR_GFXMENU=1` – Grafik-Menü beim Start öffnen
//! * `TR_WEAPON=mp5`, `TR_PAP=1` – mit dieser (aufgewerteten) Waffe starten
//! * Leistungstests: `TR_TEXMAX=512`, `TR_MSAA=0`, `TR_SHADOWS=1`, `TR_NOLIGHTS=1`, `TR_NOWALL=1`

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use super::{AppState, InGame};
use crate::player::Player;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        let cfg = DebugConfig::from_env();
        app.insert_resource(cfg)
            .add_systems(OnEnter(AppState::Playing), apply_pose)
            .add_systems(Update, timed_actions.run_if(in_state(AppState::Playing)))
            .add_systems(Update, menu_test.run_if(in_state(AppState::Menu)))
            .add_systems(
                PreUpdate,
                script_input
                    .after(bevy::input::InputSystems)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Clone, Debug)]
pub enum ScriptAction {
    Press(KeyCode),
    Release(KeyCode),
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    Look(f32, f32),
}

#[derive(Resource, Default)]
pub struct DebugConfig {
    pub shots: Vec<(f32, String)>,
    pub exit_at: Option<f32>,
    pub pose: Option<[f32; 5]>,
    pub script: Vec<(f32, ScriptAction)>,
    pub god: bool,
    pub start_round: Option<u32>,
    pub start_points: Option<u32>,
    /// Spielerzahl für die Schwierigkeit (TR_PLAYERS)
    pub players: Option<u32>,
    /// Strom von Anfang an eingeschaltet (TR_POWER)
    pub start_power: bool,
    pub elapsed: f32,
    pub active: bool,
    /// Echte Maus-/Tastatureingaben ignorieren (für automatische Tests)
    pub no_input: bool,
    /// Hauptmenü überspringen und direkt spielen
    pub skip_menu: bool,
}

pub fn parse_key(s: &str) -> Option<KeyCode> {
    Some(match s {
        "W" => KeyCode::KeyW,
        "A" => KeyCode::KeyA,
        "S" => KeyCode::KeyS,
        "D" => KeyCode::KeyD,
        "F" => KeyCode::KeyF,
        "R" => KeyCode::KeyR,
        "G" => KeyCode::KeyG,
        "V" => KeyCode::KeyV,
        "C" => KeyCode::KeyC,
        "Q" => KeyCode::KeyQ,
        "E" => KeyCode::KeyE,
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "SPACE" => KeyCode::Space,
        "SHIFT" => KeyCode::ShiftLeft,
        "CTRL" => KeyCode::ControlLeft,
        "ESC" => KeyCode::Escape,
        "TAB" => KeyCode::Tab,
        _ => return None,
    })
}

impl DebugConfig {
    fn from_env() -> Self {
        let mut c = DebugConfig::default();
        if let Ok(s) = std::env::var("TR_SHOTS") {
            for part in s.split(',') {
                if let Some((t, p)) = part.split_once(':') {
                    if let Ok(t) = t.trim().parse::<f32>() {
                        c.shots.push((t, p.trim().to_string()));
                    }
                }
            }
        }
        c.exit_at = std::env::var("TR_EXIT").ok().and_then(|s| s.parse().ok());
        if let Ok(s) = std::env::var("TR_POSE") {
            let v: Vec<f32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            if v.len() == 5 {
                c.pose = Some([v[0], v[1], v[2], v[3], v[4]]);
            }
        }
        if let Ok(s) = std::env::var("TR_SCRIPT") {
            for part in s.split(';') {
                let Some((t, a)) = part.split_once(':') else { continue };
                let Ok(t) = t.trim().parse::<f32>() else { continue };
                let a = a.trim();
                let action = if let Some(k) = a.strip_prefix("key=-") {
                    parse_key(k).map(ScriptAction::Release)
                } else if let Some(k) = a.strip_prefix("key=") {
                    parse_key(k).map(ScriptAction::Press)
                } else if a == "lmb" {
                    Some(ScriptAction::MousePress(MouseButton::Left))
                } else if a == "-lmb" {
                    Some(ScriptAction::MouseRelease(MouseButton::Left))
                } else if a == "rmb" {
                    Some(ScriptAction::MousePress(MouseButton::Right))
                } else if a == "-rmb" {
                    Some(ScriptAction::MouseRelease(MouseButton::Right))
                } else if let Some(v) = a.strip_prefix("look=") {
                    let v: Vec<f32> = v.split('/').filter_map(|x| x.parse().ok()).collect();
                    (v.len() == 2).then(|| ScriptAction::Look(v[0], v[1]))
                } else {
                    None
                };
                if let Some(action) = action {
                    c.script.push((t, action));
                }
            }
        }
        c.god = std::env::var("TR_GOD").is_ok();
        c.start_round = std::env::var("TR_ROUND").ok().and_then(|s| s.parse().ok());
        c.start_points = std::env::var("TR_POINTS").ok().and_then(|s| s.parse().ok());
        c.players = std::env::var("TR_PLAYERS").ok().and_then(|s| s.parse().ok());
        c.start_power = std::env::var("TR_POWER").is_ok();
        c.active = !c.shots.is_empty() || c.exit_at.is_some() || !c.script.is_empty();
        c.no_input = c.active || std::env::var("TR_NOINPUT").is_ok();
        c.skip_menu = c.active || std::env::var("TR_SKIPMENU").is_ok();
        c
    }
}

impl DebugConfig {
    /// Hält das Skript diese Taste gerade gedrückt?
    pub fn script_keys_held(&self, k: KeyCode) -> bool {
        let mut held = false;
        for (t, a) in &self.script {
            if *t > self.elapsed {
                break;
            }
            match a {
                ScriptAction::Press(x) if *x == k => held = true,
                ScriptAction::Release(x) if *x == k => held = false,
                _ => {}
            }
        }
        held
    }
    pub fn script_mouse_held(&self, b: MouseButton) -> bool {
        let mut held = false;
        for (t, a) in &self.script {
            if *t > self.elapsed {
                break;
            }
            match a {
                ScriptAction::MousePress(x) if *x == b => held = true,
                ScriptAction::MouseRelease(x) if *x == b => held = false,
                _ => {}
            }
        }
        held
    }
}

fn apply_pose(cfg: Res<DebugConfig>, mut q: Query<(&mut Player, &mut Transform)>) {
    let Some(p) = cfg.pose else { return };
    for (mut pl, mut tf) in q.iter_mut() {
        tf.translation = Vec3::new(p[0], p[1], p[2]);
        pl.yaw = p[3].to_radians();
        pl.pitch = p[4].to_radians();
        tf.rotation = Quat::from_rotation_y(pl.yaw);
    }
}

fn timed_actions(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut cfg: ResMut<DebugConfig>,
    ingame: Option<Res<InGame>>,
    mut exit: MessageWriter<AppExit>,
    mut fps: Local<(u32, f32)>,
) {
    if !cfg.active || ingame.is_none() {
        return;
    }
    let before = cfg.elapsed;
    cfg.elapsed += time.delta_secs();
    let now = cfg.elapsed;
    // Durchschnittliche Bildrate ab Sekunde 10 (nach Laden und Mipmaps), Ausgabe beim Beenden
    if now > 10.0 {
        fps.0 += 1;
        fps.1 += time.delta_secs();
    }
    let shots: Vec<String> = cfg
        .shots
        .iter()
        .filter(|(t, _)| *t > before && *t <= now)
        .map(|(_, p)| p.clone())
        .collect();
    for path in shots {
        info!("Screenshot: {path}");
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }
    if let Some(t) = cfg.exit_at {
        if now >= t {
            if fps.1 > 0.0 {
                info!("FPS-Mittel: {:.1}", fps.0 as f32 / fps.1);
            }
            exit.write(AppExit::Success);
        }
    }
}

fn script_input(
    cfg: Res<DebugConfig>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut player: Query<&mut Player>,
    mut last: Local<f32>,
) {
    if cfg.no_input {
        // echte Eingaben verwerfen, nur das Skript zählt
        let held: Vec<KeyCode> = keys.get_pressed().copied().collect();
        for k in held {
            if !cfg.script_keys_held(k) {
                keys.release(k);
            }
        }
        let held_m: Vec<MouseButton> = mouse.get_pressed().copied().collect();
        for b in held_m {
            if !cfg.script_mouse_held(b) {
                mouse.release(b);
            }
        }
        keys.clear();
        mouse.clear();
    }
    if cfg.script.is_empty() {
        return;
    }
    let before = *last;
    let now = cfg.elapsed;
    *last = now;
    for (t, a) in &cfg.script {
        if *t > before && *t <= now {
            match a {
                ScriptAction::Press(k) => keys.press(*k),
                ScriptAction::Release(k) => keys.release(*k),
                ScriptAction::MousePress(b) => mouse.press(*b),
                ScriptAction::MouseRelease(b) => mouse.release(*b),
                ScriptAction::Look(yaw, pitch) => {
                    for mut p in player.iter_mut() {
                        p.yaw = yaw.to_radians();
                        p.pitch = pitch.to_radians();
                    }
                }
            }
        }
    }
}

/// Automatischer Test des Hauptmenüs (Screenshot, dann "Spielen").
fn menu_test(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut t: Local<f32>,
    mut shot_done: Local<bool>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    *t += time.delta_secs();
    if !*shot_done && *t > 1.5 {
        *shot_done = true;
        if let Ok(path) = std::env::var("TR_MENUSHOT") {
            info!("Menü-Screenshot: {path}");
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
    }
    if *t > 3.0 && std::env::var("TR_MENUPLAY").is_ok() {
        keys.press(KeyCode::Enter);
    }
}
