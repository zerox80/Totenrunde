use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use web_sys::wasm_bindgen::JsCast;

use super::settings::Settings;

pub struct WebPlugin;

impl Plugin for WebPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_render_resolution);
    }
}

fn sync_render_resolution(
    settings: Res<Settings>,
    mut resized: MessageReader<WindowResized>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !settings.is_changed() && resized.is_empty() {
        return;
    }
    resized.clear();

    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let Some(browser) = web_sys::window() else {
        return;
    };
    let Some(canvas) = browser
        .document()
        .and_then(|document| document.get_element_by_id("game-canvas"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
    else {
        return;
    };

    let dpr = browser.device_pixel_ratio() as f32;
    let scale = settings.render_scale.clamp(0.5, 1.0);
    let width = (canvas.client_width().max(1) as f32 * dpr * scale).round() as u32;
    let height = (canvas.client_height().max(1) as f32 * dpr * scale).round() as u32;
    let size = UVec2::new(width.max(1), height.max(1));
    if window.resolution.physical_size() != size {
        window.resolution.set_physical_resolution(size.x, size.y);
    }
    let scale_factor = dpr / scale;
    if window.resolution.scale_factor_override() != Some(scale_factor) {
        window
            .resolution
            .set_scale_factor_override(Some(scale_factor));
    }
}
