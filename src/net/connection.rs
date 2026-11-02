//! Verbindung zum Multiplayer-Server: Anmelden, Konto verknüpfen, Lobbys und das Weiterleiten
//! der Spielnachrichten. Die Menüs lesen und steuern alles über die Ressource [`Online`].
//!
//! Die Verbindung selbst (WebSocket, Anmeldung, Pings) kommt aus dem Multiplayer-SDK
//! (`totenrunde-mp-client`); hier wird sie an die Spielzustände und Menüs angebunden.

use bevy::prelude::*;
use totenrunde_mp_client::protocol::{
    ClientMsg, CloseReason, ErrorCode, LobbyInfo, LobbyView, ServerMsg, Target, VERSION, decode,
    encode,
};
use totenrunde_mp_client::{Connection, Event, ws_url};

use super::msg::GameMsg;
use super::session::{Session, Team};
use crate::base::AppState;
use crate::base::settings::Settings;
use crate::game::balance::Balance;

/// Erneuter Verbindungsversuch nach einem Fehler (nur solange das Menü offen ist)
const RETRY_AFTER: f64 = 4.0;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Offline,
    Connecting,
    Online,
    /// Verbindung fehlgeschlagen oder abgebrochen (Grund)
    Failed(String),
    /// Spiel und Server sprechen verschiedene Protokollversionen
    Outdated,
}

/// Zustand der Server-Verbindung, so wie ihn die Menüs brauchen.
#[derive(Resource, Default)]
pub struct Online {
    pub status: Status,
    pub account: Option<String>,
    /// Code zum Verknüpfen auf der Webseite
    pub link_code: Option<String>,
    pub lobbies: Vec<LobbyInfo>,
    pub lobby: Option<LobbyView>,
    /// Letzter Hinweis für das Menü (Übersetzungsschlüssel)
    pub notice: Option<&'static str>,
    /// Laufzeit zum Server und zurück (Millisekunden)
    pub ping_ms: Option<f32>,
    /// Verbindung gewünscht (Multiplayer-Menü offen oder Partie läuft)
    wanted: bool,
    /// Verbindung neu aufbauen (z. B. neues Token)
    restart: bool,
    /// Token, mit dem zuletzt angemeldet wurde
    tried_token: Option<String>,
    outbox: Vec<ClientMsg>,
}

impl Online {
    pub fn send(&mut self, msg: ClientMsg) {
        self.outbox.push(msg);
    }

    /// Verbindung aufbauen (oder nach einem Fehler sofort neu versuchen).
    pub fn connect(&mut self) {
        self.wanted = true;
        if matches!(self.status, Status::Failed(_)) {
            self.status = Status::Offline;
        }
    }

    pub fn disconnect(&mut self) {
        self.wanted = false;
    }

    /// Neu verbinden, damit ein frisch gespeichertes Token benutzt wird (Browser).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn reconnect(&mut self) {
        self.restart = true;
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn tried_token(&self) -> Option<&str> {
        self.tried_token.as_deref()
    }

    pub fn signed_in(&self) -> bool {
        self.status == Status::Online && self.account.is_some()
    }
}

/// Die eigentliche Verbindung (nicht threadsicher, daher getrennt von [`Online`]).
#[derive(Default)]
pub struct NetLink {
    conn: Option<Connection>,
    events: Vec<Event>,
    retry_at: f64,
}

/// Nachricht eines anderen Spielers der Partie.
#[derive(Message, Clone, Debug)]
pub struct Incoming {
    pub from: u8,
    pub msg: GameMsg,
}

/// Ein Mitglied hat die laufende Partie verlassen.
#[derive(Message, Clone, Copy, Debug)]
pub struct MemberLeft(pub u8);

/// Ausgehende Spielnachrichten dieses Frames.
#[derive(Resource, Default)]
pub struct GameOut {
    msgs: Vec<(Target, GameMsg)>,
}

impl GameOut {
    pub fn send(&mut self, to: Target, msg: GameMsg) {
        self.msgs.push((to, msg));
    }
}

/// Basisadresse des Servers (`https://…`). Im Browser der Server, der das Spiel ausliefert.
pub fn server_base(settings: &Settings) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = settings;
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base = std::env::var("TR_SERVER")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| (!settings.server.is_empty()).then(|| settings.server.clone()))
            // beim Bauen eingetragener Server (Release: Repository-Variable TR_SERVER)
            .or_else(|| {
                option_env!("TR_SERVER")
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
        base.trim_end_matches('/').to_owned()
    }
}

