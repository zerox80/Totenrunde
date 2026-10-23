//! Rundenlogik mit den Formeln aus dem Original:
//!
//! * Zombie-Gesundheit: 150 in Runde 1, +100 pro Runde bis Runde 9, danach x1.1 pro Runde
//! * Anzahl (Solo): 24 + 0.5·6·Multiplikator, in den ersten Runden prozentual reduziert
//! * Spawn-Verzögerung: 2 s, pro Runde x0.95 (min. 0.08 s), max. 24 gleichzeitig
//! * Tempo: Rundenwert·8 + Zufall entscheidet über Gehen / Laufen / Sprinten

use bevy::prelude::*;
use rand::Rng;

use super::powerups::{self, ActivePowerups};
use super::spawning::pick_spawner;
use crate::audio::{Sfx, SfxEvent};
use crate::base::assets::ZombieAnims;
use crate::base::debug::DebugConfig;
use crate::base::{AppState, GameSet, InGame};
use crate::fx::FxEvent;
use crate::game::points::Score;
use crate::player::{GameOverFlag, Player};
use crate::weapons::{Loadout, MAX_GRENADES};
use crate::world::barricades::Barricades;
use crate::world::mapdata::MapFile;
use crate::world::zones::Zones;
use crate::zombies::{spawn_zombie, SpeedClass, ZState, Zombie, ZombieKilled, ZombieSpawn, MAX_ALIVE};

pub struct RoundsPlugin;

impl Plugin for RoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), reset_rounds)
            .add_systems(
                Update,
                (round_logic, count_kills, game_over_check)
                    .chain()
                    .in_set(GameSet::World)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<InGame>),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RoundPhase {
    Intro { t: f32 },
    Active,
    Intermission { t: f32 },
}

#[derive(Resource)]
pub struct Rounds {
    pub round: u32,
    pub phase: RoundPhase,
    pub to_spawn: u32,
    pub spawned: u32,
    pub killed_this_round: u32,
    pub spawn_cd: f32,
    pub total: u32,
    /// Spielzeit, zu der sich die Rundenzahl zuletzt geändert hat (HUD-Animation)
    pub changed_at: f32,
    pub game_over_t: f32,
}

pub fn zombie_health(round: u32) -> f32 {
    let r = round.max(1);
    if r <= 9 {
        150.0 + 100.0 * (r - 1) as f32
    } else {
        950.0 * 1.1f32.powi(r as i32 - 9)
    }
}

pub fn zombie_count(round: u32) -> u32 {
    let r = round.max(1) as f32;
    let mut mult = (r / 5.0).max(1.0);
    if round >= 10 {
        mult *= r * 0.15;
    }
    let mut max = 24.0 + (0.5 * 6.0 * mult).floor();
    max = match round {
        0 | 1 => (max * 0.25).floor(),
        2 => (max * 0.3).floor(),
        3 => (max * 0.5).floor(),
        4 => (max * 0.7).floor(),
        5 => (max * 0.9).floor(),
        _ => max,
    };
    max as u32
}

pub fn spawn_delay(round: u32) -> f32 {
    (2.0 * 0.95f32.powi(round.max(1) as i32 - 1)).max(0.08)
}

pub fn speed_class(round: u32, rng: &mut impl Rng) -> SpeedClass {
    let v = round as i32 * 8 + rng.random_range(-12..26);
    if v <= 35 {
        SpeedClass::Walk
    } else if v <= 70 {
        SpeedClass::Run
    } else {
        SpeedClass::Sprint
    }
}

fn reset_rounds(mut commands: Commands, cfg: Res<DebugConfig>, mut over: ResMut<GameOverFlag>) {
    over.0 = false;
    commands.insert_resource(Rounds {
        round: cfg.start_round.unwrap_or(1).saturating_sub(1),
        phase: RoundPhase::Intro { t: 0.0 },
        to_spawn: 0,
        spawned: 0,
        killed_this_round: 0,
        spawn_cd: 0.0,
        total: 0,
        changed_at: 0.0,
        game_over_t: 0.0,
    });
}

