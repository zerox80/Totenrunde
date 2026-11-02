//! Mysteriöse Kiste: 950 Punkte (Ausverkauf: 10), Waffen rotieren über der offenen Kiste,
//! die angebotene Waffe kann 12 Sekunden lang genommen werden. Nach einigen Benutzungen
//! erscheint ein Teddy – die Kiste lacht, fliegt davon und taucht an einem anderen Ort
//! wieder auf (die Punkte gibt es zurück). Ein Lichtstrahl zeigt, wo sie steht.
//!
//! Die Kiste gehört allen: Öffnen und Nehmen sind Anfragen an den Host (siehe `requests`),
//! nur er wechselt die Phasen. Die angebotene Waffe kann nur nehmen, wer bezahlt hat.
//! Gäste übernehmen Phase und Ort aus dem Schnappschuss und spielen nur die Bewegung ab.

use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use super::{Focus, InteractKind, Interactable, Prompt, cost_text};
use crate::audio::{Sfx, SfxEvent};
use crate::base::AppState;
use crate::base::i18n::tf;
use crate::game::points::Score;
use crate::game::powerups::ActivePowerups;
use crate::game::requests::WorldRequest;
use crate::net::msg::Req;
use crate::net::{Session, local_slot};
use crate::weapons::defs::BoxTier;
use crate::weapons::{Loadout, WEAPONS, def};
use crate::world::mapdata::{MapFile, asset_exists};

pub const BOX_COST: u32 = 950;
pub const FIRE_SALE_COST: u32 = 10;
const ROLL_TIME: f32 = 4.2;
const OFFER_TIME: f32 = 12.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BoxPhase {
    Closed,
    /// `weapon` = `usize::MAX`: Teddy; `owner`: Platz des Spielers, der bezahlt hat
    Rolling {
        t: f32,
        weapon: usize,
        next: f32,
        owner: u8,
    },
    Offer {
        t: f32,
        weapon: usize,
        owner: u8,
    },
    Teddy {
        t: f32,
    },
    Away {
        t: f32,
    },
}

impl BoxPhase {
    fn kind(self) -> u8 {
        match self {
            BoxPhase::Closed => 0,
            BoxPhase::Rolling { .. } => 1,
            BoxPhase::Offer { .. } => 2,
            BoxPhase::Teddy { .. } => 3,
            BoxPhase::Away { .. } => 4,
        }
    }
}

#[derive(Resource)]
pub struct MysteryBox {
    pub spot: usize,
    pub uses: u32,
    pub phase: BoxPhase,
    pub root: Entity,
    pub lid: Entity,
    pub beam: Entity,
    pub interact: Entity,
    /// Je Kistenwaffe ein fertiges, verstecktes Modell (Waffe, Entity). Beim Rollen werden
    /// sie nur ein- und ausgeblendet; neu laden würde jedes Mal ruckeln.
    pub displays: Vec<(usize, Entity)>,
    /// Gerade angezeigte Waffe
    pub shown: Option<usize>,
    pub teddy: Entity,
}

#[derive(Component)]
pub struct BoxDisplay;

fn spot_transform(map: &MapFile, i: usize) -> Transform {
    let s = &map.box_spots[i];
    Transform::from_translation(Vec3::from_array(s.pos)).with_rotation(Quat::from_rotation_y(s.yaw))
}

fn interact_point(tf: &Transform) -> (Vec3, Vec3) {
    let fwd = tf.rotation * Vec3::NEG_Z;
    (
        Vec3::new(tf.translation.x, 1.0, tf.translation.z) + fwd * 0.55,
        fwd,
    )
}

