//! Waffen- und Treffergeräusche.

use crate::dsp::*;

/// Grundbaustein Schuss: Knall (Rauschen), Druckwelle (tiefer Sinus-Abfall), Hallfahne.
pub struct Shot {
    pub crack_freq: f32,
    pub crack_q: f32,
    pub crack_tau: f32,
    pub boom_from: f32,
    pub boom_to: f32,
    pub boom_tau: f32,
    pub boom_gain: f32,
    pub tail: f32,
    pub tail_lp: f32,
    pub drive: f32,
    pub room: f32,
}

pub fn shot(p: Shot, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let dur = 0.12 + p.tail;
    let n = noise(dur, &mut rng);
    let crack = env(bandpass(n.clone(), p.crack_freq, p.crack_q), decay(0.0008, p.crack_tau));
    let body = env(lowpass(n.clone(), 3500.0), decay(0.001, p.crack_tau * 2.5));
    let boom = env(
        osc(dur, |t| p.boom_to + (p.boom_from - p.boom_to) * (-t / 0.04).exp(), Shape::Sine),
        decay(0.002, p.boom_tau),
    );
    let tail = env(lowpass(brown(dur, &mut rng), p.tail_lp), decay(0.01, p.tail * 0.35));
    let mut x = gain(crack, 1.0);
    x = mix(x, &body, 0.6);
    x = mix(x, &boom, p.boom_gain);
    x = mix(x, &tail, 0.35);
    x = drive(x, p.drive);
    x = reverb(x, p.room, 0.4, 0.25, p.tail);
    fade(normalize(x, 0.95), 0.0005, 0.05)
}

