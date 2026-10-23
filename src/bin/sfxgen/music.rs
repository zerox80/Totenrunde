//! Musik (alles eigene Kompositionen): sechs Perk-Jingles, Spieluhr der Kiste,
//! Rundenwechsel, Game Over und Menümusik.

use crate::dsp::*;
use crate::world::loop_xfade;

#[derive(Clone, Copy)]
pub enum Inst {
    MusicBox,
    Organ,
    Pluck,
    Brass,
    Choir,
    Bass,
    Chip,
    Accordion,
    Tuba,
    DistGuitar,
}

/// Note: (Beat, MIDI-Note, Länge in Beats, Lautstärke)
pub type N = (f32, f32, f32, f32);

fn tone(inst: Inst, f: f32, d: f32, seed: u64) -> Vec<f32> {
    let tail = 0.6;
    match inst {
        Inst::MusicBox => {
            let mut x = silence(d + tail);
            for (r, g, tau) in [(1.0, 1.0, 0.9), (3.0, 0.25, 0.3), (5.98, 0.08, 0.15)] {
                x = mix(x, &env(osc(d + tail, move |_| f * r, Shape::Sine), decay(0.002, tau)), g);
            }
            x
        }
        Inst::Organ => {
            let mut x = silence(d + 0.1);
            for (r, g) in [(1.0, 1.0), (2.0, 0.5), (3.0, 0.3), (4.0, 0.2)] {
                x = mix(x, &osc(d + 0.1, move |t| f * r * (1.0 + 0.003 * (t * 6.0).sin()), Shape::Sine), g);
            }
            env(x, ahr(0.02, d - 0.02, 0.1))
        }
        Inst::Pluck => {
            // Karplus-Strong
            let mut rng = Rng::new(seed);
            let p = (SR / f) as usize;
            let mut buf: Vec<f32> = (0..p.max(2)).map(|_| rng.bi()).collect();
            let n = len(d + tail);
            let mut out = Vec::with_capacity(n);
            let mut i = 0;
            for _ in 0..n {
                let j = (i + 1) % buf.len();
                let v = (buf[i] + buf[j]) * 0.497;
                out.push(buf[i]);
                buf[i] = v;
                i = j;
            }
            out
        }
        Inst::Brass => {
            let x = osc(d + 0.15, move |t| f * (1.0 + 0.004 * (t * 5.5).sin()), Shape::Saw);
            let x = biquad_sweep(x, Filt::Low, move |t| 400.0 + 2200.0 * (t / 0.08).min(1.0) * (1.0 - 0.4 * (t / d).min(1.0)), 0.9);
            env(x, ahr(0.04, d - 0.04, 0.15))
        }
        Inst::Choir => {
            let mut x = silence(d + 0.6);
            for det in [-0.006f32, 0.0, 0.005] {
                x = mix(x, &osc(d + 0.6, move |t| f * (1.0 + det) * (1.0 + 0.006 * (t * 5.0).sin()), Shape::Saw), 0.4);
            }
            let x = formant(&x, [(700.0, 4.0, 1.0), (1150.0, 5.0, 0.5), (2700.0, 7.0, 0.2)]);
            env(x, ahr(0.3, d - 0.2, 0.6))
        }
        Inst::Bass => env(lowpass(osc(d + 0.05, move |_| f, Shape::Tri), 600.0), ahr(0.01, d - 0.05, 0.06)),
        Inst::Chip => env(osc(d, move |_| f, Shape::Pulse(0.25)), ahr(0.003, d * 0.7, d * 0.3)),
        Inst::Accordion => {
            let a = osc(d + 0.05, move |_| f, Shape::Square);
            let b = osc(d + 0.05, move |_| f * 1.004, Shape::Saw);
            env(lowpass(mix(a, &b, 0.7), 2500.0), ahr(0.03, d - 0.03, 0.05))
        }
        Inst::Tuba => env(lowpass(osc(d + 0.05, move |_| f, Shape::Saw), 380.0), ahr(0.03, d * 0.5, d * 0.4)),
        Inst::DistGuitar => {
            let a = osc(d + 0.2, move |_| f, Shape::Saw);
            let b = osc(d + 0.2, move |_| f * 1.5, Shape::Saw);
            env(lowpass(drive(mix(a, &b, 0.6), 6.0), 2400.0), ahr(0.005, d, 0.2))
        }
    }
}

