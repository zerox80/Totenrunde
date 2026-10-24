//! Blutspritzer am Bildschirmrand bei Schaden und wenig Leben (Textur ui/blood_overlay.png).

use bevy::prelude::*;

use crate::base::AppState;
use crate::player::Health;

#[derive(Component)]
pub struct DamageOverlay;

pub fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DamageOverlay,
        ImageNode {
            image: asset_server.load("ui/blood_overlay.png"),
            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(-5),
        DespawnOnExit(AppState::Playing),
    ));
}

pub fn update(time: Res<Time>, health: Query<&Health>, mut q: Query<&mut ImageNode, With<DamageOverlay>>) {
    let Ok(h) = health.single() else { return };
    let low = (1.0 - h.hp / h.max.max(1.0)).clamp(0.0, 1.0);
    // bei wenig Leben pulsiert der Rand (Herzschlag)
    let beat = if low > 0.5 { ((time.elapsed_secs() * 7.5).sin() * 0.5 + 0.5) * 0.15 } else { 0.0 };
    let mut a = low * 0.9 + h.hurt_flash * 0.6 + beat;
    if h.downed {
        a = 1.0;
    }
    for mut img in q.iter_mut() {
        img.color = Color::srgba(1.0, 1.0, 1.0, a.clamp(0.0, 1.0));
    }
}