pub fn all(out: &mut Vec<(&'static str, Vec<f32>)>) {
    out.push((
        "pistol",
        shot(
            Shot { crack_freq: 2200.0, crack_q: 0.9, crack_tau: 0.03, boom_from: 180.0, boom_to: 60.0, boom_tau: 0.07, boom_gain: 0.9, tail: 0.35, tail_lp: 900.0, drive: 2.5, room: 0.9 },
            1,
        ),
    ));
    out.push((
        "revolver",
        shot(
            Shot { crack_freq: 1700.0, crack_q: 0.8, crack_tau: 0.045, boom_from: 160.0, boom_to: 45.0, boom_tau: 0.12, boom_gain: 1.2, tail: 0.6, tail_lp: 700.0, drive: 3.0, room: 1.1 },
            2,
        ),
    ));
    out.push((
        "smg",
        shot(
            Shot { crack_freq: 2600.0, crack_q: 1.0, crack_tau: 0.02, boom_from: 200.0, boom_to: 70.0, boom_tau: 0.05, boom_gain: 0.7, tail: 0.25, tail_lp: 1100.0, drive: 2.2, room: 0.8 },
            3,
        ),
    ));
    out.push((
        "rifle",
        shot(
            Shot { crack_freq: 1900.0, crack_q: 0.7, crack_tau: 0.035, boom_from: 170.0, boom_to: 50.0, boom_tau: 0.1, boom_gain: 1.1, tail: 0.55, tail_lp: 800.0, drive: 3.2, room: 1.0 },
            4,
        ),
    ));
    out.push((
        "lmg",
        shot(
            Shot { crack_freq: 1600.0, crack_q: 0.7, crack_tau: 0.04, boom_from: 150.0, boom_to: 45.0, boom_tau: 0.11, boom_gain: 1.3, tail: 0.5, tail_lp: 700.0, drive: 3.5, room: 1.0 },
            5,
        ),
    ));
    out.push((
        "shotgun",
        shot(
            Shot { crack_freq: 1200.0, crack_q: 0.6, crack_tau: 0.06, boom_from: 140.0, boom_to: 38.0, boom_tau: 0.18, boom_gain: 1.6, tail: 0.8, tail_lp: 600.0, drive: 4.0, room: 1.2 },
            6,
        ),
    ));
    {
        // Scharfschütze: harter Knall und langes Echo
        let x = shot(
            Shot { crack_freq: 2400.0, crack_q: 0.8, crack_tau: 0.04, boom_from: 160.0, boom_to: 40.0, boom_tau: 0.15, boom_gain: 1.4, tail: 1.0, tail_lp: 650.0, drive: 4.0, room: 1.3 },
            7,
        );
        out.push(("sniper", normalize(echo(x, 0.32, 0.35, 0.8), 0.95)));
    }
    {
        // Granatwerfer: dumpfes "Plopp" mit Luft
        let mut rng = Rng::new(8);
        let thump = env(osc(0.5, |t| 50.0 + 120.0 * (-t / 0.03).exp(), Shape::Sine), decay(0.002, 0.12));
        let air = env(bandpass(noise(0.5, &mut rng), 900.0, 0.8), decay(0.003, 0.06));
        let x = mix(gain(thump, 1.2), &air, 0.5);
        out.push(("launcher", fade(normalize(reverb(drive(x, 2.0), 0.9, 0.4, 0.2, 0.4), 0.95), 0.0005, 0.05)));
    }
    {
        // Strahlenwaffe: fallender FM-Zap mit Rauschen
        let mut rng = Rng::new(9);
        let car = osc(0.45, |t| 1800.0 * (-t / 0.12).exp() + 300.0, Shape::Square);
        let modu = osc(0.45, |_| 37.0, Shape::Sine);
        let mut x = mul(car, &modu.iter().map(|m| 0.6 + 0.4 * m).collect::<Vec<_>>());
        x = env(bandpass(x, 1400.0, 0.5), decay(0.002, 0.12));
        let fizz = env(highpass(noise(0.45, &mut rng), 3000.0), decay(0.001, 0.05));
        x = mix(x, &fizz, 0.3);
        out.push(("raygun", fade(normalize(reverb(x, 0.8, 0.3, 0.3, 0.4), 0.9), 0.0005, 0.05)));
    }
    for (i, name) in ["explosion_1", "explosion_2"].iter().enumerate() {
        out.push((name, explosion(10 + i as u64, 1.0 + i as f32 * 0.2)));
    }
    out.push(("dryfire", click(0.008, 3200.0, 11, 0.6)));
    out.push(("mag_out", mech(&[(0.0, 2200.0, 0.02), (0.06, 900.0, 0.05)], 12)));
    out.push(("mag_in", mech(&[(0.0, 1300.0, 0.03), (0.09, 2600.0, 0.02), (0.12, 700.0, 0.06)], 13)));
    out.push(("bolt", mech(&[(0.0, 1800.0, 0.03), (0.08, 1100.0, 0.04), (0.22, 2400.0, 0.02), (0.27, 900.0, 0.05)], 14)));
    out.push(("switch", mech(&[(0.0, 700.0, 0.06), (0.15, 2000.0, 0.02)], 15)));
    out.push(("shell", shell(16)));
    out.push(("knife_swing", whoosh(0.32, 1800.0, 600.0, 17)));
    out.push(("knife_hit", flesh(18, 0.7, true)));
    out.push(("grenade_pin", ping(2600.0, 0.25, 19)));
    out.push(("grenade_bounce", clank(20)));
    for i in 0..3 {
        out.push((["hit_flesh_1", "hit_flesh_2", "hit_flesh_3"][i], flesh(21 + i as u64, 0.5 + i as f32 * 0.1, false)));
        out.push((["impact_1", "impact_2", "impact_3"][i], impact(31 + i as u64, i == 2)));
    }
    {
        // Kopfschuss: knackiges Knacken + feuchter Spritzer
        let mut rng = Rng::new(40);
        let crack = env(bandpass(noise(0.3, &mut rng), 1500.0, 1.2), decay(0.0005, 0.015));
        let x = mix(gain(crack, 1.0), &flesh(41, 0.9, false), 0.8);
        out.push(("headshot", normalize(x, 0.9)));
    }
}

pub fn explosion(seed: u64, size: f32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let d = 2.2 * size;
    let boom = env(osc(d, |t| 30.0 + 90.0 * (-t / 0.08).exp(), Shape::Sine), decay(0.003, 0.35 * size));
    let blast = env(lowpass(noise(d, &mut rng), 2500.0), decay(0.002, 0.12));
    let rumble = env(lowpass(brown(d, &mut rng), 220.0), decay(0.02, 0.7 * size));
    let mut debris = vec![0.0; len(d)];
    for _ in 0..40 {
        let at = rng.range(0.08, 1.2 * size);
        let c = env(bandpass(noise(0.05, &mut rng), rng.range(1500.0, 4500.0), 3.0), decay(0.0005, 0.01));
        debris = mix_at(debris, &c, at, rng.range(0.05, 0.25));
    }
    let mut x = gain(boom, 1.3);
    x = mix(x, &blast, 0.9);
    x = mix(x, &rumble, 1.1);
    x = mix(x, &debris, 1.0);
    x = drive(x, 3.0);
    fade(normalize(reverb(x, 1.3, 0.5, 0.3, 1.0), 0.98), 0.001, 0.2)
}

fn click(tau: f32, f: f32, seed: u64, g: f32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let c = env(bandpass(noise(0.12, &mut rng), f, 2.0), decay(0.0003, tau));
    normalize(c, g)
}

/// Mechanische Geräusche: Folge von Klicks (Zeit, Frequenz, Abklingzeit)
fn mech(parts: &[(f32, f32, f32)], seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut x = silence(0.5);
    for (at, f, tau) in parts {
        let c = env(bandpass(noise(0.2, &mut rng), *f, 1.5), decay(0.0005, *tau));
        let thud = env(osc(0.2, |_| f * 0.12, Shape::Sine), decay(0.001, tau * 0.8));
        x = mix_at(x, &c, *at, 1.0);
        x = mix_at(x, &thud, *at, 0.4);
    }
    fade(normalize(x, 0.8), 0.0005, 0.05)
}

/// Hülse fällt und klimpert (metallische Teiltöne)
fn shell(seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut x = silence(0.6);
    for (k, at) in [0.0f32, 0.11, 0.19, 0.25].iter().enumerate() {
        let g = 1.0 / (k as f32 + 1.0);
        for f in [4100.0, 6900.0, 9300.0] {
            let p = env(osc(0.2, move |_| f * rng_f(seed, k), Shape::Sine), decay(0.0003, 0.04));
            x = mix_at(x, &p, *at, g * 0.3);
        }
        let n = env(highpass(noise(0.05, &mut rng), 3000.0), decay(0.0002, 0.006));
        x = mix_at(x, &n, *at, g * 0.4);
    }
    normalize(x, 0.6)
}

fn rng_f(seed: u64, k: usize) -> f32 {
    let mut r = Rng::new(seed * 31 + k as u64);
    r.range(0.95, 1.05)
}

pub fn whoosh(d: f32, f0: f32, f1: f32, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let n = noise(d, &mut rng);
    let x = biquad_sweep(n, Filt::Band, move |t| f0 + (f1 - f0) * (t / d), 1.2);
    normalize(env(x, |t, d| (t / (d * 0.3)).min(1.0) * ((d - t) / (d * 0.7)).min(1.0)), 0.7)
}

/// Fleischtreffer: dumpfer Schlag + feuchtes Schmatzen
pub fn flesh(seed: u64, wet: f32, stab: bool) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let thud = env(osc(0.3, |t| 70.0 + 100.0 * (-t / 0.02).exp(), Shape::Sine), decay(0.001, 0.05));
    let slap = env(lowpass(noise(0.3, &mut rng), 1800.0), decay(0.0008, 0.02));
    let squish = env(
        biquad_sweep(noise(0.3, &mut rng), Filt::Band, |t| 900.0 - 500.0 * (t / 0.3), 3.0),
        ahr(0.01, 0.05, 0.12),
    );
    let mut x = gain(thud, 1.0);
    x = mix(x, &slap, 0.7);
    x = mix(x, &squish, wet);
    if stab {
        x = mix(x, &whoosh(0.15, 2500.0, 1200.0, seed + 7), 0.3);
    }
    fade(normalize(x, 0.85), 0.0005, 0.03)
}