pub fn render(notes: &[N], bpm: f32, inst: Inst, g: f32) -> Vec<f32> {
    let beat = 60.0 / bpm;
    let end = notes.iter().fold(0.0f32, |a, n| a.max(n.0 + n.2)) * beat + 1.0;
    let mut x = silence(end);
    for (i, (b, m, l, v)) in notes.iter().enumerate() {
        let t = tone(inst, midi(*m), l * beat, 7000 + i as u64);
        x = mix_at(x, &t, b * beat, v * g);
    }
    x
}

fn kick(at: &[f32], bpm: f32, d: f32) -> Vec<f32> {
    let beat = 60.0 / bpm;
    let mut x = silence(d);
    for b in at {
        let k = env(osc(0.3, |t| 45.0 + 110.0 * (-t / 0.03).exp(), Shape::Sine), decay(0.001, 0.12));
        x = mix_at(x, &k, b * beat, 1.0);
    }
    x
}

fn snare(at: &[f32], bpm: f32, d: f32, seed: u64) -> Vec<f32> {
    let beat = 60.0 / bpm;
    let mut rng = Rng::new(seed);
    let mut x = silence(d);
    for b in at {
        let s = env(bandpass(noise(0.3, &mut rng), 2500.0, 0.7), decay(0.001, 0.07));
        let t = env(osc(0.3, |_| 190.0, Shape::Tri), decay(0.001, 0.05));
        x = mix_at(x, &mix(s, &t, 0.5), b * beat, 0.8);
    }
    x
}

fn hats(at: &[f32], bpm: f32, d: f32, seed: u64) -> Vec<f32> {
    let beat = 60.0 / bpm;
    let mut rng = Rng::new(seed);
    let mut x = silence(d);
    for b in at {
        let h = env(highpass(noise(0.1, &mut rng), 7000.0), decay(0.0005, 0.02));
        x = mix_at(x, &h, b * beat, 0.35);
    }
    x
}

fn steps(from: f32, to: f32, step: f32) -> Vec<f32> {
    let mut v = Vec::new();
    let mut b = from;
    while b < to {
        v.push(b);
        b += step;
    }
    v
}

fn master(x: Vec<f32>, room: f32, wet: f32) -> Vec<f32> {
    fade(normalize(reverb(x, room, 0.4, wet, 1.0), 0.9), 0.005, 0.6)
}

