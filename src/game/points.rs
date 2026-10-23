//! Punkte (die Währung im Zombie-Modus) und Statistiken.

use bevy::prelude::*;

use crate::base::debug::DebugConfig;
use crate::base::AppState;
use crate::game::powerups::ActivePowerups;

pub struct PointsPlugin;

impl Plugin for PointsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PointsPopup>()
            .add_systems(OnEnter(AppState::Playing), reset_score);
    }
}

pub const START_POINTS: u32 = 500;

#[derive(Resource, Default, Clone)]
pub struct Score {
    pub points: u32,
    /// Insgesamt verdiente Punkte (für Power-up-Drops und Statistik)
    pub earned: u64,
    pub kills: u32,
    pub headshots: u32,
    pub downs: u32,
    pub revives: u32,
    pub doors: u32,
    pub boards: u32,
    /// Für den Ausgabe-Popup (+10 usw.)
    pub pending_popups: Vec<i64>,
    /// Gezählte Reparatur-Punkte dieser Runde (Obergrenze wie im Original)
    pub repair_points_round: u32,
}

#[derive(Message, Clone, Copy)]
pub struct PointsPopup {
    pub amount: i64,
}

impl Score {
    /// Punkte gutschreiben (Doppelte Punkte werden berücksichtigt).
    pub fn add(&mut self, base: u32, powerups: &ActivePowerups, _at: Option<Vec3>) -> u32 {
        let amount = if powerups.double_points > 0.0 { base * 2 } else { base };
        self.points += amount;
        self.earned += amount as u64;
        self.pending_popups.push(amount as i64);
        amount
    }

    /// Versucht, `cost` Punkte auszugeben.
    pub fn spend(&mut self, cost: u32) -> bool {
        if self.points >= cost {
            self.points -= cost;
            self.pending_popups.push(-(cost as i64));
            true
        } else {
            false
        }
    }
}

fn reset_score(mut commands: Commands, cfg: Res<DebugConfig>) {
    commands.insert_resource(Score {
        points: cfg.start_points.unwrap_or(START_POINTS),
        ..default()
    });
}
