//! Der Spieler: Ego-Kamera, CoD-artige Bewegung und Gesundheit.

pub mod camera;
pub mod health;
pub mod look;
pub mod movement;
pub mod spawn;

use bevy::prelude::*;

use crate::base::{AppState, GameSet, InGame};

pub use health::{GameOverFlag, Health, PlayerDamage};
pub use spawn::spawn_player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerDamage>()
            .init_resource::<GameOverFlag>()
            .add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(
                Update,
                (look::mouse_look, movement::movement, camera::camera_effects)
                    .chain()
                    .in_set(GameSet::Movement)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            )
            .add_systems(
                Update,
                (health::player_health, movement::push_from_zombies)
                    .in_set(GameSet::World)
                    .after(crate::zombies::ZombieSystems)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            )
            .add_systems(Update, (look::cursor_grab, spawn::apply_graphics));
    }
}

pub const RADIUS: f32 = 0.32;
pub const STEP: f32 = 0.42;
pub const GRAVITY: f32 = 18.0;
pub const JUMP_VEL: f32 = 5.6;
pub const WALK_SPEED: f32 = 4.6;
pub const SPRINT_MULT: f32 = 1.5;
pub const PLAYER_STAMINA: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Stance {
    #[default]
    Stand,
    Crouch,
    Prone,
}

impl Stance {
    pub fn eye(self) -> f32 {
        match self {
            Stance::Stand => 1.62,
            Stance::Crouch => 1.05,
            Stance::Prone => 0.38,
        }
    }
    pub fn height(self) -> f32 {
        match self {
            Stance::Stand => 1.8,
            Stance::Crouch => 1.2,
            Stance::Prone => 0.6,
        }
    }
    pub fn speed(self) -> f32 {
        match self {
            Stance::Stand => 1.0,
            Stance::Crouch => 0.55,
            Stance::Prone => 0.22,
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
    pub vel: Vec3,
    pub grounded: bool,
    pub stance: Stance,
    pub eye: f32,
    pub sprinting: bool,
    pub stamina: f32,
    pub stamina_max: f32,
    pub stamina_cooldown: f32,
    /// Multiplikator der aktuellen Waffe (Beweglichkeit)
    pub weapon_mobility: f32,
    /// Zielt gerade über Kimme und Korn (0..1)
    pub ads: f32,
    pub bob_phase: f32,
    pub bob: f32,
    pub land_kick: f32,
    /// Rückstoß-Impuls, der noch auf die Blickrichtung wirkt (Pitch, Yaw)
    pub recoil_vel: Vec2,
    /// Wackeln der Kamera (z. B. Explosionen)
    pub shake: f32,
    /// Bewegung blockiert (z. B. tot)
    pub frozen: bool,
    pub speed_mult: f32,
    pub moving: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            vel: Vec3::ZERO,
            grounded: true,
            stance: Stance::Stand,
            eye: Stance::Stand.eye(),
            sprinting: false,
            stamina: PLAYER_STAMINA,
            stamina_max: PLAYER_STAMINA,
            stamina_cooldown: 0.0,
            weapon_mobility: 1.0,
            ads: 0.0,
            bob_phase: 0.0,
            bob: 0.0,
            land_kick: 0.0,
            recoil_vel: Vec2::ZERO,
            shake: 0.0,
            frozen: false,
            speed_mult: 1.0,
            moving: false,
        }
    }
}

/// Die Kamera (Kind des Spielers). Trägt auch das Waffenmodell.
#[derive(Component)]
pub struct PlayerCamera;

/// Gewünschtes Sichtfeld (vertikal, Radiant), wird von den Waffen beim Zielen gesetzt.
#[derive(Component)]
pub struct CameraFov {
    pub target: f32,
    pub current: f32,
}
