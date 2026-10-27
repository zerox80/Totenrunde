//! Barrikaden reparieren: F gedrückt halten setzt Brett für Brett wieder ein
//! (+10 Punkte pro Brett, wie im Original mit Obergrenze pro Runde).

use bevy::prelude::*;

use super::{Focus, InteractKind, Interactable, Prompt};
use crate::audio::{Sfx, SfxEvent};
use crate::base::AppState;
use crate::game::perks::{Perk, Perks};
use crate::game::points::Score;
use crate::game::powerups::ActivePowerups;
use crate::game::rounds::Rounds;
use crate::world::barricades::Barricades;

const REPAIR_TIME: f32 = 0.8;
const ROUND_CAP: u32 = 500;

pub fn spawn_repair_points(mut commands: Commands, bars: Res<Barricades>) {
    for b in &bars.list {
        let inward = (b.inside - b.outside).normalize_or_zero();
        commands.spawn((
            Name::new(format!("Barrikade {}", b.id)),
            Interactable {
                kind: InteractKind::Barricade(b.id),
                point: Vec3::new(b.inside.x, 1.0, b.inside.z),
                radius: 1.7,
                facing: Some(inward),
            },
            Transform::from_translation(b.inside),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn use_repair(
    time: Res<Time>,
    focus: Res<Focus>,
    mut prompt: ResMut<Prompt>,
    mut bars: ResMut<Barricades>,
    mut score: ResMut<Score>,
    perks: Res<Perks>,
    powerups: Res<ActivePowerups>,
    rounds: Option<Res<Rounds>>,
    mut timer: Local<f32>,
    mut last_round: Local<u32>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let round = rounds.map(|r| r.round).unwrap_or(1);
    if round != *last_round {
        *last_round = round;
        score.repair_points_round = 0;
    }
    let Some(InteractKind::Barricade(id)) = focus.kind else {
        *timer = 0.0;
        return;
    };
    let Some(b) = bars.list.iter_mut().find(|b| b.id == id) else {
        return;
    };
    if b.missing_boards() == 0 {
        *timer = 0.0;
        return;
    }
    prompt.set("Halte F, um die Barrikade zu reparieren");
    if !focus.held {
        *timer = 0.0;
        return;
    }
    let dur = if perks.has(Perk::SpeedCola) {
        REPAIR_TIME * 0.5
    } else {
        REPAIR_TIME
    };
    *timer += time.delta_secs();
    if *timer >= dur {
        *timer = 0.0;
        if b.repair_one() {
            sfx.write(SfxEvent::at(Sfx::BoardPlace, b.center + Vec3::Y * 1.3));
            score.boards += 1;
            if score.repair_points_round < ROUND_CAP {
                let got = score.add(10, &powerups, None);
                score.repair_points_round += got;
            }
        }
    }
}
