//! Eine laufende Koop-Partie: wer mitspielt, wer der Host ist und was wir von den anderen
//! Spielern wissen.

use std::collections::VecDeque;

use bevy::prelude::*;
use totenrunde_mp_client::protocol::LobbyView;

use super::msg::PlayerState;

/// Vorhanden, solange eine Koop-Partie läuft (fehlt im Solospiel).
#[derive(Resource, Clone, Debug)]
pub struct Session {
    /// Eigener Platz (0–3)
    pub you: u8,
    pub host: u8,
    /// Alle Teilnehmer beim Start (Platz, Name)
    pub names: Vec<(u8, String)>,
}

impl Session {
    pub fn from_lobby(view: &LobbyView) -> Self {
        Self {
            you: view.you,
            host: view.host,
            names: view
                .members
                .iter()
                .map(|m| (m.slot, m.name.clone()))
                .collect(),
        }
    }

    pub fn is_host(&self) -> bool {
        self.you == self.host
    }

    pub fn name(&self, slot: u8) -> &str {
        self.names
            .iter()
            .find(|(s, _)| *s == slot)
            .map(|(_, n)| n.as_str())
            .unwrap_or("?")
    }
}

/// Läuft das Spiel die Welt selbst (Solo oder Host)?
pub fn is_authority(session: Option<Res<Session>>) -> bool {
    session.is_none_or(|s| s.is_host())
}

/// Gast einer Koop-Partie (die Welt kommt vom Host)?
pub fn is_guest(session: Option<Res<Session>>) -> bool {
    session.is_some_and(|s| !s.is_host())
}

pub fn in_session(session: Option<Res<Session>>) -> bool {
    session.is_some()
}

/// Eigener Platz; im Solospiel 0.
pub fn local_slot(session: Option<&Session>) -> u8 {
    session.map(|s| s.you).unwrap_or(0)
}

/// Farbe eines Spielers (HUD, Namensschild, Hemd), wie die Spielerfarben in BO2.
pub fn slot_color(slot: u8) -> Color {
    match slot % 4 {
        0 => Color::srgb(0.95, 0.95, 0.92),
        1 => Color::srgb(0.35, 0.65, 1.0),
        2 => Color::srgb(1.0, 0.78, 0.25),
        _ => Color::srgb(0.45, 0.9, 0.45),
    }
}

/// Die anderen Spieler der Partie mit ihrem zuletzt gemeldeten Zustand.
#[derive(Resource, Default)]
pub struct Team {
    pub members: Vec<Member>,
}

pub struct Member {
    pub slot: u8,
    pub name: String,
    pub state: PlayerState,
    /// Noch verbunden?
    pub connected: bool,
    /// Schon ein Zustand empfangen?
    pub seen: bool,
    pub motion: Interp,
}

impl Team {
    pub fn for_session(session: &Session) -> Self {
        Self {
            members: session
                .names
                .iter()
                .filter(|(slot, _)| *slot != session.you)
                .map(|(slot, name)| Member {
                    slot: *slot,
                    name: name.clone(),
                    state: PlayerState::default(),
                    connected: true,
                    seen: false,
                    motion: Interp::default(),
                })
                .collect(),
        }
    }

    pub fn get(&self, slot: u8) -> Option<&Member> {
        self.members.iter().find(|m| m.slot == slot)
    }

    pub fn get_mut(&mut self, slot: u8) -> Option<&mut Member> {
        self.members.iter_mut().find(|m| m.slot == slot)
    }

    pub fn name(&self, slot: u8) -> String {
        self.get(slot)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "?".into())
    }
}

/// Verzögerung, mit der fremde Bewegungen dargestellt werden. Damit liegen fast immer zwei
/// Zustände vor, zwischen denen weich überblendet werden kann.
pub const INTERP_DELAY: f64 = 0.1;

/// Puffer empfangener Positionen für die weiche Darstellung.
#[derive(Default, Clone)]
pub struct Interp {
    samples: VecDeque<(f64, Vec3, f32)>,
}

impl Interp {
    pub fn push(&mut self, time: f64, pos: Vec3, yaw: f32) {
        // Große Sprünge (Wiederbelebung, Teleport) nicht überblenden
        if let Some((_, last, _)) = self.samples.back()
            && last.distance(pos) > 6.0
        {
            self.samples.clear();
        }
        self.samples.push_back((time, pos, yaw));
        while self.samples.len() > 12 {
            self.samples.pop_front();
        }
    }

    pub fn latest(&self) -> Option<Vec3> {
        self.samples.back().map(|s| s.1)
    }

    /// Position und Blickrichtung zum Zeitpunkt `time` (Empfangszeit).
    pub fn sample(&self, time: f64) -> Option<(Vec3, f32)> {
        let first = self.samples.front()?;
        if time <= first.0 {
            return Some((first.1, first.2));
        }
        for w in self.samples.iter().zip(self.samples.iter().skip(1)) {
            let ((t0, p0, y0), (t1, p1, y1)) = w;
            if time <= *t1 {
                let k = ((time - t0) / (t1 - t0).max(1e-4)) as f32;
                return Some((p0.lerp(*p1, k), lerp_angle(*y0, *y1, k)));
            }
        }
        let last = self.samples.back()?;
        Some((last.1, last.2))
    }
}

pub fn lerp_angle(a: f32, b: f32, k: f32) -> f32 {
    let mut d = (b - a) % std::f32::consts::TAU;
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    } else if d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    a + d * k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation_between_samples() {
        let mut i = Interp::default();
        i.push(1.0, Vec3::ZERO, 0.0);
        i.push(2.0, Vec3::X * 2.0, 1.0);
        let (p, y) = i.sample(1.5).unwrap();
        assert!((p - Vec3::X).length() < 1e-5);
        assert!((y - 0.5).abs() < 1e-5);
        assert_eq!(i.sample(5.0).unwrap().0, Vec3::X * 2.0);
        // kürzester Weg über ±π
        assert!((lerp_angle(3.0, -3.0, 0.5).abs() - std::f32::consts::PI).abs() < 0.01);
    }
}
