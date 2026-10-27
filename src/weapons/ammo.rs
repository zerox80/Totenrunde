//! Waffen vergeben (Wandkauf, Kiste, Verstärker) und Max. Munition.

use bevy::prelude::*;

use super::defs::def;
use super::loadout::{GiveWeapon, Loadout, MAX_GRENADES, WState, WeaponSlot};
use crate::audio::{Sfx, SfxEvent};
use crate::game::perks::Perks;
use crate::game::powerups::{PowerupCollected, PowerupKind};

pub fn give_weapons(
    mut events: MessageReader<GiveWeapon>,
    mut lo: Single<&mut Loadout>,
    perks: Res<Perks>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    for ev in events.read() {
        let max = Loadout::max_slots(&perks);
        if let Some(i) = lo.has(ev.weapon) {
            let s = &mut lo.slots[i];
            s.upgraded |= ev.upgraded;
            s.refill();
            if lo.current != i {
                lo.state = WState::Lowering {
                    t: 0.0,
                    to: Some(i),
                };
            }
        } else if lo.slots.len() < max {
            lo.slots.push(WeaponSlot::new(ev.weapon, ev.upgraded));
            let to = lo.slots.len() - 1;
            if lo.slots.len() == 1 {
                lo.current = 0;
                lo.state = WState::Raising { t: 0.0 };
            } else {
                lo.state = WState::Lowering {
                    t: 0.0,
                    to: Some(to),
                };
            }
        } else {
            let c = lo.current.min(lo.slots.len() - 1);
            lo.slots[c] = WeaponSlot::new(ev.weapon, ev.upgraded);
            lo.state = WState::Raising { t: 0.0 };
        }
        sfx.write(SfxEvent::ui(Sfx::Switch));
    }
}

pub fn max_ammo(mut events: MessageReader<PowerupCollected>, mut lo: Single<&mut Loadout>) {
    for ev in events.read() {
        if ev.kind == PowerupKind::MaxAmmo {
            for s in lo.slots.iter_mut() {
                let d = def(s.weapon);
                s.reserve = d.reserve_size(s.upgraded);
            }
            lo.grenades = MAX_GRENADES;
        }
    }
}

// --------------------------------------------------------------------------------------
// Ego-Waffe animieren
// --------------------------------------------------------------------------------------
