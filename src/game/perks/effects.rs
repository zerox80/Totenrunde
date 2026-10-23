//! Perk-Effekte: Ausdauer/Tempo (Ausdauer-Limo) und Aktivierung nach dem Trinken.
//! Weitere Effekte werden direkt dort abgefragt, wo sie wirken (Gesundheit, Nachladen,
//! Feuerrate, Waffenslots).

use bevy::prelude::*;

use super::{Perk, Perks};
use crate::player::{Player, PLAYER_STAMINA};

pub fn finish_drinking(time: Res<Time>, mut perks: ResMut<Perks>) {
    if let Some((p, t)) = perks.drinking {
        let t = t - time.delta_secs();
        if t <= 0.0 {
            perks.drinking = None;
            if !perks.owned.contains(&p) {
                perks.owned.push(p);
            }
        } else {
            perks.drinking = Some((p, t));
        }
    }
}

pub fn apply_perk_effects(perks: Res<Perks>, mut player: Single<&mut Player>) {
    let stamin = perks.has(Perk::StaminUp);
    player.stamina_max = if stamin { PLAYER_STAMINA * 2.0 } else { PLAYER_STAMINA };
    if player.stamina > player.stamina_max {
        player.stamina = player.stamina_max;
    }
    player.speed_mult = if player.speed_mult < 0.9 {
        player.speed_mult
    } else if stamin {
        1.07
    } else {
        1.0
    };
}
