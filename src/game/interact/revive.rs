//! Koop: niedergestreckte Mitspieler wiederbeleben. F gedrückt halten, mit Zweiter Atem
//! geht es doppelt so schnell. Die Mitspieler stehen als benutzbare Punkte in der Welt,
//! solange sie am Boden liegen (siehe `net::avatars`).

use bevy::prelude::*;
use totenrunde_mp_client::protocol::Target;

use super::{Focus, InteractKind, Prompt};
use crate::audio::{Sfx, SfxEvent};
use crate::base::i18n::tf;
use crate::game::perks::{Perk, Perks};
use crate::game::points::Score;
use crate::net::msg::{GameMsg, Life};
use crate::net::{GameOut, Team};

pub const REVIVE_TIME: f32 = 3.0;

#[allow(clippy::too_many_arguments)]
pub fn use_revive(
    time: Res<Time>,
    focus: Res<Focus>,
    mut prompt: ResMut<Prompt>,
    perks: Res<Perks>,
    team: Option<ResMut<Team>>,
    mut score: ResMut<Score>,
    mut out: ResMut<GameOut>,
    mut progress: Local<Option<(u8, f32)>>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let (Some(InteractKind::Revive(slot)), Some(mut team)) = (focus.kind, team) else {
        *progress = None;
        return;
    };
    let Some(member) = team.get_mut(slot) else {
        return;
    };
    if member.state.life != Life::Downed {
        *progress = None;
        return;
    }
    if !focus.held {
        prompt.set(tf("prompt.revive", &[("name", &member.name)]));
        *progress = None;
        return;
    }
    let dur = if perks.has(Perk::QuickRevive) {
        REVIVE_TIME * 0.5
    } else {
        REVIVE_TIME
    };
    let t = match *progress {
        Some((s, t)) if s == slot => t + time.delta_secs(),
        _ => 0.0,
    };
    let percent = ((t / dur).min(1.0) * 100.0) as u32;
    prompt.set(tf(
        "prompt.reviving",
        &[("name", &member.name), ("percent", &percent)],
    ));
    if t >= dur {
        out.send(Target::Player(slot), GameMsg::Revive);
        // bis zur Bestätigung durch seinen nächsten Zustand nicht nochmal anbieten
        member.state.life = Life::Alive;
        score.revives += 1;
        sfx.write(SfxEvent::ui(Sfx::Buy));
        *progress = None;
    } else {
        *progress = Some((slot, t));
    }
}
