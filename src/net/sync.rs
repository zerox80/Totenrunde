//! Abgleich einer Koop-Partie: Spielerzustände, Welt-Schnappschüsse, Ereignisse, Treffer,
//! Anfragen und Effekte. Welche Seite was besitzt, steht in [`super::msg`].

use std::collections::{HashMap, HashSet};

use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use totenrunde_mp_client::protocol::{ClientMsg, MatchPlayer, MatchResult, Target};

use super::connection::{GameOut, Incoming, MemberLeft, Online};
use super::msg::*;
use super::session::{Session, Team};
use super::{NetId, Notices};
use crate::audio::{Sfx, SfxEvent};
use crate::base::InGame;
use crate::base::assets::ZombieAnims;
use crate::base::i18n::tf;
use crate::fx::FxEvent;
use crate::game::interact::mystery_box::{BoxPhase, MysteryBox};
use crate::game::perks::{Perk, Perks};
use crate::game::points::Score;
use crate::game::powerups::{
    ActivePowerups, Powerup, PowerupCollected, PowerupKind, spawn_powerup,
};
use crate::game::requests::{WorldGrant, WorldRequest};
use crate::game::rounds::{RoundEnded, RoundPhase, RoundStarted, Rounds};
use crate::player::{GameOverFlag, Health, Player, PlayerDamage, Revived, Stance};
use crate::weapons::assets::WeaponAssets;
use crate::weapons::projectiles::Tracer;
use crate::weapons::{Blast, Loadout, ShotFired, WEAPONS, def};
use crate::world::barricades::{BOARDS, Barricades, BoardState};
use crate::world::doors::OpenDoor;
use crate::world::mapdata::MapFile;
use crate::world::power::Power;
use crate::world::zones::Zones;
use crate::zombies::{
    Attacker, DamageKind, HitPart, SpeedClass, ZState, Zombie, ZombieHit, ZombieRig, ZombieSpawn,
    spawn_zombie,
};

/// Abstand der Zustands- und Welt-Nachrichten (20 pro Sekunde)
const SEND_EVERY: f64 = 0.05;

/// Zuordnung der Netz-IDs zu Entities (Gäste) und Vergabe neuer IDs (Host).
#[derive(Resource, Default)]
pub struct NetIds {
    next: u16,
    pub zombies: HashMap<u16, Entity>,
    pub powerups: HashMap<u16, Entity>,
}

impl NetIds {
    fn alloc(&mut self) -> u16 {
        self.next = self.next.wrapping_add(1);
        self.next
    }
}

pub fn reset_ids(mut ids: ResMut<NetIds>) {
    *ids = NetIds::default();
}

/// Taktgeber für das Senden (Echtzeit, die Welt pausiert im Koop nie).
#[derive(Resource, Default)]
pub struct SendClock {
    next: f64,
    pub due: bool,
}

pub fn tick_clock(mut clock: ResMut<SendClock>, real: Res<Time<Real>>) {
    let now = real.elapsed_secs_f64();
    clock.due = now >= clock.next;
    if clock.due {
        clock.next = (clock.next + SEND_EVERY).max(now);
    }
}

/// Zombie vom Host, dessen Bewegung weich nachgezogen wird.
#[derive(Component, Default)]
pub struct ZombieReplica {
    motion: super::session::Interp,
    /// Zuletzt empfangener Zustand (erkennt neue Schläge und Bretter)
    last: Option<ZNet>,
}

// ---------------------------------------------------------------------------------------
// Umrechnungen
// ---------------------------------------------------------------------------------------

fn class_index(c: SpeedClass) -> u8 {
    match c {
        SpeedClass::Walk => 0,
        SpeedClass::Run => 1,
        SpeedClass::Sprint => 2,
    }
}

fn class_of(i: u8) -> SpeedClass {
    match i {
        0 => SpeedClass::Walk,
        1 => SpeedClass::Run,
        _ => SpeedClass::Sprint,
    }
}

fn znet(s: ZState) -> ZNet {
    match s {
        ZState::Rising { t } => ZNet::Rising(t),
        ZState::ToWindow { b } => ZNet::ToWindow(b as u8),
        ZState::AtWindow { b, tearing, .. } => ZNet::AtWindow {
            b: b as u8,
            tearing,
        },
        ZState::Climbing { b, t } => ZNet::Climbing { b: b as u8, t },
        ZState::Chasing => ZNet::Chasing,
        ZState::Attacking { t, .. } => ZNet::Attacking(t),
        ZState::Dying { t } => ZNet::Dying(t),
    }
}

