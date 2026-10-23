//! Geräusche und Musik. Alle Dateien erzeugt `cargo run --bin sfxgen` (Synthese + espeak-ng).

pub mod music;
pub mod playback;
pub mod sfx;

use bevy::audio::{DefaultSpatialScale, SpatialScale};
use bevy::prelude::*;

pub use music::{spawn_music, MusicTrack};
pub use playback::{SfxEvent, SfxLib};
pub use sfx::Sfx;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SfxEvent>()
            .insert_resource(DefaultSpatialScale(SpatialScale::new(0.16)))
            .add_systems(Startup, playback::load_sounds)
            .add_systems(Update, (playback::play_sfx, playback::apply_volume, music::music_fade))
            .add_systems(OnEnter(crate::base::AppState::Playing), start_ambience);
    }
}

/// Wind als leise Dauerschleife im Spiel.
fn start_ambience(mut commands: Commands, lib: Option<Res<SfxLib>>) {
    let Some(lib) = lib else { return };
    commands.spawn((
        AudioPlayer::new(lib.ambience.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.22)),
        DespawnOnExit(crate::base::AppState::Playing),
    ));
}