pub fn spawn_box(
    mut commands: Commands,
    map: Res<MapFile>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if map.box_spots.is_empty() {
        return;
    }
    let spot = map.box_spots.iter().position(|s| s.start).unwrap_or(0);
    let tf = spot_transform(&map, spot);
    let root = commands
        .spawn((
            Name::new("Mysteriöse Kiste"),
            tf,
            Visibility::default(),
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.2, 0.11),
        perceptual_roughness: 0.8,
        ..default()
    });
    let have_model = asset_exists("models/mystery_box.gltf");
    if have_model {
        commands.entity(root).with_child((
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/mystery_box.gltf")),
            ),
            Transform::IDENTITY,
        ));
    } else {
        commands.entity(root).with_child((
            Mesh3d(meshes.add(Cuboid::new(1.25, 0.5, 0.6))),
            MeshMaterial3d(wood.clone()),
            Transform::from_xyz(0.0, 0.25, 0.0),
        ));
    }
    // Deckel mit Scharnier an der Rückseite
    let lid = commands
        .spawn((
            Transform::from_xyz(0.0, 0.5, 0.3),
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    if asset_exists("models/mystery_box_lid.gltf") {
        commands.entity(lid).with_child((
            WorldAssetRoot(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("models/mystery_box_lid.gltf")),
            ),
            Transform::IDENTITY,
        ));
    } else {
        commands.entity(lid).with_child((
            Mesh3d(meshes.add(Cuboid::new(1.28, 0.08, 0.62))),
            MeshMaterial3d(wood),
            Transform::from_xyz(0.0, 0.04, -0.3),
        ));
    }
    // Lichtstrahl in den Himmel
    let beam_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.75, 1.0, 0.05),
        emissive: LinearRgba::rgb(0.25, 0.4, 0.75),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let beam = commands
        .spawn((
            Name::new("Kisten-Lichtstrahl"),
            Mesh3d(meshes.add(Cylinder::new(0.11, 40.0))),
            MeshMaterial3d(beam_mat),
            Transform::from_translation(tf.translation + Vec3::Y * 21.6),
            bevy::light::NotShadowCaster,
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    // Teddy (eigenes Modell oder Ersatz aus Kugeln)
    let teddy = commands
        .spawn((
            Transform::from_xyz(0.0, 0.3, 0.0),
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();
    if asset_exists("models/teddy.gltf") {
        commands.entity(teddy).with_child((
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/teddy.gltf")),
            ),
            Transform::IDENTITY,
        ));
    } else {
        let fur = materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.3, 0.18),
            perceptual_roughness: 1.0,
            ..default()
        });
        let s = meshes.add(Sphere::new(1.0));
        for (p, r) in [
            (Vec3::new(0.0, 0.0, 0.0), 0.16),
            (Vec3::new(0.0, 0.22, 0.0), 0.11),
            (Vec3::new(-0.08, 0.31, 0.0), 0.04),
            (Vec3::new(0.08, 0.31, 0.0), 0.04),
        ] {
            commands.entity(teddy).with_child((
                Mesh3d(s.clone()),
                MeshMaterial3d(fur.clone()),
                Transform::from_translation(p).with_scale(Vec3::splat(r)),
            ));
        }
    }
    let (point, fwd) = interact_point(&tf);
    let interact = commands
        .spawn((
            Interactable {
                kind: InteractKind::MysteryBox,
                point,
                radius: 1.7,
                facing: Some(fwd),
            },
            Transform::from_translation(point),
            DespawnOnExit(AppState::Playing),
        ))
        .id();
    let displays = box_weapons()
        .map(|w| {
            let e = commands
                .spawn((
                    BoxDisplay,
                    super::WorldWeapon,
                    WorldAssetRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset(def(w).model())),
                    ),
                    Transform::from_xyz(0.0, 0.35, 0.0)
                        .with_rotation(Quat::from_rotation_y(PI / 2.0)),
                    Visibility::Hidden,
                    ChildOf(root),
                ))
                .observe(super::hide_weapon_hands)
                .id();
            (w, e)
        })
        .collect();
    commands.insert_resource(MysteryBox {
        spot,
        uses: 0,
        phase: BoxPhase::Closed,
        root,
        lid,
        beam,
        interact,
        displays,
        shown: None,
        teddy,
    });
}

