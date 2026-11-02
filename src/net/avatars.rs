//! Mitspieler in der Welt: Figur mit Animation, Waffe in der Hand, Namensschild über dem
//! Kopf und – solange sie am Boden liegen – ein Punkt zum Wiederbeleben (Taste F).
//!
//! Eigene Spielermodelle gibt es nicht; die Figur ist das Zombie-Modell als Überlebender:
//! gesunde Haut, ohne leuchtende Augen, Hemd in der Spielerfarbe.

use std::collections::HashMap;
use std::time::Duration;

use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::PrimaryWindow;
use bevy::world_serialization::WorldInstanceReady;

use super::msg::Life;
use super::session::{INTERP_DELAY, Team, slot_color};
use crate::base::AppState;
use crate::base::assets::{GameAssets, ZombieAnims};
use crate::base::i18n::tf;
use crate::game::interact::{InteractKind, Interactable, WorldWeapon, hide_weapon_hands};
use crate::player::PlayerCamera;
use crate::weapons::{WEAPONS, def};

/// Figur eines Mitspielers (Platz).
#[derive(Component)]
pub struct Avatar {
    pub slot: u8,
}

#[derive(Component)]
pub struct AvatarModel {
    avatar: Entity,
    slot: u8,
}

/// Knochen und Animation der Figur (nach dem Laden des Modells).
#[derive(Component)]
pub struct AvatarRig {
    anim_player: Entity,
    hand: Entity,
    clip: Option<AnimationNodeIndex>,
    weapon: Option<(Entity, u8, bool)>,
}

#[derive(Component)]
pub struct NameTag {
    slot: u8,
}

#[derive(Component)]
pub struct ReviveSpot {
    slot: u8,
}

/// Punkt außerhalb jeder Reichweite (Wiederbeleben gerade nicht möglich)
const NOWHERE: Vec3 = Vec3::new(0.0, -1000.0, 0.0);

pub fn sync_avatars(
    mut commands: Commands,
    team: Res<Team>,
    anims: Res<ZombieAnims>,
    assets: Res<GameAssets>,
    avatars: Query<(Entity, &Avatar)>,
    tags: Query<(Entity, &NameTag)>,
    spots: Query<(Entity, &ReviveSpot)>,
) {
    let present: Vec<u8> = avatars.iter().map(|(_, a)| a.slot).collect();
    for m in team.members.iter().filter(|m| m.connected && m.seen) {
        if present.contains(&m.slot) {
            continue;
        }
        let Some(model) = anims.models.first() else {
            return;
        };
        let pos = Vec3::from_array(m.state.pos);
        let avatar = commands
            .spawn((
                Name::new(format!("Mitspieler {}", m.name)),
                Avatar { slot: m.slot },
                Transform::from_translation(pos),
                Visibility::default(),
                DespawnOnExit(AppState::Playing),
            ))
            .id();
        commands
            .spawn((
                AvatarModel {
                    avatar,
                    slot: m.slot,
                },
                WorldAssetRoot(model.scene.clone()),
                Transform::IDENTITY,
                ChildOf(avatar),
            ))
            .observe(on_avatar_ready);
        commands.spawn((
            NameTag { slot: m.slot },
            Text::new(m.name.clone()),
            TextFont {
                font: assets.font_bold.clone().into(),
                font_size: FontSize::Px(17.0),
                ..default()
            },
            TextColor(slot_color(m.slot)),
            TextLayout::justify(Justify::Center),
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                width: px(260),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(-2),
            DespawnOnExit(AppState::Playing),
        ));
        commands.spawn((
            Name::new(format!("Wiederbeleben {}", m.name)),
            ReviveSpot { slot: m.slot },
            Interactable {
                kind: InteractKind::Revive(m.slot),
                point: NOWHERE,
                radius: 1.8,
                facing: None,
            },
            Transform::default(),
            DespawnOnExit(AppState::Playing),
        ));
    }
    // Wer die Partie verlassen hat, verschwindet
    let gone = |slot: u8| team.get(slot).is_none_or(|m| !m.connected);
    for (e, a) in avatars.iter() {
        if gone(a.slot) {
            commands.entity(e).despawn();
        }
    }
    for (e, t) in tags.iter() {
        if gone(t.slot) {
            commands.entity(e).despawn();
        }
    }
    for (e, s) in spots.iter() {
        if gone(s.slot) {
            commands.entity(e).despawn();
        }
    }
}

