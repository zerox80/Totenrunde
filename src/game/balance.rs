//! Schwierigkeit nach Spielerzahl. Solo ist deutlich leichter als das Original: mehr Punkte
//! pro Kill, häufigere Power-ups, etwas schwächere und später schnellere Zombies, die erst
//! nach drei statt zwei Treffern umhauen. Ab zwei Spielern gelten die härteren Originalwerte
//! samt Koop-Formel für die Zombie-Anzahl.
//!
//! Das Spiel kennt bisher nur Solo; mit `TR_PLAYERS=2` lässt sich die Koop-Abstimmung testen.

use bevy::prelude::*;

use crate::base::debug::DebugConfig;

#[derive(Resource, Clone, Copy, Debug)]
pub struct Balance {
    pub players: u32,
    /// Faktor auf die Kill-Punkte (Treffer bleiben bei +10)
    pub kill_points: f32,
    /// Zufallschance auf ein Power-up pro Kill
    pub drop_chance: f32,
    /// Erste Punkteschwelle für ein sicheres Power-up
    pub drop_first: u64,
    /// Wachstum der Schwelle nach jedem Drop
    pub drop_growth: f64,
    /// Faktor auf die Zombie-Gesundheit
    pub zombie_health: f32,
    /// Schaden eines Zombie-Schlags (Spieler hat 100, mit Panzer-Tonikum 250)
    pub zombie_damage: f32,
    /// Tempo-Anstieg pro Runde (Original: 8; höher = früher Läufer und Sprinter)
    pub speed_per_round: i32,
    /// Faktor auf die Pause zwischen zwei Spawns
    pub spawn_delay: f32,
}

impl Balance {
    pub fn for_players(players: u32) -> Self {
        if players <= 1 {
            Self {
                players: 1,
                kill_points: 1.5,
                drop_chance: 0.05,
                drop_first: 1500,
                drop_growth: 1.1,
                zombie_health: 0.85,
                zombie_damage: 40.0,
                speed_per_round: 6,
                spawn_delay: 1.15,
            }
        } else {
            Self {
                players,
                kill_points: 1.0,
                drop_chance: 0.02,
                drop_first: 2000,
                drop_growth: 1.14,
                zombie_health: 1.0,
                zombie_damage: 50.0,
                speed_per_round: 8,
                spawn_delay: 1.0,
            }
        }
    }
}

pub fn setup(mut commands: Commands, cfg: Res<DebugConfig>) {
    let b = Balance::for_players(cfg.players.unwrap_or(1));
    info!("Abstimmung für {} Spieler: {b:?}", b.players);
    commands.insert_resource(b);
}
