//! Prozedural animierte Ego-Waffe wie in Call of Duty: Waffe unten rechts mit Händen,
//! Zielen über Kimme und Korn, Mausträgheit, Wippen, Sprinthaltung, Rückstoß,
//! Nachlade-Animation (Magazin raus/rein), Waffenwechsel, Messer und Mündungsfeuer.

use std::f32::consts::PI;

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use rand::Rng;

use super::assets::WeaponAssets;
use super::defs::{def, WeaponClass};
use super::loadout::{Loadout, WState};
use crate::player::{Player, PlayerCamera};

/// Maßstab der Ego-Waffe: sie wird verkleinert und nah an die Kamera gesetzt, damit sie
/// nie in Wände eintaucht (wirkt auf dem Bildschirm identisch).
pub const VM_SCALE: f32 = 0.22;

#[derive(Component)]
pub struct ViewModelRoot {
    pub sway: Vec2,
    pub sprint: f32,
    pub showing: Option<(usize, bool)>,
    pub scene: Option<Entity>,
    pub knife: Entity,
    pub time: f32,
}

#[derive(Component)]
pub struct ViewModelScene {
    pub weapon: usize,
    pub upgraded: bool,
}

#[derive(Component, Default)]
pub struct ViewModelParts {
    pub mag: Option<(Entity, Transform)>,
    pub arm_l: Option<(Entity, Transform)>,
    pub slide: Option<(Entity, Transform)>,
    pub muzzle: Option<Entity>,
}

#[derive(Component)]
pub struct MuzzleFlash;

#[derive(Component)]
pub struct MuzzleLight;

/// Ego-Waffe (Wurzel unter der Kamera), Messer und Mündungsfeuer-Licht anlegen.
pub fn spawn_viewmodel(
    mut commands: Commands,
    cam: Query<Entity, With<PlayerCamera>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(ce) = cam.single() else { return };
    let knife = commands
        .spawn((
            Name::new("Messer"),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/weapons/knife.gltf"))),
            Transform::from_xyz(-0.1, -0.1, -0.2),
            Visibility::Hidden,
        ))
        .observe(on_vm_ready)
        .id();
    let root = commands
        .spawn((
            Name::new("Ego-Waffe"),
            ViewModelRoot {
                sway: Vec2::ZERO,
                sprint: 0.0,
                showing: None,
                scene: None,
                knife,
                time: 0.0,
            },
            Transform::from_scale(Vec3::splat(VM_SCALE)),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).add_child(knife);
    commands.entity(ce).add_child(root);
    // Mündungsfeuer-Licht in der Welt
    commands.entity(ce).with_child((
        MuzzleLight,
        PointLight {
            color: Color::srgb(1.0, 0.75, 0.45),
            intensity: 0.0,
            range: 9.0,
            ..default()
        },
        Transform::from_xyz(0.1, -0.1, -0.7),
    ));
}

/// Tauscht das Modell der Ego-Waffe, wenn sich die Waffe ändert.
pub fn sync_viewmodel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player: Single<&Loadout>,
    mut roots: Query<(Entity, &mut ViewModelRoot)>,
) {
    let lo = *player;
    let want = lo.current_slot().map(|s| (s.weapon, s.upgraded));
    for (re, mut root) in roots.iter_mut() {
        if root.showing == want {
            continue;
        }
        if let Some(old) = root.scene.take() {
            commands.entity(old).despawn();
        }
        root.showing = want;
        if let Some((w, up)) = want {
            let e = commands
                .spawn((
                    ViewModelScene { weapon: w, upgraded: up },
                    WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(def(w).model()))),
                    Transform::IDENTITY,
                    ChildOf(re),
                ))
                .observe(on_vm_ready)
                .id();
            root.scene = Some(e);
        }
    }
}

