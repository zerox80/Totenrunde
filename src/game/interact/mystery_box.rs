//! Mysteriöse Kiste: 950 Punkte (Ausverkauf: 10), Waffen rotieren über der offenen Kiste,
//! die angebotene Waffe kann 12 Sekunden lang genommen werden. Nach einigen Benutzungen
//! erscheint ein Teddy – die Kiste lacht, fliegt davon und taucht an einem anderen Ort
//! wieder auf (die Punkte gibt es zurück). Ein Lichtstrahl zeigt, wo sie steht.

use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use super::{cost_text, try_buy, Focus, InteractKind, Interactable, Prompt};
use crate::audio::{Sfx, SfxEvent};
use crate::base::AppState;
use crate::game::points::Score;
use crate::game::powerups::ActivePowerups;
use crate::weapons::{def, GiveWeapon, Loadout, WEAPONS};
use crate::world::mapdata::{asset_root, MapFile};

pub const BOX_COST: u32 = 950;
pub const FIRE_SALE_COST: u32 = 10;
const ROLL_TIME: f32 = 4.2;
const OFFER_TIME: f32 = 12.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BoxPhase {
    Closed,
    Rolling { t: f32, weapon: usize, next: f32 },
    Offer { t: f32, weapon: usize },
    Teddy { t: f32 },
    Away { t: f32 },
}

#[derive(Resource)]
pub struct MysteryBox {
    pub spot: usize,
    pub uses: u32,
    pub phase: BoxPhase,
    pub root: Entity,
    pub lid: Entity,
    pub beam: Entity,
    pub interact: Entity,
    pub display: Option<Entity>,
    pub teddy: Entity,
}

#[derive(Component)]
pub struct BoxDisplay;

fn spot_transform(map: &MapFile, i: usize) -> Transform {
    let s = &map.box_spots[i];
    Transform::from_translation(Vec3::from_array(s.pos)).with_rotation(Quat::from_rotation_y(s.yaw))
}

fn interact_point(tf: &Transform) -> (Vec3, Vec3) {
    let fwd = tf.rotation * Vec3::NEG_Z;
    (Vec3::new(tf.translation.x, 1.0, tf.translation.z) + fwd * 0.55, fwd)
}

