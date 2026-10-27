//! Waffen-Verstärker ("Pack-a-Punch"): braucht Strom, kostet 5000 Punkte. Die Waffe wird
//! eingezogen, bearbeitet und kommt aufgewertet wieder heraus – sie muss innerhalb von
//! 15 Sekunden genommen werden, sonst ist sie weg.

use bevy::prelude::*;

use super::{Focus, InteractKind, Interactable, Prompt, cost_text, try_buy};
use crate::audio::{Sfx, SfxEvent};
use crate::base::AppState;
use crate::game::points::Score;
use crate::weapons::{GiveWeapon, Loadout, WeaponSlot, def, take_current};
use crate::world::mapdata::{MapFile, asset_exists};
use crate::world::power::Power;

pub const PAP_COST: u32 = 5000;
const WORK_TIME: f32 = 3.6;
const TAKE_TIME: f32 = 15.0;

#[derive(Clone, Copy, Debug)]
pub enum PapPhase {
    Idle,
    Working { t: f32, slot: WeaponSlot },
    Ready { t: f32, slot: WeaponSlot },
}

#[derive(Resource)]
pub struct PapMachine {
    pub phase: PapPhase,
    pub root: Entity,
    pub light: Entity,
    pub display: Option<Entity>,
}

pub fn spawn_pap(
    mut commands: Commands,
    map: Res<MapFile>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(pl) = &map.pap else { return };
    let tf = pl.transform();
    let fwd = pl.forward();
    let root = commands
        .spawn((
            Name::new("Waffen-Verstärker"),
            tf,
            Visibility::default(),
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    if asset_exists("models/pap.gltf") {
        commands.entity(root).with_child((
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/pap.gltf")),
            ),
            Transform::IDENTITY,
        ));
    } else {
        let m = materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.1, 0.14),
            metallic: 0.7,
            perceptual_roughness: 0.4,
            ..default()
        });
        commands.entity(root).with_child((
            Mesh3d(meshes.add(Cuboid::new(1.7, 1.25, 1.0))),
            MeshMaterial3d(m),
            Transform::from_xyz(0.0, 0.62, 0.0),
        ));
    }
    let light = commands
        .spawn((
            PointLight {
                color: Color::srgb(0.65, 0.3, 1.0),
                intensity: 0.0,
                range: 7.0,
                ..default()
            },
            Transform::from_xyz(0.0, 1.5, -0.8),
            ChildOf(root),
        ))
        .id();
    let point = Vec3::new(tf.translation.x, 1.0, tf.translation.z) + fwd * 0.9;
    commands.spawn((
        Interactable {
            kind: InteractKind::PackAPunch,
            point,
            radius: 1.8,
            facing: Some(fwd),
        },
        Transform::from_translation(point),
        DespawnOnExit(AppState::Playing),
    ));
    commands.insert_resource(PapMachine {
        phase: PapPhase::Idle,
        root,
        light,
        display: None,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn use_pap(
    mut commands: Commands,
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
    pap: Option<ResMut<PapMachine>>,
    power: Res<Power>,
    mut score: ResMut<Score>,
    mut lo: Single<&mut Loadout>,
    asset_server: Res<AssetServer>,
    mut give: MessageWriter<GiveWeapon>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(mut pap) = pap else { return };
    if focus.kind != Some(InteractKind::PackAPunch) {
        return;
    }
    if !power.on {
        prompt.set("Der Strom muss eingeschaltet sein");
        return;
    }
    match pap.phase {
        PapPhase::Idle => {
            let Some(slot) = lo.current_slot().copied() else {
                return;
            };
            if slot.upgraded {
                prompt.set("Diese Waffe ist bereits aufgewertet");
                return;
            }
            prompt.set(format!(
                "Halte F, um die Waffe aufzuwerten {}",
                cost_text(PAP_COST)
            ));
            if focus.pressed && lo.can_act() && try_buy(&mut score, PAP_COST, &mut sfx) {
                if let Some(slot) = take_current(&mut lo) {
                    pap.phase = PapPhase::Working { t: 0.0, slot };
                    sfx.write(SfxEvent::ui(Sfx::PapStart));
                    let e = commands
                        .spawn((
                            super::WorldWeapon,
                            WorldAssetRoot(asset_server.load(
                                GltfAssetLabel::Scene(0).from_asset(def(slot.weapon).model()),
                            )),
                            Transform::from_xyz(0.0, 1.05, -0.6)
                                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                            ChildOf(pap.root),
                        ))
                        .observe(super::hide_weapon_hands)
                        .id();
                    pap.display = Some(e);
                }
                focus.pressed = false;
            }
        }
        PapPhase::Ready { slot, .. } => {
            prompt.set(format!("Halte F für {}", def(slot.weapon).pap_name));
            if focus.pressed {
                give.write(GiveWeapon {
                    weapon: slot.weapon,
                    upgraded: true,
                });
                pap.phase = PapPhase::Idle;
                if let Some(d) = pap.display.take() {
                    commands.entity(d).despawn();
                }
                focus.pressed = false;
            }
        }
        PapPhase::Working { .. } => {}
    }
}

pub fn animate_pap(
    mut commands: Commands,
    time: Res<Time>,
    power: Res<Power>,
    pap: Option<ResMut<PapMachine>>,
    mut tfs: Query<&mut Transform>,
    mut lights: Query<&mut PointLight>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(mut pap) = pap else { return };
    let dt = time.delta_secs();
    let mut glow = if power.on { 60000.0 } else { 0.0 };
    match pap.phase {
        PapPhase::Idle => {}
        PapPhase::Working { t, slot } => {
            let t = t + dt;
            glow = 160000.0 + (t * 25.0).sin().abs() * 120000.0;
            // Waffe wird hineingezogen
            if let Some(d) = pap.display {
                if let Ok(mut d) = tfs.get_mut(d) {
                    d.translation.z = -0.6 + 0.6 * (t / 1.0).min(1.0);
                }
            }
            if t >= WORK_TIME {
                pap.phase = PapPhase::Ready { t: 0.0, slot };
                sfx.write(SfxEvent::ui(Sfx::PapDone));
            } else {
                pap.phase = PapPhase::Working { t, slot };
            }
        }
        PapPhase::Ready { t, slot } => {
            let t = t + dt;
            glow = 100000.0;
            if let Some(d) = pap.display {
                if let Ok(mut d) = tfs.get_mut(d) {
                    // kommt heraus, zieht sich am Ende wieder zurück
                    let out = (t / 0.8).min(1.0)
                        * (1.0 - ((t - (TAKE_TIME - 1.5)) / 1.5).clamp(0.0, 1.0));
                    d.translation.z = -0.6 * out;
                }
            }
            if t >= TAKE_TIME {
                pap.phase = PapPhase::Idle;
                if let Some(d) = pap.display.take() {
                    commands.entity(d).despawn();
                }
            } else {
                pap.phase = PapPhase::Ready { t, slot };
            }
        }
    }
    if let Ok(mut l) = lights.get_mut(pap.light) {
        l.intensity += (glow - l.intensity) * (dt * 6.0).min(1.0);
    }
}
