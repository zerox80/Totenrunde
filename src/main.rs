//! Totenrunde – ein rundenbasierter Zombie-Überlebensmodus im Stil von Black Ops 2.
//!
//! Aufbau:
//! * `base`    – Zustände, Einstellungen, Assets, Debug-Hilfen
//! * `world`   – Karte, Kollision, Navigation, Licht, Strom, Zonen, Türen, Barrikaden
//! * `player`  – Ego-Kamera, Bewegung, Gesundheit
//! * `weapons` – Waffendaten, Inventar, Schießen, Projektile, Ego-Waffe
//! * `zombies` – KI, Animation, Trefferzonen, Schaden
//! * `game`    – Runden, Punkte, Perks, Power-ups, Kaufen, Kiste, Verstärker
//! * `fx`      – Partikel, Decals, Feuer
//! * `audio`   – Geräusche und Musik
//! * `net`     – Koop-Multiplayer über den Totenrunde-Server
//! * `ui`      – HUD und Menüs
//!
//! Alle 3D-Modelle, Texturen und HUD-Grafiken erzeugen die Blender-Skripte in `blender/`,
//! alle Geräusche der Synthesizer `src/bin/sfxgen`.

// Release unter Windows: kein Konsolenfenster beim Start per Doppelklick
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod base;
mod fx;
mod game;
mod net;
mod player;
mod ui;
mod weapons;
mod world;
mod zombies;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    let settings = base::settings::Settings::load();
    base::i18n::set(&settings.language);
    base::platform::install_crash_handler();
    if !base::platform::check_assets() {
        return;
    }
    let window = Window {
        title: "Totenrunde".into(),
        // Wayland-App-ID bzw. X11-WM_CLASS (passend zum Desktop-Eintrag)
        name: Some("totenrunde".into()),
        resolution: WindowResolution::new(1600, 900),
        present_mode: PresentMode::AutoVsync,
        ..default()
    };
    #[cfg(target_arch = "wasm32")]
    let window = Window {
        canvas: Some("#game-canvas".into()),
        fit_canvas_to_parent: true,
        prevent_default_event_handling: true,
        ..window
    };
    let mut app = App::new();
    // Im Browser lädt der AssetReader sonst immer die volle 2048er-Datei, obwohl die
    // Laufzeit sie für die gewählte Grafikstufe sofort herunterrechnet. Vorab erzeugte
    // Varianten (backend/optimize-textures.py) sparen Download, Dekodierung und Ruckler.
    #[cfg(target_arch = "wasm32")]
    if option_env!("TR_TEXTURE_VARIANTS").is_some() {
        use bevy::asset::io::{
            AssetSourceBuilder, AssetSourceBuilders, AssetSourceId, wasm::HttpWasmAssetReader,
        };
        let texture_max = settings.texture_max;
        let mut builders = AssetSourceBuilders::default();
        builders.insert(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                Box::new(
                    HttpWasmAssetReader::new("assets").with_request_mapper(move |path| {
                        match base::assets::texture_variant(path, texture_max) {
                            Some(variant) => std::borrow::Cow::Owned(variant),
                            None => std::borrow::Cow::Borrowed(path),
                        }
                    }),
                )
            }),
        );
        app.insert_resource(builders);
    }
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            })
            // Es gibt keine .meta-Dateien; ohne das fragt Bevy zu jedem Asset erst danach
            // (im Browser je eine 404-Anfrage, die vor dem eigentlichen Download abgewartet wird)
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin::default_linear()),
    );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(bevy::post_process::auto_exposure::AutoExposurePlugin);
    app.add_plugins((
        base::BasePlugin,
        audio::GameAudioPlugin,
        world::WorldPlugin,
        player::PlayerPlugin,
        weapons::WeaponsPlugin,
        zombies::ZombiePlugin,
        game::GamePlugin,
        fx::FxPlugin,
        net::NetPlugin,
        ui::UiPlugin,
    ))
    .run();
}
