//! Trefferzonen: Strahltest gegen die animierten Knochen (Kopf, Hals, Torso, Gliedmaßen).

use bevy::prelude::*;

use super::{HitPart, Zombie, ZombieRig};
use crate::world::shapes::{ray_capsule, ray_sphere};

/// Ein Treffer eines Strahls auf einen Zombie.
#[derive(Clone, Copy, Debug)]
pub struct RayZombieHit {
    pub zombie: Entity,
    pub t: f32,
    pub part: HitPart,
}

/// Testet einen Strahl gegen alle lebenden Zombies (Knochen-Hitboxen).
/// Liefert alle Treffer nach Entfernung sortiert.
pub fn raycast_zombies(
    origin: Vec3,
    dir: Vec3,
    max_t: f32,
    zombies: &Query<(Entity, &Zombie, &Transform, Option<&ZombieRig>)>,
    bones: &Query<&GlobalTransform>,
) -> Vec<RayZombieHit> {
    let mut hits = Vec::new();
    for (e, z, tf, rig) in zombies.iter() {
        if !z.alive() {
            continue;
        }
        let base = tf.translation;
        // Grobtest: Kugel um den Zombie
        if ray_sphere(origin, dir, base + Vec3::Y * 0.9, 1.4).is_none_or(|t| t > max_t) {
            continue;
        }
        let Some(rig) = rig else {
            // Modell noch nicht bereit: einfache Kapsel
            if let Some(t) =
                ray_capsule(origin, dir, base + Vec3::Y * 0.3, base + Vec3::Y * 1.5, 0.3)
            {
                if t <= max_t {
                    hits.push(RayZombieHit {
                        zombie: e,
                        t,
                        part: HitPart::Torso,
                    });
                }
            }
            continue;
        };
        let p = |ent: Entity| bones.get(ent).map(|g| g.translation()).unwrap_or(base);
        let head = p(rig.head);
        let neck = p(rig.neck);
        let chest = p(rig.chest);
        let hips = p(rig.hips);
        let up = (head - neck).normalize_or(Vec3::Y);
        let mut best: Option<(f32, HitPart)> = None;
        let mut test = |t: Option<f32>, part: HitPart| {
            if let Some(t) = t {
                if t <= max_t && best.is_none_or(|(bt, _)| t < bt) {
                    best = Some((t, part));
                }
            }
        };
        if !z.headless {
            test(
                ray_sphere(origin, dir, head + up * 0.11, 0.135),
                HitPart::Head,
            );
        }
        test(
            ray_capsule(origin, dir, neck - up * 0.02, head + up * 0.02, 0.075),
            HitPart::Neck,
        );
        test(
            ray_capsule(origin, dir, hips, chest + (chest - hips) * 0.35, 0.2),
            HitPart::Torso,
        );
        for s in 0..2 {
            test(
                ray_capsule(origin, dir, p(rig.upper_arm[s]), p(rig.lower_arm[s]), 0.075),
                HitPart::Limb,
            );
            test(
                ray_capsule(origin, dir, p(rig.lower_arm[s]), p(rig.hand[s]), 0.065),
                HitPart::Limb,
            );
            test(
                ray_capsule(origin, dir, p(rig.upper_leg[s]), p(rig.lower_leg[s]), 0.1),
                HitPart::Limb,
            );
            test(
                ray_capsule(origin, dir, p(rig.lower_leg[s]), p(rig.foot[s]), 0.075),
                HitPart::Limb,
            );
        }
        if let Some((t, part)) = best {
            hits.push(RayZombieHit { zombie: e, t, part });
        }
    }
    hits.sort_by(|a, b| a.t.total_cmp(&b.t));
    hits
}
