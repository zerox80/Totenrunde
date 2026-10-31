//! Textur-Nachbearbeitung beim Laden: Mipmaps (gegen Flimmern in der Ferne) und
//! anisotrope Filterung (scharfe Böden unter flachem Blickwinkel).
//!
//! PNG-Dateien bringen keine Mipmaps mit; wir rechnen sie nach dem Laden auf der CPU aus
//! (sRGB-korrekt für Farbtexturen) und hängen sie an die Bilddaten an.
//!
//! 3D-Texturen und Lightmaps werden danach nur noch auf der GPU gebraucht. Ihre Pixeldaten
//! verlassen beim Hochladen den Arbeitsspeicher, statt dort ein zweites Mal zu liegen.

use bevy::asset::{AssetEvent, RenderAssetUsages};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, add_mipmaps);
    }
}

/// Bilder ohne Mipmaps (z. B. Lightmaps, UI-Grafik), erkannt am Dateipfad.
fn skip_path(path: &str) -> bool {
    path.contains("/lm_") || path.starts_with("lm_") || is_ui_path(path)
}

fn is_ui_path(path: &str) -> bool {
    path.starts_with("ui/") || path.contains("fonts/")
}

fn add_mipmaps(
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    server: Res<AssetServer>,
) {
    let ids: Vec<AssetId<Image>> = events
        .read()
        .filter_map(|e| match e {
            AssetEvent::LoadedWithDependencies { id } => Some(*id),
            _ => None,
        })
        .collect();
    for id in ids {
        let Some(path) = server.get_path(id) else {
            continue;
        };
        let path = path.path().to_string_lossy().replace('\\', "/");
        let is_lightmap = path.contains("lm_");
        let Some(mut img) = images.get_mut(id) else {
            continue;
        };
        // UI-Grafik bleibt unangetastet; alles andere braucht nach dem Hochladen keine CPU-Kopie
        if !is_ui_path(&path) {
            img.asset_usage = RenderAssetUsages::RENDER_WORLD;
        }
        if is_lightmap {
            // Lightmaps: weich filtern, an den Rändern nicht wiederholen
            img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::ClampToEdge,
                address_mode_v: ImageAddressMode::ClampToEdge,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                ..default()
            });
            continue;
        }
        if skip_path(&path) || img.texture_descriptor.mip_level_count > 1 {
            continue;
        }
        let srgb = match img.texture_descriptor.format {
            TextureFormat::Rgba8UnormSrgb => true,
            TextureFormat::Rgba8Unorm => false,
            _ => continue,
        };
        let w = img.texture_descriptor.size.width as usize;
        let h = img.texture_descriptor.size.height as usize;
        if w < 64 || h < 64 || img.texture_descriptor.size.depth_or_array_layers != 1 {
            continue;
        }
        if img.data.as_ref().is_none_or(|data| data.len() != w * h * 4) {
            continue;
        }
        let Some(data) = img.data.take() else {
            continue;
        };
        // Texturqualität: große Texturen beim Laden verkleinern (spart Grafikspeicher)
        let max = tex_max();
        let (mut base, mut w, mut h) = (data, w, h);
        while w > max || h > max {
            let (b, nw, nh) = halve(&base, w, h, srgb);
            base = b;
            w = nw;
            h = nh;
        }
        img.texture_descriptor.size.width = w as u32;
        img.texture_descriptor.size.height = h as u32;
        let (all, levels) = build_mips(&base, w, h, srgb);
        img.data = Some(all);
        img.texture_descriptor.mip_level_count = levels;
        // Adressmodus aus dem glTF-Sampler übernehmen, Filter + Anisotropie setzen
        let (au, av) = match &img.sampler {
            ImageSampler::Descriptor(d) => (d.address_mode_u, d.address_mode_v),
            _ => (ImageAddressMode::Repeat, ImageAddressMode::Repeat),
        };
        img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: au,
            address_mode_v: av,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            anisotropy_clamp: 16,
            ..default()
        });
    }
}

/// Größte Kantenlänge geladener Texturen (Einstellung `texture_max`, Test: TR_TEXMAX).
fn tex_max() -> usize {
    std::env::var("TR_TEXMAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| crate::base::settings::Settings::load().texture_max as usize)
}

fn halve(src: &[u8], w: usize, h: usize, srgb: bool) -> (Vec<u8>, usize, usize) {
    let (all, _) = build_mips_levels(src, w, h, srgb, 2);
    let nw = (w / 2).max(1);
    let nh = (h / 2).max(1);
    (all[w * h * 4..w * h * 4 + nw * nh * 4].to_vec(), nw, nh)
}

/// Erzeugt die komplette Mip-Kette (Box-Filter 2×2), Daten hintereinander.
fn build_mips(base: &[u8], w: usize, h: usize, srgb: bool) -> (Vec<u8>, u32) {
    build_mips_levels(base, w, h, srgb, u32::MAX)
}

fn build_mips_levels(
    base: &[u8],
    w: usize,
    h: usize,
    srgb: bool,
    max_levels: u32,
) -> (Vec<u8>, u32) {
    let to_lin: Vec<f32> = (0..256)
        .map(|i| {
            let c = i as f32 / 255.0;
            if !srgb {
                c
            } else if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    let to_u8 = |v: f32| -> u8 {
        let v = v.clamp(0.0, 1.0);
        let e = if !srgb {
            v
        } else if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (e * 255.0 + 0.5) as u8
    };
    let mut out = base.to_vec();
    let mut cur: Vec<u8> = base.to_vec();
    let (mut cw, mut ch) = (w, h);
    let mut levels = 1u32;
    while (cw > 1 || ch > 1) && levels < max_levels {
        let nw = (cw / 2).max(1);
        let nh = (ch / 2).max(1);
        let mut next = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            let y0 = (y * 2).min(ch - 1);
            let y1 = (y * 2 + 1).min(ch - 1);
            for x in 0..nw {
                let x0 = (x * 2).min(cw - 1);
                let x1 = (x * 2 + 1).min(cw - 1);
                let i = [
                    (y0 * cw + x0) * 4,
                    (y0 * cw + x1) * 4,
                    (y1 * cw + x0) * 4,
                    (y1 * cw + x1) * 4,
                ];
                let o = (y * nw + x) * 4;
                for c in 0..3 {
                    let s: f32 = i.iter().map(|&k| to_lin[cur[k + c] as usize]).sum::<f32>() * 0.25;
                    next[o + c] = to_u8(s);
                }
                let a: u32 = i.iter().map(|&k| cur[k + 3] as u32).sum();
                next[o + 3] = ((a + 2) / 4) as u8;
            }
        }
        out.extend_from_slice(&next);
        cur = next;
        cw = nw;
        ch = nh;
        levels += 1;
    }
    (out, levels)
}
