//! Erzeugt alle Geräusche und Musikstücke von Totenrunde per Synthese
//! (Ansagerstimme über espeak-ng) -> assets/sounds/*.wav
//!
//! Aufruf: cargo run --release --bin sfxgen [-- name ...]

mod dsp;
mod music;
mod voices;
mod weapons;
mod world;

use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = root.join("assets").join("sounds");
    std::fs::create_dir_all(&out_dir).expect("Ordner assets/sounds");
    let tmp = std::env::temp_dir();
    let only: Vec<String> = std::env::args().skip(1).collect();

    let mut all: Vec<(&'static str, Vec<f32>)> = Vec::new();
    weapons::all(&mut all);
    voices::zombies(&mut all);
    voices::announcer(&mut all, &tmp);
    world::all(&mut all);
    all.push(("amb_wind", world::wind()));
    all.push(("fire_loop", world::fire()));
    music::all(&mut all);

    let mut count = 0;
    for (name, samples) in &all {
        if !only.is_empty() && !only.iter().any(|o| o == name) {
            continue;
        }
        dsp::write_wav(&out_dir.join(format!("{name}.wav")), samples);
        println!("{name:>18}  {:.2} s", samples.len() as f32 / dsp::SR);
        count += 1;
    }
    println!("{count} Dateien in {}", out_dir.display());
}
