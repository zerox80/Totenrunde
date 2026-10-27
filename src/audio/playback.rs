//! Abspielen von Geräuschen (2D für die Oberfläche, 3D mit Raumklang in der Welt).

use std::collections::HashMap;

use bevy::audio::Volume;
use bevy::prelude::*;
use rand::Rng;

use super::sfx::{ALL, Sfx};
use crate::base::settings::Settings;

#[derive(Message, Clone, Copy)]
pub struct SfxEvent {
    pub sfx: Sfx,
    pub pos: Option<Vec3>,
    pub volume: f32,
    pub pitch: f32,
}

impl SfxEvent {
    pub fn ui(sfx: Sfx) -> Self {
        Self {
            sfx,
            pos: None,
            volume: 1.0,
            pitch: 1.0,
        }
    }
    pub fn at(sfx: Sfx, pos: Vec3) -> Self {
        Self {
            sfx,
            pos: Some(pos),
            volume: 1.0,
            pitch: 1.0,
        }
    }
    pub fn vol(mut self, v: f32) -> Self {
        self.volume = v;
        self
    }
    pub fn pitch(mut self, p: f32) -> Self {
        self.pitch = p;
        self
    }
}

#[derive(Resource)]
pub struct SfxLib {
    map: HashMap<Sfx, Vec<Handle<AudioSource>>>,
    pub music_menu: Handle<AudioSource>,
    pub ambience: Handle<AudioSource>,
    pub fire_loop: Handle<AudioSource>,
}

pub fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Nur vorhandene Dateien laden (die Sounds werden erst noch erzeugt) – sonst
    // füllt jede fehlende Datei das Log mit Fehlern.
    let load = |rel: String| -> Option<Handle<AudioSource>> {
        crate::world::mapdata::asset_exists(&rel).then(|| asset_server.load(rel))
    };
    let mut map = HashMap::new();
    for s in ALL {
        let handles = s
            .files()
            .iter()
            .filter_map(|f| load(format!("sounds/{f}.wav")))
            .collect();
        map.insert(*s, handles);
    }
    commands.insert_resource(SfxLib {
        map,
        music_menu: load("sounds/music_menu.wav".into()).unwrap_or_default(),
        ambience: load("sounds/amb_wind.wav".into()).unwrap_or_default(),
        fire_loop: load("sounds/fire_loop.wav".into()).unwrap_or_default(),
    });
}

pub fn play_sfx(
    mut commands: Commands,
    lib: Option<Res<SfxLib>>,
    mut events: MessageReader<SfxEvent>,
) {
    let Some(lib) = lib else {
        events.clear();
        return;
    };
    let mut rng = rand::rng();
    for ev in events.read() {
        let Some(list) = lib.map.get(&ev.sfx) else {
            continue;
        };
        if list.is_empty() {
            continue;
        }
        let h = list[rng.random_range(0..list.len())].clone();
        let j = ev.sfx.pitch_jitter();
        let pitch = ev.pitch
            * if j > 0.0 {
                rng.random_range(1.0 - j..1.0 + j)
            } else {
                1.0
            };
        let settings = PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(ev.sfx.base_volume() * ev.volume))
            .with_speed(pitch);
        match ev.pos {
            Some(p) => {
                commands.spawn((
                    AudioPlayer::new(h),
                    settings.with_spatial(true),
                    Transform::from_translation(p),
                ));
            }
            None => {
                commands.spawn((AudioPlayer::new(h), settings));
            }
        }
    }
}

/// Hintergrund-Musik/-Atmosphäre mit Ein- und Ausblenden.

pub fn apply_volume(settings: Res<Settings>, mut global: ResMut<GlobalVolume>) {
    if settings.is_changed() {
        global.volume = Volume::Linear(settings.master_volume);
    }
}
