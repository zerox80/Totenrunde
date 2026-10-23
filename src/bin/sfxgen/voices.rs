//! Stimmen: Zombies (Formant-Synthese: Stimmlippen-Pulse durch Vokalfilter, Knarren,
//! Atemrauschen, Verzerrung) und der Ansager (espeak-ng, tiefer gelegt und verfremdet).

use std::path::Path;
use std::process::Command;

use crate::dsp::*;

/// Vokal-Formanten (Frequenz, Güte, Pegel)
const AH: [(f32, f32, f32); 3] = [(700.0, 5.0, 1.0), (1150.0, 6.0, 0.6), (2600.0, 8.0, 0.25)];
const UH: [(f32, f32, f32); 3] = [(450.0, 5.0, 1.0), (850.0, 6.0, 0.5), (2400.0, 8.0, 0.2)];
const OH: [(f32, f32, f32); 3] = [(500.0, 5.0, 1.0), (800.0, 6.0, 0.6), (2500.0, 8.0, 0.2)];
const EH: [(f32, f32, f32); 3] = [(550.0, 5.0, 1.0), (1800.0, 7.0, 0.5), (2600.0, 8.0, 0.25)];

/// Stimmquelle mit Tonhöhenverlauf, Jitter und Knarren (Vocal Fry)
fn glottal(d: f32, f0: impl Fn(f32) -> f32, fry: f32, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let n = len(d);
    let mut ph = 0.0f32;
    let mut jit = 0.0f32;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR;
        if i % 400 == 0 {
            jit = rng.bi() * 0.06;
        }
        let f = f0(t) * (1.0 + jit);
        ph = (ph + f / SR).fract();
        // schmaler Puls mit Rauschanteil (heiser)
        let pulse = if ph < 0.18 { (ph / 0.18 * std::f32::consts::PI).sin() } else { 0.0 };
        let am = 1.0 - fry * (0.5 + 0.5 * (t * 31.0 * std::f32::consts::TAU).sin()).powi(3);
        out.push(pulse * am + rng.bi() * 0.25);
    }
    out
}

/// Zombie-Laut: Quelle -> zwei Vokale (überblendet) -> Verzerrung -> Raum
fn creature(d: f32, f0: impl Fn(f32) -> f32 + Copy, v1: [(f32, f32, f32); 3], v2: [(f32, f32, f32); 3], fry: f32, breath: f32, dist: f32, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed + 100);
    let src = glottal(d, f0, fry, seed);
    let a = formant(&src, v1);
    let b = formant(&src, v2);
    let mut x: Vec<f32> = a.iter().zip(b.iter()).enumerate().map(|(i, (p, q))| {
        let k = i as f32 / a.len() as f32;
        p * (1.0 - k) + q * k
    }).collect();
    let br = env(bandpass(noise(d, &mut rng), 1800.0, 0.7), |_, _| 1.0);
    x = mix(x, &br, breath);
    x = drive(normalize(x, 1.0), dist);
    x = env(x, |t, d| (t / 0.06).min(1.0) * ((d - t) / (d * 0.35)).clamp(0.0, 1.0));
    x = biquad(x, Filt::Peak(5.0), 250.0, 1.0);
    fade(normalize(reverb(x, 0.9, 0.5, 0.18, 0.35), 0.9), 0.002, 0.05)
}