/// Alle Waffen, die in der Kiste vorkommen können.
pub fn box_weapons() -> impl Iterator<Item = usize> {
    WEAPONS
        .iter()
        .enumerate()
        .filter(|(_, w)| w.box_tier != BoxTier::None && w.box_weight > 0.0)
        .map(|(i, _)| i)
}

/// Zufallswaffe: erst die Stufe (40 % schlecht, 40 % mittel, 20 % gut), dann eine Waffe
/// daraus nach Gewicht – ohne Waffen, die der Spieler schon trägt. Ist eine Stufe dadurch
/// leer, wird neu gelost (wer alle guten Waffen hat, bekommt eben eine andere).
fn roll_weapon(owned: &[usize], rng: &mut impl Rng) -> usize {
    let in_tier = |tier: BoxTier| -> Vec<(usize, f32)> {
        box_weapons()
            .filter(|i| WEAPONS[*i].box_tier == tier && !owned.contains(i))
            .map(|i| (i, WEAPONS[i].box_weight))
            .collect()
    };
    let tiers: Vec<(Vec<(usize, f32)>, f32)> = BoxTier::ODDS
        .iter()
        .map(|(tier, odds)| (in_tier(*tier), *odds))
        .filter(|(cands, _)| !cands.is_empty())
        .collect();
    let total: f32 = tiers.iter().map(|t| t.1).sum();
    if tiers.is_empty() || total <= 0.0 {
        return box_weapons().next().unwrap_or(0);
    }
    let pick = |items: &[(usize, f32)], x: f32| -> usize {
        let mut x = x;
        for (i, w) in items {
            if x < *w {
                return *i;
            }
            x -= w;
        }
        items[items.len() - 1].0
    };
    let mut x = rng.random_range(0.0..total);
    let cands = tiers
        .iter()
        .find(|(_, odds)| {
            let hit = x < *odds;
            if !hit {
                x -= odds;
            }
            hit
        })
        .map(|(c, _)| c)
        .unwrap_or(&tiers[tiers.len() - 1].0);
    let weight: f32 = cands.iter().map(|c| c.1).sum();
    pick(cands, rng.random_range(0.0..weight))
}

fn any_weapon(rng: &mut impl Rng) -> usize {
    let c: Vec<usize> = box_weapons().collect();
    c[rng.random_range(0..c.len())]
}

/// Kiste öffnen (Host): Waffe auslosen oder Teddy. Liefert die Kosten und ob es der Teddy
/// war (dann wird nichts bezahlt).
pub fn open_box(mbox: &mut MysteryBox, owned: &[usize], fire_sale: bool, owner: u8) -> (u32, bool) {
    let mut rng = rand::rng();
    let cost = if fire_sale { FIRE_SALE_COST } else { BOX_COST };
    let teddy_chance = if fire_sale || mbox.uses < 4 {
        0.0
    } else {
        (0.12 + 0.05 * (mbox.uses - 4) as f32).min(0.5)
    };
    mbox.uses += 1;
    // Teddy: erst normal rollen, dann statt einer Waffe der Teddy (Punkte zurück)
    let teddy = rng.random::<f32>() < teddy_chance;
    mbox.phase = BoxPhase::Rolling {
        t: 0.0,
        weapon: if teddy {
            usize::MAX
        } else {
            roll_weapon(owned, &mut rng)
        },
        next: 0.0,
        owner,
    };
    (cost, teddy)
}