/// Hat der Zombie seit dem letzten Schnappschuss einen neuen Schlag oder ein neues Brett
/// begonnen?
fn fresh_action(now: ZNet, last: Option<ZNet>) -> bool {
    match (now, last) {
        (ZNet::Attacking(t), Some(ZNet::Attacking(pt))) => t < pt,
        (ZNet::Attacking(_), _) => true,
        (ZNet::AtWindow { tearing, .. }, Some(ZNet::AtWindow { tearing: pt, .. })) => {
            tearing > 0.0 && (pt <= 0.0 || tearing < pt)
        }
        (ZNet::AtWindow { tearing, .. }, _) => tearing > 0.0,
        _ => false,
    }
}

/// Zustand für die Animation. Ein neuer Schlag oder ein neues Brett beginnt bei null, damit
/// die Animation von vorn startet; laufende behalten ihre lokale Zeit.
fn zstate(n: ZNet, prev: ZState, fresh: bool) -> ZState {
    match n {
        ZNet::Rising(t) => ZState::Rising { t },
        ZNet::ToWindow(b) => ZState::ToWindow { b: b as usize },
        ZNet::AtWindow { b, tearing } => {
            let tearing = match prev {
                _ if fresh => 0.001,
                ZState::AtWindow { tearing: local, .. } if local > 0.0 && tearing > 0.0 => local,
                _ => tearing,
            };
            ZState::AtWindow {
                b: b as usize,
                cd: 0.0,
                tearing,
            }
        }
        ZNet::Climbing { b, t } => ZState::Climbing { b: b as usize, t },
        ZNet::Chasing => ZState::Chasing,
        ZNet::Attacking(t) => {
            let t = match prev {
                _ if fresh => 0.0,
                ZState::Attacking { t: local, .. } => local,
                _ => t,
            };
            ZState::Attacking {
                t,
                hit: true,
                window: None,
            }
        }
        ZNet::Dying(t) => ZState::Dying { t },
    }
}

fn part_index(p: HitPart) -> u8 {
    match p {
        HitPart::Head => 0,
        HitPart::Neck => 1,
        HitPart::Torso => 2,
        HitPart::Limb => 3,
    }
}

fn part_of(i: u8) -> HitPart {
    match i {
        0 => HitPart::Head,
        1 => HitPart::Neck,
        2 => HitPart::Torso,
        _ => HitPart::Limb,
    }
}

fn kind_index(k: DamageKind) -> u8 {
    match k {
        DamageKind::Bullet => 0,
        DamageKind::Melee => 1,
        _ => 2,
    }
}

fn kind_of(i: u8) -> DamageKind {
    match i {
        0 => DamageKind::Bullet,
        1 => DamageKind::Melee,
        _ => DamageKind::Explosive,
    }
}

fn box_net(phase: BoxPhase) -> BoxNetPhase {
    match phase {
        BoxPhase::Closed => BoxNetPhase::Closed,
        BoxPhase::Rolling {
            t, weapon, owner, ..
        } => BoxNetPhase::Rolling {
            t,
            weapon: weapon.min(255) as u8,
            owner,
        },
        BoxPhase::Offer { t, weapon, owner } => BoxNetPhase::Offer {
            t,
            weapon: weapon as u8,
            owner,
        },
        BoxPhase::Teddy { t } => BoxNetPhase::Teddy { t },
        BoxPhase::Away { t } => BoxNetPhase::Away { t },
    }
}

fn box_phase(n: BoxNetPhase) -> Option<BoxPhase> {
    Some(match n {
        BoxNetPhase::Closed => BoxPhase::Closed,
        BoxNetPhase::Rolling { t, weapon, owner } => {
            if weapon != 255 && WEAPONS.get(weapon as usize).is_none() {
                return None;
            }
            BoxPhase::Rolling {
                t,
                weapon: if weapon == 255 {
                    usize::MAX
                } else {
                    weapon as usize
                },
                next: 0.0,
                owner,
            }
        }
        BoxNetPhase::Offer { t, weapon, owner } => {
            WEAPONS.get(weapon as usize)?;
            BoxPhase::Offer {
                t,
                weapon: weapon as usize,
                owner,
            }
        }
        BoxNetPhase::Teddy { t } => BoxPhase::Teddy { t },
        BoxNetPhase::Away { t } => BoxPhase::Away { t },
    })
}

