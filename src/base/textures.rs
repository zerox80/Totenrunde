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
    downsample(src, w, h, &SrgbConv::new(srgb))
}

/// Erzeugt die komplette Mip-Kette (Box-Filter 2×2), Daten hintereinander.
fn build_mips(base: &[u8], w: usize, h: usize, srgb: bool) -> (Vec<u8>, u32) {
    let conv = SrgbConv::new(srgb);
    // Die ganze Kette ist etwa 4/3 so groß wie die Basis
    let mut out = Vec::with_capacity(base.len() / 3 * 4 + 64);
    out.extend_from_slice(base);
    let (mut start, mut cw, mut ch) = (0, w, h);
    let mut levels = 1u32;
    while cw > 1 || ch > 1 {
        let (next, nw, nh) = downsample(&out[start..start + cw * ch * 4], cw, ch, &conv);
        start = out.len();
        out.extend_from_slice(&next);
        cw = nw;
        ch = nh;
        levels += 1;
    }
    (out, levels)
}

/// Umrechnung zwischen 8-Bit-Werten und linearem Licht. Für sRGB ersetzt eine Tabelle der
/// Rundungsgrenzen das `powf` pro Pixel und Kanal, im Browser der größte Posten beim Laden.
/// Das Ergebnis entspricht der Rundung von `255 · sRGB(v)`.
struct SrgbConv {
    to_lin: [f32; 256],
    /// Linearer Wert, ab dem auf k + 1 gerundet wird (k = 0..=254); `None` bei linearen Daten
    bounds: Option<[f32; 255]>,
}

impl SrgbConv {
    fn new(srgb: bool) -> Self {
        let decode = |c: f32| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        Self {
            to_lin: std::array::from_fn(|i| {
                let c = i as f32 / 255.0;
                if srgb { decode(c) } else { c }
            }),
            bounds: srgb.then(|| std::array::from_fn(|k| decode((k as f32 + 0.5) / 255.0))),
        }
    }

    fn to_u8(&self, v: f32) -> u8 {
        match &self.bounds {
            None => (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            Some(bounds) => bounds.partition_point(|&b| b <= v) as u8,
        }
    }
}

/// Halbiert ein RGBA8-Bild: Farbe in linearem Licht gemittelt, Alpha direkt.
fn downsample(cur: &[u8], cw: usize, ch: usize, conv: &SrgbConv) -> (Vec<u8>, usize, usize) {
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
                let s: f32 = i
                    .iter()
                    .map(|&k| conv.to_lin[cur[k + c] as usize])
                    .sum::<f32>()
                    * 0.25;
                next[o + c] = conv.to_u8(s);
            }
            let a: u32 = i.iter().map(|&k| cur[k + 3] as u32).sum();
            next[o + 3] = ((a + 2) / 4) as u8;
        }
    }
    (next, nw, nh)
}

#[cfg(test)]
mod tests {
    use super::{SrgbConv, halve};

    /// backend/optimize-textures.py muss exakt dasselbe Ergebnis liefern; es prüft dieselben
    /// Prüfsummen aus backend/texture-golden.json bei jedem Lauf.
    #[test]
    fn downscale_matches_web_texture_tool() {
        let golden: serde_json::Value =
            serde_json::from_str(include_str!("../../backend/texture-golden.json")).unwrap();
        let size = golden["size"].as_u64().unwrap() as usize;
        let target = golden["target"].as_u64().unwrap() as usize;
        let mut pattern = Vec::with_capacity(size * size * 4);
        for y in 0..size {
            for x in 0..size {
                for c in 0..4 {
                    pattern.push(((x * 37 + y * 101 + c * 59 + (x * y) % 7) % 256) as u8);
                }
            }
        }
        for (kind, srgb) in [("srgb", true), ("linear", false)] {
            let (mut image, mut w, mut h) = (pattern.clone(), size, size);
            while w > target || h > target {
                (image, w, h) = halve(&image, w, h, srgb);
            }
            let hash = image.iter().fold(0x811C_9DC5_u32, |value, &byte| {
                (value ^ byte as u32).wrapping_mul(0x0100_0193)
            });
            assert_eq!(u64::from(hash), golden[kind].as_u64().unwrap(), "{kind}");
        }
    }

    /// Bisherige Rundung mit `powf` als Referenz für die Tabellen-Variante.
    fn reference(v: f32) -> u8 {
        let v = v.clamp(0.0, 1.0);
        let e = if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (e * 255.0 + 0.5) as u8
    }

    #[test]
    fn srgb_rounding_matches_powf() {
        let conv = SrgbConv::new(true);
        let steps = 1_000_000;
        let off_by_one = (0..=steps)
            .map(|i| i as f32 / steps as f32)
            .filter(|&v| {
                let (a, b) = (conv.to_u8(v), reference(v));
                assert!(a.abs_diff(b) <= 1, "v={v}: Tabelle {a}, powf {b}");
                a != b
            })
            .count();
        // Abweichungen gibt es nur direkt auf einer Rundungsgrenze (Gleitkomma-Rest)
        assert!(off_by_one <= 8, "{off_by_one} Abweichungen");
    }
}
