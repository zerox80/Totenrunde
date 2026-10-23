//! Geräusche der Welt: Bretter, Schritte, Herzschlag, Kasse, Türen, Strom, Perks,
//! Kiste, Verstärker, Power-ups, Atombombe.

use crate::dsp::*;
use crate::weapons::{explosion, whoosh};

/// Holz knackt/splittert (Resonanzen + kurze Rauschimpulse)
fn wood_crack(seed: u64, n: usize, d: f32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut x = silence(d);
    for _ in 0..n {
        let at = rng.range(0.0, d * 0.6);
        let f = rng.range(250.0, 900.0);
        let c = env(bandpass(noise(0.15, &mut rng), f, 4.0), decay(0.0005, rng.range(0.01, 0.05)));
        x = mix_at(x, &c, at, rng.range(0.3, 1.0));
    }
    // Knarren: tiefer, rauer Ton mit Tonhöhenänderung
    let creak = env(
        bandpass(osc(d * 0.5, |t| 90.0 + 60.0 * t, Shape::Pulse(0.1)), 600.0, 2.0),
        ahr(0.02, d * 0.2, d * 0.2),
    );
    fade(normalize(mix(x, &creak, 0.5), 0.9), 0.001, 0.05)
}

fn thump(f: f32, tau: f32, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let s = env(osc(0.5, move |t| f * (1.0 + 1.5 * (-t / 0.015).exp()), Shape::Sine), decay(0.001, tau));
    let n = env(lowpass(noise(0.5, &mut rng), 900.0), decay(0.0008, tau * 0.4));
    normalize(mix(s, &n, 0.5), 0.9)
}

fn bell(f: f32, d: f32, tau: f32) -> Vec<f32> {
    let mut x = silence(d);
    for (r, g) in [(1.0, 1.0), (2.76, 0.5), (5.4, 0.25), (8.93, 0.12)] {
        let p = env(osc(d, move |_| f * r, Shape::Sine), decay(0.001, tau / r.sqrt()));
        x = mix(x, &p, g);
    }
    x
}

