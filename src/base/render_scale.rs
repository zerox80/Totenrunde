//! Renderauflösung (`render_scale`, im Browser einstellbar): Bei weniger als 100 % rendert die
//! Ego-Kamera in ein entsprechend kleineres Bild, das eine zweite Kamera bildschirmfüllend
//! anzeigt. Das Fenster behält seine volle Größe, daher bleiben HUD und Menüs scharf und die
//! Mauskoordinaten passen weiter zum Bild.
//!
//! Früher wurde stattdessen die Fenstergröße verkleinert. Im Browser setzt winit daraufhin die
//! CSS-Größe der Canvas, und die Mausposition meldet es weiter in voller Auflösung; das Bild
//! schrumpfte oder sprang, Klicks landeten an der falschen Stelle.

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::ui::widget::NodeImageMode;
use bevy::window::PrimaryWindow;

use super::AppState;
use super::settings::Settings;
use crate::player::PlayerCamera;

pub struct RenderScalePlugin;

impl Plugin for RenderScalePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_render_scale);
    }
}

/// Anzeige des verkleinerten Bildes (zweite Kamera und bildschirmfüllender UI-Knoten).
#[derive(Component)]
struct ScaledView;

/// Auf dem Desktop nur für Tests aktiv (`TR_RENDERSCALE=0.5`), im Browser per Einstellung.
pub fn enabled() -> bool {
    cfg!(target_arch = "wasm32") || std::env::var_os("TR_RENDERSCALE").is_some()
}

fn render_scale(settings: &Settings) -> f32 {
    std::env::var("TR_RENDERSCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(settings.render_scale)
        .clamp(0.5, 1.0)
}

fn sync_render_scale(
    mut commands: Commands,
    settings: Res<Settings>,
    window: Single<&Window, With<PrimaryWindow>>,
    cameras: Query<(Entity, &RenderTarget), With<PlayerCamera>>,
    mut views: Query<(Entity, Option<&mut ImageNode>), With<ScaledView>>,
    mut images: ResMut<Assets<Image>>,
) {
    let full = window.physical_size();
    let scale = render_scale(&settings);
    let Ok((camera, target)) = cameras.single() else {
        return;
    };
    if scale >= 0.999 || full.x == 0 || full.y == 0 {
        if target.as_image().is_some() {
            commands.entity(camera).insert(RenderTarget::default());
        }
        for (view, _) in &views {
            commands.entity(view).despawn();
        }
        return;
    }

    let size = (full.as_vec2() * scale).round().as_uvec2().max(UVec2::ONE);
    let extent = Extent3d {
        width: size.x,
        height: size.y,
        depth_or_array_layers: 1,
    };
    let handle = match target.as_image() {
        Some(handle) => {
            if images.get(handle).is_some_and(|image| image.size() != size)
                && let Some(mut image) = images.get_mut(handle)
            {
                image.resize(extent);
            }
            handle.clone()
        }
        None => {
            let handle = images.add(Image::new_target_texture(
                size.x,
                size.y,
                TextureFormat::Rgba8UnormSrgb,
                None,
            ));
            commands
                .entity(camera)
                .insert(RenderTarget::Image(handle.clone().into()));
            handle
        }
    };

    if views.is_empty() {
        commands.spawn((
            ScaledView,
            Camera2d,
            Camera {
                order: 1,
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ));
        commands.spawn((
            ScaledView,
            ImageNode {
                image: handle,
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            // hinter allen HUD-Elementen und Menüs
            GlobalZIndex(i32::MIN),
            DespawnOnExit(AppState::Playing),
        ));
    } else {
        for (_, node) in &mut views {
            if let Some(mut node) = node
                && node.image != handle
            {
                node.image = handle.clone();
            }
        }
    }
}