fn shirt_color(slot: u8) -> Color {
    match slot % 4 {
        0 => Color::srgb(0.72, 0.72, 0.68),
        1 => Color::srgb(0.18, 0.34, 0.62),
        2 => Color::srgb(0.66, 0.46, 0.12),
        _ => Color::srgb(0.22, 0.48, 0.22),
    }
}

#[allow(clippy::too_many_arguments)]
fn on_avatar_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    models: Query<&AvatarModel>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut anim_players: Query<&mut AnimationPlayer>,
    mat_names: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    anims: Res<ZombieAnims>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(model) = models.get(ready.entity) else {
        return;
    };
    let mut by_name: HashMap<String, Entity> = HashMap::new();
    let mut player = None;
    let mut cache: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>> = HashMap::new();
    for e in children.iter_descendants(ready.entity) {
        if let Ok(n) = names.get(e) {
            by_name.insert(n.as_str().to_string(), e);
        }
        if anim_players.contains(e) {
            player = Some(e);
        }
        let Ok((mname, mat)) = mat_names.get(e) else {
            continue;
        };
        // Überlebender statt Zombie: eigene Farben, keine Wunden
        let tint = match mname.0.as_str() {
            "Z_Shirt" => Some(shirt_color(model.slot)),
            "Z_Pants" => Some(Color::srgb(0.16, 0.17, 0.2)),
            "Z_Skin" | "Z_Blood" | "Z_Bone" => Some(Color::srgb(0.95, 0.74, 0.62)),
            _ => None,
        };
        let Some(tint) = tint else { continue };
        let handle = match cache.get(&mat.0.id()) {
            Some(h) => h.clone(),
            None => {
                let Some(mut m) = materials.get(&mat.0).cloned() else {
                    continue;
                };
                m.base_color = tint;
                if mname.0 != "Z_Skin" {
                    m.base_color_texture = None;
                }
                let h = materials.add(m);
                cache.insert(mat.0.id(), h.clone());
                h
            }
        };
        commands.entity(e).insert(MeshMaterial3d(handle));
    }
    for hidden in ["ZombieEyes", "ZombieStump"] {
        if let Some(e) = by_name.get(hidden) {
            commands.entity(*e).insert(Visibility::Hidden);
        }
    }
    let (Some(player), Some(anim)) = (player, anims.models.first()) else {
        return;
    };
    if let Ok(mut ap) = anim_players.get_mut(player) {
        let mut tr = AnimationTransitions::new();
        tr.play(&mut ap, anim.idle, Duration::ZERO).repeat();
        commands
            .entity(player)
            .insert((AnimationGraphHandle(anim.graph.clone()), tr));
    }
    commands.entity(model.avatar).insert(AvatarRig {
        anim_player: player,
        hand: by_name.get("Hand.R").copied().unwrap_or(ready.entity),
        clip: Some(anim.idle),
        weapon: None,
    });
}