pub fn all(out: &mut Vec<(&'static str, Vec<f32>)>) {
    out.push(("board_tear_1", wood_crack(1000, 14, 0.7)));
    out.push(("board_tear_2", wood_crack(1001, 18, 0.8)));
    {
        // Brett anbringen: Aufschlag + zwei Hammerschläge
        let mut x = thump(110.0, 0.08, 1002);
        for (i, at) in [0.18f32, 0.33].iter().enumerate() {
            let mut r = Rng::new(1003 + i as u64);
            let h = env(bandpass(noise(0.1, &mut r), 2400.0, 2.0), decay(0.0003, 0.02));
            x = mix_at(x, &h, *at, 0.8);
            x = mix_at(x, &thump(180.0, 0.03, 1005 + i as u64), *at, 0.5);
        }
        out.push(("board_place", normalize(x, 0.9)));
    }
    for (i, name) in ["step_1", "step_2", "step_3", "step_4"].iter().enumerate() {
        let mut r = Rng::new(1010 + i as u64);
        let t = thump(r.range(60.0, 80.0), 0.04, 1010 + i as u64);
        let grit = env(bandpass(noise(0.15, &mut r), r.range(1500.0, 2500.0), 1.0), decay(0.002, 0.03));
        out.push((name, normalize(mix(t, &grit, 0.5), 0.7)));
    }
    out.push(("land", normalize(mix(thump(55.0, 0.12, 1020), &thump(90.0, 0.05, 1021), 0.6), 0.9)));
    {
        let mut x = silence(1.0);
        x = mix_at(x, &thump(48.0, 0.08, 1030), 0.0, 1.0);
        x = mix_at(x, &thump(42.0, 0.1, 1031), 0.22, 0.8);
        out.push(("heartbeat", normalize(lowpass(x, 300.0), 0.9)));
    }
    {
        // Zu Boden gehen: dumpfer Schlag, dann tiefes Dröhnen
        let mut r = Rng::new(1040);
        let drone = env(lowpass(mix(osc(2.5, |_| 55.0, Shape::Saw), &brown(2.5, &mut r), 0.3), 300.0), ahr(0.3, 1.2, 1.0));
        let x = mix(thump(50.0, 0.25, 1041), &drone, 0.5);
        out.push(("down", normalize(reverb(x, 1.2, 0.5, 0.3, 0.8), 0.9)));
    }
    {
        // Kauf: Kassenklingel
        let mut x = silence(0.9);
        x = mix_at(x, &thump(300.0, 0.02, 1050), 0.0, 0.3);
        x = mix_at(x, &bell(1567.0, 0.8, 0.5), 0.05, 0.6);
        x = mix_at(x, &bell(2093.0, 0.8, 0.45), 0.12, 0.5);
        out.push(("buy", normalize(x, 0.8)));
    }
    {
        let x = env(lowpass(osc(0.35, |_| 110.0, Shape::Square), 900.0), ahr(0.005, 0.25, 0.08));
        out.push(("deny", normalize(mix(x.clone(), &env(osc(0.35, |_| 116.0, Shape::Square), ahr(0.005, 0.25, 0.08)), 0.6), 0.6)));
    }
    out.push(("click", normalize(env(bandpass(noise(0.05, &mut Rng::new(1060)), 3000.0, 2.0), decay(0.0002, 0.006)), 0.6)));
    {
        // Tür: Knarren + Anschlag
        let creak = wood_crack(1070, 6, 1.2);
        let x = mix_at(creak, &thump(70.0, 0.2, 1071), 0.9, 1.0);
        out.push(("door", normalize(reverb(x, 1.0, 0.5, 0.25, 0.6), 0.9)));
    }
    {
        // Schutt: viele Brocken poltern
        let mut r = Rng::new(1080);
        let mut x = silence(2.2);
        for _ in 0..90 {
            let at = r.range(0.0, 1.6);
            let c = env(lowpass(noise(0.12, &mut r), r.range(400.0, 1800.0)), decay(0.001, r.range(0.01, 0.06)));
            x = mix_at(x, &c, at, r.range(0.2, 0.9));
        }
        x = mix(x, &env(lowpass(brown(2.2, &mut r), 150.0), ahr(0.1, 0.8, 1.0)), 0.6);
        out.push(("debris", normalize(reverb(x, 1.1, 0.5, 0.2, 0.5), 0.9)));
    }
    {
        // Strom an: schwerer Schalter, ansteigendes Brummen, Funken
        let mut r = Rng::new(1090);
        let clunk = thump(65.0, 0.18, 1091);
        let mut hum = silence(3.0);
        for (h, g) in [(1.0, 1.0), (2.0, 0.6), (3.0, 0.4), (5.0, 0.2)] {
            hum = mix(hum, &osc(3.0, move |t| 50.0 * h * (0.6 + 0.4 * (t / 1.5).min(1.0)), Shape::Saw), g);
        }
        let hum = env(lowpass(hum, 900.0), ahr(1.2, 1.0, 0.8));
        let mut sparks = silence(3.0);
        for _ in 0..30 {
            let at = r.range(0.1, 1.5);
            let s = env(highpass(noise(0.03, &mut r), 4000.0), decay(0.0002, 0.006));
            sparks = mix_at(sparks, &s, at, r.range(0.2, 0.6));
        }
        let x = mix(mix(clunk, &hum, 0.4), &sparks, 0.6);
        out.push(("power_on", normalize(reverb(x, 1.3, 0.4, 0.35, 1.0), 0.95)));
    }
    {
        // Perk trinken: Kronkorken, Schlucke, Glasklirren
        let mut r = Rng::new(1100);
        let pop = env(bandpass(noise(0.1, &mut r), 1800.0, 3.0), decay(0.0005, 0.02));
        let mut x = silence(2.6);
        x = mix_at(x, &pop, 0.05, 1.0);
        for k in 0..4 {
            let g = env(osc(0.18, |t| 180.0 + 120.0 * t / 0.18, Shape::Sine), ahr(0.02, 0.06, 0.08));
            let g = mix(g, &env(lowpass(noise(0.18, &mut r), 500.0), ahr(0.02, 0.05, 0.08)), 0.6);
            x = mix_at(x, &g, 0.55 + k as f32 * 0.32, 0.8);
        }
        x = mix_at(x, &bell(3100.0, 0.6, 0.15), 2.05, 0.3);
        out.push(("perk_drink", normalize(x, 0.85)));
    }
    {
        // Verstärker: Maschine läuft an, mahlt, zischt
        let mut r = Rng::new(1110);
        let d = 3.6;
        let motor = env(lowpass(osc(d, |t| 40.0 + 80.0 * (t / d), Shape::Saw), 700.0), ahr(0.4, 2.6, 0.6));
        let grind = env(bandpass(noise(d, &mut r), 1300.0, 1.5), ahr(0.8, 2.0, 0.8));
        let mut hits = silence(d);
        for k in 0..12 {
            hits = mix_at(hits, &thump(90.0, 0.05, 1111 + k), 0.4 + k as f32 * 0.25, 0.4);
        }
        let x = mix(mix(motor, &grind, 0.5), &hits, 0.8);
        out.push(("pap_start", normalize(reverb(drive(x, 2.0), 1.1, 0.5, 0.25, 0.5), 0.9)));
        let mut done = bell(1046.0, 1.5, 0.9);
        done = mix(done, &bell(1568.0, 1.5, 0.8), 0.6);
        let hiss = env(highpass(noise(1.5, &mut r), 2500.0), ahr(0.01, 0.2, 0.8));
        out.push(("pap_done", normalize(reverb(mix(done, &hiss, 0.25), 1.2, 0.4, 0.35, 0.8), 0.85)));
    }
    {
        out.push(("box_open", normalize(mix(wood_crack(1120, 5, 0.6), &bell(1318.0, 0.6, 0.4), 0.3), 0.85)));
        out.push(("box_close", normalize(mix(thump(80.0, 0.12, 1121), &wood_crack(1122, 3, 0.3), 0.4), 0.85)));
        let mut r = Rng::new(1123);
        let w = whoosh(2.0, 300.0, 2500.0, 1124);
        let rumble = env(lowpass(brown(2.0, &mut r), 120.0), ahr(0.2, 1.0, 0.8));
        out.push(("box_move", normalize(reverb(mix(w, &rumble, 0.8), 1.3, 0.5, 0.3, 0.8), 0.9)));
    }
    {
        // Power-up erscheint: Glitzern; eingesammelt: aufsteigender Klang
        let mut x = silence(1.2);
        for (k, n) in [84.0f32, 88.0, 91.0, 96.0].iter().enumerate() {
            x = mix_at(x, &bell(midi(*n), 0.8, 0.35), k as f32 * 0.07, 0.4);
        }
        out.push(("pu_spawn", normalize(reverb(x, 1.0, 0.3, 0.4, 0.6), 0.7)));
        let s = env(osc(0.9, |t| 300.0 * 2f32.powf(t * 3.0), Shape::Tri), ahr(0.01, 0.5, 0.35));
        out.push(("pu_grab", normalize(reverb(s, 0.9, 0.3, 0.3, 0.4), 0.7)));
    }
    {
        // Atombombe: gewaltige Explosion mit langem Grollen
        let mut x = explosion(1130, 2.0);
        x = mix(x, &env(lowpass(brown(4.0, &mut Rng::new(1131)), 90.0), ahr(0.3, 1.5, 2.0)), 0.8);
        out.push(("nuke", normalize(x, 0.98)));
        // Zimmermann: schnelles Hämmern und Sägen
        let mut c = silence(2.2);
        for k in 0..10 {
            c = mix_at(c, &thump(160.0, 0.03, 1140 + k), k as f32 * 0.18, 0.7);
        }
        let saw = env(bandpass(noise(1.0, &mut Rng::new(1150)), 2800.0, 2.0), |t, _| 0.5 + 0.5 * (t * 14.0).sin().abs());
        c = mix_at(c, &saw, 1.0, 0.5);
        out.push(("carpenter", normalize(c, 0.85)));
    }
}

/// Wind (Schleife, 20 s): gefiltertes Rauschen mit langsamer Bewegung
pub fn wind() -> Vec<f32> {
    let mut r = Rng::new(1200);
    let d = 20.0;
    let n = noise(d, &mut r);
    let x = biquad_sweep(n, Filt::Band, |t| 380.0 + 220.0 * (t * 0.31).sin() + 120.0 * (t * 0.83).sin(), 1.4);
    let gust = |t: f32| 0.55 + 0.3 * (t * 0.22 * std::f32::consts::TAU).sin().powi(2) + 0.15 * (t * 0.07 * std::f32::consts::TAU).sin();
    let x = env(x, move |t, _| gust(t));
    loop_xfade(normalize(x, 0.7), 1.5)
}

/// Feuer (Schleife, 8 s): Grollen + Knistern
pub fn fire() -> Vec<f32> {
    let mut r = Rng::new(1210);
    let d = 8.0;
    let mut x = lowpass(brown(d, &mut r), 300.0);
    for _ in 0..260 {
        let at = r.range(0.0, d - 0.05);
        let c = env(highpass(noise(0.02, &mut r), r.range(1500.0, 5000.0)), decay(0.0002, r.range(0.001, 0.006)));
        x = mix_at(x, &c, at, r.range(0.2, 0.9));
    }
    loop_xfade(normalize(x, 0.7), 0.8)
}

/// Schleife ohne Knacks: Ende über den Anfang blenden
pub fn loop_xfade(x: Vec<f32>, sec: f32) -> Vec<f32> {
    let n = len(sec).min(x.len() / 3);
    let body = x.len() - n;
    let mut out = x[..body].to_vec();
    for i in 0..n {
        let k = i as f32 / n as f32;
        out[i] = x[i] * k + x[body + i] * (1.0 - k);
    }
    out
}
