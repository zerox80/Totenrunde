//! Lichter der Karte: Flackern und Anlaufen der Lampen, wenn der Strom eingeschaltet wird.

use bevy::prelude::*;

use super::mapdata::MapFile;
use super::power::Power;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (flicker_lights, power_lights).run_if(resource_exists::<MapFile>));
    }
}

/// Multiplikator für die Lichtstärken aus der Karte (Lumen).
pub const LIGHT_SCALE: f32 = 8.0;

#[derive(Component)]
pub struct MapLight {
    pub base: f32,
    pub flicker: f32,
    pub needs_power: bool,
    pub seed: f32,
}

pub fn spawn_map_lights(commands: &mut Commands, map: &MapFile) {
    for (i, l) in map.lights.iter().enumerate() {
        let base = l.intensity * LIGHT_SCALE;
        commands.spawn((
            Name::new(format!("Licht {i}")),
            PointLight {
                color: Color::linear_rgb(l.color[0], l.color[1], l.color[2]),
                intensity: if l.needs_power { 0.0 } else { base },
                range: l.range,
                radius: 0.1,
                shadow_maps_enabled: l.shadows,
                ..default()
            },
            Transform::from_translation(Vec3::from_array(l.pos)),
            MapLight {
                base,
                flicker: l.flicker,
                needs_power: l.needs_power,
                seed: i as f32 * 7.31,
            },
        ));
    }
}

fn flicker_lights(time: Res<Time>, power: Res<Power>, mut lights: Query<(&mut PointLight, &MapLight)>) {
    let t = time.elapsed_secs();
    for (mut pl, ml) in lights.iter_mut() {
        if ml.needs_power && !power.on {
            pl.intensity = 0.0;
            continue;
        }
        if ml.flicker <= 0.0 {
            continue;
        }
        // Mischung aus Rauschen und gelegentlichen Aussetzern
        let n = (t * 13.0 + ml.seed).sin() * 0.5 + (t * 31.7 + ml.seed * 2.0).sin() * 0.3 + (t * 5.3).sin() * 0.2;
        let cut = ((t * 2.3 + ml.seed).sin() > 0.93) as i32 as f32;
        let f = (1.0 - ml.flicker * 0.35 * (n * 0.5 + 0.5) - cut * ml.flicker * 0.8).clamp(0.05, 1.0);
        pl.intensity = ml.base * f;
    }
}

fn power_lights(time: Res<Time>, power: Res<Power>, mut lights: Query<(&mut PointLight, &MapLight)>) {
    if !power.on {
        return;
    }
    let dt = time.elapsed_secs() - power.since;
    for (mut pl, ml) in lights.iter_mut() {
        if !ml.needs_power || ml.flicker > 0.0 {
            continue;
        }
        if dt > 3.0 {
            pl.intensity = ml.base;
            continue;
        }
        // Anlaufen der Lampen mit kurzem Flackern
        let k = if dt < 0.8 {
            ((dt * 40.0).sin() > 0.2) as i32 as f32 * 0.6
        } else {
            (dt / 1.5).min(1.0)
        };
        pl.intensity = ml.base * k;
    }
}