/// Anzeigename des Servers (ohne Schema).
pub fn server_label(settings: &Settings) -> String {
    let base = server_base(settings);
    base.split_once("://")
        .map(|(_, rest)| rest.to_owned())
        .unwrap_or(base)
}

/// Konto-Seite des Servers, optional mit Verknüpfungscode für dieses Gerät.
pub fn account_url(settings: &Settings, code: Option<&str>) -> String {
    totenrunde_mp_client::account_url(&server_base(settings), code)
}

/// Gespeichertes Sitzungstoken: `TR_TOKEN` (Tests), Einstellungen, im Browser auch die
/// Anmeldung der Konto-Webseite (gleiche Herkunft).
pub fn stored_token(settings: &Settings) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(token) = std::env::var("TR_TOKEN") {
        return Some(token);
    }
    if let Some(token) = settings.mp_token.clone().filter(|t| !t.is_empty()) {
        return Some(token);
    }
    website_token()
}

#[cfg(target_arch = "wasm32")]
pub const WEBSITE_TOKEN_KEY: &str = "totenrunde.session";

pub fn website_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(WEBSITE_TOKEN_KEY).ok().flatten())
            .filter(|t| t.len() == 64)
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

/// Abgelehntes Token vergessen (`None`: abgemeldet, alle vergessen).
fn forget_token(settings: &mut Settings, token: Option<&str>) {
    if settings.mp_token.is_some() && (token.is_none() || settings.mp_token.as_deref() == token) {
        settings.mp_token = None;
        settings.save();
    }
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let stored = storage.get_item(WEBSITE_TOKEN_KEY).ok().flatten();
        if token.is_none() || stored.as_deref() == token {
            let _ = storage.remove_item(WEBSITE_TOKEN_KEY);
        }
    }
}

fn error_notice(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::LobbyNotFound => "mp.err_not_found",
        ErrorCode::LobbyFull => "mp.err_full",
        ErrorCode::AlreadyPlaying => "mp.err_playing",
        ErrorCode::NotReady => "mp.err_not_ready",
        ErrorCode::NotSignedIn => "mp.err_signed_out",
        ErrorCode::TooManyRequests => "mp.err_rate",
        _ => "mp.err_generic",
    }
}

