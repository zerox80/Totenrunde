//! Hintergrundmusik und Atmosphäre mit weichem Ein- und Ausblenden.

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::base::settings::Settings;

#[derive(Component)]
pub struct MusicTrack {
    pub target: f32,
    pub current: f32,
    pub fade_out_despawn: bool,
}

pub fn spawn_music(commands: &mut Commands, handle: Handle<AudioSource>, volume: f32) -> Entity {
    commands
        .spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
            MusicTrack {
                target: volume,
                current: 0.0,
                fade_out_despawn: false,
            },
        ))
        .id()
}

pub fn music_fade(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut MusicTrack, Option<&mut AudioSink>)>,
) {
    let dt = time.delta_secs();
    for (e, mut m, sink) in q.iter_mut() {
        let goal = if m.fade_out_despawn { 0.0 } else { m.target };
        let step = dt * 0.6;
        if m.current < goal {
            m.current = (m.current + step).min(goal);
        } else {
            m.current = (m.current - step).max(goal);
        }
        if let Some(mut s) = sink {
            s.set_volume(Volume::Linear(m.current * settings.music_volume));
        }
        if m.fade_out_despawn && m.current <= 0.001 {
            commands.entity(e).despawn();
        }
    }
}
