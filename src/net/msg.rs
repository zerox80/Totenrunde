//! Nachrichten zwischen den Spielen einer Partie (über den Server weitergeleitet).
//!
//! Aufteilung der Verantwortung:
//! * Jedes Spiel besitzt seinen eigenen Spieler: Bewegung, Waffen, Punkte, Perks. Es meldet
//!   seinen Zustand (`State`) etwa 20-mal pro Sekunde an alle.
//! * Der Host besitzt die Welt: Zombies, Runden, Türen, Strom, Barrikaden, Kiste und
//!   Power-ups. Er schickt einen Schnappschuss (`World`) und Ereignisse (`Event`).
//! * Treffer prüft der Schütze auf seinem Bild der Zombies und meldet sie dem Host (`Hit`);
//!   der Host rechnet den Schaden und schickt die Punkte zurück (`Points`).
//! * Käufe, die die Welt verändern (Tür, Strom, Kiste, Bretter), fragt das Spiel beim Host an
//!   (`Request`); bezahlt wird erst, wenn der Host zusagt (`Grant`). Zwei Spieler können so
//!   nicht dieselbe Tür gleichzeitig kaufen.
//!
//! Die Verbindung ist zuverlässig und geordnet (WebSocket über TCP).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum GameMsg {
    State(PlayerState),
    /// Schuss (Geräusch und Leuchtspur bei den anderen)
    Shot(ShotFx),
    /// Explosion (nur Effekt; Schaden rechnet der Host)
    Blast(BlastFx),
    Hit(NetHit),
    Request(Req),
    /// An einen niedergestreckten Spieler: du wurdest wiederbelebt
    Revive,
    World(Box<Snapshot>),
    Event(WorldEvent),
    Grant(Grant),
    Damage {
        amount: f32,
        from: [f32; 3],
    },
    Points {
        amount: u32,
        kill: bool,
        headshot: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Life {
    #[default]
    Alive,
    Downed,
    Dead,
}

/// Zustand eines Spielers, wie ihn die anderen brauchen.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct PlayerState {
    pub pos: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    /// Waagerechte Geschwindigkeit (m/s) für die Laufanimation
    pub speed: f32,
    /// 0 stehen, 1 hocken, 2 liegen
    pub stance: u8,
    pub sprinting: bool,
    pub life: Life,
    /// Restzeit bis zum Ausbluten (nur niedergestreckt)
    pub bleed: f32,
    pub weapon: u8,
    pub upgraded: bool,
    pub points: u32,
    pub earned: u64,
    pub kills: u32,
    pub headshots: u32,
    pub downs: u32,
    pub revives: u32,
    /// Bitmaske der Perks (Reihenfolge wie `Perk::ALL`)
    pub perks: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ShotFx {
    pub weapon: u8,
    pub upgraded: bool,
    pub origin: [f32; 3],
    pub end: [f32; 3],
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct BlastFx {
    pub pos: [f32; 3],
    pub radius: f32,
    /// Strahlenpistole (grün/rot) statt Granate
    pub ray: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct NetHit {
    pub zombie: u16,
    pub damage: f32,
    /// 0 Kopf, 1 Hals, 2 Torso, 3 Gliedmaßen
    pub part: u8,
    /// 0 Kugel, 1 Messer, 2 Explosion
    pub kind: u8,
    pub point: [f32; 3],
    pub dir: [f32; 3],
}

/// Anfrage an den Host (Kauf oder Aktion, die die Welt ändert).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Req {
    Door(u16),
    Power,
    /// Kiste öffnen; `owned` sind die Waffen, die der Spieler schon trägt
    BoxOpen {
        owned: Vec<u8>,
    },
    BoxTake,
    Repair(u8),
}

/// Antwort des Hosts auf eine Anfrage.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Grant {
    Door { id: u16, cost: u32 },
    Power,
    BoxOpen { cost: u32, teddy: bool },
    BoxTake { weapon: u8 },
    Repair { barricade: u8 },
    Denied,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WorldEvent {
    RoundStarted(u32),
    RoundEnded,
    PowerupSpawned { id: u16, kind: u8, pos: [f32; 3] },
    PowerupGone { id: u16 },
    PowerupCollected { kind: u8, by: u8 },
    GameOver,
}

/// Zustand der Welt, wie ihn der Host sieht.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Snapshot {
    pub round: u32,
    /// 0 Einleitung, 1 läuft, 2 Pause zwischen den Runden
    pub phase: u8,
    pub insta_kill: f32,
    pub double_points: f32,
    pub fire_sale: f32,
    pub power: bool,
    pub doors: Vec<u16>,
    /// Je Barrikade eine Bitmaske der stehenden Bretter
    pub boards: Vec<u8>,
    pub mystery: Option<BoxNet>,
    pub zombies: Vec<ZombieNet>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ZombieNet {
    pub id: u16,
    /// Position in Zentimetern
    pub pos: [i16; 3],
    /// Blickrichtung in 1/256-Umdrehungen
    pub yaw: u8,
    pub state: ZNet,
    pub class: u8,
    pub model: u8,
    pub variant: u8,
    /// Geschwindigkeit in cm/s (für die Animation)
    pub speed: u16,
    pub headless: bool,
    pub death_back: bool,
}

/// Zustand eines Zombies, so weit ihn die Animation braucht.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ZNet {
    Rising(f32),
    ToWindow(u8),
    AtWindow { b: u8, tearing: f32 },
    Climbing { b: u8, t: f32 },
    Chasing,
    Attacking(f32),
    Dying(f32),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct BoxNet {
    pub spot: u8,
    pub phase: BoxNetPhase,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum BoxNetPhase {
    Closed,
    /// `weapon` = 255: Teddy
    Rolling {
        t: f32,
        weapon: u8,
        owner: u8,
    },
    Offer {
        t: f32,
        weapon: u8,
        owner: u8,
    },
    Teddy {
        t: f32,
    },
    Away {
        t: f32,
    },
}

pub fn q_pos(p: Vec3) -> [i16; 3] {
    let q = |v: f32| (v * 100.0).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
    [q(p.x), q(p.y), q(p.z)]
}

pub fn dq_pos(p: [i16; 3]) -> Vec3 {
    Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32) / 100.0
}

pub fn q_yaw(yaw: f32) -> u8 {
    (yaw / std::f32::consts::TAU * 256.0)
        .round()
        .rem_euclid(256.0) as u8
}

pub fn dq_yaw(yaw: u8) -> f32 {
    yaw as f32 / 256.0 * std::f32::consts::TAU
}

pub fn v3(a: [f32; 3]) -> Vec3 {
    Vec3::from_array(a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use totenrunde_mp_client::protocol::{decode, encode};

    #[test]
    fn snapshot_round_trip_and_size() {
        let z = ZombieNet {
            id: 513,
            pos: q_pos(Vec3::new(-12.34, 0.0, 30.5)),
            yaw: q_yaw(-2.0),
            state: ZNet::AtWindow { b: 3, tearing: 0.4 },
            class: 1,
            model: 0,
            variant: 22,
            speed: 260,
            headless: false,
            death_back: true,
        };
        let snap = GameMsg::World(Box::new(Snapshot {
            round: 7,
            zombies: vec![z; 24],
            boards: vec![0b111111; 8],
            ..default()
        }));
        let bytes = encode(&snap);
        assert_eq!(decode::<GameMsg>(&bytes), Some(snap));
        // 24 Zombies sollen deutlich unter 1 KB bleiben (20 Schnappschüsse pro Sekunde)
        assert!(bytes.len() < 900, "{} Bytes", bytes.len());
        assert!((dq_pos(z.pos) - Vec3::new(-12.34, 0.0, 30.5)).length() < 0.01);
        let yaw = dq_yaw(q_yaw(-2.0));
        assert!(((yaw - (-2.0f32).rem_euclid(std::f32::consts::TAU)).abs()) < 0.03);
    }
}
