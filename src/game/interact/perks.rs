//! Perk-Automaten benutzen: kaufen und trinken (Waffe gesenkt, Flasche an den Mund).

use bevy::prelude::*;

use super::{cost_text, try_buy, Focus, InteractKind, Prompt};
use crate::audio::{Sfx, SfxEvent};
use crate::game::perks::Perks;
use crate::game::points::Score;
use crate::weapons::Loadout;
use crate::world::power::Power;

pub const DRINK_TIME: f32 = 2.7;

pub fn use_perk_machine(
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
    mut score: ResMut<Score>,
    mut perks: ResMut<Perks>,
    power: Res<Power>,
    mut lo: Single<&mut Loadout>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(InteractKind::Perk(perk)) = focus.kind else { return };
    if perk.needs_power() && !power.on {
        prompt.set("Der Strom muss eingeschaltet sein");
        return;
    }
    if perks.has(perk) || perks.drinking.map(|d| d.0 == perk).unwrap_or(false) {
        return;
    }
    if perks.full() {
        prompt.set("Du kannst keine weiteren Perks tragen");
        return;
    }
    prompt.set(format!("Halte F für {} {}", perk.name(), cost_text(perk.cost())));
    if focus.pressed && lo.can_act() && try_buy(&mut score, perk.cost(), &mut sfx) {
        perks.drinking = Some((perk, DRINK_TIME));
        lo.busy(DRINK_TIME);
        sfx.write(SfxEvent::ui(Sfx::PerkDrink));
        focus.pressed = false;
    }
}
