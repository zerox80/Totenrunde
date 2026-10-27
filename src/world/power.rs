//! Stromversorgung der Karte (Perks, Waffen-Verstärker und Lampen brauchen Strom).

use bevy::prelude::*;

use crate::base::AppState;

pub struct PowerPlugin;

impl Plugin for PowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Power>()
            .add_systems(OnEnter(AppState::Playing), |mut p: ResMut<Power>| {
                *p = Power::default()
            });
    }
}

#[derive(Resource, Default)]
pub struct Power {
    pub on: bool,
    /// Zeitpunkt (Sekunden seit Programmstart) des Einschaltens
    pub since: f32,
}
