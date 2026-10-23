//! Roter Bildschirmrand bei Schaden und wenig Leben (prozedurale Vignette).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::base::AppState;
use crate::player::Health;

#[derive(Component)]
pub struct DamageOverlay;

pub fn spawn(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let n = 128u32;
    let mut data = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let u = x as f32 / (n - 1) as f32 * 2.0 - 1.0;
            let v = y as f32 / (n - 1) as f32 * 2.0 - 1.0;
            let r = (u * u * 0.8 + v * v).sqrt();
            let a = ((r - 0.55) / 0.6).clamp(0.0, 1.0).powf(1.6);
            data.extend_from_slice(&[150, 6, 4, (a * 255.0) as u8]);
        }
    }
    let img = Image::new(
        Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    commands.spawn((
        DamageOverlay,
        ImageNode {
            image: images.add(img),
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

pub fn update(health: Query<&Health>, mut q: Query<&mut ImageNode, With<DamageOverlay>>) {
    let Ok(h) = health.single() else { return };
    let low = (1.0 - h.hp / h.max.max(1.0)).clamp(0.0, 1.0);
    let mut a = low * 0.85 + h.hurt_flash * 0.5;
    if h.downed {
        a = 0.9;
    }
    for mut img in q.iter_mut() {
        img.color = Color::srgba(1.0, 1.0, 1.0, a.clamp(0.0, 1.0));
    }
}
