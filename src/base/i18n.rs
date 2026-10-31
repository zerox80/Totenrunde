//! Übersetzungen. Jede Sprache steht in `src/base/lang/<code>.txt`: eine Zeile pro Text,
//! `schlüssel = Text`, `\n` für Zeilenumbrüche und `{name}` für Platzhalter. Fehlt ein
//! Schlüssel, gilt der englische Text.
//!
//! Die Sprache folgt der Einstellung `language` (Sprachcode oder "auto"); bei "auto" der
//! Browser- bzw. Systemsprache und ohne passende Übersetzung Englisch. Zum Testen
//! erzwingt `TR_LANG=fr` eine Sprache auf dem Desktop.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::prelude::*;

pub struct Language {
    pub code: &'static str,
    /// Eigenname für das Menü
    pub name: &'static str,
    source: &'static str,
}

macro_rules! languages {
    ($($code:literal => $name:literal),* $(,)?) => {
        /// Alle Sprachen; die Schrift Montserrat deckt Latein und Kyrillisch ab.
        pub const LANGUAGES: &[Language] = &[$(Language {
            code: $code,
            name: $name,
            source: include_str!(concat!("lang/", $code, ".txt")),
        }),*];
    };
}

languages! {
    "de" => "DEUTSCH",
    "en" => "ENGLISH",
    "fr" => "FRANÇAIS",
    "es" => "ESPAÑOL",
    "it" => "ITALIANO",
    "pt" => "PORTUGUÊS",
    "nl" => "NEDERLANDS",
    "pl" => "POLSKI",
    "cs" => "ČEŠTINA",
    "tr" => "TÜRKÇE",
    "ru" => "РУССКИЙ",
    "uk" => "УКРАЇНСЬКА",
}

const ENGLISH: usize = 1;
static CURRENT: AtomicUsize = AtomicUsize::new(ENGLISH);
static TABLES: OnceLock<Vec<HashMap<&'static str, &'static str>>> = OnceLock::new();

fn tables() -> &'static [HashMap<&'static str, &'static str>] {
    TABLES.get_or_init(|| LANGUAGES.iter().map(|l| parse(l.source)).collect())
}

fn parse(source: &'static str) -> HashMap<&'static str, &'static str> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let value = value.trim();
            let value: &'static str = if value.contains("\\n") {
                Box::leak(value.replace("\\n", "\n").into_boxed_str())
            } else {
                value
            };
            Some((key.trim(), value))
        })
        .collect()
}

/// Wählt die Sprache nach der Einstellung (Sprachcode oder "auto").
pub fn set(setting: &str) {
    let index = LANGUAGES
        .iter()
        .position(|l| l.code == setting)
        .unwrap_or_else(detect);
    CURRENT.store(index, Ordering::Relaxed);
}

pub fn current() -> &'static Language {
    &LANGUAGES[CURRENT.load(Ordering::Relaxed)]
}

/// Text zum Schlüssel in der aktuellen Sprache, sonst Englisch.
pub fn lookup(key: &str) -> Option<&'static str> {
    let tables = tables();
    tables[CURRENT.load(Ordering::Relaxed)]
        .get(key)
        .or_else(|| tables[ENGLISH].get(key))
        .copied()
}

/// Wie [`lookup`]; ein fehlender Schlüssel erscheint unübersetzt.
pub fn t(key: &'static str) -> &'static str {
    lookup(key).unwrap_or(key)
}

/// Text mit Platzhaltern: `tf("prompt.buy", &[("item", &name), ("cost", &cost)])`.
pub fn tf(key: &'static str, args: &[(&str, &dyn Display)]) -> String {
    let mut text = t(key).to_owned();
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), &value.to_string());
    }
    text
}

fn detect() -> usize {
    preferred_locales()
        .iter()
        .find_map(|locale| {
            let primary = locale.split(['-', '_']).next()?.to_ascii_lowercase();
            LANGUAGES.iter().position(|l| l.code == primary)
        })
        .unwrap_or(ENGLISH)
}

#[cfg(target_arch = "wasm32")]
fn preferred_locales() -> Vec<String> {
    let Some(navigator) = web_sys::window().map(|window| window.navigator()) else {
        return Vec::new();
    };
    let mut locales: Vec<String> = navigator
        .languages()
        .iter()
        .filter_map(|value| value.as_string())
        .collect();
    if locales.is_empty() {
        locales.extend(navigator.language());
    }
    locales
}

#[cfg(not(target_arch = "wasm32"))]
fn preferred_locales() -> Vec<String> {
    if let Ok(locale) = std::env::var("TR_LANG") {
        return vec![locale];
    }
    sys_locale::get_locales().collect()
}

/// Text, der beim Sprachwechsel neu übersetzt wird.
#[derive(Component)]
pub struct Translated(pub &'static str);

pub struct I18nPlugin;

impl Plugin for I18nPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, retranslate);
    }
}

fn retranslate(mut shown: Local<Option<usize>>, mut texts: Query<(&Translated, &mut Text)>) {
    let index = CURRENT.load(Ordering::Relaxed);
    if *shown == Some(index) {
        return;
    }
    *shown = Some(index);
    for (key, mut text) in &mut texts {
        let translated = t(key.0);
        if text.0 != translated {
            text.0 = translated.to_owned();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LANGUAGES, parse};

    fn placeholders(text: &str) -> Vec<&str> {
        let mut found: Vec<&str> = text
            .split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect();
        found.sort_unstable();
        found
    }

    /// Jede Sprache übersetzt alle Texte der deutschen Vorlage mit denselben Platzhaltern.
    #[test]
    fn all_languages_are_complete() {
        let reference = parse(LANGUAGES[0].source);
        for language in LANGUAGES {
            let table = parse(language.source);
            for (key, text) in &reference {
                let translated = table
                    .get(key)
                    .unwrap_or_else(|| panic!("{}: Schlüssel {key} fehlt", language.code));
                assert_eq!(
                    placeholders(text),
                    placeholders(translated),
                    "{}: Platzhalter in {key}",
                    language.code
                );
            }
            assert_eq!(
                table.len(),
                reference.len(),
                "{}: unbekannte Schlüssel",
                language.code
            );
        }
    }
}
