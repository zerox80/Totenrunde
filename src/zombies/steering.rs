//! Steuerung: Flow-Field zu den Spielern aktualisieren, Zombies voneinander trennen.

use bevy::prelude::*;

use super::{ZOMBIE_RADIUS, ZState, Zombie};
use crate::game::survivors::Survivors;
use crate::world::collision::{CollisionWorld, F_ZOMBIE};
use crate::world::nav::NavGrid;

#[derive(Resource, Default)]
pub struct FlowTimer(f32);

/// Frühestens nach dieser Zeit (s) wird das Flow-Field neu berechnet.
const FLOW_INTERVAL: f32 = 0.2;

/// Das Flow-Field wird nur neu berechnet, wenn sich die Startzellen der Spieler geändert haben
/// (oder eine Tür aufging): Bei stehenden Spielern käme dasselbe Feld heraus, und die
/// Berechnung kostet im Browser (ein Thread) bei 160 Hz einen ganzen Bildschirmtakt.
pub fn update_flow(
    time: Res<Time>,
    mut timer: ResMut<FlowTimer>,
    mut nav: ResMut<NavGrid>,
    survivors: Res<Survivors>,
) {
    timer.0 = (timer.0 - time.delta_secs()).max(0.0);
    if timer.0 > 0.0 && !nav.goal_cells.is_empty() {
        return;
    }
    let goals = survivors.goals();
    if nav.flow_is_stale(&goals) {
        timer.0 = FLOW_INTERVAL;
        nav.compute_flow(&goals);
    }
}

/// Zombies schieben sich gegenseitig und den Spieler nicht ineinander.
pub fn zombie_separation(
    mut zombies: Query<(Entity, &Zombie, &mut Transform)>,
    col: Res<CollisionWorld>,
    survivors: Res<Survivors>,
) {
    let list: Vec<(Entity, Vec3)> = zombies
        .iter()
        .filter(|(_, z, _)| matches!(z.state, ZState::Chasing | ZState::Attacking { .. }))
        .map(|(e, _, t)| (e, t.translation))
        .collect();
    for (e, z, mut tf) in zombies.iter_mut() {
        if !matches!(z.state, ZState::Chasing | ZState::Attacking { .. }) {
            continue;
        }
        let mut push = Vec3::ZERO;
        let p = tf.translation;
        for (o, op) in &list {
            if *o == e {
                continue;
            }
            let d = Vec3::new(p.x - op.x, 0.0, p.z - op.z);
            let l = d.length();
            let min = ZOMBIE_RADIUS * 2.0;
            if l < min && l > 1e-4 {
                push += d / l * (min - l) * 0.5;
            } else if l <= 1e-4 {
                push += Vec3::new((e.index_u32() % 7) as f32 * 0.01 - 0.03, 0.0, 0.02);
            }
        }
        for s in &survivors.list {
            let d = Vec3::new(p.x - s.pos.x, 0.0, p.z - s.pos.z);
            let l = d.length();
            let min = ZOMBIE_RADIUS + crate::player::RADIUS;
            if l < min && l > 1e-4 {
                push += d / l * (min - l);
            }
        }
        if push != Vec3::ZERO {
            let mut np = p + push;
            col.resolve_cylinder(&mut np, ZOMBIE_RADIUS, 1.7, 0.35, F_ZOMBIE);
            tf.translation = np;
        }
    }
}
