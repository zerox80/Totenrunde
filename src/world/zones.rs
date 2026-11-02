//! Zonen der Karte: welche Bereiche offen sind, wo die Spieler stehen und welche Zonen
//! gerade "aktiv" sind (dort spawnen Zombies).

use bevy::prelude::*;

use super::mapdata::MapFile;
use crate::base::{AppState, GameSet, InGame};
use crate::game::survivors::Survivors;

pub struct ZonesPlugin;

impl Plugin for ZonesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), init_zones)
            .add_systems(
                Update,
                track_player_zones
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
    /// Zonen, in denen gerade ein Spieler steht (zuletzt bekannte, falls außerhalb)
    pub player_zones: Vec<usize>,
}

impl Zones {
    /// Zonen der Spieler und alle über offene Türen angrenzenden Zonen.
    pub fn active_zones(&self) -> Vec<usize> {
        let mut v = self.player_zones.clone();
        for (d, a, b) in &self.links {
            if !self.door_open.get(*d as usize).copied().unwrap_or(false) {
                continue;
            }
            if self.player_zones.contains(a) && !v.contains(b) {
                v.push(*b);
            }
            if self.player_zones.contains(b) && !v.contains(a) {
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
        player_zones: vec![player_zone],
    });
}

fn track_player_zones(map: Res<MapFile>, mut zones: ResMut<Zones>, survivors: Res<Survivors>) {
    let mut now: Vec<usize> = survivors
        .list
        .iter()
        .filter_map(|s| map.zone_at(s.pos))
        .collect();
    now.sort_unstable();
    now.dedup();
    // Außerhalb aller Zonen (Fensterbank, Sprung): letzte bekannte Zonen behalten
    if !now.is_empty() && zones.player_zones != now {
        zones.player_zones = now;
    }
}
