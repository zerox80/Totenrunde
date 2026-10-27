//! Animationen der Zombies: passenden Clip je Zustand wählen, weich überblenden und
//! die Abspielgeschwindigkeit an die Laufgeschwindigkeit anpassen (kein Rutschen).

use std::time::Duration;

use bevy::prelude::*;

use super::{SpeedClass, ZState, Zombie, ZombieRig};
use crate::base::assets::ZombieAnims;

#[derive(Component, Default)]
pub struct ZombieAnimState {
    pub current: Option<AnimationNodeIndex>,
    pub serial: u32,
    pub applied_serial: u32,
}

pub fn animate_zombies(
    anims: Res<ZombieAnims>,
    mut zombies: Query<(&Zombie, &ZombieRig, &mut ZombieAnimState)>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (z, rig, mut st) in zombies.iter_mut() {
        let Ok((mut ap, mut tr)) = players.get_mut(rig.anim_player) else {
            continue;
        };
        let Some(anims) = anims.models.get(z.model) else {
            continue;
        };
        let moving = z.vel.length();
        let (want, speed, looping, restart) = match z.state {
            ZState::Rising { .. } => (anims.rise, 1.0, false, false),
            ZState::ToWindow { .. } | ZState::Chasing => {
                if moving < 0.15 {
                    (anims.idle, 1.0, true, false)
                } else {
                    let node = match z.class {
                        SpeedClass::Walk => anims.walk,
                        SpeedClass::Run => anims.run,
                        SpeedClass::Sprint => anims.sprint,
                    };
                    (
                        node,
                        (moving / z.class.native_speed()).clamp(0.4, 1.8),
                        true,
                        false,
                    )
                }
            }
            ZState::AtWindow { tearing, .. } => {
                if tearing > 0.0 {
                    (anims.tear, 1.0, false, tearing < 0.05)
                } else {
                    (anims.idle, 1.0, true, false)
                }
            }
            ZState::Climbing { .. } => (anims.climb, 1.0, false, false),
            ZState::Attacking { t, .. } => (anims.attack, 1.0, false, t < 0.02),
            ZState::Dying { .. } => (
                if z.death_back {
                    anims.death_back
                } else {
                    anims.death_fwd
                },
                1.0,
                false,
                false,
            ),
        };
        if restart {
            st.serial = st.serial.wrapping_add(1);
        }
        if st.current != Some(want) || st.serial != st.applied_serial {
            st.applied_serial = st.serial;
            st.current = Some(want);
            let blend = if matches!(z.state, ZState::Dying { .. }) {
                Duration::from_millis(120)
            } else {
                Duration::from_millis(220)
            };
            let a = tr.play(&mut ap, want, blend);
            if looping {
                a.repeat();
            }
            a.set_speed(speed);
        } else if looping {
            if let Some(a) = ap.animation_mut(want) {
                a.set_speed(speed);
            }
        }
    }
}
