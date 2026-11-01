//! Multiplayer-Menü: mit dem Server verbinden, Konto verknüpfen (über die Webseite), Lobby
//! erstellen oder beitreten, bereit melden und als Host die Partie starten.
//!
//! Der Inhalt wird neu aufgebaut, sobald sich etwas Sichtbares ändert (Verbindung, Konto,
//! Lobby, Liste, eingetippter Code).

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};
use totenrunde_mp_client::protocol::{ClientMsg, LOBBY_CODE_LEN, MAX_PLAYERS, normalize_code};

use super::widgets::{BLOOD, HOVER, IDLE, PRESS, root, text};
use crate::base::AppState;
use crate::base::assets::GameAssets;
use crate::base::i18n::{t, tf};
use crate::base::settings::Settings;
use crate::net::connection::{account_url, server_label};
use crate::net::{Online, Status, slot_color};

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CodeField>()
            .add_systems(OnEnter(AppState::Multiplayer), spawn)
            .add_systems(
                Update,
                (typing, keys, refresh_list, rebuild, visuals, auto_play)
                    .chain()
                    .run_if(in_state(AppState::Multiplayer)),
            );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            adopt_website_login.run_if(in_state(AppState::Multiplayer)),
        );
    }
}

/// Browser: Wer sich im selben Browser auf der Konto-Webseite anmeldet, ist sofort auch im
/// Spiel angemeldet (gleiche Herkunft, das Token liegt im localStorage).
#[cfg(target_arch = "wasm32")]
fn adopt_website_login(mut online: ResMut<Online>, time: Res<Time<Real>>, mut next: Local<f32>) {
    if online.status != Status::Online || online.account.is_some() {
        return;
    }
    let now = time.elapsed_secs();
    if now < *next {
        return;
    }
    *next = now + 1.0;
    if let Some(token) = crate::net::connection::website_token()
        && online.tried_token() != Some(token.as_str())
    {
        online.reconnect();
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
enum MpAction {
    SignIn,
    ReopenPage,
    Retry,
    SignOut,
    Create { public: bool },
    JoinTyped,
    Join(String),
    Refresh,
    Leave,
    Ready(bool),
    Start,
    Back,
}

/// Knopf, der gerade nicht benutzt werden kann
#[derive(Component)]
struct Disabled;

#[derive(Component)]
struct Content;

/// Eingetippter Lobby-Code
#[derive(Resource, Default)]
struct CodeField(String);

const WIDE: f32 = 420.0;
const NARROW: f32 = 205.0;
const DIM: Color = Color::srgba(1.0, 1.0, 1.0, 0.55);
const LIGHT: Color = Color::srgb(0.92, 0.9, 0.86);

fn spawn(
    mut commands: Commands,
    assets: Res<GameAssets>,
    lib: Option<Res<crate::audio::SfxLib>>,
    mut online: ResMut<Online>,
    mut code: ResMut<CodeField>,
) {
    online.connect();
    code.0.clear();
    commands.spawn((Camera2d, DespawnOnExit(AppState::Multiplayer)));
    if let Some(lib) = lib {
        commands.spawn((
            AudioPlayer::new(lib.music_menu.clone()),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.4)),
            DespawnOnExit(AppState::Multiplayer),
        ));
    }
    commands
        .spawn((
            root(Color::srgb(0.025, 0.02, 0.02)),
            DespawnOnExit(AppState::Multiplayer),
        ))
        .with_children(|p| {
            p.spawn(text(t("mp.title"), &assets.font_black, 72.0, BLOOD));
            p.spawn((
                Content,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(px(18)),
                    ..default()
                },
            ));
        });
}

fn mp_button(
    p: &mut ChildSpawnerCommands,
    label: impl Into<String>,
    action: MpAction,
    font: &Handle<Font>,
    width: f32,
    enabled: bool,
) {
    let mut b = p.spawn((
        Button,
        Hovered::default(),
        action,
        Node {
            width: px(width),
            height: px(50),
            margin: UiRect::all(px(5)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::left(px(4)),
            ..default()
        },
        BorderColor::all(if enabled { BLOOD } else { DIM }),
        BackgroundColor(IDLE),
    ));
    b.with_children(|b| {
        b.spawn(text(label, font, 21.0, if enabled { LIGHT } else { DIM }));
    });
    if enabled {
        b.observe(on_activate);
    } else {
        b.insert(Disabled);
    }
}

fn row(p: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(f);
}

fn line(
    p: &mut ChildSpawnerCommands,
    s: impl Into<String>,
    font: &Handle<Font>,
    size: f32,
    color: Color,
) {
    p.spawn((
        text(s, font, size, color),
        TextLayout::justify(Justify::Center),
        Node {
            max_width: px(760),
            margin: UiRect::vertical(px(5)),
            ..default()
        },
    ));
}

/// Alles, was der Inhalt zeigt; ändert sich das, wird neu aufgebaut.
fn signature(online: &Online, code: &CodeField) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{}",
        online.status,
        online.account,
        online.link_code,
        online.lobby,
        online.lobbies,
        online.notice,
        code.0
    )
}

