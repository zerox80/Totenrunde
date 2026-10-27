//! Fernrohrbild beim Zielen mit einer Zielfernrohr-Waffe: runder Ausschnitt mit weichem
//! Rand, schwarze Seitenbalken und Duplex-Fadenkreuz. Die Ego-Waffe ist dann ausgeblendet
//! (siehe `viewmodel::SCOPE_IN`), sonst blickte man durch ein enges Rohr.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::base::AppState;
use crate::weapons::viewmodel::SCOPE_IN;
use crate::weapons::{Loadout, def};

#[derive(Component)]
pub struct ScopeOverlay;

const MASK: u32 = 512;
const RETICLE: Color = Color::srgba(0.0, 0.0, 0.0, 0.92);

/// Schwarz außerhalb des Kreises, innen zum Rand hin leicht abgedunkelt.
fn mask_image() -> Image {
    let n = MASK as usize;
    let mut data = Vec::with_capacity(n * n * 4);
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let r = (dx * dx + dy * dy).sqrt();
            let edge = smoothstep(0.955, 0.99, r);
            let vignette = smoothstep(0.6, 0.97, r) * 0.6;
            let a = edge.max(vignette);
            data.extend_from_slice(&[0, 0, 0, (a * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: MASK,
            height: MASK,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Strich des Fadenkreuzes; Lage und Größe in Prozent des Kreises, Dicke in Pixeln.
fn line(horizontal: bool, from: f32, len: f32, thick: f32) -> impl Bundle {
    let node = if horizontal {
        Node {
            position_type: PositionType::Absolute,
            left: percent(from),
            width: percent(len),
            top: percent(50),
            height: px(thick),
            margin: UiRect::top(px(-thick / 2.0)),
            ..default()
        }
    } else {
        Node {
            position_type: PositionType::Absolute,
            top: percent(from),
            height: percent(len),
            left: percent(50),
            width: px(thick),
            margin: UiRect::left(px(-thick / 2.0)),
            ..default()
        }
    };
    (node, BackgroundColor(RETICLE))
}

pub fn spawn(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut mask: Local<Option<Handle<Image>>>,
) {
    let mask = mask.get_or_insert_with(|| images.add(mask_image())).clone();
    let bar = || {
        (
            Node {
                flex_grow: 1.0,
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        )
    };
    commands
        .spawn((
            ScopeOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            // unter Blutrand (-5) und HUD-Texten
            GlobalZIndex(-6),
            Visibility::Hidden,
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|p| {
            p.spawn(bar());
            p.spawn((
                ImageNode::new(mask),
                Node {
                    height: percent(100),
                    aspect_ratio: Some(1.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .with_children(|c| {
                // feine Linien durch die Mitte, dicke Balken von außen (Duplex)
                c.spawn(line(true, 0.0, 100.0, 1.5));
                c.spawn(line(false, 0.0, 100.0, 1.5));
                for (from, len) in [(0.0, 36.0), (64.0, 36.0)] {
                    c.spawn(line(true, from, len, 6.0));
                    c.spawn(line(false, from, len, 6.0));
                }
                c.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        top: percent(50),
                        width: px(4),
                        height: px(4),
                        margin: UiRect::new(px(-2), px(0), px(-2), px(0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.9, 0.08, 0.05)),
                ));
            });
            p.spawn(bar());
        });
}

pub fn update(lo: Single<&Loadout>, mut q: Query<&mut Visibility, With<ScopeOverlay>>) {
    let scoped = lo.current_slot().is_some_and(|s| def(s.weapon).scope) && lo.ads > SCOPE_IN;
    for mut v in q.iter_mut() {
        v.set_if_neq(if scoped {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