/// Position, Animation und Waffe der Figuren; Wiederbelebungspunkt mitführen.
#[allow(clippy::too_many_arguments)]
pub fn update_avatars(
    mut commands: Commands,
    real: Res<Time<Real>>,
    team: Res<Team>,
    anims: Res<ZombieAnims>,
    asset_server: Res<AssetServer>,
    mut avatars: Query<(&Avatar, &mut Transform, Option<&mut AvatarRig>), Without<ReviveSpot>>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut spots: Query<(&ReviveSpot, &mut Interactable, &mut Transform), Without<Avatar>>,
) {
    let at = real.elapsed_secs_f64() - INTERP_DELAY;
    let Some(anim) = anims.models.first() else {
        return;
    };
    for (avatar, mut tf, rig) in avatars.iter_mut() {
        let Some(m) = team.get(avatar.slot) else {
            continue;
        };
        if let Some((pos, yaw)) = m.motion.sample(at) {
            tf.translation = pos;
            tf.rotation = Quat::from_rotation_y(yaw);
        }
        let Some(mut rig) = rig else { continue };
        let s = &m.state;
        let (clip, speed, looping) = match s.life {
            Life::Downed => (anim.crawl, 0.6, true),
            Life::Dead => (anim.death_fwd, 1.0, false),
            Life::Alive if s.stance == 2 => (anim.crawl, (s.speed / 0.8).clamp(0.3, 1.5), true),
            Life::Alive if s.speed < 0.3 => (anim.idle, 1.0, true),
            Life::Alive if s.sprinting => (anim.sprint, (s.speed / 5.0).clamp(0.6, 1.6), true),
            Life::Alive => (anim.run, (s.speed / 2.6).clamp(0.5, 1.8), true),
        };
        if let Ok((mut ap, mut tr)) = players.get_mut(rig.anim_player) {
            if rig.clip != Some(clip) {
                rig.clip = Some(clip);
                let a = tr.play(&mut ap, clip, Duration::from_millis(200));
                if looping {
                    a.repeat();
                }
                a.set_speed(speed);
            } else if let Some(a) = ap.animation_mut(clip) {
                a.set_speed(speed);
            }
        }
        // Waffe in der rechten Hand (wechselt mit der Waffe des Mitspielers)
        let want = (s.weapon.min(WEAPONS.len() as u8 - 1), s.upgraded);
        let show = s.life != Life::Dead;
        if rig.weapon.map(|(_, w, u)| (w, u)) != Some(want) || !show {
            if let Some((e, ..)) = rig.weapon.take() {
                commands.entity(e).despawn();
            }
            if show {
                let e = commands
                    .spawn((
                        WorldWeapon,
                        WorldAssetRoot(asset_server.load(
                            GltfAssetLabel::Scene(0).from_asset(def(want.0 as usize).model()),
                        )),
                        // Lauf nach vorn aus der Hand heraus
                        Transform::from_xyz(0.0, 0.08, 0.02)
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::splat(0.9)),
                        ChildOf(rig.hand),
                    ))
                    .observe(hide_weapon_hands)
                    .id();
                rig.weapon = Some((e, want.0, want.1));
            }
        }
    }
    for (spot, mut it, mut tf) in spots.iter_mut() {
        let downed = team
            .get(spot.slot)
            .filter(|m| m.connected && m.state.life == Life::Downed);
        let point = match downed.and_then(|m| m.motion.sample(at)) {
            Some((pos, _)) => pos + Vec3::Y * 0.4,
            None => NOWHERE,
        };
        if it.point != point {
            it.point = point;
            tf.translation = point;
        }
    }
}

/// Namensschilder über den Köpfen (in Bildschirmkoordinaten, unabhängig von der
/// Renderauflösung).
pub fn update_nametags(
    team: Res<Team>,
    real: Res<Time<Real>>,
    camera: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut tags: Query<(
        &NameTag,
        &mut Node,
        &mut Text,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    let Ok((cam, cam_tf)) = camera.single() else {
        return;
    };
    let at = real.elapsed_secs_f64() - INTERP_DELAY;
    let size = Vec2::new(window.width(), window.height());
    for (tag, mut node, mut text, mut color, mut vis) in tags.iter_mut() {
        let Some(m) = team.get(tag.slot) else {
            continue;
        };
        let Some((pos, _)) = m.motion.sample(at) else {
            continue;
        };
        let head = pos
            + Vec3::Y
                * match m.state.life {
                    Life::Alive if m.state.stance == 0 => 2.05,
                    Life::Alive if m.state.stance == 1 => 1.5,
                    _ => 0.9,
                };
        let dist = cam_tf.translation().distance(head);
        let ndc = cam
            .world_to_ndc(cam_tf, head)
            .filter(|n| n.z > 0.0 && n.z < 1.0);
        let Some(ndc) = ndc.filter(|_| dist < 60.0) else {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
            continue;
        };
        let screen = Vec2::new((ndc.x + 1.0) * 0.5 * size.x, (1.0 - ndc.y) * 0.5 * size.y);
        node.left = px(screen.x - 130.0);
        node.top = px(screen.y - 22.0);
        let label = match m.state.life {
            Life::Downed => tf(
                "mp.tag_downed",
                &[
                    ("name", &m.name),
                    ("seconds", &(m.state.bleed.ceil() as u32)),
                ],
            ),
            Life::Dead => tf("mp.tag_dead", &[("name", &m.name)]),
            Life::Alive => m.name.clone(),
        };
        if text.0 != label {
            text.0 = label;
        }
        let fade = (1.0 - (dist - 30.0) / 30.0).clamp(0.25, 1.0);
        let base = if m.state.life == Life::Downed {
            Color::srgb(1.0, 0.3, 0.25)
        } else {
            slot_color(tag.slot)
        };
        color.0 = base.with_alpha(fade);
        if *vis != Visibility::Inherited {
            *vis = Visibility::Inherited;
        }
    }
}
