//! Strahltests gegen einfache Formen (Kugel, Kapsel) für Trefferzonen.

use bevy::prelude::*;

/// Strahl gegen Kugel.
pub fn ray_sphere(o: Vec3, d: Vec3, c: Vec3, r: f32) -> Option<f32> {
    let m = o - c;
    let b = m.dot(d);
    let cc = m.dot(m) - r * r;
    if cc > 0.0 && b > 0.0 {
        return None;
    }
    let disc = b * b - cc;
    if disc < 0.0 {
        return None;
    }
    let t = -b - disc.sqrt();
    Some(t.max(0.0))
}

/// Strahl gegen Kapsel (Segment a-b, Radius r).
pub fn ray_capsule(o: Vec3, d: Vec3, a: Vec3, b: Vec3, r: f32) -> Option<f32> {
    // Annäherung: kleinster Abstand Strahl-Segment, dann Kugeltest am nächsten Punkt
    let ab = b - a;
    let ao = o - a;
    let ab_ab = ab.dot(ab);
    let ab_d = ab.dot(d);
    let ab_ao = ab.dot(ao);
    let denom = ab_ab - ab_d * ab_d;
    let s = if denom.abs() > 1e-8 {
        ((ab_ao - ab_d * ao.dot(d)) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Punkt auf Segment, dann Strahl gegen Kugel um diesen Punkt
    let p = a + ab * s;
    let t1 = ray_sphere(o, d, p, r);
    // Endkappen zusätzlich prüfen
    let t2 = ray_sphere(o, d, a, r);
    let t3 = ray_sphere(o, d, b, r);
    [t1, t2, t3].into_iter().flatten().reduce(f32::min)
}
