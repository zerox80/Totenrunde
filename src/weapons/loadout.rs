//! Waffen-Inventar des Spielers: Slots, Munition, Zustandsautomat und Waffenvergabe.

use bevy::prelude::*;

use super::defs::{def, weapon_index, WeaponClass};
use crate::game::perks::{Perk, Perks};
use crate::player::Player;

pub const MAX_GRENADES: u32 = 4;
pub const KNIFE_DAMAGE: f32 = 150.0;

#[derive(Clone, Copy, Debug)]
pub struct WeaponSlot {
    pub weapon: usize,
    pub upgraded: bool,
    pub mag: u32,
    pub reserve: u32,
}

impl WeaponSlot {
    pub fn new(weapon: usize, upgraded: bool) -> Self {
        let d = def(weapon);
        Self {
            weapon,
            upgraded,
            mag: d.mag_size(upgraded),
            reserve: d.reserve_size(upgraded),
        }
    }
    pub fn refill(&mut self) {
        let d = def(self.weapon);
        self.mag = d.mag_size(self.upgraded);
        self.reserve = d.reserve_size(self.upgraded);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WState {
    Ready,
    Reloading { t: f32, dur: f32 },
    ShellReload { t: f32, per: f32 },
    Lowering { t: f32, to: Option<usize> },
    Raising { t: f32 },
    Melee { t: f32, hit: bool },
    Throw { t: f32, thrown: bool },
    /// Waffe gesenkt (Perk trinken, Kiste, ...)
    Busy { t: f32, dur: f32 },
    Empty,
}

#[derive(Component)]
pub struct Loadout {
    pub slots: Vec<WeaponSlot>,
    pub current: usize,
    pub state: WState,
    pub fire_cd: f32,
    pub burst_left: u32,
    pub ads: f32,
    pub bloom: f32,
    pub grenades: u32,
    pub sprint_out: f32,
    /// zum Wiederherstellen nach "Letztes Gefecht"
    pub saved_current: Option<usize>,
    pub last_stand_slot: bool,
    pub flash: f32,
    pub kick: f32,
    pub shots: u64,
}

impl Loadout {
    pub fn current_slot(&self) -> Option<&WeaponSlot> {
        self.slots.get(self.current)
    }
    pub fn current_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots.get_mut(self.current)
    }
    pub fn has(&self, weapon: usize) -> Option<usize> {
        self.slots.iter().position(|s| s.weapon == weapon)
    }
    pub fn max_slots(perks: &Perks) -> usize {
        if perks.has(Perk::MuleKick) {
            3
        } else {
            2
        }
    }
    pub fn busy(&mut self, dur: f32) {
        self.state = WState::Busy { t: 0.0, dur };
    }
    pub fn can_act(&self) -> bool {
        matches!(self.state, WState::Ready | WState::Empty)
    }
}

/// Waffe geben (Wandkauf, Kiste, Verstärker).
#[derive(Message, Clone, Copy)]
pub struct GiveWeapon {
    pub weapon: usize,
    pub upgraded: bool,
}

/// Aktuelle Waffe abgeben (für den Waffen-Verstärker). Liefert den Slot.
pub fn take_current(lo: &mut Loadout) -> Option<WeaponSlot> {
    if lo.slots.is_empty() {
        return None;
    }
    let s = lo.slots.remove(lo.current);
    lo.current = 0;
    lo.state = if lo.slots.is_empty() { WState::Empty } else { WState::Raising { t: 0.0 } };
    Some(s)
}

/// Am Boden: auf eine Pistole wechseln (notfalls eine M1911 leihen).
pub fn enter_last_stand(lo: &mut Loadout) {
    lo.saved_current = Some(lo.current);
    let pistol = lo
        .slots
        .iter()
        .position(|s| matches!(def(s.weapon).class, WeaponClass::Pistol | WeaponClass::Wonder));
    match pistol {
        Some(i) => {
            lo.current = i;
            lo.last_stand_slot = false;
        }
        None => {
            let w = weapon_index("m1911").unwrap();
            lo.slots.push(WeaponSlot::new(w, false));
            lo.current = lo.slots.len() - 1;
            lo.last_stand_slot = true;
        }
    }
    lo.state = WState::Raising { t: 0.0 };
}

pub fn leave_last_stand(lo: &mut Loadout) {
    if lo.last_stand_slot && !lo.slots.is_empty() {
        lo.slots.pop();
        lo.last_stand_slot = false;
    }
    if let Some(c) = lo.saved_current.take() {
        lo.current = c.min(lo.slots.len().saturating_sub(1));
    }
    lo.state = if lo.slots.is_empty() { WState::Empty } else { WState::Raising { t: 0.0 } };
}

/// Inventar beim Spielstart: M1911 mit Messer und 2 Granaten (wie in BO2).
pub fn spawn_loadout(mut commands: Commands, player: Query<Entity, With<Player>>) {
    let Ok(pe) = player.single() else { return };
    let start = weapon_index("m1911").unwrap();
    commands.entity(pe).insert(Loadout {
        slots: vec![WeaponSlot::new(start, false)],
        current: 0,
        state: WState::Raising { t: 0.0 },
        fire_cd: 0.0,
        burst_left: 0,
        ads: 0.0,
        bloom: 0.0,
        grenades: 2,
        sprint_out: 0.0,
        saved_current: None,
        last_stand_slot: false,
        flash: 0.0,
        kick: 0.0,
        shots: 0,
    });
}
