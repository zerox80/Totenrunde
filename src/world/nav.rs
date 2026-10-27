//! Navigationsgitter und Flow-Field für die Zombies.
//!
//! Das begehbare Gebiet wird einmal aus den Kollisionsboxen gerastert. Ein Dijkstra-
//! Flow-Field (8 Nachbarn) vom Spieler aus wird regelmäßig neu berechnet; jeder Zombie
//! folgt dem fallenden Distanzwert.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use bevy::prelude::*;

use super::collision::{CollisionWorld, F_ZOMBIE};
use super::mapdata::MapFile;

pub const UNREACHABLE: u32 = u32::MAX;

#[derive(Resource)]
pub struct NavGrid {
    pub min: Vec2,
    pub cell: f32,
    pub w: usize,
    pub h: usize,
    /// begehbar ohne Türen
    static_ok: Vec<bool>,
    /// Tür, die die Zelle blockiert (-1 = keine)
    door_of: Vec<i16>,
    pub walk: Vec<bool>,
    pub dist: Vec<u32>,
    pub zone_of: Vec<i8>,
    pub goal_cell: Option<usize>,
    heap: BinaryHeap<Reverse<(u32, u32)>>,
}

impl NavGrid {
    pub fn build(map: &MapFile, col: &CollisionWorld) -> Self {
        let min = Vec2::from_array(map.nav.min);
        let max = Vec2::from_array(map.nav.max);
        let cell = map.nav.cell;
        let w = ((max.x - min.x) / cell).ceil() as usize;
        let h = ((max.y - min.y) / cell).ceil() as usize;
        let n = w * h;
        let mut static_ok = vec![false; n];
        let mut door_of = vec![-1i16; n];
        let mut zone_of = vec![-1i8; n];
        let radius = 0.3;
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let p = Vec3::new(
                    min.x + (x as f32 + 0.5) * cell,
                    0.0,
                    min.y + (y as f32 + 0.5) * cell,
                );
                let mut blocked = false;
                let mut door = -1i16;
                for b in &col.boxes {
                    if b.flags & F_ZOMBIE == 0 {
                        continue;
                    }
                    if b.bottom() >= 1.7 || b.top() <= 0.4 {
                        continue;
                    }
                    if p.x + radius < b.min.x
                        || p.x - radius > b.max.x
                        || p.z + radius < b.min.z
                        || p.z - radius > b.max.z
                    {
                        continue;
                    }
                    let l = b.to_local(p);
                    let cx = l.x.clamp(-b.h.x, b.h.x);
                    let cz = l.z.clamp(-b.h.z, b.h.z);
                    let d2 = (l.x - cx).powi(2) + (l.z - cz).powi(2);
                    if d2 < radius * radius {
                        if let Some(dr) = b.door {
                            door = dr as i16;
                        } else {
                            blocked = true;
                            break;
                        }
                    }
                }
                static_ok[i] = !blocked;
                if !blocked && door >= 0 {
                    door_of[i] = door;
                }
                if let Some(z) = map.zone_at(p) {
                    zone_of[i] = z as i8;
                }
            }
        }
        let walk: Vec<bool> = (0..n).map(|i| static_ok[i] && door_of[i] < 0).collect();
        Self {
            min,
            cell,
            w,
            h,
            static_ok,
            door_of,
            walk,
            dist: vec![UNREACHABLE; n],
            zone_of,
            goal_cell: None,
            heap: BinaryHeap::new(),
        }
    }

    pub fn open_door(&mut self, door: u16) {
        for i in 0..self.walk.len() {
            if self.door_of[i] == door as i16 {
                self.door_of[i] = -1;
                self.walk[i] = self.static_ok[i];
            }
        }
        self.goal_cell = None;
    }

    #[inline]
    pub fn cell_of(&self, p: Vec3) -> Option<(usize, usize)> {
        let fx = (p.x - self.min.x) / self.cell;
        let fy = (p.z - self.min.y) / self.cell;
        if fx < 0.0 || fy < 0.0 {
            return None;
        }
        let (x, y) = (fx as usize, fy as usize);
        if x >= self.w || y >= self.h {
            return None;
        }
        Some((x, y))
    }

    #[inline]
    pub fn index_of(&self, p: Vec3) -> Option<usize> {
        self.cell_of(p).map(|(x, y)| y * self.w + x)
    }

    #[inline]
    pub fn center(&self, i: usize) -> Vec3 {
        let x = i % self.w;
        let y = i / self.w;
        Vec3::new(
            self.min.x + (x as f32 + 0.5) * self.cell,
            0.0,
            self.min.y + (y as f32 + 0.5) * self.cell,
        )
    }

    pub fn walkable(&self, p: Vec3) -> bool {
        self.index_of(p).map(|i| self.walk[i]).unwrap_or(false)
    }

    /// Nächstgelegene begehbare Zelle (Spirale), falls der Punkt selbst blockiert ist.
    pub fn nearest_walkable(&self, p: Vec3) -> Option<usize> {
        let (cx, cy) = self.cell_of(p)?;
        for r in 0..6i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let x = cx as i32 + dx;
                    let y = cy as i32 + dy;
                    if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
                        continue;
                    }
                    let i = y as usize * self.w + x as usize;
                    if self.walk[i] {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Berechnet das Flow-Field zum Ziel (Spielerposition) neu.
    pub fn compute_flow(&mut self, goal: Vec3) {
        let Some(start) = self.nearest_walkable(goal) else {
            return;
        };
        self.goal_cell = Some(start);
        self.dist.fill(UNREACHABLE);
        self.heap.clear();
        self.dist[start] = 0;
        self.heap.push(Reverse((0, start as u32)));
        let w = self.w as i32;
        let h = self.h as i32;
        const N: [(i32, i32, u32); 8] = [
            (1, 0, 10),
            (-1, 0, 10),
            (0, 1, 10),
            (0, -1, 10),
            (1, 1, 14),
            (1, -1, 14),
            (-1, 1, 14),
            (-1, -1, 14),
        ];
        while let Some(Reverse((d, i))) = self.heap.pop() {
            let i = i as usize;
            if d > self.dist[i] {
                continue;
            }
            let x = (i % self.w) as i32;
            let y = (i / self.w) as i32;
            for &(dx, dy, cost) in &N {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let ni = (ny * w + nx) as usize;
                if !self.walk[ni] {
                    continue;
                }
                if dx != 0 && dy != 0 {
                    // keine Ecken schneiden
                    let a = (y * w + nx) as usize;
                    let b = (ny * w + x) as usize;
                    if !self.walk[a] || !self.walk[b] {
                        continue;
                    }
                }
                let nd = d + cost;
                if nd < self.dist[ni] {
                    self.dist[ni] = nd;
                    self.heap.push(Reverse((nd, ni as u32)));
                }
            }
        }
    }

    pub fn dist_at(&self, p: Vec3) -> u32 {
        self.index_of(p)
            .map(|i| self.dist[i])
            .unwrap_or(UNREACHABLE)
    }

    /// Bewegungsrichtung (XZ, normiert) entlang des Flow-Fields. Schaut zwei Zellen voraus,
    /// damit die Wege weicher werden.
    pub fn flow_dir(&self, p: Vec3) -> Option<Vec3> {
        let i = self.index_of(p)?;
        let i = if self.walk[i] && self.dist[i] != UNREACHABLE {
            i
        } else {
            // Zombie steht knapp in einer blockierten Zelle: beste Nachbarzelle suchen
            self.best_neighbor(i)?
        };
        let mut cur = i;
        let mut target = None;
        for _ in 0..3 {
            match self.best_neighbor(cur) {
                Some(n) if self.dist[n] < self.dist[cur] => {
                    target = Some(n);
                    cur = n;
                }
                _ => break,
            }
        }
        let t = target.unwrap_or(cur);
        let d = self.center(t) - Vec3::new(p.x, 0.0, p.z);
        let d = Vec3::new(d.x, 0.0, d.z);
        if d.length_squared() < 1e-6 {
            return None;
        }
        Some(d.normalize())
    }

    fn best_neighbor(&self, i: usize) -> Option<usize> {
        let x = (i % self.w) as i32;
        let y = (i / self.w) as i32;
        let mut best = None;
        let mut best_d = UNREACHABLE;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= self.w as i32 || ny >= self.h as i32 {
                    continue;
                }
                let ni = ny as usize * self.w + nx as usize;
                if !self.walk[ni] {
                    continue;
                }
                if dx != 0 && dy != 0 {
                    let a = y as usize * self.w + nx as usize;
                    let b = ny as usize * self.w + x as usize;
                    if !self.walk[a] || !self.walk[b] {
                        continue;
                    }
                }
                if self.dist[ni] < best_d {
                    best_d = self.dist[ni];
                    best = Some(ni);
                }
            }
        }
        best
    }

    /// Gerade, begehbare Linie zwischen zwei Punkten (Rasterprüfung)?
    pub fn straight_walkable(&self, a: Vec3, b: Vec3) -> bool {
        let d = Vec3::new(b.x - a.x, 0.0, b.z - a.z);
        let len = d.length();
        let steps = (len / (self.cell * 0.5)).ceil() as usize;
        for s in 0..=steps {
            let t = if steps == 0 {
                0.0
            } else {
                s as f32 / steps as f32
            };
            let p = a + d * t;
            if !self.walkable(p) {
                return false;
            }
        }
        true
    }

    pub fn zone_at(&self, p: Vec3) -> Option<usize> {
        self.index_of(p).and_then(|i| {
            let z = self.zone_of[i];
            (z >= 0).then_some(z as usize)
        })
    }
}
