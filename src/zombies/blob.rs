//! Weicher Kontaktschatten unter jedem Zombie. Die Karte ist gebacken (Lightmaps), daher
//! werfen bewegte Figuren dort keinen echten Schatten – der Fleck erdet sie am Boden.

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::Zombie;

#[derive(Resource)]
pub struct BlobAssets {
    mesh: Handle<Mesh>,
    mat: Handle<StandardMaterial>,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let n = 64u32;
    let mut data = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let u = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let r = (u * u + v * v).sqrt();
            let a = (1.0 - r).clamp(0.0, 1.0).powf(1.6);
            data.extend_from_slice(&[0, 0, 0, (a * 255.0) as u8]);
        }
    }
    let img = images.add(Image::new(
        Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ));
    commands.insert_resource(BlobAssets {
        mesh: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
        mat: mats.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.55),
            base_color_texture: Some(img),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            depth_bias: 50.0,
            ..default()
        }),
    });
}

pub fn attach(
    mut commands: Commands,
    blob: Option<Res<BlobAssets>>,
    added: Query<Entity, Added<Zombie>>,
) {
    let Some(blob) = blob else { return };
    for z in added.iter() {
        commands.spawn((
            Name::new("Kontaktschatten"),
            Mesh3d(blob.mesh.clone()),
            MeshMaterial3d(blob.mat.clone()),
            Transform::from_xyz(0.0, 0.03, 0.0).with_scale(Vec3::new(1.0, 1.0, 1.15)),
            NotShadowCaster,
            ChildOf(z),
        ));
    }
}
