//! Renderauflösung (`render_scale`, im Browser einstellbar): Bei weniger als 100 % rendert die
//! Ego-Kamera in ein entsprechend kleineres Bild, das eine zweite Kamera bildschirmfüllend
//! anzeigt. Das Fenster behält seine volle Größe, daher bleiben HUD und Menüs scharf und die
//! Mauskoordinaten passen weiter zum Bild.
//!
//! Früher wurde stattdessen die Fenstergröße verkleinert. Im Browser setzt winit daraufhin die
//! CSS-Größe der Canvas, und die Mausposition meldet es weiter in voller Auflösung; das Bild
//! schrumpfte oder sprang, Klicks landeten an der falschen Stelle.
//!
//! Dazu kommt ein Pixelbudget für hochauflösende Bildschirme (Retina-MacBooks, Windows mit
//! 150 % Skalierung, Browser mit Pixelfaktor 2): Dort hätte die 3D-Szene sonst das Doppelte
//! an Breite und Höhe, also viermal so viele Pixel, und integrierte Grafik bricht ein. Die
//! Szene rendert höchstens in logischer Auflösung bzw. im Budget der Grafikstufe; HUD und
//! Schrift bleiben in voller Auflösung scharf. Bildschirme ohne Skalierung betrifft das nicht.

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

/// Pixel der 3D-Szene, die eine Grafikstufe auf hochauflösenden Bildschirmen höchstens
/// bekommt (Extrem: unbegrenzt).
fn pixel_budget(quality: u8) -> f32 {
    match quality {
        0 => 1280.0 * 720.0,
        1 => 1600.0 * 900.0,
        3 => f32::INFINITY,
        _ => 1920.0 * 1080.0,
    }
}

/// Höchster Maßstab für dieses Fenster: mindestens die logische Auflösung, sonst das Budget.
/// Ohne Bildschirmskalierung (logisch = physisch) ist das immer 1.
pub fn hidpi_cap(physical: UVec2, scale_factor: f32, quality: u8) -> f32 {
    let physical_px = physical.x as f32 * physical.y as f32;
    if physical_px <= 0.0 {
        return 1.0;
    }
    let logical_px = physical_px / (scale_factor * scale_factor).max(1.0);
    let allowed = logical_px.max(pixel_budget(quality));
    (allowed / physical_px).sqrt().min(1.0)
}

/// Maßstab der 3D-Szene: Test-Überschreibung (`TR_RENDERSCALE=0.5`), im Browser die
/// Einstellung; auf dem Desktop volle Auflösung. Jeweils begrenzt durch das Pixelbudget.
fn render_scale(settings: &Settings, window: &Window) -> f32 {
    let chosen = std::env::var("TR_RENDERSCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(if cfg!(target_arch = "wasm32") {
            settings.render_scale
        } else {
            1.0
        });
    let cap = hidpi_cap(
        window.physical_size(),
        window.scale_factor(),
        settings.quality,
    );
    chosen.min(cap).clamp(0.4, 1.0)
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
    let scale = render_scale(&settings, &window);
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

#[cfg(test)]
mod tests {
    use super::hidpi_cap;
    use bevy::math::UVec2;

    #[test]
    fn only_scaled_screens_are_capped() {
        // Normale Bildschirme ohne Skalierung: immer volle Auflösung
        assert_eq!(hidpi_cap(UVec2::new(1920, 1080), 1.0, 2), 1.0);
        assert_eq!(hidpi_cap(UVec2::new(2560, 1440), 1.0, 2), 1.0);
        assert_eq!(hidpi_cap(UVec2::new(3840, 2160), 1.0, 0), 1.0);
        // Retina-MacBook (2880 x 1800, Faktor 2): Stufe Hoch rendert etwa 1814 x 1134
        let cap = hidpi_cap(UVec2::new(2880, 1800), 2.0, 2);
        assert!((cap - 0.63).abs() < 0.01, "{cap}");
        // Stufe Niedrig: mindestens die logische Auflösung (1440 x 900)
        let cap = hidpi_cap(UVec2::new(2880, 1800), 2.0, 0);
        assert!((cap - 0.5).abs() < 0.01, "{cap}");
        // Extrem: keine Begrenzung
        assert_eq!(hidpi_cap(UVec2::new(2880, 1800), 2.0, 3), 1.0);
    }
}
