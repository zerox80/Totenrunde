//! Zonen der Karte: welche Bereiche offen sind, wo der Spieler steht und welche Zonen
//! gerade "aktiv" sind (dort spawnen Zombies).

use bevy::prelude::*;

use super::mapdata::MapFile;
use crate::base::{AppState, GameSet, InGame};
use crate::player::Player;

pub struct ZonesPlugin;

impl Plugin for ZonesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), init_zones)
            .add_systems(
                Update,
                track_player_zone
                    .in_set(GameSet::World)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

#[derive(Resource, Default)]
pub struct Zones {
    pub open: Vec<bool>,
    pub door_open: Vec<bool>,
    /// (Tür-ID, Zone A, Zone B)
    pub links: Vec<(u16, usize, usize)>,
    pub player_zone: usize,
}

impl Zones {
    /// Zone des Spielers und alle über offene Türen angrenzenden Zonen.
    pub fn active_zones(&self) -> Vec<usize> {
        let mut v = vec![self.player_zone];
        for (d, a, b) in &self.links {
            if !self.door_open.get(*d as usize).copied().unwrap_or(false) {
                continue;
            }
            if *a == self.player_zone && !v.contains(b) {
                v.push(*b);
            }
            if *b == self.player_zone && !v.contains(a) {
                v.push(*a);
            }
        }
        v
    }

    pub fn is_door_open(&self, id: u16) -> bool {
        self.door_open.get(id as usize).copied().unwrap_or(false)
    }
}

fn init_zones(mut commands: Commands, map: Res<MapFile>) {
    let max_door = map.doors.iter().map(|d| d.id as usize).max().unwrap_or(0);
    let mut open = vec![false; map.zones.len()];
    for z in &map.zones {
        if z.open {
            open[z.id] = true;
        }
    }
    let links = map
        .doors
        .iter()
        .filter(|d| d.zones.len() >= 2)
        .map(|d| (d.id, d.zones[0], d.zones[1]))
        .collect();
    let player_zone = map.zone_at(map.player_start.pos()).unwrap_or(0);
    commands.insert_resource(Zones {
        open,
        door_open: vec![false; max_door + 1],
        links,
        player_zone,
    });
}

fn track_player_zone(
    map: Res<MapFile>,
    mut zones: ResMut<Zones>,
    player: Single<&Transform, With<Player>>,
) {
    if let Some(z) = map.zone_at(player.translation) {
        if zones.player_zone != z {
            zones.player_zone = z;
        }
    }
}