#[allow(clippy::too_many_arguments)]
fn rebuild(
    mut commands: Commands,
    online: Res<Online>,
    code: Res<CodeField>,
    settings: Res<Settings>,
    assets: Res<GameAssets>,
    content: Single<Entity, With<Content>>,
    mut shown: Local<String>,
) {
    let sig = signature(&online, &code);
    if *shown == sig {
        return;
    }
    *shown = sig;
    let bold = &assets.font_bold;
    let regular = &assets.font_regular;
    let black = &assets.font_black;
    commands.entity(*content).despawn_children();
    commands.entity(*content).with_children(|p| {
        if let Some(notice) = online.notice {
            line(p, t(notice), bold, 20.0, Color::srgb(1.0, 0.45, 0.35));
        }
        match &online.status {
            Status::Offline | Status::Connecting => {
                line(
                    p,
                    tf("mp.connecting", &[("server", &server_label(&settings))]),
                    regular,
                    22.0,
                    LIGHT,
                );
                mp_button(p, t("settings.back"), MpAction::Back, bold, WIDE, true);
            }
            Status::Failed(reason) => {
                line(
                    p,
                    tf("mp.unreachable", &[("server", &server_label(&settings))]),
                    bold,
                    22.0,
                    LIGHT,
                );
                line(p, reason.clone(), regular, 15.0, DIM);
                #[cfg(not(target_arch = "wasm32"))]
                line(p, t("mp.server_hint"), regular, 15.0, DIM);
                mp_button(p, t("mp.retry"), MpAction::Retry, bold, WIDE, true);
                mp_button(p, t("settings.back"), MpAction::Back, bold, WIDE, true);
            }
            Status::Outdated => {
                line(p, t("mp.outdated"), bold, 22.0, LIGHT);
                mp_button(p, t("settings.back"), MpAction::Back, bold, WIDE, true);
            }
            Status::Online => match (&online.account, &online.lobby) {
                (None, _) => {
                    line(p, t("mp.need_account"), regular, 22.0, LIGHT);
                    if let Some(link) = &online.link_code {
                        let shown = format!(
                            "{}-{}",
                            &link[..4.min(link.len())],
                            &link[4.min(link.len())..]
                        );
                        p.spawn((
                            text(shown, black, 54.0, Color::WHITE),
                            Node {
                                margin: UiRect::vertical(px(10)),
                                ..default()
                            },
                        ));
                        line(p, t("mp.link_waiting"), regular, 18.0, DIM);
                        mp_button(p, t("mp.reopen"), MpAction::ReopenPage, bold, WIDE, true);
                    } else {
                        mp_button(p, t("mp.sign_in"), MpAction::SignIn, bold, WIDE, true);
                    }
                    mp_button(p, t("settings.back"), MpAction::Back, bold, WIDE, true);
                }
                (Some(name), None) => {
                    line(
                        p,
                        tf("mp.signed_in_as", &[("name", name)]),
                        regular,
                        20.0,
                        DIM,
                    );
                    row(p, |r| {
                        mp_button(
                            r,
                            t("mp.create"),
                            MpAction::Create { public: true },
                            bold,
                            NARROW,
                            true,
                        );
                        mp_button(
                            r,
                            t("mp.create_private"),
                            MpAction::Create { public: false },
                            bold,
                            NARROW,
                            true,
                        );
                    });
                    line(p, t("mp.join_code"), bold, 17.0, DIM);
                    row(p, |r| {
                        let mut shown: String = code.0.clone();
                        while shown.chars().count() < LOBBY_CODE_LEN {
                            shown.push('_');
                        }
                        r.spawn((
                            Node {
                                width: px(NARROW),
                                height: px(50),
                                margin: UiRect::all(px(5)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(2)),
                                ..default()
                            },
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                        ))
                        .with_children(|b| {
                            b.spawn(text(shown, black, 28.0, Color::WHITE));
                        });
                        mp_button(
                            r,
                            t("mp.join"),
                            MpAction::JoinTyped,
                            bold,
                            NARROW,
                            code.0.len() == LOBBY_CODE_LEN,
                        );
                    });
                    line(p, t("mp.open_lobbies"), bold, 17.0, DIM);
                    if online.lobbies.is_empty() {
                        line(p, t("mp.no_lobbies"), regular, 17.0, DIM);
                    }
                    for l in online.lobbies.iter().take(5) {
                        row(p, |r| {
                            r.spawn((
                                text(
                                    format!("{}   {}/{}", l.host, l.players, MAX_PLAYERS),
                                    bold,
                                    20.0,
                                    LIGHT,
                                ),
                                Node {
                                    width: px(WIDE - 10.0),
                                    ..default()
                                },
                            ));
                            if l.playing {
                                mp_button(
                                    r,
                                    t("mp.running"),
                                    MpAction::Refresh,
                                    bold,
                                    NARROW,
                                    false,
                                );
                            } else {
                                mp_button(
                                    r,
                                    t("mp.join"),
                                    MpAction::Join(l.code.clone()),
                                    bold,
                                    NARROW,
                                    true,
                                );
                            }
                        });
                    }
                    row(p, |r| {
                        mp_button(r, t("mp.sign_out"), MpAction::SignOut, bold, NARROW, true);
                        mp_button(r, t("settings.back"), MpAction::Back, bold, NARROW, true);
                    });
                }
                (Some(_), Some(lobby)) => {
                    p.spawn(text(
                        tf("mp.lobby_code", &[("code", &lobby.code)]),
                        black,
                        38.0,
                        Color::WHITE,
                    ));
                    line(
                        p,
                        t(if lobby.public {
                            "mp.lobby_public"
                        } else {
                            "mp.lobby_private"
                        }),
                        regular,
                        17.0,
                        DIM,
                    );
                    p.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::vertical(px(12)),
                        row_gap: px(6),
                        ..default()
                    })
                    .with_children(|list| {
                        for slot in 0..MAX_PLAYERS as u8 {
                            let member = lobby.member(slot);
                            list.spawn((
                                Node {
                                    width: px(WIDE + 60.0),
                                    height: px(44),
                                    padding: UiRect::horizontal(px(14)),
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    border: UiRect::left(px(5)),
                                    ..default()
                                },
                                BorderColor::all(if member.is_some() {
                                    slot_color(slot)
                                } else {
                                    Color::srgba(1.0, 1.0, 1.0, 0.1)
                                }),
                                BackgroundColor(Color::srgba(0.08, 0.06, 0.06, 0.85)),
                            ))
                            .with_children(|r| match member {
                                Some(m) => {
                                    let you = if m.slot == lobby.you { t("mp.you") } else { "" };
                                    r.spawn(text(
                                        format!("{}  {you}", m.name),
                                        bold,
                                        21.0,
                                        slot_color(slot),
                                    ));
                                    let (state, color) = if m.slot == lobby.host {
                                        (t("mp.host"), Color::srgb(1.0, 0.8, 0.3))
                                    } else if m.ready {
                                        (t("mp.ready"), Color::srgb(0.45, 0.95, 0.45))
                                    } else {
                                        (t("mp.waiting"), DIM)
                                    };
                                    r.spawn(text(state, bold, 17.0, color));
                                }
                                None => {
                                    r.spawn(text(t("mp.free_slot"), regular, 18.0, DIM));
                                }
                            });
                        }
                    });
                    if lobby.playing {
                        line(p, t("mp.match_running"), regular, 18.0, DIM);
                    } else if lobby.is_host() {
                        let ready = lobby.all_ready();
                        mp_button(p, t("mp.start"), MpAction::Start, bold, WIDE, ready);
                        if !ready {
                            line(p, t("mp.wait_ready"), regular, 16.0, DIM);
                        }
                    } else {
                        let me = lobby.member(lobby.you).is_some_and(|m| m.ready);
                        mp_button(
                            p,
                            t(if me { "mp.not_ready" } else { "mp.set_ready" }),
                            MpAction::Ready(!me),
                            bold,
                            WIDE,
                            true,
                        );
                    }
                    mp_button(p, t("mp.leave"), MpAction::Leave, bold, WIDE, true);
                }
            },
        }
    });
}

