//! Auswahl der Spawnpunkte: nur in der Zone des Spielers und angrenzenden offenen Zonen,
//! Aufsteiger nicht zu nah am Spieler, nähere Spawner werden bevorzugt.

use bevy::prelude::*;
use rand::Rng;

use crate::world::barricades::Barricades;
use crate::world::mapdata::MapFile;
use crate::world::zones::Zones;

/// Wählt einen Spawner in einer aktiven Zone (Zone des Spielers + angrenzende offene Zonen).
pub fn pick_spawner(map: &MapFile, zones: &Zones, barricades: &Barricades, ppos: Vec3, rng: &mut impl Rng) -> Option<usize> {
    let active = zones.active_zones();
    let mut cands: Vec<(usize, f32)> = Vec::new();
    for (i, sp) in map.spawners.iter().enumerate() {
        if !active.contains(&sp.zone) {
            continue;
        }
        let pos = Vec3::from_array(sp.pos);
        let d = Vec3::new(pos.x - ppos.x, 0.0, pos.z - ppos.z).length();
        if sp.kind == "riser" {
            if d < 5.0 {
                continue;
            }
        } else if let Some(b) = sp.barricade {
            // Fenster mit wartenden Zombies weniger bevorzugen
            if b < barricades.list.len() && barricades.list[b].occupant.is_some() && rng.random::<f32>() < 0.5 {
                continue;
            }
        }
        let w = 1.0 / (1.0 + d / 14.0);
        cands.push((i, w));
    }
    if cands.is_empty() {
        return None;
    }
    let total: f32 = cands.iter().map(|c| c.1).sum();
    let mut x = rng.random::<f32>() * total;
    for (i, w) in &cands {
        if x <= *w {
            return Some(*i);
        }
        x -= w;
    }
    Some(cands[cands.len() - 1].0)
}