pub fn spawn_box(
    mut commands: Commands,
    map: Res<MapFile>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if map.box_spots.is_empty() {
        return;
    }
    let spot = map.box_spots.iter().position(|s| s.start).unwrap_or(0);
    let tf = spot_transform(&map, spot);
    let root = commands
        .spawn((Name::new("Mysteriöse Kiste"), tf, Visibility::default(), DespawnOnExit(AppState::Playing)))
        .id();
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.2, 0.11),
        perceptual_roughness: 0.8,
        ..default()
    });
    let have_model = asset_root().join("models/mystery_box.gltf").exists();
    if have_model {
        commands.entity(root).with_child((
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/mystery_box.gltf"))),
            Transform::IDENTITY,
        ));
    } else {
        commands.entity(root).with_child((
            Mesh3d(meshes.add(Cuboid::new(1.25, 0.5, 0.6))),
            MeshMaterial3d(wood.clone()),
            Transform::from_xyz(0.0, 0.25, 0.0),
        ));
    }
    // Deckel mit Scharnier an der Rückseite
    let lid = commands
        .spawn((Transform::from_xyz(0.0, 0.5, 0.3), Visibility::default(), ChildOf(root)))
        .id();
    if asset_root().join("models/mystery_box_lid.gltf").exists() {
        commands.entity(lid).with_child((
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/mystery_box_lid.gltf"))),
            Transform::IDENTITY,
        ));
    } else {
        commands.entity(lid).with_child((
            Mesh3d(meshes.add(Cuboid::new(1.28, 0.08, 0.62))),
            MeshMaterial3d(wood),
            Transform::from_xyz(0.0, 0.04, -0.3),
        ));
    }
    // Lichtstrahl in den Himmel
    let beam_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.75, 1.0, 0.05),
        emissive: LinearRgba::rgb(0.25, 0.4, 0.75),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let beam = commands
        .spawn((
            Name::new("Kisten-Lichtstrahl"),
            Mesh3d(meshes.add(Cylinder::new(0.11, 40.0))),
            MeshMaterial3d(beam_mat),
            Transform::from_translation(tf.translation + Vec3::Y * 21.6),
            bevy::light::NotShadowCaster,
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    // Teddy (eigenes Modell oder Ersatz aus Kugeln)
    let teddy = commands
        .spawn((Transform::from_xyz(0.0, 0.3, 0.0), Visibility::Hidden, ChildOf(root)))
        .id();
    if asset_root().join("models/teddy.gltf").exists() {
        commands.entity(teddy).with_child((
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/teddy.gltf"))),
            Transform::IDENTITY,
        ));
    } else {
        let fur = materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.3, 0.18),
            perceptual_roughness: 1.0,
            ..default()
        });
        let s = meshes.add(Sphere::new(1.0));
        for (p, r) in [(Vec3::new(0.0, 0.0, 0.0), 0.16), (Vec3::new(0.0, 0.22, 0.0), 0.11), (Vec3::new(-0.08, 0.31, 0.0), 0.04), (Vec3::new(0.08, 0.31, 0.0), 0.04)] {
            commands.entity(teddy).with_child((
                Mesh3d(s.clone()),
                MeshMaterial3d(fur.clone()),
                Transform::from_translation(p).with_scale(Vec3::splat(r)),
            ));
        }
    }
    let (point, fwd) = interact_point(&tf);
    let interact = commands
        .spawn((
            Interactable {
                kind: InteractKind::MysteryBox,
                point,
                radius: 1.7,
                facing: Some(fwd),
            },
            Transform::from_translation(point),
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    commands.insert_resource(MysteryBox {
        spot,
        uses: 0,
        phase: BoxPhase::Closed,
        root,
        lid,
        beam,
        interact,
        display: None,
        teddy,
    });
}

/// Zufallswaffe nach Gewichten, ohne Waffen, die der Spieler schon trägt.
fn roll_weapon(lo: &Loadout, rng: &mut impl Rng) -> usize {
    let cands: Vec<(usize, f32)> = WEAPONS
        .iter()
        .enumerate()
        .filter(|(i, w)| w.box_weight > 0.0 && lo.has(*i).is_none())
        .map(|(i, w)| (i, w.box_weight))
        .collect();
    let total: f32 = cands.iter().map(|c| c.1).sum();
    if cands.is_empty() || total <= 0.0 {
        return 0;
    }
    let mut x = rng.random_range(0.0..total);
    for (i, w) in &cands {
        if x < *w {
            return *i;
        }
        x -= w;
    }
    cands[cands.len() - 1].0
}

fn any_weapon(rng: &mut impl Rng) -> usize {
    let c: Vec<usize> = WEAPONS.iter().enumerate().filter(|(_, w)| w.box_weight > 0.0).map(|(i, _)| i).collect();
    c[rng.random_range(0..c.len())]
}

#[allow(clippy::too_many_arguments)]
pub fn use_box(
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
    mbox: Option<ResMut<MysteryBox>>,
    mut score: ResMut<Score>,
    pu: Res<ActivePowerups>,
    lo: Single<&Loadout>,
    tfs: Query<&Transform>,
    mut give: MessageWriter<GiveWeapon>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(mut mbox) = mbox else { return };
    if focus.kind != Some(InteractKind::MysteryBox) {
        return;
    }
    let pos = tfs.get(mbox.root).map(|t| t.translation).unwrap_or_default() + Vec3::Y * 0.6;
    let fire_sale = pu.fire_sale > 0.0;
    match mbox.phase {
        BoxPhase::Closed => {
            let cost = if fire_sale { FIRE_SALE_COST } else { BOX_COST };
            prompt.set(format!("Halte F für eine Zufallswaffe {}", cost_text(cost)));
            if focus.pressed && try_buy(&mut score, cost, &mut sfx) {
                let mut rng = rand::rng();
                let teddy_chance = if fire_sale || mbox.uses < 4 {
                    0.0
                } else {
                    (0.12 + 0.05 * (mbox.uses - 4) as f32).min(0.5)
                };
                mbox.uses += 1;
                sfx.write(SfxEvent::at(Sfx::BoxOpen, pos));
                sfx.write(SfxEvent::at(Sfx::BoxMusic, pos));
                if rng.random::<f32>() < teddy_chance {
                    // Teddy: erst normal rollen, dann statt einer Waffe der Teddy
                    mbox.phase = BoxPhase::Rolling { t: 0.0, weapon: usize::MAX, next: 0.0 };
                    score.points += cost; // wie im Original: Punkte zurück
                    score.pending_popups.push(cost as i64);
                } else {
                    mbox.phase = BoxPhase::Rolling { t: 0.0, weapon: roll_weapon(&lo, &mut rng), next: 0.0 };
                }
                focus.pressed = false;
            }
        }
        BoxPhase::Offer { weapon, .. } => {
            prompt.set(format!("Halte F für {}", def(weapon).name));
            if focus.pressed {
                give.write(GiveWeapon { weapon, upgraded: false });
                mbox.phase = BoxPhase::Closed;
                sfx.write(SfxEvent::at(Sfx::BoxClose, pos));
                focus.pressed = false;
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
pub fn animate_box(
    mut commands: Commands,
    time: Res<Time>,
    map: Res<MapFile>,
    mbox: Option<ResMut<MysteryBox>>,
    asset_server: Res<AssetServer>,
    mut tfs: Query<&mut Transform>,
    mut vis: Query<&mut Visibility>,
    mut items: Query<&mut Interactable>,
    mut pu: ResMut<ActivePowerups>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(mut mbox) = mbox else { return };
    let dt = time.delta_secs();
    let mut rng = rand::rng();
    let root_tf = tfs.get(mbox.root).map(|t| *t).unwrap_or_default();
    let base = root_tf.translation;
    let mut lid_open = 0.0;
    let mut display_h: Option<f32> = None;
    let mut want_weapon: Option<usize> = None;
    match mbox.phase {
        BoxPhase::Closed => {}
        BoxPhase::Rolling { t, weapon, next } => {
            let t = t + dt;
            lid_open = (t / 0.4).min(1.0);
            display_h = Some(0.35 + 0.6 * (t / ROLL_TIME).min(1.0));
            let mut next = next - dt;
            let mut current = mbox.display.and_then(|e| tfs.get(e).ok()).map(|_| ());
            if next <= 0.0 || current.is_none() {
                // Waffen wechseln immer langsamer
                next = 0.06 + 0.4 * (t / ROLL_TIME).powi(2);
                want_weapon = Some(any_weapon(&mut rng));
                current = None;
            }
            let _ = current;
            if t >= ROLL_TIME {
                if weapon == usize::MAX {
                    mbox.phase = BoxPhase::Teddy { t: 0.0 };
                    sfx.write(SfxEvent::at(Sfx::BoxLaugh, base + Vec3::Y));
                    if let Some(d) = mbox.display.take() {
                        commands.entity(d).despawn();
                    }
                } else {
                    mbox.phase = BoxPhase::Offer { t: 0.0, weapon };
                    want_weapon = Some(weapon);
                }
            } else {
                mbox.phase = BoxPhase::Rolling { t, weapon, next };
            }
        }
        BoxPhase::Offer { t, weapon } => {
            let t = t + dt;
            lid_open = 1.0;
            // Waffe sinkt in den letzten Sekunden zurück in die Kiste
            let sink = ((t - (OFFER_TIME - 2.0)) / 2.0).clamp(0.0, 1.0);
            display_h = Some(0.95 - 0.6 * sink);
            if t >= OFFER_TIME {
                mbox.phase = BoxPhase::Closed;
                sfx.write(SfxEvent::at(Sfx::BoxClose, base + Vec3::Y * 0.5));
            } else {
                mbox.phase = BoxPhase::Offer { t, weapon };
            }
        }
        BoxPhase::Teddy { t } => {
            let t = t + dt;
            lid_open = 1.0;
            if let Ok(mut tt) = tfs.get_mut(mbox.teddy) {
                tt.translation = Vec3::new(0.0, 0.3 + 0.7 * (t / 1.0).min(1.0), 0.0);
                tt.rotation = Quat::from_rotation_y(PI + (t * 3.0).sin() * 0.3);
            }
            if let Ok(mut v) = vis.get_mut(mbox.teddy) {
                *v = Visibility::Inherited;
            }
            if t >= 2.6 {
                mbox.phase = BoxPhase::Away { t: 0.0 };
                sfx.write(SfxEvent::at(Sfx::BoxMove, base + Vec3::Y));
                if let Ok(mut v) = vis.get_mut(mbox.teddy) {
                    *v = Visibility::Hidden;
                }
            } else {
                mbox.phase = BoxPhase::Teddy { t };
            }
        }
        BoxPhase::Away { t } => {
            let t = t + dt;
            // hochfliegen, verschwinden, an neuem Ort wieder herabkommen
            if t < 2.0 {
                if let Ok(mut rt) = tfs.get_mut(mbox.root) {
                    let k = t / 2.0;
                    rt.translation.y = map.box_spots[mbox.spot].pos[1] + k * k * 6.0;
                    rt.rotation = Quat::from_rotation_y(map.box_spots[mbox.spot].yaw + k * k * 8.0);
                }
                if let Ok(mut v) = vis.get_mut(mbox.beam) {
                    *v = Visibility::Hidden;
                }
            } else if t < 2.1 {
                // neuen Ort wählen
                let n = map.box_spots.len();
                if n > 1 {
                    let mut ns = mbox.spot;
                    while ns == mbox.spot {
                        ns = rng.random_range(0..n);
                    }
                    mbox.spot = ns;
                }
                mbox.uses = 0;
                pu.fire_sale_allowed = true;
            } else {
                let tf = spot_transform(&map, mbox.spot);
                let k = ((t - 2.1) / 1.2).min(1.0);
                if let Ok(mut rt) = tfs.get_mut(mbox.root) {
                    rt.translation = tf.translation + Vec3::Y * (1.0 - k) * (1.0 - k) * 6.0;
                    rt.rotation = tf.rotation;
                }
                if let Ok(mut bt) = tfs.get_mut(mbox.beam) {
                    bt.translation = tf.translation + Vec3::Y * 21.6;
                }
                if k >= 1.0 {
                    if let Ok(mut v) = vis.get_mut(mbox.beam) {
                        *v = Visibility::Inherited;
                    }
                    if let Ok(mut it) = items.get_mut(mbox.interact) {
                        let (p, f) = interact_point(&tf);
                        it.point = p;
                        it.facing = Some(f);
                    }
                    mbox.phase = BoxPhase::Closed;
                }
            }
            if let BoxPhase::Away { .. } = mbox.phase {
                mbox.phase = BoxPhase::Away { t };
            }
        }
    }
    // Deckel
    if let Ok(mut lt) = tfs.get_mut(mbox.lid) {
        let cur = lt.rotation.to_euler(EulerRot::XYZ).0;
        let goal = -lid_open * 1.9;
        let a = cur + (goal - cur) * (dt * 10.0).min(1.0);
        lt.rotation = Quat::from_rotation_x(a);
    }
    // angezeigte Waffe
    match display_h {
        None => {
            if let Some(d) = mbox.display.take() {
                commands.entity(d).despawn();
            }
        }
        Some(h) => {
            if let Some(w) = want_weapon {
                if let Some(d) = mbox.display.take() {
                    commands.entity(d).despawn();
                }
                let e = commands
                    .spawn((
                        BoxDisplay,
                        super::WorldWeapon,
                        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(def(w).model()))),
                        Transform::from_xyz(0.0, h, 0.0).with_rotation(Quat::from_rotation_y(PI / 2.0)),
                        ChildOf(mbox.root),
                    ))
                    .observe(super::hide_weapon_hands)
                    .id();
                mbox.display = Some(e);
            } else if let Some(d) = mbox.display {
                if let Ok(mut dt_) = tfs.get_mut(d) {
                    dt_.translation.y = h;
                }
            }
        }
    }
}
