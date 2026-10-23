//! Die Spielwelt: Kartendaten, Modell, Beleuchtung, Kollision, Navigation, Zonen,
//! Türen, Barrikaden und Stromversorgung.

pub mod barricades;
pub mod collision;
pub mod doors;
pub mod level;
pub mod lighting;
pub mod mapdata;
pub mod nav;
pub mod power;
pub mod shapes;
pub mod zones;

use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            level::LevelPlugin,
            lighting::LightingPlugin,
            power::PowerPlugin,
            zones::ZonesPlugin,
            doors::DoorsPlugin,
            barricades::BarricadePlugin,
        ));
    }
}