/// Angebotene Waffe nehmen (Host): nur wer bezahlt hat.
pub fn take_offer(mbox: &mut MysteryBox, by: u8) -> Option<usize> {
    match mbox.phase {
        BoxPhase::Offer { weapon, owner, .. } if owner == by => {
            mbox.phase = BoxPhase::Closed;
            Some(weapon)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn use_box(
    mut focus: ResMut<Focus>,
    mut prompt: ResMut<Prompt>,
    mbox: Option<Res<MysteryBox>>,
    score: Res<Score>,
    pu: Res<ActivePowerups>,
    lo: Single<&Loadout>,
    session: Option<Res<Session>>,
    mut requests: MessageWriter<WorldRequest>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(mbox) = mbox else { return };
    if focus.kind != Some(InteractKind::MysteryBox) {
        return;
    }
    let me = local_slot(session.as_deref());
    match mbox.phase {
        BoxPhase::Closed => {
            let cost = if pu.fire_sale > 0.0 {
                FIRE_SALE_COST
            } else {
                BOX_COST
            };
            prompt.set(tf("prompt.box", &[("cost", &cost_text(cost))]));
            if focus.pressed {
                if score.points >= cost {
                    let owned = lo.slots.iter().map(|s| s.weapon as u8).collect();
                    requests.write(WorldRequest {
                        by: me,
                        req: Req::BoxOpen { owned },
                    });
                } else {
                    sfx.write(SfxEvent::ui(Sfx::Deny));
                }
                focus.pressed = false;
            }
        }
        BoxPhase::Offer { weapon, owner, .. } if owner == me => {
            prompt.set(tf(
                "prompt.take",
                &[("item", &def(weapon).display_name(false))],
            ));
            if focus.pressed {
                requests.write(WorldRequest {
                    by: me,
                    req: Req::BoxTake,
                });
                focus.pressed = false;
            }
        }
        _ => {}
    }
}

/// Geräusche bei jedem Phasenwechsel (beim Host wie bei den Gästen).
pub fn box_sounds(
    mbox: Option<Res<MysteryBox>>,
    tfs: Query<&Transform>,
    mut last: Local<u8>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let Some(mbox) = mbox else { return };
    let kind = mbox.phase.kind();
    if kind == *last {
        return;
    }
    let pos = tfs
        .get(mbox.root)
        .map(|t| t.translation)
        .unwrap_or_default()
        + Vec3::Y * 0.6;
    match (*last, kind) {
        (_, 1) => {
            sfx.write(SfxEvent::at(Sfx::BoxOpen, pos));
            sfx.write(SfxEvent::at(Sfx::BoxMusic, pos));
        }
        (2, 0) => {
            sfx.write(SfxEvent::at(Sfx::BoxClose, pos));
        }
        (_, 3) => {
            sfx.write(SfxEvent::at(Sfx::BoxLaugh, pos + Vec3::Y * 0.4));
        }
        (_, 4) => {
            sfx.write(SfxEvent::at(Sfx::BoxMove, pos + Vec3::Y * 0.4));
        }
        _ => {}
    }
    *last = kind;
}

/// Kiste an ihren Platz stellen: Lichtstrahl sichtbar, Benutzungspunkt davor.
fn settle(
    mbox: &MysteryBox,
    tf: &Transform,
    tfs: &mut Query<&mut Transform>,
    vis: &mut Query<&mut Visibility>,
    items: &mut Query<&mut Interactable>,
) {
    if let Ok(mut rt) = tfs.get_mut(mbox.root)
        && (rt.translation != tf.translation || rt.rotation != tf.rotation)
    {
        rt.translation = tf.translation;
        rt.rotation = tf.rotation;
    }
    let beam = tf.translation + Vec3::Y * 21.6;
    if let Ok(mut bt) = tfs.get_mut(mbox.beam)
        && bt.translation != beam
    {
        bt.translation = beam;
    }
    if let Ok(mut v) = vis.get_mut(mbox.beam)
        && *v != Visibility::Inherited
    {
        *v = Visibility::Inherited;
    }
    if let Ok(mut it) = items.get_mut(mbox.interact) {
        let (p, f) = interact_point(tf);
        if it.point != p {
            it.point = p;
            it.facing = Some(f);
        }
    }
}

/// Bewegung der Kiste. Phasen wechselt nur, wer die Welt berechnet (`authority`); bei Gästen
/// läuft die Zeit innerhalb der Phase weiter, bis der Schnappschuss die nächste bringt.
#[allow(clippy::too_many_arguments)]
pub fn animate_box(
    time: Res<Time>,
    map: Res<MapFile>,
    mbox: Option<ResMut<MysteryBox>>,
    session: Option<Res<Session>>,
    mut tfs: Query<&mut Transform>,
    mut vis: Query<&mut Visibility>,
    mut items: Query<&mut Interactable>,
    mut pu: ResMut<ActivePowerups>,
) {
    let Some(mut mbox) = mbox else { return };
    let authority = session.is_none_or(|s| s.is_host());
    let dt = time.delta_secs();
    let mut rng = rand::rng();
    // Gäste bleiben kurz vor dem Phasenende stehen
    let hold = |t: f32, end: f32| if authority { t } else { t.min(end - 1e-3) };
    let mut lid_open = 0.0;
    let mut display_h: Option<f32> = None;
    let mut want_weapon: Option<usize> = None;
    match mbox.phase {
        BoxPhase::Closed => {
            let tf = spot_transform(&map, mbox.spot);
            settle(&mbox, &tf, &mut tfs, &mut vis, &mut items);
            if let Ok(mut v) = vis.get_mut(mbox.teddy)
                && *v != Visibility::Hidden
            {
                *v = Visibility::Hidden;
            }
        }
        BoxPhase::Rolling {
            t,
            weapon,
            next,
            owner,
        } => {
            let t = hold(t + dt, ROLL_TIME);
            lid_open = (t / 0.4).min(1.0);
            display_h = Some(0.35 + 0.6 * (t / ROLL_TIME).min(1.0));
            let mut next = next - dt;
            if next <= 0.0 || mbox.shown.is_none() {
                // Waffen wechseln immer langsamer
                next = 0.06 + 0.4 * (t / ROLL_TIME).powi(2);
                want_weapon = Some(any_weapon(&mut rng));
            }
            if t >= ROLL_TIME {
                if weapon == usize::MAX {
                    mbox.phase = BoxPhase::Teddy { t: 0.0 };
                    display_h = None;
                } else {
                    mbox.phase = BoxPhase::Offer {
                        t: 0.0,
                        weapon,
                        owner,
                    };
                    want_weapon = Some(weapon);
                }
            } else {
                mbox.phase = BoxPhase::Rolling {
                    t,
                    weapon,
                    next,
                    owner,
                };
            }
        }
        BoxPhase::Offer { t, weapon, owner } => {
            let t = hold(t + dt, OFFER_TIME);
            lid_open = 1.0;
            // Waffe sinkt in den letzten Sekunden zurück in die Kiste
            let sink = ((t - (OFFER_TIME - 2.0)) / 2.0).clamp(0.0, 1.0);
            display_h = Some(0.95 - 0.6 * sink);
            if mbox.shown != Some(weapon) {
                want_weapon = Some(weapon);
            }
            if t >= OFFER_TIME {
                mbox.phase = BoxPhase::Closed;
            } else {
                mbox.phase = BoxPhase::Offer { t, weapon, owner };
            }
        }
        BoxPhase::Teddy { t } => {
            let t = hold(t + dt, 2.6);
            lid_open = 1.0;
            if let Ok(mut tt) = tfs.get_mut(mbox.teddy) {
                tt.translation = Vec3::new(0.0, 0.3 + 0.7 * (t / 1.0).min(1.0), 0.0);
                tt.rotation = Quat::from_rotation_y(PI + (t * 3.0).sin() * 0.3);
            }
            if let Ok(mut v) = vis.get_mut(mbox.teddy) {
                *v = Visibility::Inherited;
            }
            if t >= 2.6 {
                mbox.phase = BoxPhase::Away { t: 0.0 };
                if let Ok(mut v) = vis.get_mut(mbox.teddy) {
                    *v = Visibility::Hidden;
                }
            } else {
                mbox.phase = BoxPhase::Teddy { t };
            }
        }
        BoxPhase::Away { t } => {
            let t = hold(t + dt, 3.3);
            if let Ok(mut v) = vis.get_mut(mbox.teddy) {
                *v = Visibility::Hidden;
            }
            // hochfliegen, verschwinden, an neuem Ort wieder herabkommen
            if t < 2.0 {
                if let Ok(mut rt) = tfs.get_mut(mbox.root) {
                    let k = t / 2.0;
                    let from = &map.box_spots[mbox.spot];
                    rt.translation = Vec3::from_array(from.pos) + Vec3::Y * k * k * 6.0;
                    rt.rotation = Quat::from_rotation_y(from.yaw + k * k * 8.0);
                }
                if let Ok(mut v) = vis.get_mut(mbox.beam) {
                    *v = Visibility::Hidden;
                }
            } else if t < 2.1 {
                // neuen Ort wählen (nur der Host; Gäste bekommen ihn mit dem Schnappschuss)
                if authority && mbox.uses > 0 {
                    let n = map.box_spots.len();
                    if n > 1 {
                        let mut ns = mbox.spot;
                        while ns == mbox.spot {
                            ns = rng.random_range(0..n);
                        }
                        mbox.spot = ns;
                    }
                    mbox.uses = 0;
                    pu.fire_sale_allowed = true;
                }
            } else {
                let tf = spot_transform(&map, mbox.spot);
                let k = ((t - 2.1) / 1.2).min(1.0);
                if let Ok(mut rt) = tfs.get_mut(mbox.root) {
                    rt.translation = tf.translation + Vec3::Y * (1.0 - k) * (1.0 - k) * 6.0;
                    rt.rotation = tf.rotation;
                }
                if k >= 1.0 {
                    mbox.phase = BoxPhase::Closed;
                }
            }
            if let BoxPhase::Away { .. } = mbox.phase {
                mbox.phase = BoxPhase::Away { t };
            }
        }
    }
    // Deckel
    if let Ok(mut lt) = tfs.get_mut(mbox.lid) {
        let cur = lt.rotation.to_euler(EulerRot::XYZ).0;
        let goal = -lid_open * 1.9;
        let a = cur + (goal - cur) * (dt * 10.0).min(1.0);
        lt.rotation = Quat::from_rotation_x(a);
    }
    // angezeigte Waffe: fertige Modelle nur ein- und ausblenden
    if display_h.is_none() {
        mbox.shown = None;
    } else if let Some(w) = want_weapon {
        mbox.shown = Some(w);
    }
    for &(w, e) in &mbox.displays {
        let show = mbox.shown == Some(w);
        if let Ok(mut v) = vis.get_mut(e) {
            let want = if show {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *v != want {
                *v = want;
            }
        }
        if let (true, Some(h)) = (show, display_h)
            && let Ok(mut tf) = tfs.get_mut(e)
        {
            tf.translation.y = h;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn tiers_follow_the_odds() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let mut counts = [0u32; 3];
        let n = 20_000;
        for _ in 0..n {
            let w = roll_weapon(&[], &mut rng);
            let tier = WEAPONS[w].box_tier;
            assert_ne!(tier, BoxTier::None);
            counts[match tier {
                BoxTier::Bad => 0,
                BoxTier::Medium => 1,
                _ => 2,
            }] += 1;
        }
        let share = |c: u32| c as f32 / n as f32;
        assert!((share(counts[0]) - 0.4).abs() < 0.02, "{counts:?}");
        assert!((share(counts[1]) - 0.4).abs() < 0.02, "{counts:?}");
        assert!((share(counts[2]) - 0.2).abs() < 0.02, "{counts:?}");
    }

    #[test]
    fn owned_weapons_are_never_offered() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let good: Vec<usize> = box_weapons()
            .filter(|i| WEAPONS[*i].box_tier == BoxTier::Good)
            .collect();
        for _ in 0..2000 {
            let w = roll_weapon(&good, &mut rng);
            assert!(!good.contains(&w));
        }
    }
}
