//! Visuelle Effekte: Partikel, Decals, Lichtblitze und Feuer.

pub mod decals;
pub mod events;
pub mod fire;
pub mod particles;

use bevy::prelude::*;

use crate::base::{AppState, GameSet, InGame};

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FxEvent>()
            .add_systems(Startup, setup_fx)
            .add_systems(OnEnter(AppState::Playing), fire::spawn_fires)
            .add_systems(
                Update,
                (
                    events::handle_fx,
                    particles::update_particles,
                    decals::update_decals,
                    particles::update_flashes,
                    fire::animate_fires,
                )
                    .chain()
                    .in_set(GameSet::Ui)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

#[derive(Message, Clone, Copy)]
pub enum FxEvent {
    Blood { pos: Vec3, dir: Vec3, amount: f32 },
    Gib { pos: Vec3 },
    Impact { pos: Vec3, normal: Vec3 },
    Explosion { pos: Vec3, radius: f32 },
    RayImpact { pos: Vec3, upgraded: bool },
    Dirt { pos: Vec3 },
    Sparks { pos: Vec3 },
}

#[derive(Resource)]
pub struct FxAssets {
    pub quad: Handle<Mesh>,
    pub cube: Handle<Mesh>,
    pub blood: Handle<StandardMaterial>,
    pub blood_drop: Handle<StandardMaterial>,
    pub smoke: Handle<StandardMaterial>,
    pub dust: Handle<StandardMaterial>,
    pub spark: Handle<StandardMaterial>,
    pub fire: Handle<StandardMaterial>,
    pub glow_green: Handle<StandardMaterial>,
    pub glow_red: Handle<StandardMaterial>,
    pub decal_blood: Handle<StandardMaterial>,
    pub decal_hole: Handle<StandardMaterial>,
    pub flame: Handle<StandardMaterial>,
    pub dirt: Handle<StandardMaterial>,
}

fn setup_fx(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let tex = |n: &str| -> Handle<Image> { asset_server.load(format!("textures/{n}.png")) };
    let sprite = |materials: &mut Assets<StandardMaterial>, t: Handle<Image>, c: Color, emissive: LinearRgba, unlit: bool| {
        materials.add(StandardMaterial {
            base_color: c,
            base_color_texture: Some(t),
            emissive,
            alpha_mode: AlphaMode::Blend,
            unlit,
            double_sided: true,
            cull_mode: None,
            ..default()
        })
    };
    let fx = FxAssets {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        blood: sprite(&mut materials, tex("fx_blood"), Color::srgba(0.55, 0.05, 0.04, 0.95), LinearRgba::BLACK, false),
        blood_drop: materials.add(StandardMaterial {
            base_color: Color::srgb(0.32, 0.02, 0.02),
            perceptual_roughness: 0.3,
            ..default()
        }),
        smoke: sprite(&mut materials, tex("fx_smoke"), Color::srgba(0.5, 0.48, 0.45, 0.55), LinearRgba::BLACK, false),
        dust: sprite(&mut materials, tex("fx_smoke"), Color::srgba(0.6, 0.55, 0.48, 0.5), LinearRgba::BLACK, false),
        spark: sprite(&mut materials, tex("fx_spark"), Color::WHITE, LinearRgba::rgb(8.0, 5.0, 2.0), true),
        fire: sprite(&mut materials, tex("fx_glow"), Color::srgba(1.0, 0.6, 0.2, 0.9), LinearRgba::rgb(12.0, 5.0, 1.2), true),
        glow_green: sprite(&mut materials, tex("fx_glow"), Color::srgba(0.4, 1.0, 0.4, 0.9), LinearRgba::rgb(2.0, 12.0, 2.0), true),
        glow_red: sprite(&mut materials, tex("fx_glow"), Color::srgba(1.0, 0.3, 0.3, 0.9), LinearRgba::rgb(12.0, 2.0, 2.0), true),
        decal_blood: materials.add(StandardMaterial {
            base_color: Color::srgba(0.4, 0.03, 0.03, 0.9),
            base_color_texture: Some(tex("fx_blood")),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.25,
            ..default()
        }),
        decal_hole: materials.add(StandardMaterial {
            base_color: Color::srgba(0.02, 0.02, 0.02, 0.9),
            base_color_texture: Some(tex("fx_glow")),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        flame: sprite(&mut materials, tex("fx_flame"), Color::WHITE, LinearRgba::rgb(6.0, 2.5, 0.6), true),
        dirt: materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.17, 0.12),
            perceptual_roughness: 1.0,
            ..default()
        }),
    };
    commands.insert_resource(fx);
}