/// Partie beenden (Host weg, Verbindung verloren, verlassen) und ins Multiplayer-Menü.
pub fn end_session(
    commands: &mut Commands,
    state: &State<AppState>,
    next: &mut NextState<AppState>,
    players: Option<u32>,
) {
    commands.remove_resource::<Session>();
    commands.insert_resource(Balance::for_players(players.unwrap_or(1)));
    if matches!(state.get(), AppState::Playing | AppState::GameOver) {
        next.set(AppState::Multiplayer);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn poll(
    mut link: NonSendMut<NetLink>,
    mut online: ResMut<Online>,
    mut settings: ResMut<Settings>,
    real: Res<Time<Real>>,
    session: Option<Res<Session>>,
    state: Res<State<AppState>>,
    cfg: Res<crate::base::debug::DebugConfig>,
    mut next: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mut incoming: MessageWriter<Incoming>,
    mut left: MessageWriter<MemberLeft>,
) {
    let now = real.elapsed_secs_f64();
    let link = &mut *link;
    if std::mem::take(&mut online.restart) && session.is_none() && link.conn.take().is_some() {
        online.status = Status::Offline;
        online.lobby = None;
    }

    // Verbindung auf- oder abbauen
    if !online.wanted && session.is_none() {
        if link.conn.take().is_some() {
            online.status = Status::Offline;
            online.lobby = None;
            online.link_code = None;
        }
        online.outbox.clear();
        return;
    }
    if link.conn.is_none() {
        let retry = match online.status {
            Status::Offline => true,
            Status::Failed(_) => now >= link.retry_at,
            _ => false,
        };
        if retry {
            let url = ws_url(&server_base(&settings));
            info!("Verbinde mit {url}");
            let token = stored_token(&settings);
            online.tried_token = token.clone();
            link.conn = Some(Connection::connect(&url, token));
            online.status = Status::Connecting;
        }
    }
    let Some(conn) = link.conn.as_mut() else {
        return;
    };
    let mut events = std::mem::take(&mut link.events);
    conn.poll(now, &mut events);
    online.ping_ms = conn.rtt_ms();
    // Token, mit dem sich diese Verbindung angemeldet hat
    let hello_token = conn.token().map(str::to_owned);

    for event in events.drain(..) {
        let msg = match event {
            Event::Connected => continue,
            Event::Closed(reason) => {
                warn!("Server-Verbindung beendet: {reason}");
                link.conn = None;
                link.retry_at = now + RETRY_AFTER;
                if online.status != Status::Outdated {
                    online.status = Status::Failed(reason);
                }
                online.lobby = None;
                online.link_code = None;
                online.outbox.clear();
                if session.is_some() {
                    online.notice = Some("mp.connection_lost");
                    end_session(&mut commands, &state, &mut next, cfg.players);
                }
                break;
            }
            Event::Message(msg) => msg,
        };
        match msg {
            ServerMsg::Welcome { account } => {
                online.status = Status::Online;
                if account.is_none() && hello_token.is_some() {
                    // Sitzung abgelaufen oder gelöscht
                    forget_token(&mut settings, hello_token.as_deref());
                }
                online.account = account.map(|a| a.name);
                if online.account.is_some() {
                    online.link_code = None;
                    online.send(ClientMsg::ListLobbies);
                }
            }
            ServerMsg::Outdated { server } => {
                warn!("Server spricht Protokoll {server}, Spiel {VERSION}");
                online.status = Status::Outdated;
            }
            ServerMsg::LinkCode { code } => {
                crate::base::platform::open_url(&account_url(&settings, Some(&code)));
                online.link_code = Some(code);
            }
            ServerMsg::Linked { token, account } => {
                if std::env::var_os("TR_TOKEN").is_none() {
                    settings.mp_token = Some(token);
                    settings.save();
                }
                online.account = Some(account.name);
                online.link_code = None;
                online.send(ClientMsg::ListLobbies);
            }
            ServerMsg::SignedOut => {
                forget_token(&mut settings, None);
                online.account = None;
                online.lobby = None;
            }
            ServerMsg::Lobbies { list } => online.lobbies = list,
            ServerMsg::Lobby(view) => online.lobby = Some(view),
            ServerMsg::LobbyClosed { reason } => {
                online.lobby = None;
                if reason == CloseReason::HostLeft {
                    online.notice = Some("mp.host_left");
                }
                if session.is_some() {
                    end_session(&mut commands, &state, &mut next, cfg.players);
                }
                online.send(ClientMsg::ListLobbies);
            }
            ServerMsg::Error { code } => {
                warn!("Server meldet {code:?}");
                online.notice = Some(error_notice(code));
            }
            ServerMsg::Start { .. } => {
                let Some(view) = online.lobby.clone() else {
                    continue;
                };
                let session = Session::from_lobby(&view);
                info!(
                    "Partie beginnt: {} Spieler, wir sind Platz {} ({})",
                    session.names.len(),
                    session.you,
                    if session.is_host() { "Host" } else { "Gast" }
                );
                commands.insert_resource(Balance::for_players(session.names.len() as u32));
                commands.insert_resource(Team::for_session(&session));
                commands.insert_resource(session);
                online.notice = None;
                next.set(AppState::Playing);
            }
            ServerMsg::Relay { from, data } => match decode::<GameMsg>(&data) {
                Some(msg) => {
                    incoming.write(Incoming { from, msg });
                }
                None => warn!("Unverständliche Spielnachricht von Platz {from}"),
            },
            ServerMsg::PlayerLeft { slot } => {
                left.write(MemberLeft(slot));
            }
            // Pings beantwortet das SDK selbst
            ServerMsg::Pong { .. } => {}
        }
    }
    events.clear();
    link.events = events;
}

/// Gesammelte Nachrichten senden (am Ende des Frames).
pub fn flush(mut link: NonSendMut<NetLink>, mut online: ResMut<Online>, mut out: ResMut<GameOut>) {
    let Some(conn) = link.conn.as_mut() else {
        out.msgs.clear();
        return;
    };
    if online.status != Status::Online {
        out.msgs.clear();
        return;
    }
    for msg in online.outbox.drain(..) {
        conn.send(&msg);
    }
    for (to, msg) in out.msgs.drain(..) {
        conn.relay(to, encode(&msg));
    }
}
