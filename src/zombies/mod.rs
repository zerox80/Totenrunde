//! Zombies: Daten, Spawnen, KI, Steuerung, Animation, Trefferzonen und Schaden.

pub mod ai;
pub mod anim;
pub mod damage;
pub mod hitbox;
pub mod spawn;
pub mod steering;

use bevy::prelude::*;

use crate::base::{AppState, GameSet, InGame};

pub use hitbox::{raycast_zombies, RayZombieHit};
pub use spawn::{spawn_zombie, ZombieSpawn};

pub struct ZombiePlugin;

/// Systemmenge der Zombie-KI (andere Systeme ordnen sich danach ein).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZombieSystems;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ZombieHit>()
            .add_message::<ZombieKilled>()
            .init_resource::<steering::FlowTimer>()
            .init_resource::<spawn::ZombieMats>()
            .add_systems(
                Update,
                (steering::update_flow, ai::zombie_ai, steering::zombie_separation, anim::animate_zombies)
                    .chain()
                    .in_set(ZombieSystems)
                    .in_set(GameSet::World)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            )
            .add_systems(
                Update,
                damage::apply_damage
                    .in_set(GameSet::Combat)
                    .after(crate::weapons::WeaponSystems)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

pub const ZOMBIE_RADIUS: f32 = 0.3;
pub const ATTACK_RANGE: f32 = 1.25;
pub const ATTACK_DAMAGE: f32 = 50.0;
pub const MAX_ALIVE: usize = 24;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpeedClass {
    Walk,
    Run,
    Sprint,
}

impl SpeedClass {
    /// Geschwindigkeit, bei der die Animation ohne Rutschen läuft (m/s)
    pub fn native_speed(self) -> f32 {
        match self {
            SpeedClass::Walk => 1.05,
            SpeedClass::Run => 2.6,
            SpeedClass::Sprint => 5.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ZState {
    Rising { t: f32 },
    ToWindow { b: usize },
    AtWindow { b: usize, cd: f32, tearing: f32 },
    Climbing { b: usize, t: f32 },
    Chasing,
    Attacking { t: f32, hit: bool, window: Option<usize> },
    Dying { t: f32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HitPart {
    Head,
    Neck,
    Torso,
    Limb,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageKind {
    Bullet,
    Melee,
    Explosive,
    Nuke,
}

#[derive(Component)]
pub struct Zombie {
    pub health: f32,
    pub max_health: f32,
    pub state: ZState,
    pub class: SpeedClass,
    pub speed: f32,
    pub vel: Vec3,
    pub yaw: f32,
    pub attack_cd: f32,
    pub stuck: f32,
    pub progress_pos: Vec3,
    pub headless: bool,
    pub death_back: bool,
    pub groan_cd: f32,
    pub variant: u8,
    /// Index in ZombieAnims::models (Mann/Frau)
    pub model: usize,
    pub flinch: f32,
}

impl Zombie {
    pub fn alive(&self) -> bool {
        !matches!(self.state, ZState::Dying { .. })
    }
}

/// Knochen-Entities des Modells (für Trefferzonen).
#[derive(Component)]
pub struct ZombieRig {
    pub anim_player: Entity,
    pub head: Entity,
    pub neck: Entity,
    pub chest: Entity,
    pub hips: Entity,
    pub upper_arm: [Entity; 2],
    pub lower_arm: [Entity; 2],
    pub hand: [Entity; 2],
    pub upper_leg: [Entity; 2],
    pub lower_leg: [Entity; 2],
    pub foot: [Entity; 2],
    pub head_mesh: Option<Entity>,
    /// leuchtende Augen (verschwinden mit dem Kopf)
    pub eyes_mesh: Option<Entity>,
    /// Halsstumpf (erst nach dem Kopfschuss sichtbar)
    pub stump_mesh: Option<Entity>,
}

#[derive(Message)]
pub struct ZombieHit {
    pub zombie: Entity,
    pub damage: f32,
    pub part: HitPart,
    pub kind: DamageKind,
    pub point: Vec3,
    pub dir: Vec3,
}

#[derive(Message, Clone, Copy)]
pub struct ZombieKilled {
    pub zombie: Entity,
    pub pos: Vec3,
    pub part: HitPart,
    pub kind: DamageKind,
}
