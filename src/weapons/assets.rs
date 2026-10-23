//! Gemeinsame Meshes und Materialien der Waffen (Mündungsfeuer, Tarnmuster, Projektile).

use bevy::prelude::*;

#[derive(Resource)]
pub struct WeaponAssets {
    pub flash_mesh: Handle<Mesh>,
    pub flash_mat: Handle<StandardMaterial>,
    pub camo: Handle<StandardMaterial>,
    pub bolt_mesh: Handle<Mesh>,
    pub bolt_green: Handle<StandardMaterial>,
    pub bolt_red: Handle<StandardMaterial>,
    pub grenade_mesh: Handle<Mesh>,
    pub grenade_mat: Handle<StandardMaterial>,
    pub tracer_mat: Handle<StandardMaterial>,
}

pub fn setup_weapon_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let flash_tex: Handle<Image> = asset_server.load("textures/fx_muzzle.png");
    let camo_tex: Handle<Image> = asset_server.load("textures/pap_camo.png");
    commands.insert_resource(WeaponAssets {
        flash_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        flash_mat: materials.add(StandardMaterial {
            base_color_texture: Some(flash_tex),
            emissive: LinearRgba::rgb(6.0, 3.5, 1.2),
            alpha_mode: AlphaMode::Add,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        camo: materials.add(StandardMaterial {
            base_color_texture: Some(camo_tex.clone()),
            emissive_texture: Some(camo_tex),
            emissive: LinearRgba::rgb(0.8, 0.8, 0.8),
            metallic: 0.6,
            perceptual_roughness: 0.3,
            ..default()
        }),
        bolt_mesh: meshes.add(Capsule3d::new(0.06, 0.5)),
        bolt_green: materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 1.0, 0.4),
            emissive: LinearRgba::rgb(4.0, 30.0, 4.0),
            unlit: true,
            ..default()
        }),
        bolt_red: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.4, 0.4),
            emissive: LinearRgba::rgb(30.0, 4.0, 4.0),
            unlit: true,
            ..default()
        }),
        grenade_mesh: meshes.add(Sphere::new(0.05).mesh().uv(10, 6)),
        grenade_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.25, 0.18),
            perceptual_roughness: 0.6,
            ..default()
        }),
        tracer_mat: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.85, 0.5, 0.6),
            emissive: LinearRgba::rgb(6.0, 4.0, 1.5),
            unlit: true,
            alpha_mode: AlphaMode::Add,
            ..default()
        }),
    });
}