/// Kugel schlägt in Beton/Holz ein (optional mit Querschläger-Pfeifen)
fn impact(seed: u64, ricochet: bool) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let hit = env(bandpass(noise(0.4, &mut rng), rng.range(1800.0, 3200.0), 1.0), decay(0.0003, 0.012));
    let dust = env(lowpass(noise(0.4, &mut rng), 1500.0), decay(0.002, 0.05));
    let mut x = mix(hit, &dust, 0.4);
    if ricochet {
        let w = env(osc(0.4, |t| 3200.0 - 1600.0 * t / 0.4, Shape::Sine), ahr(0.01, 0.15, 0.2));
        x = mix_at(x, &w, 0.01, 0.25);
    }
    fade(normalize(x, 0.7), 0.0003, 0.05)
}

fn ping(f: f32, tau: f32, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut x = env(osc(0.4, move |_| f, Shape::Sine), decay(0.0005, tau));
    x = mix(x, &env(osc(0.4, move |_| f * 2.76, Shape::Sine), decay(0.0005, tau * 0.4)), 0.4);
    x = mix(x, &env(highpass(noise(0.05, &mut rng), 4000.0), decay(0.0002, 0.005)), 0.5);
    normalize(x, 0.6)
}

fn clank(seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut x = silence(0.5);
    for f in [820.0, 1460.0, 2380.0, 3900.0] {
        let p = env(osc(0.5, move |_| f, Shape::Sine), decay(0.0005, 0.08));
        x = mix(x, &p, 0.3);
    }
    x = mix(x, &env(bandpass(noise(0.1, &mut rng), 1200.0, 1.0), decay(0.0003, 0.01)), 0.6);
    normalize(x, 0.7)
}
