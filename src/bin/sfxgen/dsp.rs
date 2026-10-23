//! Kleine DSP-Bibliothek für die Klangsynthese: Puffer, Oszillatoren, Rauschen, Filter,
//! Hüllkurven, Verzerrung, Hall und WAV-Ein-/Ausgabe.

use std::f32::consts::TAU;
use std::fs;
use std::io::Write;
use std::path::Path;

pub const SR: f32 = 44100.0;

/// Deterministischer Zufall (xorshift), damit die Klänge reproduzierbar sind.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    /// gleichverteilt in [0, 1)
    pub fn f(&mut self) -> f32 {
        self.next_u32() as f32 / 4294967296.0
    }
    /// gleichverteilt in [-1, 1)
    pub fn bi(&mut self) -> f32 {
        self.f() * 2.0 - 1.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

pub fn len(sec: f32) -> usize {
    (sec * SR) as usize
}

pub fn silence(sec: f32) -> Vec<f32> {
    vec![0.0; len(sec)]
}

/// Weißes Rauschen
pub fn noise(sec: f32, rng: &mut Rng) -> Vec<f32> {
    (0..len(sec)).map(|_| rng.bi()).collect()
}

/// Rosa-ähnliches (tieferes) Rauschen
pub fn brown(sec: f32, rng: &mut Rng) -> Vec<f32> {
    let mut v = 0.0f32;
    (0..len(sec))
        .map(|_| {
            v = (v + rng.bi() * 0.08).clamp(-1.0, 1.0) * 0.995;
            v * 3.0
        })
        .collect()
}

/// Oszillator mit frei veränderlicher Frequenz f(t) (Sekunden -> Hz)
pub fn osc(sec: f32, f: impl Fn(f32) -> f32, shape: Shape) -> Vec<f32> {
    let n = len(sec);
    let mut ph = 0.0f32;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR;
        ph = (ph + f(t) / SR).fract();
        out.push(shape.at(ph));
    }
    out
}

#[derive(Clone, Copy)]
pub enum Shape {
    Sine,
    Saw,
    Square,
    Tri,
    Pulse(f32),
}

