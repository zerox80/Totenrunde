//! Lichter der Karte: Flackern und Anlaufen der Lampen, wenn der Strom eingeschaltet wird.

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::mapdata::MapFile;
use super::power::Power;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                flicker_lights,
                power_lights,
                light_cones,
                apply_light_settings,
            )
                .chain()
                .run_if(resource_exists::<MapFile>),
        );
    }
}

/// Multiplikator für die Lichtstärken aus der Karte (Lumen).
pub const LIGHT_SCALE: f32 = 8.0;

#[derive(Component)]
pub struct MapLight {
    /// Lampe darf Echtzeit-Schatten werfen (laut Karte)
    pub wants_shadow: bool,
    pub base: f32,
    pub flicker: f32,
    pub needs_power: bool,
    pub seed: f32,
}

/// Sichtbarer Lichtkegel (Staub in der Luft) unter einer Deckenlampe/Laterne.
#[derive(Component)]
pub struct LightCone {
    light: Entity,
    base: f32,
    color: Vec3,
    strength: f32,
}

/// Offener Kegel (Spitze oben bei y = 0, Öffnung unten), Helligkeit über Vertexfarben:
/// oben hell, unten auslaufend.
fn cone_mesh(r_top: f32, r_bot: f32, h: f32) -> Mesh {
    let seg = 28;
    let mut pos = Vec::new();
    let mut col = Vec::new();
    let mut nrm = Vec::new();
    let mut idx = Vec::new();
    for i in 0..=seg {
        let a = i as f32 / seg as f32 * std::f32::consts::TAU;
        let (c, s) = (a.cos(), a.sin());
        for (k, (r, y, b)) in [
            (r_top, 0.0, 1.0f32),
            (r_bot * 0.55, -h * 0.45, 0.55),
            (r_bot, -h, 0.0),
        ]
        .into_iter()
        .enumerate()
        {
            pos.push([c * r, y, s * r]);
            nrm.push([c, 0.3, s]);
            col.push([b, b, b, 1.0]);
            let _ = k;
        }
    }
    for i in 0..seg as u32 {
        for k in 0..2u32 {
            let a = i * 3 + k;
            let b = (i + 1) * 3 + k;
            idx.extend_from_slice(&[a, b, a + 1, b, b + 1, a + 1]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nrm)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
    .with_inserted_indices(Indices::U32(idx))
}

pub fn spawn_map_lights(
    commands: &mut Commands,
    map: &MapFile,
    shadows: bool,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    let cone_indoor = meshes.add(cone_mesh(0.25, 1.35, 2.6));
    let cone_street = meshes.add(cone_mesh(0.2, 2.6, 6.2));
    let no_lights = std::env::var("TR_NOLIGHTS").is_ok();
    for (i, l) in map.lights.iter().enumerate() {
        if no_lights {
            break;
        }
        let base = l.intensity * LIGHT_SCALE;
        let col = Vec3::from_array(l.color);
        let light = commands
            .spawn((
                Name::new(format!("Licht {i}")),
                PointLight {
                    color: Color::linear_rgb(l.color[0], l.color[1], l.color[2]),
                    intensity: if l.needs_power { 0.0 } else { base },
                    range: l.range,
                    radius: 0.1,
                    shadow_maps_enabled: l.shadows && shadows,
                    // Gebackene Lichter beleuchten statische Flächen nur noch spekular,
                    // Zombies, Waffen und Türen aber weiterhin voll.
                    affects_lightmapped_mesh_diffuse: !l.baked,
                    ..default()
                },
                Transform::from_translation(Vec3::from_array(l.pos)),
                MapLight {
                    wants_shadow: l.shadows,
                    base,
                    flicker: l.flicker,
                    needs_power: l.needs_power,
                    seed: i as f32 * 7.31,
                },
            ))
            .id();
        // Lichtkegel nur für Decken- und Mastleuchten (hoch, kräftig)
        let street = l.pos[1] > 5.0;
        if l.pos[1] > 2.2 && l.range >= 6.0 && l.intensity >= 15000.0 {
            let mat = mats.add(StandardMaterial {
                base_color: Color::BLACK,
                alpha_mode: AlphaMode::Add,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                ..default()
            });
            commands.spawn((
                Name::new("Lichtkegel"),
                LightCone {
                    light,
                    base,
                    color: col,
                    strength: if street { 0.018 } else { 0.006 },
                },
                Mesh3d(if street {
                    cone_street.clone()
                } else {
                    cone_indoor.clone()
                }),
                MeshMaterial3d(mat),
                Transform::from_translation(Vec3::from_array(l.pos) + Vec3::Y * 0.2),
                NotShadowCaster,
            ));
        }
    }
}

/// Helligkeit der Lichtkegel folgt der Lampe (Flackern, Strom).
fn light_cones(
    lights: Query<&PointLight>,
    cones: Query<(&LightCone, &MeshMaterial3d<StandardMaterial>)>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    for (c, m) in cones.iter() {
        let k = lights
            .get(c.light)
            .map(|l| l.intensity / c.base.max(1.0))
            .unwrap_or(0.0);
        if let Some(mut mat) = mats.get_mut(&m.0) {
            let v = c.color * c.strength * k;
            mat.base_color = Color::linear_rgb(v.x, v.y, v.z);
        }
    }
}

fn flicker_lights(
    time: Res<Time>,
    power: Res<Power>,
    mut lights: Query<(&mut PointLight, &MapLight)>,
) {
    let t = time.elapsed_secs();
    for (mut pl, ml) in lights.iter_mut() {
        if ml.needs_power && !power.on {
            pl.intensity = 0.0;
            continue;
        }
        if ml.flicker <= 0.0 {
            continue;
        }
        // Mischung aus Rauschen und gelegentlichen Aussetzern
        let n = (t * 13.0 + ml.seed).sin() * 0.5
            + (t * 31.7 + ml.seed * 2.0).sin() * 0.3
            + (t * 5.3).sin() * 0.2;
        let cut = ((t * 2.3 + ml.seed).sin() > 0.93) as i32 as f32;
        let f =
            (1.0 - ml.flicker * 0.35 * (n * 0.5 + 0.5) - cut * ml.flicker * 0.8).clamp(0.05, 1.0);
        pl.intensity = ml.base * f;
    }
}

fn power_lights(
    time: Res<Time>,
    power: Res<Power>,
    mut lights: Query<(&mut PointLight, &MapLight)>,
) {
    if !power.on {
        return;
    }
    let dt = time.elapsed_secs() - power.since;
    for (mut pl, ml) in lights.iter_mut() {
        if !ml.needs_power || ml.flicker > 0.0 {
            continue;
        }
        if dt > 3.0 {
            pl.intensity = ml.base;
            continue;
        }
        // Anlaufen der Lampen mit kurzem Flackern
        let k = if dt < 0.8 {
            ((dt * 40.0).sin() > 0.2) as i32 as f32 * 0.6
        } else {
            (dt / 1.5).min(1.0)
        };
        pl.intensity = ml.base * k;
    }
}

/// Grafik-Einstellungen: Lichtkegel ein/aus, Echtzeit-Schatten der flackernden Lampen und Feuer.
fn apply_light_settings(
    settings: Res<crate::base::settings::Settings>,
    mut cones: Query<&mut Visibility, With<LightCone>>,
    mut lights: Query<(&MapLight, &mut PointLight)>,
    added: Query<(), Added<LightCone>>,
) {
    if !settings.is_changed() && added.is_empty() {
        return;
    }
    let vis = if settings.light_cones {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in cones.iter_mut() {
        *v = vis;
    }
    for (ml, mut pl) in lights.iter_mut() {
        let want = ml.wants_shadow && settings.shadows;
        if pl.shadow_maps_enabled != want {
            pl.shadow_maps_enabled = want;
        }
    }
}
