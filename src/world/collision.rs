//! Einfache, schnelle Kollision gegen die statische Kartengeometrie.
//!
//! Die Karte besteht aus um die Hochachse gedrehten Boxen (OBBs). Spieler und Zombies
//! sind senkrechte Zylinder. Kugeln sind Strahlen.

use bevy::prelude::*;

use super::mapdata::MapFile;

pub const F_PLAYER: u8 = 1;
pub const F_ZOMBIE: u8 = 2;
pub const F_BULLET: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColKind {
    Solid,
    Window,
    Door,
}

#[derive(Clone, Debug)]
pub struct Obb {
    pub c: Vec3,
    pub h: Vec3,
    pub yaw: f32,
    cs: f32,
    sn: f32,
    pub flags: u8,
    pub kind: ColKind,
    pub door: Option<u16>,
    pub enabled: bool,
    pub min: Vec3,
    pub max: Vec3,
}

impl Obb {
    pub fn new(c: Vec3, h: Vec3, yaw: f32, flags: u8, kind: ColKind, door: Option<u16>) -> Self {
        let (sn, cs) = yaw.sin_cos();
        let ex = cs.abs() * h.x + sn.abs() * h.z;
        let ez = sn.abs() * h.x + cs.abs() * h.z;
        let ext = Vec3::new(ex, h.y, ez);
        Self {
            c,
            h,
            yaw,
            cs,
            sn,
            flags,
            kind,
            door,
            enabled: true,
            min: c - ext,
            max: c + ext,
        }
    }

    #[inline]
    pub fn to_local(&self, p: Vec3) -> Vec3 {
        let d = p - self.c;
        Vec3::new(
            d.x * self.cs - d.z * self.sn,
            d.y,
            d.x * self.sn + d.z * self.cs,
        )
    }

    #[inline]
    pub fn dir_to_local(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            v.x * self.cs - v.z * self.sn,
            v.y,
            v.x * self.sn + v.z * self.cs,
        )
    }

    #[inline]
    pub fn dir_to_world(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            v.x * self.cs + v.z * self.sn,
            v.y,
            -v.x * self.sn + v.z * self.cs,
        )
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.c.y + self.h.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.c.y - self.h.y
    }

    /// Schiebt einen Kreis (lokal XZ) aus dem Rechteck. Ergebnis in Weltkoordinaten (XZ).
    fn push_circle(&self, p: Vec3, r: f32) -> Option<Vec2> {
        let l = self.to_local(p);
        let (hx, hz) = (self.h.x, self.h.z);
        let cx = l.x.clamp(-hx, hx);
        let cz = l.z.clamp(-hz, hz);
        let d = Vec2::new(l.x - cx, l.z - cz);
        let d2 = d.length_squared();
        if d2 >= r * r {
            return None;
        }
        let local = if d2 > 1e-10 {
            let dist = d2.sqrt();
            d / dist * (r - dist)
        } else {
            let px = hx - l.x.abs();
            let pz = hz - l.z.abs();
            if px < pz {
                Vec2::new((px + r) * if l.x >= 0.0 { 1.0 } else { -1.0 }, 0.0)
            } else {
                Vec2::new(0.0, (pz + r) * if l.z >= 0.0 { 1.0 } else { -1.0 })
            }
        };
        let w = self.dir_to_world(Vec3::new(local.x, 0.0, local.y));
        Some(Vec2::new(w.x, w.z))
    }

    /// Überlappt der Kreis (XZ) das Rechteck?
    fn overlaps_circle(&self, p: Vec3, r: f32) -> bool {
        let l = self.to_local(p);
        let cx = l.x.clamp(-self.h.x, self.h.x);
        let cz = l.z.clamp(-self.h.z, self.h.z);
        let dx = l.x - cx;
        let dz = l.z - cz;
        dx * dx + dz * dz < r * r
    }

    /// Strahl gegen Box (Slab-Test). Liefert (t, Normale).
    pub fn ray(&self, o: Vec3, d: Vec3, max_t: f32) -> Option<(f32, Vec3)> {
        let lo = self.to_local(o);
        let ld = self.dir_to_local(d);
        let mut tmin = 0.0f32;
        let mut tmax = max_t;
        let mut n_local = -ld;
        let oa = [lo.x, lo.y, lo.z];
        let da = [ld.x, ld.y, ld.z];
        let ha = [self.h.x, self.h.y, self.h.z];
        for a in 0..3 {
            if da[a].abs() < 1e-9 {
                if oa[a].abs() > ha[a] {
                    return None;
                }
                continue;
            }
            let inv = 1.0 / da[a];
            let mut t1 = (-ha[a] - oa[a]) * inv;
            let mut t2 = (ha[a] - oa[a]) * inv;
            let mut s = -1.0;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
                s = 1.0;
            }
            if t1 > tmin {
                tmin = t1;
                let mut n = [0.0f32; 3];
                n[a] = s;
                n_local = Vec3::new(n[0], n[1], n[2]);
            }
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
        Some((tmin, self.dir_to_world(n_local).normalize_or_zero()))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub index: usize,
}

#[derive(Resource, Default)]
pub struct CollisionWorld {
    pub boxes: Vec<Obb>,
}

impl CollisionWorld {
    pub fn from_map(map: &MapFile) -> Self {
        let boxes = map
            .colliders
            .iter()
            .map(|c| {
                let kind = match c.kind.as_str() {
                    "window" => ColKind::Window,
                    "door" => ColKind::Door,
                    _ => ColKind::Solid,
                };
                Obb::new(
                    Vec3::from_array(c.c),
                    Vec3::from_array(c.h),
                    c.yaw,
                    c.f,
                    kind,
                    c.door,
                )
            })
            .collect();
        Self { boxes }
    }

