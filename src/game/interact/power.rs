//! Stromschalter: großer Hebel an der Wand. Einmal umlegen – Perks, Waffen-Verstärker und
//! die Lampen der Karte bekommen Strom.

use bevy::prelude::*;

use super::{Focus, InteractKind, Interactable, Prompt};
use crate::audio::{Sfx, SfxEvent};
use crate::base::AppState;
use crate::base::debug::DebugConfig;
use crate::world::mapdata::{MapFile, asset_exists};
use crate::world::power::Power;

#[derive(Component)]
pub struct PowerLever {
    pub t: f32,
}

pub fn spawn_power_switch(
    mut commands: Commands,
    map: Res<MapFile>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cfg: Res<DebugConfig>,
    mut power: ResMut<Power>,
    time: Res<Time>,
) {
    if std::env::var("TR_POWER").is_ok() || cfg.start_power {
        power.on = true;
        power.since = time.elapsed_secs();
    }
    let Some(pl) = &map.power else { return };
    let pos = pl.pos();
    let rot = Quat::from_rotation_y(pl.yaw);
    let fwd = rot * Vec3::NEG_Z;
    commands.spawn((
        Name::new("Stromschalter"),
        Interactable {
            kind: InteractKind::Power,
            point: Vec3::new(pos.x, 1.0, pos.z) + fwd * 0.3,
            radius: 1.7,
            facing: Some(fwd),
        },
        Transform::from_translation(pos),
        DespawnOnExit(AppState::Playing),
    ));
    let root = commands
        .spawn((
            Name::new("Stromkasten"),
            Transform::from_translation(pos).with_rotation(rot),
            Visibility::default(),
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    if asset_exists("models/power_switch.gltf") {
        commands.entity(root).with_child((
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/power_switch.gltf")),
            ),
            Transform::IDENTITY,
        ));
    } else {
        let metal = materials.add(StandardMaterial {
            base_color: Color::srgb(0.32, 0.34, 0.3),
            metallic: 0.6,
            perceptual_roughness: 0.55,
            ..default()
        });
        commands.entity(root).with_child((
            Mesh3d(meshes.add(Cuboid::new(0.7, 0.9, 0.22))),
            MeshMaterial3d(metal),
            Transform::from_xyz(0.0, 0.0, -0.11),
        ));
    }
    // Hebel (dreht sich beim Einschalten nach unten)
    let lever_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.08, 0.06),
        perceptual_roughness: 0.5,
        ..default()
    });
    let on = power.on;
    commands.entity(root).with_children(|p| {
        p.spawn((
            PowerLever {
                t: if on { 1.0 } else { 0.0 },
            },
            Transform::from_xyz(0.18, 0.0, -0.26),
            Visibility::default(),
        ))
        .with_child((
            Mesh3d(meshes.add(Cuboid::new(0.05, 0.42, 0.05))),
            MeshMaterial3d(lever_mat.clone()),
            Transform::from_xyz(0.0, 0.2, 0.0),
        ));
    });
}

pub fn use_power_switch(
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
    mut power: ResMut<Power>,
    time: Res<Time>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    if focus.kind != Some(InteractKind::Power) || power.on {
        return;
    }
    prompt.set("Halte F, um den Strom einzuschalten");
    if focus.pressed {
        power.on = true;
        power.since = time.elapsed_secs();
        sfx.write(SfxEvent::ui(Sfx::PowerOn));
        focus.pressed = false;
    }
}

pub fn animate_power_switch(
    time: Res<Time>,
    power: Res<Power>,
    mut q: Query<(&mut PowerLever, &mut Transform)>,
) {
    for (mut l, mut tf) in q.iter_mut() {
        let goal = if power.on { 1.0 } else { 0.0 };
        l.t += (goal - l.t).clamp(-time.delta_secs() * 2.5, time.delta_secs() * 2.5);
        tf.rotation = Quat::from_rotation_x(-l.t * 2.6);
    }
}
