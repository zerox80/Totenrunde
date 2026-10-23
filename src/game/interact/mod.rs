//! Interaktion mit der Welt (Taste F): Wandwaffen, Türen und Schutt, Perk-Automaten,
//! Stromschalter, Mysteriöse Kiste, Waffen-Verstärker und Barrikaden reparieren.
//!
//! Ein zentrales System wählt das Objekt, auf das der Spieler schaut und das in Reichweite
//! ist (`Focus`). Jedes Modul schreibt dann seinen Hinweistext (`Prompt`) und reagiert auf F.

pub mod doors;
pub mod mystery_box;
pub mod pap;
pub mod perks;
pub mod power;
pub mod repair;
pub mod wallbuy;

use bevy::prelude::*;

use crate::audio::{Sfx, SfxEvent};
use crate::base::debug::DebugConfig;
use crate::base::{AppState, GameSet, InGame};
use crate::game::perks::Perk;
use crate::game::points::Score;
use crate::player::{Health, Player};

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Focus>()
            .init_resource::<Prompt>()
            .add_systems(
                OnEnter(AppState::Playing),
                (
                    wallbuy::spawn_wallbuys,
                    doors::spawn_door_prompts,
                    power::spawn_power_switch,
                    mystery_box::spawn_box,
                    pap::spawn_pap,
                    repair::spawn_repair_points.after(crate::world::barricades::spawn_barricades),
                    reset_focus,
                ),
            )
            .add_systems(
                Update,
                (
                    find_focus,
                    (
                        wallbuy::use_wallbuy,
                        doors::use_door,
                        perks::use_perk_machine,
                        power::use_power_switch,
                        mystery_box::use_box,
                        pap::use_pap,
                        repair::use_repair,
                    ),
                )
                    .chain()
                    .in_set(GameSet::World)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            )
            .add_systems(
                Update,
                (power::animate_power_switch, mystery_box::animate_box, pap::animate_pap, wallbuy::chalk_glow)
                    .in_set(GameSet::World)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

/// Wofür ein Objekt benutzt werden kann.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InteractKind {
    WallBuy(usize),
    Door(u16),
    Perk(Perk),
    Power,
    MysteryBox,
    PackAPunch,
    Barricade(usize),
}

/// Benutzbares Objekt: Punkt in der Welt, Reichweite, optional die Seite, von der aus es
/// benutzt werden kann (Normale).
#[derive(Component, Clone, Copy)]
pub struct Interactable {
    pub kind: InteractKind,
    pub point: Vec3,
    pub radius: f32,
    pub facing: Option<Vec3>,
}

/// Das Objekt, auf das der Spieler gerade schaut.
#[derive(Resource, Default)]
pub struct Focus {
    pub entity: Option<Entity>,
    pub kind: Option<InteractKind>,
    /// F wurde in diesem Frame gedrückt
    pub pressed: bool,
    /// F wird gehalten
    pub held: bool,
}

/// Hinweistext unten in der Bildmitte ("Halte F für ...").
#[derive(Resource, Default)]
pub struct Prompt {
    pub text: Option<String>,
}

impl Prompt {
    pub fn set(&mut self, text: impl Into<String>) {
        self.text = Some(text.into());
    }
}

fn reset_focus(mut focus: ResMut<Focus>, mut prompt: ResMut<Prompt>) {
    *focus = Focus::default();
    prompt.text = None;
}

/// Kosten-Zusatz wie im Original: "[Kosten: 950]"
pub fn cost_text(cost: u32) -> String {
    format!("[Kosten: {cost}]")
}

/// Punkte ausgeben, sonst Ablehnungs-Geräusch. Liefert true bei Erfolg.
pub fn try_buy(score: &mut Score, cost: u32, sfx: &mut MessageWriter<SfxEvent>) -> bool {
    if score.spend(cost) {
        sfx.write(SfxEvent::ui(Sfx::Buy));
        true
    } else {
        sfx.write(SfxEvent::ui(Sfx::Deny));
        false
    }
}

#[allow(clippy::too_many_arguments)]
fn find_focus(
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<DebugConfig>,
    game: Res<InGame>,
    player: Single<(&Transform, &Player, &Health)>,
    items: Query<(Entity, &Interactable)>,
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
) {
    prompt.text = None;
    let (tf, p, h) = *player;
    let pressed = keys.just_pressed(KeyCode::KeyF);
    let held = keys.pressed(KeyCode::KeyF) || (cfg.no_input && cfg.script_keys_held(KeyCode::KeyF));
    focus.pressed = pressed && !game.paused;
    focus.held = held && !game.paused;
    focus.entity = None;
    focus.kind = None;
    if game.paused || h.downed || h.dead {
        return;
    }
    let eye = tf.translation + Vec3::Y * p.eye;
    let look = Quat::from_euler(EulerRot::YXZ, p.yaw, p.pitch, 0.0) * Vec3::NEG_Z;
    let mut best: Option<(f32, Entity, InteractKind)> = None;
    for (e, it) in items.iter() {
        let to = it.point - eye;
        let flat = Vec2::new(to.x, to.z).length();
        if flat > it.radius || to.y.abs() > 2.2 {
            continue;
        }
        if let Some(n) = it.facing {
            if (tf.translation - it.point).dot(n) < -0.05 {
                continue;
            }
        }
        let dir = to.normalize_or_zero();
        let dot = dir.dot(look);
        if flat > 0.9 && dot < 0.35 {
            continue;
        }
        let score = flat * (2.2 - dot);
        if best.map(|b| score < b.0).unwrap_or(true) {
            best = Some((score, e, it.kind));
        }
    }
    if let Some((_, e, k)) = best {
        focus.entity = Some(e);
        focus.kind = Some(k);
    }
}

/// Ausgestellte Waffe in der Welt (Wand, Kiste, Verstärker): die Ego-Hände und Ärmel,
/// die in den Waffenmodellen stecken, werden ausgeblendet.
#[derive(Component)]
pub struct WorldWeapon;

pub fn hide_weapon_hands(
    ready: On<bevy::world_serialization::WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mats: Query<&bevy::gltf::GltfMaterialName>,
) {
    for e in children.iter_descendants(ready.entity) {
        if let Ok(m) = mats.get(e) {
            if matches!(m.0.as_str(), "W_Hands" | "W_HandsSkin" | "W_Sleeve") {
                commands.entity(e).insert(Visibility::Hidden);
            }
        }
    }
}