fn on_vm_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    names: Query<(&Name, &Transform)>,
    meshes: Query<(), With<Mesh3d>>,
    mats: Query<&bevy::gltf::GltfMaterialName>,
    scenes: Query<&ViewModelScene>,
    wa: Res<WeaponAssets>,
) {
    let upgraded = scenes.get(ready.entity).map(|s| s.upgraded).unwrap_or(false);
    let mut parts = ViewModelParts::default();
    for e in children.iter_descendants(ready.entity) {
        if meshes.contains(e) {
            commands.entity(e).insert(NotShadowCaster);
            if upgraded {
                let keep = mats
                    .get(e)
                    .map(|m| m.0.starts_with("W_Hand") || m.0.starts_with("W_Sleeve") || m.0.starts_with("W_Glow"))
                    .unwrap_or(false);
                if !keep {
                    commands.entity(e).insert(MeshMaterial3d(wa.camo.clone()));
                }
            }
        }
        if let Ok((n, tf)) = names.get(e) {
            match n.as_str() {
                "Mag" => parts.mag = Some((e, *tf)),
                "ArmL" => parts.arm_l = Some((e, *tf)),
                "Slide" => parts.slide = Some((e, *tf)),
                "Muzzle" => {
                    parts.muzzle = Some(e);
                    commands.entity(e).with_child((
                        MuzzleFlash,
                        Mesh3d(wa.flash_mesh.clone()),
                        MeshMaterial3d(wa.flash_mat.clone()),
                        Transform::from_scale(Vec3::splat(0.0001)),
                        NotShadowCaster,
                    ));
                }
                _ => {}
            }
        }
    }
    commands.entity(ready.entity).insert(parts);
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[allow(clippy::too_many_arguments)]
pub fn animate_viewmodel(
    time: Res<Time>,
    motion: Res<AccumulatedMouseMotion>,
    player: Single<(&Player, &Loadout)>,
    mut roots: Query<(&mut ViewModelRoot, &mut Transform), Without<ViewModelScene>>,
    scenes: Query<&ViewModelParts>,
    mut part_tfs: Query<&mut Transform, (Without<ViewModelRoot>, Without<MuzzleFlash>)>,
    mut flashes: Query<(&mut Transform, &mut Visibility), (With<MuzzleFlash>, Without<ViewModelRoot>)>,
    mut lights: Query<&mut PointLight, With<MuzzleLight>>,
    mut vis: Query<&mut Visibility, (Without<MuzzleFlash>, Without<ViewModelRoot>)>,
) {
    let dt = time.delta_secs();
    let (pl, lo) = *player;
    let mut rng = rand::rng();
    for (mut root, mut tf) in roots.iter_mut() {
        root.time += dt;
        let Some(slot) = lo.current_slot() else {
            // keine Waffe: Wurzel ausblenden bis auf das Messer
            tf.translation = Vec3::new(0.0, -0.4, -0.3) * VM_SCALE;
            continue;
        };
        let d = def(slot.weapon);
        let ads = ease(lo.ads);
        let mut pos = d.hip.lerp(d.ads, ads);
        let mut rot = Quat::IDENTITY;

        // Mausträgheit
        let target_sway = Vec2::new(-motion.delta.x, -motion.delta.y) * 0.0006 * (1.0 - ads * 0.85);
        root.sway = root.sway.lerp(target_sway.clamp(Vec2::splat(-0.04), Vec2::splat(0.04)), (dt * 10.0).min(1.0));
        pos += Vec3::new(root.sway.x, root.sway.y, 0.0);
        rot *= Quat::from_euler(EulerRot::YXZ, root.sway.x * 2.0, root.sway.y * 1.5, root.sway.x * 1.5);

        // Wippen beim Laufen
        let bob = pl.bob * (1.0 - ads * 0.92);
        let ph = pl.bob_phase;
        pos += Vec3::new(ph.sin() * 0.012 * bob, -(ph * 2.0).sin().abs() * 0.01 * bob, 0.0);
        rot *= Quat::from_rotation_z(ph.sin() * 0.02 * bob);
        // Atmen
        let breathe = (root.time * 1.6).sin() * 0.0025 * (1.0 - ads * 0.8);
        pos.y += breathe;

        // Sprinten
        let sprint_target = if pl.sprinting { 1.0 } else { 0.0 };
        root.sprint += (sprint_target - root.sprint) * (dt * 9.0).min(1.0);
        let s = ease(root.sprint);
        pos += Vec3::new(-0.06, -0.05, 0.03) * s;
        rot *= Quat::from_euler(EulerRot::YXZ, 0.75 * s, -0.25 * s, 0.35 * s);

        // Rückstoß
        let k = lo.kick * lo.kick;
        let kick_amt = match d.class {
            WeaponClass::Shotgun | WeaponClass::Sniper => 1.8,
            WeaponClass::Pistol => 1.1,
            _ => 0.7,
        };
        pos.z += 0.035 * k * kick_amt;
        pos.y += 0.008 * k * kick_amt;
        rot *= Quat::from_rotation_x(0.12 * k * kick_amt * (1.0 - ads * 0.6));

        // Zustände
        let mut mag_off = 0.0f32;
        let mut mag_hidden = false;
        match lo.state {
            WState::Reloading { t, dur } => {
                let u = t / dur;
                let w = ease((u / 0.15).min(1.0)) * ease(((1.0 - u) / 0.15).min(1.0));
                pos += Vec3::new(-0.02, -0.05, 0.02) * w;
                rot *= Quat::from_euler(EulerRot::YXZ, 0.2 * w, 0.25 * w, -0.45 * w);
                // Magazin raus / rein
                if u > 0.2 && u < 0.75 {
                    let m = if u < 0.35 { (u - 0.2) / 0.15 } else if u < 0.55 { 1.0 } else { 1.0 - (u - 0.55) / 0.2 };
                    mag_off = ease(m) * 0.25;
                    mag_hidden = u > 0.33 && u < 0.5;
                }
            }
            WState::ShellReload { .. } => {
                pos += Vec3::new(-0.02, -0.04, 0.01);
                rot *= Quat::from_euler(EulerRot::YXZ, 0.15, 0.2, -0.35);
            }
            WState::Lowering { t, .. } => {
                let u = ease(t / 0.22);
                pos.y -= 0.3 * u;
                rot *= Quat::from_rotation_x(-0.6 * u);
            }
            WState::Raising { t } => {
                let u = 1.0 - ease(t / 0.35);
                pos.y -= 0.3 * u;
                rot *= Quat::from_rotation_x(-0.6 * u);
            }
            WState::Busy { t, dur } => {
                let u = ease((t / 0.25).min(1.0)) * ease(((dur - t) / 0.25).min(1.0));
                pos.y -= 0.35 * u;
                rot *= Quat::from_rotation_x(-0.7 * u);
            }
            WState::Melee { t, .. } => {
                let u = (t / 0.55).min(1.0);
                let w = (u * PI).sin();
                pos += Vec3::new(0.05, -0.12, 0.05) * w;
                rot *= Quat::from_euler(EulerRot::YXZ, -0.3 * w, -0.3 * w, 0.4 * w);
            }
            WState::Throw { t, .. } => {
                let w = ((t / 0.6).min(1.0) * PI).sin();
                pos.y -= 0.25 * w;
                rot *= Quat::from_rotation_x(-0.5 * w);
            }
            _ => {}
        }
        tf.translation = pos * VM_SCALE;
        tf.rotation = rot;
        tf.scale = Vec3::splat(VM_SCALE);

        // Messer
        if let Ok(mut v) = vis.get_mut(root.knife) {
            *v = if matches!(lo.state, WState::Melee { .. }) { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let WState::Melee { t, .. } = lo.state {
            if let Ok(mut ktf) = part_tfs.get_mut(root.knife) {
                let u = (t / 0.55).min(1.0);
                let swing = if u < 0.25 { u / 0.25 } else { 1.0 - ((u - 0.25) / 0.75) };
                let a = ease(u.min(0.4) / 0.4);
                ktf.translation = Vec3::new(0.25 - 0.45 * a, -0.12 + 0.05 * swing, -0.32 - 0.08 * swing) - pos;
                ktf.rotation = Quat::from_euler(EulerRot::YXZ, 0.9 - 1.6 * a, 0.1, -1.2 + 0.4 * swing) * rot.inverse();
            }
        }

        // Teile der aktuellen Waffe
        if let Some(se) = root.scene {
            if let Ok(parts) = scenes.get(se) {
                if let Some((m, rest)) = parts.mag {
                    if let Ok(mut mt) = part_tfs.get_mut(m) {
                        mt.translation = rest.translation - Vec3::Y * mag_off;
                        mt.scale = if mag_hidden { Vec3::splat(0.001) } else { rest.scale };
                    }
                }
                if let Some((a, rest)) = parts.arm_l {
                    if let Ok(mut at) = part_tfs.get_mut(a) {
                        at.translation = rest.translation - Vec3::new(0.0, mag_off * 0.8, mag_off * -0.3);
                    }
                }
                if let Some((sl, rest)) = parts.slide {
                    if let Ok(mut st) = part_tfs.get_mut(sl) {
                        let back = if slot.mag == 0 { 0.03 } else { 0.03 * k };
                        st.translation = rest.translation + Vec3::Z * back;
                    }
                }
            }
        }
    }
    // Mündungsfeuer
    for (mut ftf, mut v) in flashes.iter_mut() {
        if lo.flash > 0.0 {
            *v = Visibility::Inherited;
            let s = rng.random_range(0.12..0.2);
            ftf.scale = Vec3::splat(s);
            ftf.rotation = Quat::from_rotation_z(rng.random_range(0.0..PI));
        } else {
            *v = Visibility::Hidden;
        }
    }
    for mut l in lights.iter_mut() {
        l.intensity = if lo.flash > 0.0 { 250000.0 } else { 0.0 };
    }
}

/// Perk-Flasche in der Ego-Ansicht (beim Trinken).
#[derive(Component)]
pub struct PerkBottle;

pub fn spawn_bottle(mut commands: Commands, cam: Query<Entity, With<PlayerCamera>>, asset_server: Res<AssetServer>) {
    let Ok(ce) = cam.single() else { return };
    commands.entity(ce).with_child((
        Name::new("Perk-Flasche"),
        PerkBottle,
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/weapons/perk_bottle.gltf"))),
        Transform::from_scale(Vec3::splat(VM_SCALE)),
        Visibility::Hidden,
    ));
}

/// Flasche hochnehmen, ansetzen, trinken, absetzen (Dauer = Trinkzeit des Perks).
pub fn animate_bottle(
    perks: Res<crate::game::perks::Perks>,
    lo: Single<&Loadout>,
    mut q: Query<(&mut Transform, &mut Visibility), With<PerkBottle>>,
) {
    let drinking = perks.drinking.is_some();
    for (mut tf, mut vis) in q.iter_mut() {
        let WState::Busy { t, dur } = lo.state else {
            *vis = Visibility::Hidden;
            continue;
        };
        if !drinking {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        let up = ease((t / 0.5).min(1.0)) * ease(((dur - t) / 0.45).clamp(0.0, 1.0));
        let tilt = ease(((t - 0.5) / 0.6).clamp(0.0, 1.0)) * ease(((dur - 0.45 - t) / 0.4).clamp(0.0, 1.0));
        let low = Vec3::new(0.16, -0.42, -0.36);
        let mouth = Vec3::new(0.07, -0.13, -0.33);
        tf.translation = low.lerp(mouth, up) * VM_SCALE;
        tf.rotation = Quat::from_euler(EulerRot::YXZ, -0.2 * up, 0.75 * tilt + 0.1 * up, 0.25 * up);
        tf.scale = Vec3::splat(VM_SCALE);
    }
}
