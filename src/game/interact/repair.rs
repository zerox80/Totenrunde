//! Barrikaden reparieren: F gedrückt halten setzt Brett für Brett wieder ein
//! (+10 Punkte pro Brett, wie im Original mit Obergrenze pro Runde). Jedes Brett ist eine
//! Anfrage an den Host; die Punkte gibt es mit seiner Zusage (siehe `requests`).

use bevy::prelude::*;

use super::{Focus, InteractKind, Interactable, Prompt};
use crate::base::AppState;
use crate::base::i18n::t;
use crate::game::perks::{Perk, Perks};
use crate::game::points::Score;
use crate::game::requests::WorldRequest;
use crate::game::rounds::Rounds;
use crate::net::msg::Req;
use crate::net::{Session, local_slot};
use crate::world::barricades::Barricades;

const REPAIR_TIME: f32 = 0.8;
/// Höchstens so viele Reparatur-Punkte pro Runde
pub const ROUND_CAP: u32 = 500;

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
    bars: Res<Barricades>,
    mut score: ResMut<Score>,
    perks: Res<Perks>,
    rounds: Option<Res<Rounds>>,
    session: Option<Res<Session>>,
    mut timer: Local<f32>,
    mut last_round: Local<u32>,
    mut requests: MessageWriter<WorldRequest>,
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
    let Some(b) = bars.list.iter().find(|b| b.id == id) else {
        return;
    };
    if b.missing_boards() == 0 {
        *timer = 0.0;
        return;
    }
    prompt.set(t("prompt.repair"));
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
        requests.write(WorldRequest {
            by: local_slot(session.as_deref()),
            req: Req::Repair(id as u8),
        });
    }
}