fn box_time(p: BoxPhase) -> f32 {
    match p {
        BoxPhase::Closed => 0.0,
        BoxPhase::Rolling { t, .. }
        | BoxPhase::Offer { t, .. }
        | BoxPhase::Teddy { t }
        | BoxPhase::Away { t } => t,
    }
}

fn board_mask(boards: &[BoardState; BOARDS]) -> u8 {
    boards.iter().enumerate().fold(0, |m, (i, b)| {
        if matches!(b, BoardState::Intact | BoardState::Repairing(_)) {
            m | (1 << i)
        } else {
            m
        }
    })
}

fn stance_index(s: Stance) -> u8 {
    match s {
        Stance::Stand => 0,
        Stance::Crouch => 1,
        Stance::Prone => 2,
    }
}

// ---------------------------------------------------------------------------------------
// Alle Spieler
// ---------------------------------------------------------------------------------------

/// Eigenen Zustand an die Mitspieler.
pub fn send_state(
    clock: Res<SendClock>,
    player: Single<(&Transform, &Player, &Health, &Loadout)>,
    score: Res<Score>,
    perks: Res<Perks>,
    mut out: ResMut<GameOut>,
) {
    if !clock.due {
        return;
    }
    let (tf, p, h, lo) = *player;
    let slot = lo.current_slot();
    out.send(
        Target::Others,
        GameMsg::State(PlayerState {
            pos: tf.translation.to_array(),
            yaw: p.yaw,
            pitch: p.pitch,
            speed: Vec2::new(p.vel.x, p.vel.z).length(),
            stance: stance_index(p.stance),
            sprinting: p.sprinting,
            life: if h.dead {
                Life::Dead
            } else if h.downed {
                Life::Downed
            } else {
                Life::Alive
            },
            bleed: if h.downed {
                h.revive_left.max(0.0)
            } else {
                0.0
            },
            weapon: slot.map(|s| s.weapon as u8).unwrap_or(0),
            upgraded: slot.is_some_and(|s| s.upgraded),
            points: score.points,
            earned: score.earned,
            kills: score.kills,
            headshots: score.headshots,
            downs: score.downs,
            revives: score.revives,
            perks: Perk::ALL
                .iter()
                .enumerate()
                .filter(|(_, p)| perks.has(**p))
                .fold(0, |m, (i, _)| m | (1 << i)),
        }),
    );
}

/// Zustände, Schaden, Punkte, Wiederbelebung und Effekte der Mitspieler.
#[allow(clippy::too_many_arguments)]
pub fn receive_common(
    mut commands: Commands,
    mut incoming: MessageReader<Incoming>,
    real: Res<Time<Real>>,
    mut team: ResMut<Team>,
    mut notices: ResMut<Notices>,
    mut score: ResMut<Score>,
    wa: Res<WeaponAssets>,
    mut damage: MessageWriter<PlayerDamage>,
    mut revived: MessageWriter<Revived>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let now = real.elapsed_secs_f64();
    for Incoming { from, msg } in incoming.read() {
        match msg {
            GameMsg::State(state) => {
                let Some(m) = team.get_mut(*from) else {
                    continue;
                };
                let before = m.state.life;
                if m.seen && before != state.life {
                    let key = match (before, state.life) {
                        (_, Life::Downed) => Some("mp.notice_down"),
                        (Life::Downed, Life::Alive) => Some("mp.notice_revived"),
                        (_, Life::Dead) => Some("mp.notice_dead"),
                        (Life::Dead, Life::Alive) => Some("mp.notice_back"),
                        _ => None,
                    };
                    if let Some(key) = key {
                        notices.push(tf(key, &[("name", &m.name)]));
                    }
                }
                m.motion.push(now, v3(state.pos), state.yaw);
                m.state = state.clone();
                m.seen = true;
            }
            GameMsg::Damage { amount, from } => {
                damage.write(PlayerDamage {
                    amount: *amount,
                    from: v3(*from),
                });
            }
            GameMsg::Points {
                amount,
                kill,
                headshot,
            } => {
                score.add_final(*amount);
                score.kills += *kill as u32;
                score.headshots += *headshot as u32;
            }
            GameMsg::Revive => {
                revived.write(Revived);
                notices.push(tf("mp.notice_revived_by", &[("name", &team.name(*from))]));
            }
            GameMsg::Shot(shot) => {
                let w = (shot.weapon as usize).min(crate::weapons::WEAPONS.len() - 1);
                let origin = v3(shot.origin);
                let end = v3(shot.end);
                sfx.write(SfxEvent::at(def(w).sound, origin).pitch(if shot.upgraded {
                    0.92
                } else {
                    1.0
                }));
                let len = (end - origin).length();
                if len > 2.0 {
                    let start = origin + (end - origin) / len * 0.9;
                    let mid = (start + end) / 2.0;
                    commands.spawn((
                        Mesh3d(wa.bolt_mesh.clone()),
                        MeshMaterial3d(wa.tracer_mat.clone()),
                        Transform::from_translation(mid)
                            .looking_to(end - start, Vec3::Y)
                            .with_scale(Vec3::new(0.12, 0.12, (end - start).length())),
                        Tracer { life: 0.04 },
                        NotShadowCaster,
                        DespawnOnExit(crate::base::AppState::Playing),
                    ));
                }
            }
            GameMsg::Blast(b) => match b.ray {
                Some(upgraded) => {
                    fx.write(FxEvent::RayImpact {
                        pos: v3(b.pos),
                        upgraded,
                    });
                    sfx.write(SfxEvent::at(Sfx::Explosion, v3(b.pos)).vol(0.35).pitch(1.6));
                }
                None => {
                    fx.write(FxEvent::Explosion {
                        pos: v3(b.pos),
                        radius: b.radius,
                    });
                    sfx.write(SfxEvent::at(Sfx::Explosion, v3(b.pos)));
                }
            },
            _ => {}
        }
    }
}

