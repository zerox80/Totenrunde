//! Zombies spawnen: Modell (Mann/Frau) instanziieren, Skelett-Knochen finden, Animation
//! starten, Varianten (Hemd-/Hosenfarbe).

use std::collections::HashMap;
use std::time::Duration;

use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use rand::Rng;

use super::{SpeedClass, ZState, Zombie, ZombieRig};
use super::anim::ZombieAnimState;
use crate::base::assets::ZombieAnims;
use crate::base::AppState;

/// Wurzel der Modell-Instanz eines Zombies (Kind der Zombie-Entity).
#[derive(Component)]
pub struct ZombieModelRoot {
    zombie: Entity,
}

/// Varianten-Materialien (Hemd/Hose in verschiedenen Farben)
#[derive(Resource, Default)]
pub struct ZombieMats {
    shirts: Vec<Handle<StandardMaterial>>,
    pants: Vec<Handle<StandardMaterial>>,
}

const SHIRT_COLORS: [(f32, f32, f32); 7] = [
    (0.55, 0.52, 0.45),
    (0.30, 0.36, 0.48),
    (0.45, 0.20, 0.17),
    (0.32, 0.40, 0.30),
    (0.62, 0.58, 0.50),
    (0.25, 0.24, 0.26),
    (0.50, 0.42, 0.28),
];
const PANT_COLORS: [(f32, f32, f32); 5] = [
    (0.20, 0.22, 0.28),
    (0.30, 0.26, 0.20),
    (0.17, 0.17, 0.17),
    (0.34, 0.33, 0.30),
    (0.22, 0.26, 0.20),
];

pub struct ZombieSpawn {
    pub pos: Vec3,
    pub yaw: f32,
    pub state: ZState,
    pub health: f32,
    pub class: SpeedClass,
}

pub fn spawn_zombie(commands: &mut Commands, anims: &ZombieAnims, s: ZombieSpawn) -> Entity {
    let mut rng = rand::rng();
    let speed = match s.class {
        SpeedClass::Walk => rng.random_range(0.75..1.15),
        SpeedClass::Run => rng.random_range(2.3..2.9),
        SpeedClass::Sprint => rng.random_range(4.0..4.7),
    };
    let variant = rng.random_range(0..35u8);
    let model = rng.random_range(0..anims.models.len().max(1));
    let z = commands
        .spawn((
            Name::new("Zombie"),
            Zombie {
                health: s.health,
                max_health: s.health,
                state: s.state,
                class: s.class,
                speed,
                vel: Vec3::ZERO,
                yaw: s.yaw,
                attack_cd: 0.0,
                stuck: 0.0,
                progress_pos: s.pos,
                headless: false,
                death_back: false,
                groan_cd: rng.random_range(1.0..6.0),
                variant,
                model,
                flinch: 0.0,
            },
            ZombieAnimState::default(),
            Transform::from_translation(s.pos).with_rotation(Quat::from_rotation_y(s.yaw)),
            Visibility::default(),
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    commands
        .spawn((
            ZombieModelRoot { zombie: z },
            WorldAssetRoot(anims.models[model].scene.clone()),
            Transform::IDENTITY,
            ChildOf(z),
        ))
        .observe(on_zombie_ready);
    z
}

#[allow(clippy::too_many_arguments)]
fn on_zombie_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    models: Query<&ZombieModelRoot>,
    zombies: Query<&Zombie>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut anim_players: Query<&mut AnimationPlayer>,
    mat_names: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    anims: Res<ZombieAnims>,
    mut zmats: ResMut<ZombieMats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(model) = models.get(ready.entity) else { return };
    let Ok(zombie) = zombies.get(model.zombie) else { return };
    let mut by_name: HashMap<String, Entity> = HashMap::new();
    let mut player = None;
    for e in children.iter_descendants(ready.entity) {
        if let Ok(n) = names.get(e) {
            by_name.insert(n.as_str().to_string(), e);
        }
        if anim_players.contains(e) {
            player = Some(e);
        }
        if let Ok((mname, mat)) = mat_names.get(e) {
            // Varianten-Material anlegen (einmalig) und zuweisen
            let v = zombie.variant as usize;
            match mname.0.as_str() {
                "Z_Shirt" => {
                    if zmats.shirts.is_empty() {
                        if let Some(base) = materials.get(&mat.0).cloned() {
                            for c in SHIRT_COLORS {
                                let mut m = base.clone();
                                m.base_color = Color::srgb(c.0, c.1, c.2);
                                zmats.shirts.push(materials.add(m));
                            }
                        }
                    }
                    if !zmats.shirts.is_empty() {
                        commands
                            .entity(e)
                            .insert(MeshMaterial3d(zmats.shirts[v % zmats.shirts.len()].clone()));
                    }
                }
                "Z_Pants" => {
                    if zmats.pants.is_empty() {
                        if let Some(base) = materials.get(&mat.0).cloned() {
                            for c in PANT_COLORS {
                                let mut m = base.clone();
                                m.base_color = Color::srgb(c.0, c.1, c.2);
                                zmats.pants.push(materials.add(m));
                            }
                        }
                    }
                    if !zmats.pants.is_empty() {
                        commands
                            .entity(e)
                            .insert(MeshMaterial3d(zmats.pants[(v / 7) % zmats.pants.len()].clone()));
                    }
                }
                _ => {}
            }
        }
    }
    let Some(player) = player else {
        warn!("Zombie ohne AnimationPlayer");
        return;
    };
    let get = |n: &str| by_name.get(n).copied().unwrap_or(ready.entity);
    let stump = by_name.get("ZombieStump").copied();
    if let Some(s) = stump {
        commands.entity(s).insert(Visibility::Hidden);
    }
    let Some(anims) = anims.models.get(zombie.model) else { return };
    if let Ok(mut ap) = anim_players.get_mut(player) {
        let mut tr = AnimationTransitions::new();
        let first = match zombie.state {
            ZState::Rising { .. } => anims.rise,
            _ => anims.idle,
        };
        tr.play(&mut ap, first, Duration::ZERO).repeat();
        commands
            .entity(player)
            .insert((AnimationGraphHandle(anims.graph.clone()), tr));
    }
    commands.entity(model.zombie).insert(ZombieRig {
        anim_player: player,
        head: get("Head"),
        neck: get("Neck"),
        chest: get("Chest"),
        hips: get("Hips"),
        upper_arm: [get("UpperArm.L"), get("UpperArm.R")],
        lower_arm: [get("LowerArm.L"), get("LowerArm.R")],
        hand: [get("Hand.L"), get("Hand.R")],
        upper_leg: [get("UpperLeg.L"), get("UpperLeg.R")],
        lower_leg: [get("LowerLeg.L"), get("LowerLeg.R")],
        foot: [get("Foot.L"), get("Foot.R")],
        head_mesh: by_name.get("ZombieHead").copied(),
        eyes_mesh: by_name.get("ZombieEyes").copied(),
        stump_mesh: stump,
    });
}
