//! Türen und Schutthaufen kaufen.

use bevy::prelude::*;

use super::{Focus, InteractKind, Interactable, Prompt, cost_text, try_buy};
use crate::audio::SfxEvent;
use crate::base::AppState;
use crate::game::points::Score;
use crate::world::doors::OpenDoor;
use crate::world::mapdata::MapFile;
use crate::world::zones::Zones;

#[derive(Component)]
pub struct DoorPrompt {
    pub id: u16,
    pub cost: u32,
    pub debris: bool,
}

pub fn spawn_door_prompts(mut commands: Commands, map: Res<MapFile>) {
    for d in &map.doors {
        let p = Vec3::from_array(d.prompt);
        commands.spawn((
            Name::new(format!("Tür {}", d.id)),
            DoorPrompt {
                id: d.id,
                cost: d.cost,
                debris: d.kind == "debris",
            },
            Interactable {
                kind: InteractKind::Door(d.id),
                point: Vec3::new(p.x, 1.0, p.z),
                radius: 2.3,
                facing: None,
            },
            Transform::from_translation(p),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn use_door(
    mut commands: Commands,
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
    mut score: ResMut<Score>,
    zones: Res<Zones>,
    doors: Query<(Entity, &DoorPrompt)>,
    mut open: MessageWriter<OpenDoor>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    // Bereits geöffnete Türen nicht mehr anbieten
    for (e, d) in doors.iter() {
        if zones.is_door_open(d.id) {
            commands.entity(e).despawn();
        }
    }
    let Some(InteractKind::Door(id)) = focus.kind else {
        return;
    };
    let Some(e) = focus.entity else { return };
    let Ok((_, d)) = doors.get(e) else { return };
    if zones.is_door_open(id) {
        return;
    }
    let what = if d.debris {
        "den Schutt wegzuräumen"
    } else {
        "die Tür zu öffnen"
    };
    prompt.set(format!("Halte F, um {what} {}", cost_text(d.cost)));
    if focus.pressed && try_buy(&mut score, d.cost, &mut sfx) {
        open.write(OpenDoor(id));
        score.doors += 1;
        focus.pressed = false;
    }
}