fn on_activate(
    ev: On<Activate>,
    actions: Query<&MpAction>,
    mut online: ResMut<Online>,
    mut code: ResMut<CodeField>,
    settings: Res<Settings>,
    mut next: ResMut<NextState<AppState>>,
) {
    if let Ok(action) = actions.get(ev.entity) {
        perform(action.clone(), &mut online, &mut code, &settings, &mut next);
    }
}

fn perform(
    action: MpAction,
    online: &mut Online,
    code: &mut CodeField,
    settings: &Settings,
    next: &mut NextState<AppState>,
) {
    online.notice = None;
    match action {
        MpAction::SignIn => online.send(ClientMsg::LinkStart),
        MpAction::ReopenPage => {
            crate::base::platform::open_url(&account_url(settings, online.link_code.as_deref()));
        }
        MpAction::Retry => online.connect(),
        MpAction::SignOut => online.send(ClientMsg::Logout),
        MpAction::Create { public } => online.send(ClientMsg::CreateLobby { public }),
        MpAction::JoinTyped => {
            if code.0.len() == LOBBY_CODE_LEN {
                online.send(ClientMsg::JoinLobby {
                    code: code.0.clone(),
                });
                code.0.clear();
            }
        }
        MpAction::Join(c) => online.send(ClientMsg::JoinLobby { code: c }),
        MpAction::Refresh => online.send(ClientMsg::ListLobbies),
        MpAction::Leave => {
            online.send(ClientMsg::LeaveLobby);
            online.send(ClientMsg::ListLobbies);
        }
        MpAction::Ready(ready) => online.send(ClientMsg::SetReady { ready }),
        MpAction::Start => online.send(ClientMsg::StartGame),
        MpAction::Back => next.set(AppState::Menu),
    }
}

