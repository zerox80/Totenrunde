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
//! * `ui`      – HUD und Menüs
//!
//! Alle 3D-Modelle, Texturen und HUD-Grafiken erzeugen die Blender-Skripte in `blender/`,
//! alle Geräusche der Synthesizer `src/bin/sfxgen`.

mod audio;
mod base;
mod fx;
mod game;
mod player;
mod ui;
mod weapons;
mod world;
mod zombies;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Totenrunde".into(),
                        resolution: WindowResolution::new(1600, 900),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_linear()),
        )
        .add_plugins((
            base::BasePlugin,
            audio::GameAudioPlugin,
            world::WorldPlugin,
            player::PlayerPlugin,
            weapons::WeaponsPlugin,
            zombies::ZombiePlugin,
            game::GamePlugin,
            fx::FxPlugin,
            ui::UiPlugin,
        ))
        .run();
}