    pub fn set_door_open(&mut self, door: u16) {
        for b in self.boxes.iter_mut() {
            if b.door == Some(door) {
                b.enabled = false;
            }
        }
    }

    /// Schiebt einen senkrechten Zylinder (Fußpunkt `pos`) aus allen Boxen heraus.
    /// Hindernisse, deren Oberkante unter `pos.y + step` liegt, gelten als Stufe.
    pub fn resolve_cylinder(
        &self,
        pos: &mut Vec3,
        radius: f32,
        height: f32,
        step: f32,
        mask: u8,
    ) -> bool {
        let mut hit = false;
        for _ in 0..3 {
            let mut moved = false;
            for b in &self.boxes {
                if !b.enabled || b.flags & mask == 0 {
                    continue;
                }
                if pos.x + radius < b.min.x
                    || pos.x - radius > b.max.x
                    || pos.z + radius < b.min.z
                    || pos.z - radius > b.max.z
                {
                    continue;
                }
                if b.bottom() >= pos.y + height || b.top() <= pos.y + step {
                    continue;
                }
                if let Some(push) = b.push_circle(*pos, radius) {
                    pos.x += push.x;
                    pos.z += push.y;
                    moved = true;
                    hit = true;
                }
            }
            if !moved {
                break;
            }
        }
        hit
    }

    /// Bodenhöhe unter einem Zylinder: höchste Box-Oberkante unter `pos.y + step` (mind. 0).
    pub fn ground_height(&self, pos: Vec3, radius: f32, step: f32, mask: u8) -> f32 {
        let mut g = 0.0f32;
        let r = radius * 0.7;
        for b in &self.boxes {
            if !b.enabled || b.flags & mask == 0 {
                continue;
            }
            let top = b.top();
            if top > pos.y + step || top <= g {
                continue;
            }
            if pos.x + r < b.min.x
                || pos.x - r > b.max.x
                || pos.z + r < b.min.z
                || pos.z - r > b.max.z
            {
                continue;
            }
            if b.overlaps_circle(pos, r) {
                g = top;
            }
        }
        g
    }

    /// Niedrigste Unterkante über dem Kopf (für Aufstehen / Springen).
    pub fn ceiling_height(&self, pos: Vec3, radius: f32, mask: u8) -> f32 {
        let mut c = f32::INFINITY;
        for b in &self.boxes {
            if !b.enabled || b.flags & mask == 0 {
                continue;
            }
            let bottom = b.bottom();
            if bottom < pos.y + 0.3 || bottom >= c {
                continue;
            }
            if pos.x + radius < b.min.x
                || pos.x - radius > b.max.x
                || pos.z + radius < b.min.z
                || pos.z - radius > b.max.z
            {
                continue;
            }
            if b.overlaps_circle(pos, radius) {
                c = bottom;
            }
        }
        c
    }

    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_t: f32, mask: u8) -> Option<RayHit> {
        let inv = Vec3::new(
            if dir.x.abs() > 1e-9 {
                1.0 / dir.x
            } else {
                f32::INFINITY
            },
            if dir.y.abs() > 1e-9 {
                1.0 / dir.y
            } else {
                f32::INFINITY
            },
            if dir.z.abs() > 1e-9 {
                1.0 / dir.z
            } else {
                f32::INFINITY
            },
        );
        let mut best: Option<RayHit> = None;
        let mut best_t = max_t;
        for (i, b) in self.boxes.iter().enumerate() {
            if !b.enabled || b.flags & mask == 0 {
                continue;
            }
            // Breitphase: Strahl gegen AABB
            let t1 = (b.min - origin) * inv;
            let t2 = (b.max - origin) * inv;
            let tmin = t1.min(t2);
            let tmax = t1.max(t2);
            let enter = tmin.x.max(tmin.y).max(tmin.z).max(0.0);
            let exit = tmax.x.min(tmax.y).min(tmax.z);
            if enter > exit || enter > best_t {
                continue;
            }
            if let Some((t, n)) = b.ray(origin, dir, best_t) {
                if t < best_t {
                    best_t = t;
                    best = Some(RayHit {
                        t,
                        point: origin + dir * t,
                        normal: n,
                        index: i,
                    });
                }
            }
        }
        best
    }

    /// Freie Sichtlinie zwischen zwei Punkten?
    pub fn line_clear(&self, a: Vec3, b: Vec3, mask: u8) -> bool {
        let d = b - a;
        let len = d.length();
        if len < 1e-4 {
            return true;
        }
        self.raycast(a, d / len, len, mask).is_none()
    }

    /// Überlappt ein Zylinder irgendeine Box (ohne Stufen)?
    pub fn cylinder_blocked(&self, pos: Vec3, radius: f32, height: f32, mask: u8) -> bool {
        for b in &self.boxes {
            if !b.enabled || b.flags & mask == 0 {
                continue;
            }
            if b.bottom() >= pos.y + height || b.top() <= pos.y + 0.35 {
                continue;
            }
            if pos.x + radius < b.min.x
                || pos.x - radius > b.max.x
                || pos.z + radius < b.min.z
                || pos.z - radius > b.max.z
            {
                continue;
            }
            if b.overlaps_circle(pos, radius) {
                return true;
            }
        }
        false
    }
}