fn start_round(r: &mut Rounds, now: f32, pu: &mut ActivePowerups, lo: &mut Loadout, sfx: &mut MessageWriter<SfxEvent>) {
    r.round += 1;
    r.to_spawn = zombie_count(r.round);
    r.total = r.to_spawn;
    r.spawned = 0;
    r.killed_this_round = 0;
    r.spawn_cd = 3.0;
    r.phase = RoundPhase::Active;
    r.changed_at = now;
    powerups::new_round(pu);
    lo.grenades = (lo.grenades + 2).min(MAX_GRENADES);
    sfx.write(SfxEvent::ui(Sfx::RoundStart));
    info!("Runde {} beginnt: {} Zombies, {} HP", r.round, r.to_spawn, zombie_health(r.round));
}

#[allow(clippy::too_many_arguments)]
fn round_logic(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<InGame>,
    mut rounds: ResMut<Rounds>,
    map: Res<MapFile>,
    zones: Res<Zones>,
    barricades: Res<Barricades>,
    anims: Res<ZombieAnims>,
    zombies: Query<&Zombie>,
    player: Single<(&Transform, &mut Loadout), With<Player>>,
    mut pu: ResMut<ActivePowerups>,
    mut sfx: MessageWriter<SfxEvent>,
    mut fx: MessageWriter<FxEvent>,
    mut score: ResMut<Score>,
) {
    let dt = time.delta_secs();
    let (ptf, mut lo) = player.into_inner();
    let now = game.time;
    match rounds.phase {
        RoundPhase::Intro { t } => {
            let t = t + dt;
            if t >= 2.0 {
                start_round(&mut rounds, now, &mut pu, &mut lo, &mut sfx);
            } else {
                rounds.phase = RoundPhase::Intro { t };
            }
        }
        RoundPhase::Intermission { t } => {
            let t = t + dt;
            if t >= 11.0 {
                score.repair_points_round = 0;
                start_round(&mut rounds, now, &mut pu, &mut lo, &mut sfx);
            } else {
                rounds.phase = RoundPhase::Intermission { t };
            }
        }
        RoundPhase::Active => {
            let alive = zombies.iter().filter(|z| z.alive()).count();
            if rounds.to_spawn == 0 && alive == 0 && pu.nuke_queue.is_empty() {
                rounds.phase = RoundPhase::Intermission { t: 0.0 };
                rounds.changed_at = now;
                sfx.write(SfxEvent::ui(Sfx::RoundEnd));
                return;
            }
            rounds.spawn_cd -= dt;
            if rounds.to_spawn > 0 && alive < MAX_ALIVE && rounds.spawn_cd <= 0.0 {
                let mut rng = rand::rng();
                if let Some(si) = pick_spawner(&map, &zones, &barricades, ptf.translation, &mut rng) {
                    let sp = &map.spawners[si];
                    let pos = Vec3::from_array(sp.pos);
                    let to_player = ptf.translation - pos;
                    let yaw = (-to_player.x).atan2(-to_player.z);
                    let class = speed_class(rounds.round, &mut rng);
                    let (state, start_pos) = if sp.kind == "riser" {
                        fx.write(FxEvent::Dirt { pos });
                        sfx.write(SfxEvent::at(Sfx::ZombieRise, pos));
                        (ZState::Rising { t: 0.0 }, pos - Vec3::Y * 1.65)
                    } else {
                        let b = sp.barricade.unwrap_or(0);
                        (ZState::ToWindow { b }, pos)
                    };
                    spawn_zombie(
                        &mut commands,
                        &anims,
                        ZombieSpawn {
                            pos: start_pos,
                            yaw,
                            state,
                            health: zombie_health(rounds.round),
                            class,
                        },
                    );
                    rounds.to_spawn -= 1;
                    rounds.spawned += 1;
                    rounds.spawn_cd = spawn_delay(rounds.round) * rng.random_range(0.8..1.2);
                } else {
                    rounds.spawn_cd = 0.5;
                }
            }
        }
    }
}

fn count_kills(mut killed: MessageReader<ZombieKilled>, mut rounds: ResMut<Rounds>) {
    for _ in killed.read() {
        rounds.killed_this_round += 1;
    }
}

fn game_over_check(
    time: Res<Time>,
    over: Res<GameOverFlag>,
    mut rounds: ResMut<Rounds>,
    mut next: ResMut<NextState<AppState>>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    if !over.0 {
        return;
    }
    if rounds.game_over_t == 0.0 {
        sfx.write(SfxEvent::ui(Sfx::GameOver));
    }
    rounds.game_over_t += time.delta_secs();
    if rounds.game_over_t > 4.0 {
        next.set(AppState::GameOver);
    }
}
