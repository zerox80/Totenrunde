//! Türen und Schutthaufen: Öffnen (Kollision, Navigation, Zonen) und Animation.

use bevy::prelude::*;

use super::collision::CollisionWorld;
use super::level::DoorPart;
use super::mapdata::MapFile;
use super::nav::NavGrid;
use super::zones::Zones;
use crate::audio::{Sfx, SfxEvent};
use crate::base::{AppState, GameSet, InGame};

pub struct DoorsPlugin;

impl Plugin for DoorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<OpenDoor>().add_systems(
            Update,
            (open_doors, animate_doors)
                .chain()
                .in_set(GameSet::World)
                .run_if(in_state(AppState::Playing))
                .run_if(resource_exists::<InGame>),
        );
    }
}

/// Tür mit dieser ID öffnen (nach dem Kauf).
#[derive(Message, Clone, Copy)]
pub struct OpenDoor(pub u16);

#[derive(Component)]
pub struct DoorAnim {
    pub t: f32,
}

fn open_doors(
    mut commands: Commands,
    mut events: MessageReader<OpenDoor>,
    map: Res<MapFile>,
    mut zones: ResMut<Zones>,
    mut col: ResMut<CollisionWorld>,
    mut nav: ResMut<NavGrid>,
    parts: Query<(Entity, &DoorPart)>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    for OpenDoor(id) in events.read() {
        let id = *id;
        if zones.is_door_open(id) || id as usize >= zones.door_open.len() {
            continue;
        }
        zones.door_open[id as usize] = true;
        col.set_door_open(id);
        nav.open_door(id);
        if let Some(d) = map.doors.iter().find(|d| d.id == id) {
            for z in &d.zones {
                if *z < zones.open.len() {
                    zones.open[*z] = true;
                }
            }
            let pos = Vec3::from_array(d.prompt);
            sfx.write(SfxEvent::at(if d.kind == "debris" { Sfx::Debris } else { Sfx::Door }, pos));
        }
        for (e, p) in parts.iter() {
            if p.door == id {
                commands.entity(e).insert(DoorAnim { t: 0.0 });
            }
        }
    }
}

fn animate_doors(
    time: Res<Time>,
    mut commands: Commands,
    map: Res<MapFile>,
    mut q: Query<(Entity, &DoorPart, &mut DoorAnim, &mut Transform, &mut Visibility)>,
) {
    for (e, part, mut anim, mut tf, mut vis) in q.iter_mut() {
        anim.t += time.delta_secs();
        let debris = map.doors.iter().any(|d| d.id == part.door && d.kind == "debris");
        let dur = if debris { 1.4 } else { 1.0 };
        let u = (anim.t / dur).min(1.0);
        let k = if debris { u * u } else { 1.0 - (1.0 - u) * (1.0 - u) };
        tf.translation = part.closed.translation.lerp(part.open.translation, k);
        tf.rotation = part.closed.rotation.slerp(part.open.rotation, k);
        if debris {
            // Schutt schwebt nach oben und verschwindet
            tf.scale = part.closed.scale * (1.0 - u * 0.6);
            if u >= 1.0 {
                *vis = Visibility::Hidden;
            }
        }
        if u >= 1.0 {
            commands.entity(e).remove::<DoorAnim>();
        }
    }
}
