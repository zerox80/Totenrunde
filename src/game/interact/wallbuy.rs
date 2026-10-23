//! Wandwaffen: an der Wand hängende Waffe (mit Kreideumriss), kaufen für den Wandpreis,
//! Munition nachkaufen für die Hälfte (aufgewertete Waffen: 4500).

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::{cost_text, try_buy, Focus, InteractKind, Interactable, Prompt};
use crate::audio::SfxEvent;
use crate::base::AppState;
use crate::game::points::Score;
use crate::weapons::{def, weapon_index, GiveWeapon, Loadout};
use crate::world::mapdata::{asset_root, MapFile};

pub const UPGRADED_AMMO_COST: u32 = 4500;

/// Kreideumriss hinter der Wandwaffe (leuchtet leicht, wenn man davorsteht)
#[derive(Component)]
pub struct Chalk {
    pub point: Vec3,
}

pub fn spawn_wallbuys(
    mut commands: Commands,
    map: Res<MapFile>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for spec in &map.wallbuys {
        // eigenes Material je Wandwaffe, damit nur die angesehene aufleuchtet
        let chalk_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.92, 0.92, 0.88, 0.55),
            emissive: LinearRgba::rgb(0.25, 0.25, 0.23),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let Some(w) = weapon_index(&spec.weapon) else {
            warn!("Wandwaffe '{}' unbekannt", spec.weapon);
            continue;
        };
        let pos = Vec3::from_array(spec.pos);
        let rot = Quat::from_rotation_y(spec.yaw);
        let fwd = rot * Vec3::NEG_Z;
        let right = rot * Vec3::X;
        let point = pos + fwd * 0.3;
        commands.spawn((
            Name::new(format!("Wandwaffe {}", def(w).name)),
            Interactable {
                kind: InteractKind::WallBuy(w),
                point: Vec3::new(point.x, 1.0, point.z),
                radius: 1.6,
                facing: Some(fwd),
            },
            Transform::from_translation(pos),
            DespawnOnExit(AppState::Playing),
        ));
        // Kreideumriss: eigenes Modell (Blender) oder ein Rahmen aus Kreidestrichen
        let chalk_model = format!("models/chalk_{}.gltf", spec.weapon);
        let chalk_tf = Transform::from_translation(pos + fwd * 0.012).with_rotation(rot);
        if asset_root().join(&chalk_model).exists() {
            commands.spawn((
                Chalk { point },
                WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(chalk_model))),
                chalk_tf,
                DespawnOnExit(AppState::Playing),
            ));
        } else {
            let len = 0.9;
            for (c, size) in [
                (Vec3::new(0.0, 0.2, 0.0), Vec3::new(len, 0.02, 0.002)),
                (Vec3::new(0.0, -0.2, 0.0), Vec3::new(len, 0.02, 0.002)),
                (Vec3::new(len / 2.0, 0.0, 0.0), Vec3::new(0.02, 0.42, 0.002)),
                (Vec3::new(-len / 2.0, 0.0, 0.0), Vec3::new(0.02, 0.42, 0.002)),
            ] {
                commands.spawn((
                    Chalk { point },
                    Mesh3d(meshes.add(Cuboid::from_size(size))),
                    MeshMaterial3d(chalk_mat.clone()),
                    Transform::from_translation(pos + fwd * 0.012 + right * c.x + Vec3::Y * c.y).with_rotation(rot),
                    DespawnOnExit(AppState::Playing),
                ));
            }
        }
        // Die Waffe selbst hängt flach an der Wand (Lauf entlang der Wand)
        commands
            .spawn((
                Name::new("Wandwaffe Modell"),
                super::WorldWeapon,
                WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(def(w).model()))),
                Transform::from_translation(pos + fwd * 0.05).with_rotation(rot * Quat::from_rotation_y(FRAC_PI_2)),
                DespawnOnExit(AppState::Playing),
            ))
            .observe(super::hide_weapon_hands);
    }
}

pub fn chalk_glow(
    focus: Res<Focus>,
    items: Query<&Interactable>,
    chalks: Query<(&Chalk, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let focused = focus.entity.and_then(|e| items.get(e).ok()).map(|i| i.point);
    for (c, m) in chalks.iter() {
        let on = focused.map(|p| p.distance(Vec3::new(c.point.x, 1.0, c.point.z)) < 0.1).unwrap_or(false);
        let k = if on { 0.7 } else { 0.22 };
        // erst lesen, nur bei Änderung schreiben (get_mut markiert das Material als geändert)
        let differs = materials.get(&m.0).map(|mat| (mat.emissive.red - k).abs() > 0.01).unwrap_or(false);
        if differs {
            if let Some(mut mat) = materials.get_mut(&m.0) {
                mat.emissive = LinearRgba::rgb(k, k, k * 0.95);
            }
        }
    }
}

pub fn use_wallbuy(
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
    mut score: ResMut<Score>,
    lo: Single<&Loadout>,
    mut give: MessageWriter<GiveWeapon>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(InteractKind::WallBuy(w)) = focus.kind else { return };
    let d = def(w);
    match lo.has(w) {
        Some(i) => {
            let slot = lo.slots[i];
            let cost = if slot.upgraded { UPGRADED_AMMO_COST } else { d.ammo_cost() };
            let full = slot.reserve >= d.reserve_size(slot.upgraded) && slot.mag >= d.mag_size(slot.upgraded);
            if full {
                prompt.set(format!("{} – Munition voll", d.display_name(slot.upgraded)));
                return;
            }
            prompt.set(format!("Halte F für Munition {}", cost_text(cost)));
            if focus.pressed && try_buy(&mut score, cost, &mut sfx) {
                give.write(GiveWeapon { weapon: w, upgraded: slot.upgraded });
                focus.pressed = false;
            }
        }
        None => {
            prompt.set(format!("Halte F für {} {}", d.name, cost_text(d.wall_cost)));
            if focus.pressed && try_buy(&mut score, d.wall_cost, &mut sfx) {
                give.write(GiveWeapon { weapon: w, upgraded: false });
                focus.pressed = false;
            }
        }
    }
}