/// Eigene Schüsse und Explosionen an die Mitspieler (nur als Effekt).
pub fn forward_effects(
    mut shots: MessageReader<ShotFired>,
    mut blasts: MessageReader<Blast>,
    mut out: ResMut<GameOut>,
) {
    for s in shots.read() {
        out.send(
            Target::Others,
            GameMsg::Shot(ShotFx {
                weapon: s.weapon as u8,
                upgraded: s.upgraded,
                origin: s.origin.to_array(),
                end: s.end.to_array(),
            }),
        );
    }
    for b in blasts.read() {
        out.send(
            Target::Others,
            GameMsg::Blast(BlastFx {
                pos: b.pos.to_array(),
                radius: b.radius,
                ray: b.ray,
            }),
        );
    }
}

pub fn member_left(
    mut left: MessageReader<MemberLeft>,
    mut team: ResMut<Team>,
    mut notices: ResMut<Notices>,
) {
    for MemberLeft(slot) in left.read() {
        if let Some(m) = team.get_mut(*slot)
            && m.connected
        {
            m.connected = false;
            notices.push(tf("mp.notice_left", &[("name", &m.name)]));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Host
// ---------------------------------------------------------------------------------------

/// Neue Zombies und Power-ups bekommen eine Netz-ID.
pub fn assign_net_ids(
    mut commands: Commands,
    mut ids: ResMut<NetIds>,
    zombies: Query<Entity, (With<Zombie>, Without<NetId>)>,
    powerups: Query<Entity, (With<Powerup>, Without<NetId>)>,
) {
    for e in zombies.iter().chain(powerups.iter()) {
        let id = ids.alloc();
        commands.entity(e).insert(NetId(id));
    }
}

/// Welt-Schnappschuss an alle Gäste.
#[allow(clippy::too_many_arguments)]
pub fn send_snapshot(
    clock: Res<SendClock>,
    rounds: Option<Res<Rounds>>,
    pu: Res<ActivePowerups>,
    power: Res<Power>,
    zones: Res<Zones>,
    bars: Res<Barricades>,
    mbox: Option<Res<MysteryBox>>,
    zombies: Query<(&NetId, &Zombie, &Transform)>,
    mut out: ResMut<GameOut>,
) {
    if !clock.due {
        return;
    }
    let (round, phase) = rounds
        .map(|r| {
            (
                r.round,
                match r.phase {
                    RoundPhase::Intro { .. } => 0,
                    RoundPhase::Active => 1,
                    RoundPhase::Intermission { .. } => 2,
                },
            )
        })
        .unwrap_or((0, 0));
    let snapshot = Snapshot {
        round,
        phase,
        insta_kill: pu.insta_kill,
        double_points: pu.double_points,
        fire_sale: pu.fire_sale,
        power: power.on,
        doors: zones
            .door_open
            .iter()
            .enumerate()
            .filter(|(_, open)| **open)
            .map(|(i, _)| i as u16)
            .collect(),
        boards: bars.list.iter().map(|b| board_mask(&b.boards)).collect(),
        mystery: mbox.map(|b| BoxNet {
            spot: b.spot as u8,
            phase: box_net(b.phase),
        }),
        zombies: zombies
            .iter()
            .map(|(id, z, tf)| ZombieNet {
                id: id.0,
                pos: q_pos(tf.translation),
                yaw: q_yaw(z.yaw),
                state: znet(z.state),
                class: class_index(z.class),
                model: z.model as u8,
                variant: z.variant,
                speed: (z.vel.length() * 100.0).min(u16::MAX as f32) as u16,
                headless: z.headless,
                death_back: z.death_back,
            })
            .collect(),
    };
    out.send(Target::Others, GameMsg::World(Box::new(snapshot)));
}

/// Power-ups auftauchen und verschwinden lassen (bei den Gästen).
pub fn announce_powerups(
    powerups: Query<(&NetId, &Powerup, &Transform)>,
    mut known: Local<HashSet<u16>>,
    mut out: ResMut<GameOut>,
) {
    let mut now = HashSet::new();
    for (id, p, tf) in powerups.iter() {
        now.insert(id.0);
        if !known.contains(&id.0) {
            out.send(
                Target::Others,
                GameMsg::Event(WorldEvent::PowerupSpawned {
                    id: id.0,
                    kind: p.kind.index(),
                    pos: p.base.to_array(),
                }),
            );
            let _ = tf;
        }
    }
    for gone in known.difference(&now) {
        out.send(
            Target::Others,
            GameMsg::Event(WorldEvent::PowerupGone { id: *gone }),
        );
    }
    *known = now;
}

/// Rundenwechsel, Power-ups und Spielende an alle; das Ergebnis an den Server.
#[allow(clippy::too_many_arguments)]
pub fn forward_world_events(
    mut started: MessageReader<RoundStarted>,
    mut ended: MessageReader<RoundEnded>,
    mut collected: MessageReader<PowerupCollected>,
    over: Res<GameOverFlag>,
    rounds: Option<Res<Rounds>>,
    score: Res<Score>,
    team: Res<Team>,
    session: Res<Session>,
    mut reported: Local<bool>,
    mut out: ResMut<GameOut>,
    mut online: ResMut<Online>,
) {
    for RoundStarted(r) in started.read() {
        out.send(Target::Others, GameMsg::Event(WorldEvent::RoundStarted(*r)));
    }
    for _ in ended.read() {
        out.send(Target::Others, GameMsg::Event(WorldEvent::RoundEnded));
    }
    for c in collected.read() {
        out.send(
            Target::Others,
            GameMsg::Event(WorldEvent::PowerupCollected {
                kind: c.kind.index(),
                by: c.by,
            }),
        );
    }
    if !over.0 {
        *reported = false;
        return;
    }
    if *reported {
        return;
    }
    *reported = true;
    out.send(Target::Others, GameMsg::Event(WorldEvent::GameOver));
    let mut players = vec![MatchPlayer {
        slot: session.you,
        kills: score.kills,
        headshots: score.headshots,
        downs: score.downs,
        revives: score.revives,
        points: score.earned,
    }];
    players.extend(team.members.iter().filter(|m| m.seen).map(|m| MatchPlayer {
        slot: m.slot,
        kills: m.state.kills,
        headshots: m.state.headshots,
        downs: m.state.downs,
        revives: m.state.revives,
        points: m.state.earned,
    }));
    online.send(ClientMsg::GameOver {
        result: MatchResult {
            round: rounds.map(|r| r.round).unwrap_or(0),
            players,
        },
    });
}

/// Treffer und Anfragen der Gäste.
pub fn receive_host(
    mut incoming: MessageReader<Incoming>,
    ids: Query<(Entity, &NetId), With<Zombie>>,
    mut hits: MessageWriter<ZombieHit>,
    mut requests: MessageWriter<WorldRequest>,
) {
    let mut by_id: Option<HashMap<u16, Entity>> = None;
    for Incoming { from, msg } in incoming.read() {
        match msg {
            GameMsg::Hit(h) => {
                let map =
                    by_id.get_or_insert_with(|| ids.iter().map(|(e, id)| (id.0, e)).collect());
                let Some(&zombie) = map.get(&h.zombie) else {
                    continue;
                };
                debug!(
                    "Treffer von Platz {from}: Zombie {} ({} Schaden)",
                    h.zombie, h.damage
                );
                hits.write(ZombieHit {
                    zombie,
                    by: Attacker::Remote(*from),
                    damage: h.damage.clamp(0.0, 1.0e6),
                    part: part_of(h.part),
                    kind: kind_of(h.kind),
                    point: v3(h.point),
                    dir: v3(h.dir).normalize_or(Vec3::NEG_Z),
                });
            }
            GameMsg::Request(req) => {
                requests.write(WorldRequest {
                    by: *from,
                    req: req.clone(),
                });
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------------------
// Gäste
// ---------------------------------------------------------------------------------------

/// Eigene Treffer an den Host; das Blut gibt es sofort.
pub fn forward_hits(
    mut hits: MessageReader<ZombieHit>,
    ids: Query<&NetId>,
    mut fx: MessageWriter<FxEvent>,
    mut out: ResMut<GameOut>,
) {
    for h in hits.read() {
        if h.by != Attacker::Local {
            continue;
        }
        let Ok(id) = ids.get(h.zombie) else {
            continue;
        };
        fx.write(FxEvent::Blood {
            pos: h.point,
            dir: h.dir,
            amount: if h.part == HitPart::Head { 2.0 } else { 1.0 },
        });
        out.send(
            Target::Host,
            GameMsg::Hit(NetHit {
                zombie: id.0,
                damage: h.damage,
                part: part_index(h.part),
                kind: kind_index(h.kind),
                point: h.point.to_array(),
                dir: h.dir.to_array(),
            }),
        );
    }
}

pub fn forward_requests(
    mut requests: MessageReader<WorldRequest>,
    session: Res<Session>,
    mut out: ResMut<GameOut>,
) {
    for r in requests.read() {
        if r.by == session.you {
            out.send(Target::Host, GameMsg::Request(r.req.clone()));
        }
    }
}

/// Ereignisse und Zusagen des Hosts.
#[allow(clippy::too_many_arguments)]
pub fn receive_guest(
    mut commands: Commands,
    mut incoming: MessageReader<Incoming>,
    session: Res<Session>,
    asset_server: Res<AssetServer>,
    mut ids: ResMut<NetIds>,
    mut over: ResMut<GameOverFlag>,
    mut started: MessageWriter<RoundStarted>,
    mut ended: MessageWriter<RoundEnded>,
    mut collected: MessageWriter<PowerupCollected>,
    mut grants: MessageWriter<WorldGrant>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    for Incoming { from, msg } in incoming.read() {
        if *from != session.host {
            continue;
        }
        match msg {
            GameMsg::Event(event) => match event {
                WorldEvent::RoundStarted(r) => {
                    started.write(RoundStarted(*r));
                }
                WorldEvent::RoundEnded => {
                    ended.write(RoundEnded);
                }
                WorldEvent::PowerupSpawned { id, kind, pos } => {
                    let Some(kind) = PowerupKind::from_index(*kind) else {
                        continue;
                    };
                    let e = spawn_powerup(&mut commands, &asset_server, kind, v3(*pos));
                    commands.entity(e).insert(NetId(*id));
                    ids.powerups.insert(*id, e);
                    sfx.write(SfxEvent::at(Sfx::PowerupSpawn, v3(*pos) + Vec3::Y));
                }
                WorldEvent::PowerupGone { id } => {
                    if let Some(e) = ids.powerups.remove(id)
                        && let Ok(mut ec) = commands.get_entity(e)
                    {
                        ec.despawn();
                    }
                }
                WorldEvent::PowerupCollected { kind, by } => {
                    if let Some(kind) = PowerupKind::from_index(*kind) {
                        collected.write(PowerupCollected { kind, by: *by });
                    }
                }
                WorldEvent::GameOver => over.0 = true,
            },
            GameMsg::Grant(grant) => {
                grants.write(WorldGrant(*grant));
            }
            _ => {}
        }
    }
}

/// Welt-Schnappschuss des Hosts übernehmen.
#[allow(clippy::too_many_arguments)]
pub fn apply_snapshot(
    mut commands: Commands,
    mut incoming: MessageReader<Incoming>,
    session: Res<Session>,
    real: Res<Time<Real>>,
    time: Res<Time>,
    game: Res<InGame>,
    anims: Res<ZombieAnims>,
    mut ids: ResMut<NetIds>,
    mut world: WorldState,
    mut zombies: Query<(&mut Zombie, &mut ZombieReplica, Option<&ZombieRig>)>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxEvent>,
    mut open: MessageWriter<OpenDoor>,
) {
    let Some(snap) = incoming
        .read()
        .filter(|i| i.from == session.host)
        .filter_map(|i| match &i.msg {
            GameMsg::World(s) => Some(s),
            _ => None,
        })
        .last()
    else {
        return;
    };
    let now = real.elapsed_secs_f64();

    // Runde
    if let Some(r) = world.rounds.as_deref_mut() {
        let phase = match snap.phase {
            0 => RoundPhase::Intro { t: 0.0 },
            1 => RoundPhase::Active,
            _ => RoundPhase::Intermission { t: 0.0 },
        };
        let changed = std::mem::discriminant(&r.phase) != std::mem::discriminant(&phase);
        if r.round != snap.round || changed {
            r.changed_at = game.time;
            r.round = snap.round;
        }
        if changed {
            r.phase = phase;
        }
    }
    // Power-ups, Strom, Türen
    world.pu.insta_kill = snap.insta_kill;
    world.pu.double_points = snap.double_points;
    world.pu.fire_sale = snap.fire_sale;
    if snap.power && !world.power.on {
        world.power.on = true;
        world.power.since = time.elapsed_secs();
        sfx.write(SfxEvent::ui(Sfx::PowerOn));
    }
    for id in &snap.doors {
        if !world.zones.is_door_open(*id) {
            open.write(OpenDoor(*id));
        }
    }
    // Bretter: Unterschiede als Abreißen bzw. Anbringen abspielen
    for (bar, mask) in world.bars.list.iter_mut().zip(&snap.boards) {
        for i in 0..BOARDS {
            let want = mask & (1 << i) != 0;
            let have = matches!(bar.boards[i], BoardState::Intact | BoardState::Repairing(_));
            if want != have {
                bar.boards[i] = if want {
                    BoardState::Repairing(0.0)
                } else {
                    BoardState::Tearing(0.0)
                };
            }
        }
    }
    // Kiste
    // Ort und Phase zusammen prüfen, bevor einer der beiden Werte übernommen wird.
    let box_state = snap.mystery.and_then(|net| {
        let spot = net.spot as usize;
        world.map.box_spots.get(spot)?;
        Some((spot, box_phase(net.phase)?))
    });
    if let (Some((spot, want)), Some(mbox)) = (box_state, world.mbox.as_deref_mut()) {
        mbox.spot = spot;
        let same_kind = std::mem::discriminant(&want) == std::mem::discriminant(&mbox.phase);
        let same_item = match (want, mbox.phase) {
            (
                BoxPhase::Rolling {
                    weapon: a,
                    owner: oa,
                    ..
                },
                BoxPhase::Rolling {
                    weapon: b,
                    owner: ob,
                    ..
                },
            )
            | (
                BoxPhase::Offer {
                    weapon: a,
                    owner: oa,
                    ..
                },
                BoxPhase::Offer {
                    weapon: b,
                    owner: ob,
                    ..
                },
            ) => a == b && oa == ob,
            _ => true,
        };
        if !same_kind || !same_item || (box_time(want) - box_time(mbox.phase)).abs() > 0.5 {
            mbox.phase = match (want, mbox.phase) {
                // Wechselnde Anzeige-Waffe nicht zurücksetzen
                (
                    BoxPhase::Rolling {
                        t, weapon, owner, ..
                    },
                    BoxPhase::Rolling { next, .. },
                ) => BoxPhase::Rolling {
                    t,
                    weapon,
                    next,
                    owner,
                },
                _ => want,
            };
        }
    }
    // Zombies
    let mut alive: HashSet<u16> = HashSet::with_capacity(snap.zombies.len());
    for zn in &snap.zombies {
        alive.insert(zn.id);
        let pos = dq_pos(zn.pos);
        let yaw = dq_yaw(zn.yaw);
        let existing = ids
            .zombies
            .get(&zn.id)
            .copied()
            .filter(|e| zombies.contains(*e));
        match existing {
            Some(e) => {
                let Ok((mut z, mut rep, rig)) = zombies.get_mut(e) else {
                    continue;
                };
                rep.motion.push(now, pos, yaw);
                let fresh = fresh_action(zn.state, rep.last);
                rep.last = Some(zn.state);
                z.state = zstate(zn.state, z.state, fresh);
                z.class = class_of(zn.class);
                z.vel = Vec3::new(-yaw.sin(), 0.0, -yaw.cos()) * (zn.speed as f32 / 100.0);
                z.death_back = zn.death_back;
                if zn.headless && !z.headless {
                    z.headless = true;
                    if let Some(rig) = rig {
                        for hide in [rig.head_mesh, rig.eyes_mesh].into_iter().flatten() {
                            commands.entity(hide).insert(Visibility::Hidden);
                        }
                        if let Some(s) = rig.stump_mesh {
                            commands.entity(s).insert(Visibility::Inherited);
                        }
                    }
                    fx.write(FxEvent::Gib {
                        pos: pos + Vec3::Y * 1.6,
                    });
                }
            }
            None => {
                let state = zstate(zn.state, ZState::Chasing, true);
                if let ZState::Rising { t } = state
                    && t < 0.3
                {
                    let ground = Vec3::new(pos.x, 0.0, pos.z);
                    fx.write(FxEvent::Dirt { pos: ground });
                    sfx.write(SfxEvent::at(Sfx::ZombieRise, ground));
                }
                let e = spawn_zombie(
                    &mut commands,
                    &anims,
                    ZombieSpawn {
                        pos,
                        yaw,
                        state,
                        health: 1.0,
                        class: class_of(zn.class),
                        look: Some((zn.model as usize, zn.variant)),
                    },
                );
                let mut rep = ZombieReplica {
                    last: Some(zn.state),
                    ..default()
                };
                rep.motion.push(now, pos, yaw);
                commands.entity(e).insert((NetId(zn.id), rep));
                ids.zombies.insert(zn.id, e);
            }
        }
    }
    ids.zombies.retain(|id, e| {
        if alive.contains(id) {
            return true;
        }
        if let Ok(mut ec) = commands.get_entity(*e) {
            ec.despawn();
        }
        false
    });
}

/// Was `apply_snapshot` an der Welt ändert (gebündelt wegen der Parameterzahl).
#[derive(bevy::ecs::system::SystemParam)]
pub struct WorldState<'w> {
    map: Res<'w, MapFile>,
    rounds: Option<ResMut<'w, Rounds>>,
    pu: ResMut<'w, ActivePowerups>,
    power: ResMut<'w, Power>,
    zones: Res<'w, Zones>,
    bars: ResMut<'w, Barricades>,
    mbox: Option<ResMut<'w, MysteryBox>>,
}

/// Zombies des Hosts weich bewegen; Schlag und Bretterreißen laufen lokal weiter, damit die
/// Animation nicht bei jedem Schnappschuss neu beginnt.
pub fn animate_replicas(
    real: Res<Time<Real>>,
    time: Res<Time>,
    mut zombies: Query<(&mut Zombie, &ZombieReplica, &mut Transform)>,
) {
    let at = real.elapsed_secs_f64() - super::session::INTERP_DELAY;
    let dt = time.delta_secs();
    for (mut z, rep, mut tf) in zombies.iter_mut() {
        if let Some((pos, yaw)) = rep.motion.sample(at) {
            tf.translation = pos;
            tf.rotation = Quat::from_rotation_y(yaw);
            z.yaw = yaw;
        }
        match &mut z.state {
            ZState::Attacking { t, .. } => *t += dt,
            ZState::AtWindow { tearing, .. } if *tearing > 0.0 => *tearing += dt,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legitimate_box_phases_round_trip() {
        let phases = [
            BoxPhase::Closed,
            BoxPhase::Rolling {
                t: 1.0,
                weapon: 0,
                next: 0.0,
                owner: 1,
            },
            BoxPhase::Rolling {
                t: 1.0,
                weapon: usize::MAX,
                next: 0.0,
                owner: 1,
            },
            BoxPhase::Offer {
                t: 1.0,
                weapon: WEAPONS.len() - 1,
                owner: 1,
            },
            BoxPhase::Teddy { t: 1.0 },
            BoxPhase::Away { t: 1.0 },
        ];
        for phase in phases {
            assert_eq!(box_phase(box_net(phase)), Some(phase));
        }
    }
}
