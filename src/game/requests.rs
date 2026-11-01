//! Käufe und Aktionen, die die gemeinsame Welt verändern: Türen, Strom, Kiste und Bretter.
//!
//! Der Spieler stellt eine Anfrage ([`WorldRequest`]); wer die Welt berechnet (Solo: das
//! eigene Spiel, Koop: der Host), prüft sie, führt sie aus und antwortet ([`WorldGrant`]).
//! Bezahlt wird erst mit der Zusage – kaufen zwei Spieler gleichzeitig dieselbe Tür, zahlt
//! nur einer. Im Solospiel ist das derselbe Weg, nur ohne Netz dazwischen.

use bevy::prelude::*;
use totenrunde_mp_client::protocol::Target;

use crate::audio::{Sfx, SfxEvent};
use crate::game::interact::mystery_box::{MysteryBox, open_box, take_offer};
use crate::game::interact::repair::ROUND_CAP;
use crate::game::points::Score;
use crate::game::powerups::ActivePowerups;
use crate::net::msg::{GameMsg, Grant, Req};
use crate::net::{GameOut, Session, local_slot};
use crate::weapons::{GiveWeapon, WEAPONS};
use crate::world::barricades::Barricades;
use crate::world::doors::OpenDoor;
use crate::world::mapdata::MapFile;
use crate::world::power::Power;
use crate::world::zones::Zones;

/// Anfrage eines Spielers (Platz `by`).
#[derive(Message, Clone, Debug)]
pub struct WorldRequest {
    pub by: u8,
    pub req: Req,
}

/// Zusage oder Absage an den eigenen Spieler.
#[derive(Message, Clone, Copy, Debug)]
pub struct WorldGrant(pub Grant);

/// Anfragen prüfen und ausführen (nur wo die Welt berechnet wird).
#[allow(clippy::too_many_arguments)]
pub fn handle_requests(
    mut requests: MessageReader<WorldRequest>,
    session: Option<Res<Session>>,
    map: Res<MapFile>,
    zones: Res<Zones>,
    time: Res<Time>,
    pu: Res<ActivePowerups>,
    mut power: ResMut<Power>,
    mut mbox: Option<ResMut<MysteryBox>>,
    mut bars: ResMut<Barricades>,
    mut open: MessageWriter<OpenDoor>,
    mut grants: MessageWriter<WorldGrant>,
    mut out: ResMut<GameOut>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    let me = local_slot(session.as_deref());
    // Türen, die in diesem Frame schon geöffnet wurden (OpenDoor wirkt erst danach)
    let mut opened: Vec<u16> = Vec::new();
    for r in requests.read() {
        let grant = match &r.req {
            Req::Door(id) => match map.doors.iter().find(|d| d.id == *id) {
                Some(d) if !zones.is_door_open(*id) && !opened.contains(id) => {
                    opened.push(*id);
                    open.write(OpenDoor(*id));
                    Grant::Door {
                        id: *id,
                        cost: d.cost,
                    }
                }
                _ => Grant::Denied,
            },
            Req::Power => {
                if power.on {
                    Grant::Denied
                } else {
                    power.on = true;
                    power.since = time.elapsed_secs();
                    sfx.write(SfxEvent::ui(Sfx::PowerOn));
                    Grant::Power
                }
            }
            Req::BoxOpen { owned } => match mbox.as_deref_mut() {
                Some(b) if b.phase == crate::game::interact::mystery_box::BoxPhase::Closed => {
                    let owned: Vec<usize> = owned.iter().map(|w| *w as usize).collect();
                    let (cost, teddy) = open_box(b, &owned, pu.fire_sale > 0.0, r.by);
                    Grant::BoxOpen { cost, teddy }
                }
                _ => Grant::Denied,
            },
            Req::BoxTake => match mbox.as_deref_mut().and_then(|b| take_offer(b, r.by)) {
                Some(weapon) => Grant::BoxTake {
                    weapon: weapon as u8,
                },
                None => Grant::Denied,
            },
            Req::Repair(b) => match bars.list.iter_mut().find(|x| x.id == *b as usize) {
                Some(bar) if bar.missing_boards() > 0 => {
                    bar.repair_one();
                    Grant::Repair { barricade: *b }
                }
                _ => Grant::Denied,
            },
        };
        debug!("Anfrage von Platz {}: {:?} -> {:?}", r.by, r.req, grant);
        if r.by == me {
            grants.write(WorldGrant(grant));
        } else {
            out.send(Target::Player(r.by), GameMsg::Grant(grant));
        }
    }
}

/// Zusagen für den eigenen Spieler: bezahlen, Waffe nehmen, Punkte fürs Brett.
pub fn apply_grants(
    mut grants: MessageReader<WorldGrant>,
    mut score: ResMut<Score>,
    powerups: Res<ActivePowerups>,
    bars: Res<Barricades>,
    mut give: MessageWriter<GiveWeapon>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    for WorldGrant(grant) in grants.read() {
        match *grant {
            Grant::Door { cost, .. } => {
                if score.spend(cost) {
                    score.doors += 1;
                    sfx.write(SfxEvent::ui(Sfx::Buy));
                }
            }
            Grant::Power => {}
            Grant::BoxOpen { cost, teddy } => {
                // Beim Teddy gibt es die Punkte zurück – also gar nicht erst abbuchen
                if !teddy && score.spend(cost) {
                    sfx.write(SfxEvent::ui(Sfx::Buy));
                }
            }
            Grant::BoxTake { weapon } => {
                if WEAPONS.get(weapon as usize).is_none() {
                    continue;
                }
                give.write(GiveWeapon {
                    weapon: weapon as usize,
                    upgraded: false,
                });
            }
            Grant::Repair { barricade } => {
                score.boards += 1;
                if score.repair_points_round < ROUND_CAP {
                    let got = score.add(10, &powerups, None);
                    score.repair_points_round += got;
                }
                if let Some(b) = bars.list.iter().find(|b| b.id == barricade as usize) {
                    sfx.write(SfxEvent::at(Sfx::BoardPlace, b.center + Vec3::Y * 1.3));
                }
            }
            Grant::Denied => {
                sfx.write(SfxEvent::ui(Sfx::Deny));
            }
        }
    }
}
