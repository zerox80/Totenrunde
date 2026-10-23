//! Perk-Automaten: Platzierung aus der Karte, Leuchten (mit Strom), zufällige Jingles.

use bevy::prelude::*;
use rand::Rng;

use super::Perk;
use crate::audio::SfxEvent;
use crate::base::AppState;
use crate::player::Player;
use crate::world::mapdata::MapFile;
use crate::world::power::Power;

#[derive(Component)]
pub struct PerkMachine {
    pub perk: Perk,
    pub jingle_cd: f32,
}

#[derive(Component)]
pub struct MachineLight {
    pub perk: Perk,
}

pub fn spawn_machines(mut commands: Commands, map: Res<MapFile>, asset_server: Res<AssetServer>) {
    let mut rng = rand::rng();
    for spec in &map.perks {
        let Some(perk) = Perk::from_id(&spec.perk) else {
            warn!("Unbekannter Perk '{}'", spec.perk);
            continue;
        };
        let pos = Vec3::from_array(spec.pos);
        let rot = Quat::from_rotation_y(spec.yaw);
        let fwd = rot * Vec3::NEG_Z;
        let point = Vec3::new(pos.x, 1.0, pos.z) + fwd * 0.8;
        commands.spawn((
            crate::game::interact::Interactable {
                kind: crate::game::interact::InteractKind::Perk(perk),
                point,
                radius: 1.7,
                facing: Some(fwd),
            },
            Transform::from_translation(point),
            DespawnOnExit(AppState::Playing),
        ));
        commands
            .spawn((
                Name::new(format!("Automat {}", perk.name())),
                PerkMachine {
                    perk,
                    jingle_cd: rng.random_range(20.0..90.0),
                },
                Transform::from_translation(pos).with_rotation(rot),
                Visibility::default(),
                DespawnOnExit(AppState::Playing),
            ))
            .with_children(|p| {
                // Automaten-Modell nur laden, wenn es schon gebaut ist
                if crate::world::mapdata::asset_root().join(perk.model()).exists() {
                    p.spawn((
                        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(perk.model()))),
                        Transform::IDENTITY,
                    ));
                }
                p.spawn((
                    MachineLight { perk },
                    PointLight {
                        color: perk.color(),
                        intensity: 0.0,
                        range: 6.0,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 1.6, -0.9),
                ));
            });
    }
}

pub fn machine_lights(time: Res<Time>, power: Res<Power>, mut q: Query<(&MachineLight, &mut PointLight)>) {
    let t = time.elapsed_secs();
    for (m, mut l) in q.iter_mut() {
        let on = power.on || !m.perk.needs_power();
        let target = if on { 90000.0 * (1.0 + (t * 3.0 + m.perk as u8 as f32).sin() * 0.06) } else { 0.0 };
        l.intensity += (target - l.intensity) * (time.delta_secs() * 3.0).min(1.0);
    }
}

pub fn machine_jingles(
    time: Res<Time>,
    power: Res<Power>,
    player: Single<&Transform, With<Player>>,
    mut q: Query<(&mut PerkMachine, &Transform)>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let mut rng = rand::rng();
    for (mut m, tf) in q.iter_mut() {
        if m.perk.needs_power() && !power.on {
            continue;
        }
        m.jingle_cd -= time.delta_secs();
        if m.jingle_cd <= 0.0 {
            m.jingle_cd = rng.random_range(70.0..160.0);
            if tf.translation.distance(player.translation) < 22.0 {
                sfx.write(SfxEvent::at(m.perk.jingle(), tf.translation + Vec3::Y * 1.5));
            }
        }
    }
}