impl Shape {
    pub fn at(self, ph: f32) -> f32 {
        match self {
            Shape::Sine => (ph * TAU).sin(),
            Shape::Saw => ph * 2.0 - 1.0,
            Shape::Square => {
                if ph < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Shape::Tri => 1.0 - 4.0 * (ph - 0.5).abs(),
            Shape::Pulse(w) => {
                if ph < w {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }
}

/// Hüllkurve anwenden: env(t, dauer) -> Faktor
pub fn env(mut x: Vec<f32>, e: impl Fn(f32, f32) -> f32) -> Vec<f32> {
    let d = x.len() as f32 / SR;
    for (i, s) in x.iter_mut().enumerate() {
        *s *= e(i as f32 / SR, d);
    }
    x
}

/// Exponentieller Abfall mit kurzem Anschlag
pub fn decay(attack: f32, tau: f32) -> impl Fn(f32, f32) -> f32 {
    move |t, _| {
        let a = if attack > 0.0 { (t / attack).min(1.0) } else { 1.0 };
        a * (-(t - attack).max(0.0) / tau).exp()
    }
}

/// Anschlag, Halten, Ausklingen (linear)
pub fn ahr(a: f32, h: f32, r: f32) -> impl Fn(f32, f32) -> f32 {
    move |t, _| {
        if t < a {
            t / a
        } else if t < a + h {
            1.0
        } else {
            (1.0 - (t - a - h) / r).max(0.0)
        }
    }
}

/// Ein-/Ausblenden an den Rändern (gegen Knackser)
pub fn fade(x: Vec<f32>, fi: f32, fo: f32) -> Vec<f32> {
    env(x, move |t, d| (t / fi.max(1e-4)).min(1.0) * ((d - t) / fo.max(1e-4)).min(1.0))
}

pub fn gain(mut x: Vec<f32>, g: f32) -> Vec<f32> {
    for s in x.iter_mut() {
        *s *= g;
    }
    x
}

/// Summe (Länge = längster Puffer); b beginnt bei `at` Sekunden
pub fn mix_at(mut a: Vec<f32>, b: &[f32], at: f32, g: f32) -> Vec<f32> {
    let off = len(at);
    if a.len() < off + b.len() {
        a.resize(off + b.len(), 0.0);
    }
    for (i, s) in b.iter().enumerate() {
        a[off + i] += s * g;
    }
    a
}

pub fn mix(a: Vec<f32>, b: &[f32], g: f32) -> Vec<f32> {
    mix_at(a, b, 0.0, g)
}

pub fn mul(mut a: Vec<f32>, b: &[f32]) -> Vec<f32> {
    for (i, s) in a.iter_mut().enumerate() {
        *s *= b.get(i).copied().unwrap_or(0.0);
    }
    a
}

/// Weiche Sättigung
pub fn drive(mut x: Vec<f32>, amount: f32) -> Vec<f32> {
    let n = amount.tanh();
    for s in x.iter_mut() {
        *s = (*s * amount).tanh() / n;
    }
    x
}

/// Biquad-Filter (RBJ-Kochbuch)
#[derive(Clone, Copy)]
pub enum Filt {
    Low,
    High,
    Band,
    Peak(f32),
}

pub fn biquad(x: Vec<f32>, kind: Filt, freq: f32, q: f32) -> Vec<f32> {
    biquad_sweep(x, kind, move |_| freq, q)
}

/// Biquad mit zeitabhängiger Frequenz
pub fn biquad_sweep(mut x: Vec<f32>, kind: Filt, freq: impl Fn(f32) -> f32, q: f32) -> Vec<f32> {
    let (mut x1, mut x2, mut y1, mut y2) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for (i, s) in x.iter_mut().enumerate() {
        let f = freq(i as f32 / SR).clamp(10.0, SR * 0.45);
        let w = TAU * f / SR;
        let (sn, cs) = w.sin_cos();
        let alpha = sn / (2.0 * q);
        let (b0, b1, b2, a0, a1, a2) = match kind {
            Filt::Low => ((1.0 - cs) / 2.0, 1.0 - cs, (1.0 - cs) / 2.0, 1.0 + alpha, -2.0 * cs, 1.0 - alpha),
            Filt::High => ((1.0 + cs) / 2.0, -(1.0 + cs), (1.0 + cs) / 2.0, 1.0 + alpha, -2.0 * cs, 1.0 - alpha),
            Filt::Band => (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cs, 1.0 - alpha),
            Filt::Peak(db) => {
                let a = 10f32.powf(db / 40.0);
                (1.0 + alpha * a, -2.0 * cs, 1.0 - alpha * a, 1.0 + alpha / a, -2.0 * cs, 1.0 - alpha / a)
            }
        };
        let y = (b0 * *s + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2) / a0;
        x2 = x1;
        x1 = *s;
        y2 = y1;
        y1 = y;
        *s = y;
    }
    x
}

pub fn lowpass(x: Vec<f32>, f: f32) -> Vec<f32> {
    biquad(x, Filt::Low, f, 0.707)
}

pub fn highpass(x: Vec<f32>, f: f32) -> Vec<f32> {
    biquad(x, Filt::High, f, 0.707)
}

pub fn bandpass(x: Vec<f32>, f: f32, q: f32) -> Vec<f32> {
    biquad(x, Filt::Band, f, q)
}

/// Formantfilter (Vokal) aus drei Bandpässen
pub fn formant(x: &[f32], f: [(f32, f32, f32); 3]) -> Vec<f32> {
    let mut out = vec![0.0; x.len()];
    for (fr, q, g) in f {
        let b = bandpass(x.to_vec(), fr, q);
        for (o, s) in out.iter_mut().zip(b.iter()) {
            *o += s * g;
        }
    }
    out
}

/// Einfacher Hall (Schroeder: 4 Kammfilter + 2 Allpässe)
pub fn reverb(x: Vec<f32>, size: f32, damp: f32, wet: f32, tail: f32) -> Vec<f32> {
    let mut input = x.clone();
    input.resize(x.len() + len(tail), 0.0);
    let combs = [1557, 1617, 1491, 1422].map(|d| (d as f32 * size) as usize);
    let mut out = vec![0.0f32; input.len()];
    for d in combs {
        let mut buf = vec![0.0f32; d.max(1)];
        let mut idx = 0;
        let mut lp = 0.0f32;
        for (i, s) in input.iter().enumerate() {
            let y = buf[idx];
            lp = y * (1.0 - damp) + lp * damp;
            buf[idx] = s + lp * 0.84;
            idx = (idx + 1) % buf.len();
            out[i] += y * 0.25;
        }
    }
    for d in [225, 556].map(|d| (d as f32 * size) as usize) {
        let mut buf = vec![0.0f32; d.max(1)];
        let mut idx = 0;
        for s in out.iter_mut() {
            let b = buf[idx];
            let y = -*s + b;
            buf[idx] = *s + b * 0.5;
            idx = (idx + 1) % buf.len();
            *s = y;
        }
    }
    let mut res = input;
    for (r, o) in res.iter_mut().zip(out.iter()) {
        *r = *r * (1.0 - wet * 0.5) + o * wet;
    }
    res
}

/// Echo (Verzögerung mit Rückkopplung)
pub fn echo(x: Vec<f32>, delay: f32, fb: f32, tail: f32) -> Vec<f32> {
    let d = len(delay).max(1);
    let mut y = x;
    y.resize(y.len() + len(tail), 0.0);
    for i in d..y.len() {
        y[i] += y[i - d] * fb;
    }
    y
}

/// Neu abtasten (Tonhöhe und Tempo zusammen ändern), factor < 1 = tiefer
pub fn resample(x: &[f32], factor: f32) -> Vec<f32> {
    let n = (x.len() as f32 / factor) as usize;
    (0..n)
        .map(|i| {
            let p = i as f32 * factor;
            let j = p as usize;
            let fr = p - j as f32;
            let a = x.get(j).copied().unwrap_or(0.0);
            let b = x.get(j + 1).copied().unwrap_or(0.0);
            a + (b - a) * fr
        })
        .collect()
}

/// Auf Spitzenpegel normieren
pub fn normalize(mut x: Vec<f32>, peak: f32) -> Vec<f32> {
    let m = x.iter().fold(0.0f32, |a, s| a.max(s.abs())).max(1e-6);
    for s in x.iter_mut() {
        *s *= peak / m;
    }
    x
}

/// Note (MIDI) -> Frequenz
pub fn midi(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

/// 16-Bit-Mono-WAV schreiben
pub fn write_wav(path: &Path, x: &[f32]) {
    let mut data = Vec::with_capacity(x.len() * 2);
    for s in x {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        data.extend_from_slice(&v.to_le_bytes());
    }
    let mut f = fs::File::create(path).expect("WAV anlegen");
    let sr = SR as u32;
    let mut h = Vec::new();
    h.extend_from_slice(b"RIFF");
    h.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    h.extend_from_slice(b"WAVEfmt ");
    h.extend_from_slice(&16u32.to_le_bytes());
    h.extend_from_slice(&1u16.to_le_bytes());
    h.extend_from_slice(&1u16.to_le_bytes());
    h.extend_from_slice(&sr.to_le_bytes());
    h.extend_from_slice(&(sr * 2).to_le_bytes());
    h.extend_from_slice(&2u16.to_le_bytes());
    h.extend_from_slice(&16u16.to_le_bytes());
    h.extend_from_slice(b"data");
    h.extend_from_slice(&(data.len() as u32).to_le_bytes());
    f.write_all(&h).unwrap();
    f.write_all(&data).unwrap();
}

/// 16-Bit-PCM-WAV lesen (für espeak-ng); liefert Samples und Abtastrate
pub fn read_wav(path: &Path) -> Option<(Vec<f32>, f32)> {
    let b = fs::read(path).ok()?;
    if b.len() < 44 || &b[0..4] != b"RIFF" {
        return None;
    }
    let mut pos = 12;
    let mut sr = 22050.0;
    let mut channels = 1usize;
    while pos + 8 <= b.len() {
        let id = &b[pos..pos + 4];
        let size = u32::from_le_bytes([b[pos + 4], b[pos + 5], b[pos + 6], b[pos + 7]]) as usize;
        let body = pos + 8;
        if id == b"fmt " {
            channels = u16::from_le_bytes([b[body + 2], b[body + 3]]) as usize;
            sr = u32::from_le_bytes([b[body + 4], b[body + 5], b[body + 6], b[body + 7]]) as f32;
        } else if id == b"data" {
            let end = (body + size).min(b.len());
            let samples: Vec<f32> = b[body..end]
                .chunks_exact(2 * channels)
                .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
                .collect();
            return Some((samples, sr));
        }
        pos = body + size + (size & 1);
    }
    None
}