pub fn all(out: &mut Vec<(&'static str, Vec<f32>)>) {
    // --- Zweiter Atem: sanfter Walzer (C-Dur), Spieluhr + Orgel ---
    {
        let bpm = 132.0;
        let mel: Vec<N> = vec![
            (0.0, 72.0, 1.0, 1.0), (1.0, 76.0, 1.0, 0.8), (2.0, 79.0, 1.0, 0.8), (3.0, 84.0, 2.0, 1.0), (5.0, 83.0, 1.0, 0.8),
            (6.0, 81.0, 1.0, 0.9), (7.0, 79.0, 1.0, 0.8), (8.0, 76.0, 1.0, 0.8), (9.0, 77.0, 2.0, 0.9), (11.0, 76.0, 1.0, 0.7),
            (12.0, 74.0, 1.0, 0.9), (13.0, 79.0, 1.0, 0.8), (14.0, 77.0, 1.0, 0.8), (15.0, 76.0, 1.5, 0.9), (16.5, 74.0, 0.5, 0.7), (17.0, 72.0, 3.0, 1.0),
        ];
        let chords: Vec<N> = [(0.0, 48.0), (3.0, 53.0), (6.0, 55.0), (9.0, 53.0), (12.0, 55.0), (15.0, 55.0), (17.0, 48.0)]
            .iter()
            .flat_map(|(b, r)| [(*b, *r, 3.0, 0.5), (*b, r + 4.0, 3.0, 0.35), (*b, r + 7.0, 3.0, 0.35)])
            .collect();
        let x = mix(render(&mel, bpm, Inst::MusicBox, 0.8), &render(&chords, bpm, Inst::Organ, 0.18), 1.0);
        out.push(("jingle_revive", master(x, 1.1, 0.3)));
    }
    // --- Panzer-Tonikum: schweres Riff (e-Moll), verzerrte Gitarre + Schlagzeug ---
    {
        let bpm = 100.0;
        let riff: Vec<N> = vec![
            (0.0, 40.0, 0.5, 1.0), (0.5, 40.0, 0.5, 0.8), (1.0, 43.0, 0.5, 1.0), (1.5, 40.0, 0.5, 0.8), (2.0, 45.0, 1.0, 1.0), (3.0, 43.0, 1.0, 0.9),
            (4.0, 40.0, 0.5, 1.0), (4.5, 40.0, 0.5, 0.8), (5.0, 43.0, 0.5, 1.0), (5.5, 46.0, 0.5, 0.9), (6.0, 45.0, 1.0, 1.0), (7.0, 47.0, 1.0, 1.0),
            (8.0, 40.0, 0.5, 1.0), (8.5, 40.0, 0.5, 0.8), (9.0, 43.0, 0.5, 1.0), (9.5, 40.0, 0.5, 0.8), (10.0, 38.0, 1.0, 1.0), (11.0, 35.0, 1.0, 1.0), (12.0, 40.0, 2.0, 1.0),
        ];
        let lead: Vec<N> = vec![(0.0, 64.0, 2.0, 0.7), (2.0, 67.0, 2.0, 0.7), (4.0, 69.0, 2.0, 0.7), (6.0, 71.0, 2.0, 0.8), (8.0, 72.0, 2.0, 0.7), (10.0, 71.0, 2.0, 0.7), (12.0, 64.0, 2.0, 0.9)];
        let d = 14.0 * 60.0 / bpm + 1.0;
        let mut x = render(&riff, bpm, Inst::DistGuitar, 0.6);
        x = mix(x, &render(&lead, bpm, Inst::Brass, 0.35), 1.0);
        x = mix(x, &kick(&steps(0.0, 14.0, 1.0), bpm, d), 0.8);
        x = mix(x, &snare(&steps(1.0, 14.0, 2.0), bpm, d, 7100), 0.6);
        out.push(("jingle_jugg", master(x, 0.9, 0.2)));
    }
    // --- Turbo-Cola: schneller Chiptune (G-Dur) ---
    {
        let bpm = 168.0;
        let mut mel: Vec<N> = Vec::new();
        let pattern = [79.0, 83.0, 86.0, 83.0, 81.0, 84.0, 88.0, 84.0, 79.0, 83.0, 86.0, 91.0, 90.0, 86.0, 83.0, 79.0];
        for (i, n) in pattern.iter().enumerate() {
            mel.push((i as f32 * 0.5, *n, 0.45, 0.8));
        }
        let tail = [(8.0, 86.0, 1.0), (9.0, 88.0, 1.0), (10.0, 90.0, 1.0), (11.0, 91.0, 3.0)];
        for (b, n, l) in tail {
            mel.push((b, n, l, 0.9));
        }
        let bass: Vec<N> = [(0.0, 43.0), (2.0, 48.0), (4.0, 43.0), (6.0, 50.0), (8.0, 47.0), (10.0, 50.0), (11.0, 43.0)].iter().map(|(b, n)| (*b, *n, 1.8, 0.8)).collect();
        let d = 14.0 * 60.0 / bpm + 1.0;
        let mut x = render(&mel, bpm, Inst::Chip, 0.4);
        x = mix(x, &render(&bass, bpm, Inst::Bass, 0.8), 1.0);
        x = mix(x, &hats(&steps(0.0, 14.0, 0.5), bpm, d, 7200), 1.0);
        x = mix(x, &kick(&steps(0.0, 14.0, 1.0), bpm, d), 0.6);
        out.push(("jingle_speed", master(x, 0.8, 0.2)));
    }
    // --- Doppelschuss-Bräu: Western-Zupfmusik (D-Dur) ---
    {
        let bpm = 120.0;
        let mel: Vec<N> = vec![
            (0.0, 62.0, 0.5, 1.0), (0.5, 66.0, 0.5, 0.8), (1.0, 69.0, 1.0, 1.0), (2.0, 74.0, 1.0, 0.9), (3.0, 73.0, 0.5, 0.8), (3.5, 71.0, 0.5, 0.8),
            (4.0, 69.0, 1.0, 1.0), (5.0, 66.0, 1.0, 0.8), (6.0, 67.0, 0.5, 0.9), (6.5, 69.0, 0.5, 0.8), (7.0, 71.0, 1.0, 0.9), (8.0, 69.0, 0.5, 0.8),
            (8.5, 67.0, 0.5, 0.8), (9.0, 66.0, 1.0, 0.9), (10.0, 64.0, 1.0, 0.8), (11.0, 62.0, 2.0, 1.0),
        ];
        let bass: Vec<N> = steps(0.0, 13.0, 1.0)
            .iter()
            .enumerate()
            .map(|(i, b)| (*b, if i % 2 == 0 { 50.0 } else { 57.0 }, 0.8, 0.8))
            .collect();
        let x = mix(render(&mel, bpm, Inst::Pluck, 0.9), &render(&bass, bpm, Inst::Pluck, 0.5), 1.0);
        out.push(("jingle_doubletap", master(x, 1.0, 0.25)));
    }
    // --- Ausdauer-Limo: hüpfender Synth-Pop mit Arpeggios (A-Dur) ---
    {
        let bpm = 140.0;
        let mut arp: Vec<N> = Vec::new();
        let chords = [[57.0, 61.0, 64.0, 69.0], [62.0, 66.0, 69.0, 74.0], [64.0, 68.0, 71.0, 76.0], [57.0, 61.0, 64.0, 69.0]];
        for (c, ch) in chords.iter().enumerate() {
            for k in 0..8 {
                arp.push((c as f32 * 4.0 + k as f32 * 0.5, ch[k % 4] + 12.0, 0.4, 0.6));
            }
        }
        let mel: Vec<N> = vec![(0.0, 81.0, 1.5, 0.9), (1.5, 80.0, 0.5, 0.8), (2.0, 78.0, 2.0, 0.9), (4.0, 78.0, 1.5, 0.9), (5.5, 76.0, 0.5, 0.8), (6.0, 74.0, 2.0, 0.9), (8.0, 76.0, 1.0, 0.9), (9.0, 80.0, 1.0, 0.9), (10.0, 83.0, 2.0, 1.0), (12.0, 81.0, 4.0, 1.0)];
        let d = 16.0 * 60.0 / bpm + 1.0;
        let mut x = render(&arp, bpm, Inst::Chip, 0.25);
        x = mix(x, &render(&mel, bpm, Inst::Organ, 0.3), 1.0);
        x = mix(x, &kick(&steps(0.0, 16.0, 1.0), bpm, d), 0.6);
        x = mix(x, &snare(&steps(1.0, 16.0, 2.0), bpm, d, 7300), 0.4);
        out.push(("jingle_stamin", master(x, 0.9, 0.25)));
    }
    // --- Maultier-Malz: Polka (F-Dur), Tuba + Akkordeon ---
    {
        let bpm = 150.0;
        let mut tuba: Vec<N> = Vec::new();
        let roots = [41.0, 48.0, 41.0, 48.0, 46.0, 41.0, 48.0, 41.0];
        for (i, r) in roots.iter().enumerate() {
            tuba.push((i as f32 * 2.0, *r, 0.9, 1.0));
        }
        let mut chords: Vec<N> = Vec::new();
        for i in 0..8 {
            let r = [65.0, 64.0, 65.0, 64.0, 70.0, 65.0, 64.0, 65.0][i];
            chords.push((i as f32 * 2.0 + 1.0, r, 0.6, 0.6));
            chords.push((i as f32 * 2.0 + 1.0, r + 4.0, 0.6, 0.5));
        }
        let mel: Vec<N> = vec![
            (0.0, 77.0, 1.0, 1.0), (1.0, 79.0, 0.5, 0.8), (1.5, 81.0, 0.5, 0.8), (2.0, 84.0, 2.0, 1.0), (4.0, 82.0, 1.0, 0.9), (5.0, 81.0, 1.0, 0.8),
            (6.0, 79.0, 2.0, 0.9), (8.0, 82.0, 1.0, 0.9), (9.0, 81.0, 1.0, 0.8), (10.0, 79.0, 1.0, 0.8), (11.0, 76.0, 1.0, 0.8), (12.0, 77.0, 3.0, 1.0),
        ];
        let mut x = render(&tuba, bpm, Inst::Tuba, 0.9);
        x = mix(x, &render(&chords, bpm, Inst::Accordion, 0.25), 1.0);
        x = mix(x, &render(&mel, bpm, Inst::Accordion, 0.35), 1.0);
        out.push(("jingle_mule", master(x, 0.9, 0.2)));
    }
    // --- Spieluhr der Kiste (a-Moll, leicht schief) ---
    {
        let bpm = 150.0;
        let mel: Vec<N> = vec![
            (0.0, 81.0, 1.0, 1.0), (1.0, 84.0, 1.0, 0.8), (2.0, 88.0, 1.0, 0.9), (3.0, 87.0, 1.0, 0.8), (4.0, 84.0, 1.0, 0.9), (5.0, 81.0, 1.0, 0.8),
            (6.0, 80.0, 1.0, 0.9), (7.0, 83.0, 1.0, 0.8), (8.0, 86.0, 2.0, 1.0),
        ];
        let x = render(&mel, bpm, Inst::MusicBox, 0.8);
        let x = resample(&x, 1.0 - 0.01);
        out.push(("box_music", master(x, 1.0, 0.35)));
    }
    // --- Rundenbeginn: dunkler Chor schwillt an, Pauke, Glocke ---
    {
        let ch: Vec<N> = vec![(0.0, 45.0, 4.0, 0.9), (0.0, 52.0, 4.0, 0.7), (0.0, 57.0, 4.0, 0.7), (0.0, 60.0, 4.0, 0.6)];
        let mut x = render(&ch, 60.0, Inst::Choir, 0.6);
        x = mix(x, &kick(&[0.0, 1.5, 3.0], 60.0, 5.0), 0.9);
        let b = env(osc(4.0, |_| midi(69.0), Shape::Sine), decay(0.002, 1.2));
        x = mix_at(x, &b, 3.0, 0.3);
        out.push(("round_start", master(x, 1.5, 0.45)));
    }
    // --- Rundenende: absteigende Akkorde (Chor + verzerrte Gitarre) ---
    {
        let prog = [(0.0, 52.0), (2.0, 50.0), (4.0, 48.0), (6.0, 47.0)];
        let mut ch: Vec<N> = Vec::new();
        let mut gt: Vec<N> = Vec::new();
        for (b, r) in prog {
            ch.push((b, r, 2.2, 0.8));
            ch.push((b, r + 3.0, 2.2, 0.6));
            ch.push((b, r + 7.0, 2.2, 0.6));
            gt.push((b, r - 12.0, 1.8, 0.8));
        }
        let mut x = render(&ch, 72.0, Inst::Choir, 0.5);
        x = mix(x, &render(&gt, 72.0, Inst::DistGuitar, 0.35), 1.0);
        x = mix(x, &kick(&[0.0, 2.0, 4.0, 6.0], 72.0, 8.0), 0.8);
        out.push(("round_end", master(x, 1.4, 0.4)));
    }
    // --- Game Over: langsame, traurige Folge ---
    {
        let prog = [(0.0, 45.0, 0.0), (3.0, 41.0, 4.0), (6.0, 43.0, 4.0), (9.0, 40.0, 3.0)];
        let mut ch: Vec<N> = Vec::new();
        for (b, r, _) in prog {
            ch.push((b, r, 3.2, 0.8));
            ch.push((b, r + 3.0, 3.2, 0.6));
            ch.push((b, r + 7.0, 3.2, 0.6));
        }
        let mel: Vec<N> = vec![(0.0, 76.0, 2.0, 0.7), (2.0, 72.0, 1.0, 0.6), (3.0, 74.0, 3.0, 0.7), (6.0, 71.0, 3.0, 0.7), (9.0, 69.0, 4.0, 0.8)];
        let mut x = render(&ch, 60.0, Inst::Organ, 0.25);
        x = mix(x, &render(&mel, 60.0, Inst::MusicBox, 0.6), 1.0);
        out.push(("game_over", master(x, 1.5, 0.45)));
    }
    // --- Menü: dunkle Fläche mit vereinzelten Glockentönen (Schleife) ---
    {
        let d = 40.0;
        let mut x = silence(d);
        for (f, g) in [(55.0, 0.5), (82.4, 0.3), (110.0, 0.25), (130.8, 0.15)] {
            let o = osc(d, move |t| f * (1.0 + 0.002 * (t * 0.3 + f).sin()), Shape::Saw);
            x = mix(x, &lowpass(o, 400.0), g);
        }
        x = env(x, |t, _| 0.7 + 0.3 * (t * 0.11).sin());
        let mut rng = Rng::new(7400);
        for k in 0..12 {
            let n = [69.0, 72.0, 76.0, 74.0, 81.0, 79.0][rng.next_u32() as usize % 6];
            let b = tone(Inst::MusicBox, midi(n), 0.6, 7400 + k);
            x = mix_at(x, &b, k as f32 * 3.2 + rng.range(0.0, 1.0), 0.25);
        }
        let x = reverb(x, 1.6, 0.4, 0.5, 0.0);
        out.push(("music_menu", loop_xfade(normalize(x, 0.6), 3.0)));
    }
}