type MpButton<'a> = (
    &'a Hovered,
    Has<Pressed>,
    Has<Disabled>,
    &'a mut BackgroundColor,
);

fn visuals(mut q: Query<MpButton, With<MpAction>>) {
    for (hovered, pressed, disabled, mut bg) in q.iter_mut() {
        let c = if disabled {
            IDLE
        } else if pressed {
            PRESS
        } else if hovered.0 {
            HOVER
        } else {
            IDLE
        };
        if bg.0 != c {
            bg.0 = c;
        }
    }
}

/// Lobby-Code tippen (nur in der Lobby-Auswahl).
fn typing(
    mut keys: MessageReader<KeyboardInput>,
    mut code: ResMut<CodeField>,
    mut online: ResMut<Online>,
    settings: Res<Settings>,
    mut next: ResMut<NextState<AppState>>,
) {
    let browsing = online.signed_in() && online.lobby.is_none();
    for k in keys.read() {
        if !browsing || k.state != ButtonState::Pressed {
            continue;
        }
        match &k.logical_key {
            Key::Backspace => {
                code.0.pop();
            }
            Key::Enter => {
                perform(
                    MpAction::JoinTyped,
                    &mut online,
                    &mut code,
                    &settings,
                    &mut next,
                );
            }
            _ => {
                if let Some(text) = &k.text {
                    let add = normalize_code(text);
                    for c in add.chars() {
                        if code.0.len() < LOBBY_CODE_LEN {
                            code.0.push(c);
                        }
                    }
                }
            }
        }
    }
}

fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut online: ResMut<Online>,
    mut next: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if online.lobby.is_some() {
            online.send(ClientMsg::LeaveLobby);
        } else {
            next.set(AppState::Menu);
        }
    }
}

/// Liste offener Lobbys alle paar Sekunden auffrischen.
fn refresh_list(time: Res<Time<Real>>, mut online: ResMut<Online>, mut next_at: Local<f32>) {
    if !online.signed_in() || online.lobby.is_some() {
        return;
    }
    let now = time.elapsed_secs();
    if now >= *next_at {
        *next_at = now + 3.0;
        online.send(ClientMsg::ListLobbies);
    }
}

/// Automatische Tests (`TR_MP=host` oder `TR_MP=join`, `TR_MP_PLAYERS=2`): Lobby anlegen
/// bzw. der ersten beitreten und starten, sobald alle da und bereit sind.
fn auto_play(mut online: ResMut<Online>, mut cooldown: Local<f32>, time: Res<Time<Real>>) {
    let Ok(mode) = std::env::var("TR_MP") else {
        return;
    };
    if time.elapsed_secs() < *cooldown || !online.signed_in() {
        return;
    }
    *cooldown = time.elapsed_secs() + 0.5;
    let players: usize = std::env::var("TR_MP_PLAYERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    match (mode.as_str(), online.lobby.clone()) {
        ("host", None) => online.send(ClientMsg::CreateLobby { public: true }),
        ("host", Some(l))
            if l.is_host() && !l.playing && l.members.len() >= players && l.all_ready() =>
        {
            online.send(ClientMsg::StartGame);
            *cooldown = time.elapsed_secs() + 3.0;
        }
        ("join", None) => {
            if let Some(l) = online.lobbies.iter().find(|l| !l.playing).cloned() {
                online.send(ClientMsg::JoinLobby { code: l.code });
            }
        }
        ("join", Some(l)) if !l.playing && l.member(l.you).is_some_and(|m| !m.ready) => {
            online.send(ClientMsg::SetReady { ready: true });
        }
        _ => {}
    }
}
