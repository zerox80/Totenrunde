//! Fenster-Barrikaden mit je 6 Brettern. Zombies reißen sie ab, der Spieler baut sie
//! wieder auf (+10 Punkte pro Brett, wie im Original).

use bevy::prelude::*;
use rand::Rng;

use super::mapdata::MapFile;
use crate::base::assets::GameAssets;
use crate::base::{AppState, GameSet, InGame};

pub struct BarricadePlugin;

impl Plugin for BarricadePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BoardEvent>()
            .add_systems(OnEnter(AppState::Playing), spawn_barricades)
            .add_systems(
                Update,
                animate_boards
                    .in_set(GameSet::World)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

pub const BOARDS: usize = 6;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BoardState {
    Intact,
    /// fliegt gerade weg (Zeit)
    Tearing(f32),
    Gone,
    /// fliegt gerade zurück (Zeit)
    Repairing(f32),
}

pub struct BarricadeState {
    pub id: usize,
    pub zone: usize,
    pub center: Vec3,
    pub inside: Vec3,
    pub outside: Vec3,
    pub sill: f32,
    pub yaw: f32,
    pub boards: [BoardState; BOARDS],
    pub board_entities: [Entity; BOARDS],
    pub board_rest: [Transform; BOARDS],
    pub occupant: Option<Entity>,
}

impl BarricadeState {
    pub fn intact_boards(&self) -> usize {
        self.boards
            .iter()
            .filter(|b| matches!(b, BoardState::Intact | BoardState::Repairing(_)))
            .count()
    }

    pub fn missing_boards(&self) -> usize {
        BOARDS - self.intact_boards()
    }

    /// Reißt das zuletzt angebrachte Brett ab.
    pub fn tear_one(&mut self) -> bool {
        for i in (0..BOARDS).rev() {
            if self.boards[i] == BoardState::Intact {
                self.boards[i] = BoardState::Tearing(0.0);
                return true;
            }
        }
        false
    }

    pub fn repair_one(&mut self) -> bool {
        for i in 0..BOARDS {
            if self.boards[i] == BoardState::Gone {
                self.boards[i] = BoardState::Repairing(0.0);
                return true;
            }
        }
        false
    }

    pub fn repair_all(&mut self) {
        for i in 0..BOARDS {
            if matches!(self.boards[i], BoardState::Gone | BoardState::Tearing(_)) {
                self.boards[i] = BoardState::Repairing(0.0);
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct Barricades {
    pub list: Vec<BarricadeState>,
}

impl Barricades {
    pub fn release(&mut self, e: Entity) {
        for b in self.list.iter_mut() {
            if b.occupant == Some(e) {
                b.occupant = None;
            }
        }
    }
}

#[derive(Message)]
pub enum BoardEvent {
    Torn { pos: Vec3 },
    Repaired { pos: Vec3 },
}

#[derive(Component)]
pub struct Board;

pub fn spawn_barricades(
    mut commands: Commands,
    map: Res<MapFile>,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();
    let board_mesh = meshes.add(Cuboid::new(1.0, 0.16, 0.035));
    let board_mat = materials.add(StandardMaterial {
        base_color_texture: Some(assets.planks.clone()),
        base_color: Color::srgb(0.85, 0.75, 0.62),
        perceptual_roughness: 0.85,
        ..default()
    });
    let mut list = Vec::new();
    for spec in &map.barricades {
        let center = Vec3::from_array(spec.pos);
        let rot = Quat::from_rotation_y(spec.yaw);
        let inward = rot * Vec3::NEG_Z;
        let right = rot * Vec3::X;
        let mut ents = [Entity::PLACEHOLDER; BOARDS];
        let mut rests = [Transform::IDENTITY; BOARDS];
        let h = spec.top - spec.sill;
        for i in 0..BOARDS {
            // Bretter über die Öffnung verteilt, leicht schräg und versetzt
            let fy = spec.sill + h * (0.12 + 0.76 * (i as f32 / (BOARDS - 1) as f32));
            let tilt = rng.random_range(-0.35f32..0.35) + if i % 2 == 0 { 0.12 } else { -0.12 };
            let shift = rng.random_range(-0.08f32..0.08);
            let pos = Vec3::new(center.x, fy, center.z) + inward * (0.19 + i as f32 * 0.008) + right * shift;
            let len = spec.width + 0.35 + rng.random_range(0.0..0.15);
            let tf = Transform::from_translation(pos)
                .with_rotation(rot * Quat::from_rotation_z(tilt))
                .with_scale(Vec3::new(len, 1.0, 1.0));
            rests[i] = tf;
            ents[i] = commands
                .spawn((
                    Board,
                    Mesh3d(board_mesh.clone()),
                    MeshMaterial3d(board_mat.clone()),
                    tf,
                    DespawnOnExit(AppState::Playing),
                ))
                .id();
        }
        list.push(BarricadeState {
            id: spec.id,
            zone: spec.zone,
            center: Vec3::new(center.x, 0.0, center.z),
            inside: Vec3::from_array(spec.inside),
            outside: Vec3::from_array(spec.outside),
            sill: spec.sill,
            yaw: spec.yaw,
            boards: [BoardState::Intact; BOARDS],
            board_entities: ents,
            board_rest: rests,
            occupant: None,
        });
    }
    commands.insert_resource(Barricades { list });
}

fn animate_boards(
    time: Res<Time>,
    mut bars: ResMut<Barricades>,
    mut boards: Query<(&mut Transform, &mut Visibility), With<Board>>,
    mut events: MessageWriter<BoardEvent>,
) {
    let dt = time.delta_secs();
    for b in bars.list.iter_mut() {
        let outward = (b.outside - b.inside).normalize_or_zero();
        for i in 0..BOARDS {
            let Ok((mut tf, mut vis)) = boards.get_mut(b.board_entities[i]) else { continue };
            let rest = b.board_rest[i];
            match b.boards[i] {
                BoardState::Intact => {
                    *tf = rest;
                    *vis = Visibility::Inherited;
                }
                BoardState::Gone => {
                    *vis = Visibility::Hidden;
                }
                BoardState::Tearing(t) => {
                    let t = t + dt;
                    if t < 0.02 {
                        events.write(BoardEvent::Torn { pos: rest.translation });
                    }
                    // nach außen gerissen, fällt und dreht sich
                    let k = t / 0.45;
                    tf.translation = rest.translation + outward * (k * 1.6) + Vec3::Y * (k * 0.5 - k * k * 1.6);
                    tf.rotation = rest.rotation * Quat::from_rotation_x(k * 2.5) * Quat::from_rotation_z(k * 1.2);
                    if t >= 0.45 {
                        b.boards[i] = BoardState::Gone;
                        *vis = Visibility::Hidden;
                    } else {
                        b.boards[i] = BoardState::Tearing(t);
                        *vis = Visibility::Inherited;
                    }
                }
                BoardState::Repairing(t) => {
                    let t = t + dt;
                    let k = (t / 0.35).min(1.0);
                    let start = rest.translation - outward * 0.9 + Vec3::new(0.0, -rest.translation.y + 0.1, 0.0);
                    let p = start.lerp(rest.translation, k) + Vec3::Y * ((k * std::f32::consts::PI).sin() * 0.4);
                    tf.translation = p;
                    tf.rotation = rest.rotation * Quat::from_rotation_x((1.0 - k) * 1.5);
                    tf.scale = rest.scale;
                    *vis = Visibility::Inherited;
                    if k >= 1.0 {
                        b.boards[i] = BoardState::Intact;
                        events.write(BoardEvent::Repaired { pos: rest.translation });
                    } else {
                        b.boards[i] = BoardState::Repairing(t);
                    }
                }
            }
        }
    }
}