pub fn zombies(out: &mut Vec<(&'static str, Vec<f32>)>) {
    let groans = ["z_groan_1", "z_groan_2", "z_groan_3", "z_groan_4", "z_groan_5", "z_groan_6"];
    let vow = [(AH, UH), (UH, OH), (OH, AH), (EH, UH), (AH, OH), (UH, AH)];
    for (i, name) in groans.iter().enumerate() {
        let mut r = Rng::new(200 + i as u64);
        let base = r.range(68.0, 105.0);
        let d = r.range(1.1, 2.0);
        let wob = r.range(2.0, 5.0);
        let f0 = move |t: f32| base * (1.0 + 0.12 * (t * wob).sin() - 0.15 * t / d);
        out.push((name, creature(d, f0, vow[i].0, vow[i].1, 0.7, 0.35, 3.0, 200 + i as u64)));
    }
    for (i, name) in ["z_attack_1", "z_attack_2", "z_attack_3"].iter().enumerate() {
        let mut r = Rng::new(300 + i as u64);
        let base = r.range(120.0, 170.0);
        let f0 = move |t: f32| base * (1.0 + 0.6 * (-t / 0.15).exp());
        out.push((name, creature(0.75, f0, AH, EH, 0.5, 0.6, 5.0, 300 + i as u64)));
    }
    for (i, name) in ["z_scream_1", "z_scream_2"].iter().enumerate() {
        let base = 260.0 + i as f32 * 60.0;
        let f0 = move |t: f32| base * (1.0 + 0.4 * (t / 1.2).min(1.0)) * (1.0 + 0.03 * (t * 40.0).sin());
        out.push((name, creature(1.3, f0, AH, EH, 0.3, 0.9, 6.0, 400 + i as u64)));
    }
    for (i, name) in ["z_death_1", "z_death_2", "z_death_3", "z_death_4"].iter().enumerate() {
        let mut r = Rng::new(500 + i as u64);
        let base = r.range(110.0, 160.0);
        let d = r.range(0.8, 1.3);
        let f0 = move |t: f32| base * (1.0 - 0.55 * (t / d));
        let mut x = creature(d, f0, AH, UH, 0.9, 0.5, 4.0, 500 + i as u64);
        // Gurgeln am Ende
        let gurgle = env(bandpass(brown(0.6, &mut r), 400.0, 2.0), ahr(0.05, 0.2, 0.3));
        x = mix_at(x, &gurgle, d * 0.6, 0.5);
        out.push((name, normalize(x, 0.9)));
    }
    {
        // Aus dem Boden steigen: Erde bröckelt + Stöhnen
        let mut rng = Rng::new(600);
        let mut dirt = silence(2.0);
        for _ in 0..60 {
            let at = rng.range(0.0, 1.6);
            let g = env(lowpass(noise(0.08, &mut rng), rng.range(600.0, 1800.0)), decay(0.001, 0.02));
            dirt = mix_at(dirt, &g, at, rng.range(0.1, 0.4));
        }
        let groan = creature(1.6, |t| 80.0 * (1.0 + 0.2 * t), UH, AH, 0.8, 0.4, 3.0, 601);
        out.push(("z_rise", normalize(mix_at(dirt, &groan, 0.3, 0.8), 0.9)));
    }
    {
        // Spieler: Schmerzlaute und Zu-Boden-Gehen
        for (i, name) in ["hurt_1", "hurt_2"].iter().enumerate() {
            let base = 140.0 + i as f32 * 20.0;
            let f0 = move |t: f32| base * (1.0 - 0.3 * t / 0.35);
            out.push((name, creature(0.35, f0, EH, AH, 0.2, 0.5, 2.5, 700 + i as u64)));
        }
    }
    {
        // Lachen aus der Kiste: "ha ha ha" tief und hallig
        let mut x = silence(2.4);
        for k in 0..5 {
            let base = 95.0 - k as f32 * 6.0;
            let h = creature(0.28, move |_| base, AH, AH, 0.6, 0.9, 3.0, 800 + k);
            x = mix_at(x, &h, k as f32 * 0.3, 1.0 - k as f32 * 0.12);
        }
        out.push(("box_laugh", normalize(reverb(x, 1.4, 0.4, 0.45, 1.2), 0.95)));
    }
}

/// Ansager über espeak-ng; ohne espeak-ng wird ein synthetischer Ersatz erzeugt.
pub fn announcer(out: &mut Vec<(&'static str, Vec<f32>)>, tmp: &Path) {
    let lines = [
        ("vo_maxammo", "Maximale Munition!"),
        ("vo_instakill", "Sofort-Tod!"),
        ("vo_double", "Doppelte Punkte!"),
        ("vo_kaboom", "Kabum!"),
        ("vo_carpenter", "Zimmermann!"),
        ("vo_firesale", "Ausverkauf!"),
    ];
    for (name, text) in lines {
        let wav = tmp.join(format!("{name}_raw.wav"));
        let ok = Command::new("espeak-ng")
            .args(["-v", "de+m3", "-s", "125", "-p", "18", "-a", "180", "-w"])
            .arg(&wav)
            .arg(text)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        let voice = if ok { read_wav(&wav) } else { None };
        let x = match voice {
            Some((v, sr)) => demonic(&v, sr),
            None => {
                eprintln!("espeak-ng nicht verfügbar – Ersatzklang für {name}");
                creature(1.0, |_| 70.0, AH, OH, 0.5, 0.3, 3.0, 900)
            }
        };
        out.push((name, x));
        let _ = std::fs::remove_file(&wav);
    }
}

/// Stimme tiefer legen (Unteroktave dazumischen), anrauen und in einen großen Raum stellen.
fn demonic(v: &[f32], sr: f32) -> Vec<f32> {
    // auf 44,1 kHz bringen und dabei um ~5 Halbtöne tiefer
    let low = resample(v, sr / SR * 0.74);
    let sub = resample(v, sr / SR * 0.5);
    let mut x = low.clone();
    x = mix(x, &sub[..sub.len().min(low.len() * 2)], 0.55);
    x = biquad(x, Filt::Peak(6.0), 140.0, 0.8);
    x = lowpass(x, 5200.0);
    x = drive(normalize(x, 1.0), 2.2);
    x = reverb(x, 1.6, 0.35, 0.4, 1.6);
    fade(normalize(x, 0.95), 0.005, 0.2)
}
