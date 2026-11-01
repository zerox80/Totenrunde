//! Koop-Multiplayer für bis zu vier Spieler über den Totenrunde-Server.
//!
//! * `connection` – Anmeldung, Konto verknüpfen, Lobbys; Ressource [`Online`] für die Menüs.
//!   WebSocket, Protokoll und Pings kommen aus dem SDK `totenrunde-mp-client`.
//! * `session`    – laufende Partie: Teilnehmer, Host, Zustand der Mitspieler
//! * `msg`        – Nachrichten zwischen den Spielen und wer was besitzt
//! * `sync`       – Abgleich von Spielern, Welt, Treffern und Käufen
//! * `avatars`    – Mitspieler als Figuren mit Namensschild
//!
//! Das Spiel des Hosts berechnet die Welt; die anderen schicken ihm Treffer und Anfragen und
//! zeigen, was er meldet. Ohne Partie (Solo) ist das ganze Modul untätig.

pub mod avatars;
pub mod connection;
pub mod msg;
pub mod session;
pub mod sync;

use bevy::prelude::*;

use crate::base::{AppState, GameSet, InGame};

pub use connection::{GameOut, Incoming, MemberLeft, Online, Status};
#[allow(unused_imports)]
pub use session::slot_color;
pub use session::{Session, Team, in_session, is_authority, is_guest, local_slot};

/// Kennung eines Zombies oder Power-ups, die bei allen Spielern gleich ist.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NetId(pub u16);

/// Kurze Meldungen im Spiel ("Anna ist niedergestreckt!").
#[derive(Resource, Default)]
pub struct Notices {
    /// Text und Alter in Sekunden
    pub list: Vec<(String, f32)>,
}

impl Notices {
    pub fn push(&mut self, text: String) {
        self.list.push((text, 0.0));
        if self.list.len() > 5 {
            self.list.remove(0);
        }
    }
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        let playing = || playing_together;
        app.init_non_send::<connection::NetLink>()
            .init_resource::<Online>()
            .init_resource::<GameOut>()
            .init_resource::<Notices>()
            .init_resource::<Team>()
            .init_resource::<sync::NetIds>()
            .init_resource::<sync::SendClock>()
            .add_message::<Incoming>()
            .add_message::<MemberLeft>()
            .add_systems(PreUpdate, connection::poll)
            .add_systems(Last, connection::flush)
            .add_systems(
                OnEnter(AppState::Playing),
                (sync::reset_ids, |mut n: ResMut<Notices>| n.list.clear()),
            )
            .add_systems(OnEnter(AppState::Menu), leave_online)
            .add_systems(OnExit(AppState::GameOver), clear_session)
            .add_systems(
                Update,
                (
                    sync::tick_clock,
                    sync::receive_common,
                    sync::member_left,
                    (sync::receive_host, sync::assign_net_ids)
                        .chain()
                        .run_if(is_authority),
                    (
                        sync::receive_guest,
                        sync::apply_snapshot,
                        sync::animate_replicas,
                    )
                        .chain()
                        .run_if(is_guest),
                )
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(playing()),
            )
            .add_systems(
                Update,
                (
                    (sync::forward_hits, sync::forward_requests).run_if(is_guest),
                    sync::forward_effects,
                )
                    .in_set(GameSet::Ui)
                    .run_if(playing()),
            )
            .add_systems(
                Update,
                (
                    avatars::sync_avatars,
                    avatars::update_avatars,
                    avatars::update_nametags,
                )
                    .chain()
                    .in_set(GameSet::Ui)
                    .run_if(playing()),
            )
            // Senden am Ende des Frames, wenn alles berechnet ist
            .add_systems(
                PostUpdate,
                (
                    sync::send_state,
                    (
                        sync::send_snapshot,
                        sync::announce_powerups,
                        sync::forward_world_events,
                    )
                        .run_if(is_authority),
                )
                    .run_if(playing()),
            );
    }
}

/// Koop-Partie läuft (Spielzustand, Partie-Ressource und Sitzung vorhanden).
fn playing_together(
    state: Option<Res<State<AppState>>>,
    game: Option<Res<InGame>>,
    session: Option<Res<Session>>,
) -> bool {
    state.is_some_and(|s| *s.get() == AppState::Playing) && game.is_some() && session.is_some()
}

/// Partie vorbei: Sitzung entfernen, Abstimmung wieder für Solo.
fn clear_session(mut commands: Commands, cfg: Res<crate::base::debug::DebugConfig>) {
    commands.remove_resource::<Session>();
    commands.insert_resource(crate::game::balance::Balance::for_players(
        cfg.players.unwrap_or(1),
    ));
}

/// Zurück im Hauptmenü: Partie und Lobby verlassen, Verbindung schließen.
fn leave_online(
    commands: Commands,
    cfg: Res<crate::base::debug::DebugConfig>,
    mut online: ResMut<Online>,
) {
    clear_session(commands, cfg);
    online.lobby = None;
    online.disconnect();
}
