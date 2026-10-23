//! A French word to phonemes: lexicon or spelling rules, elision, and the stress on the last
//! syllable.

use super::consonants::consonant;
use super::lexicon;
use super::stem::{Word, is_vowel, prepare};
use super::vowels::vowel;

const STRESS: char = 'ˈ';
const VOWEL_SOUNDS: &str = "aeiouyɑɛɔøœə";

fn letters_to_sounds(w: &Word) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < w.len {
        let Some(c) = w.at(i) else { break };
        let (sound, used) = if is_vowel(c) {
            vowel(w, i, c)
        } else {
            consonant(w, i, c)
        };
        out.push_str(&sound);
        i += used.max(1);
    }
    out
}

/// The sounds of one word without stress.
fn plain(word: &str) -> String {
    lexicon::spelled(word).map_or_else(|| letters_to_sounds(&prepare(word)), str::to_string)
}

/// The mark goes before the last vowel of a word of more than one syllable.
fn stressed(sounds: &str) -> String {
    let vowels: Vec<usize> = sounds
        .char_indices()
        .filter(|(_, c)| VOWEL_SOUNDS.contains(*c))
        .map(|(at, _)| at)
        .collect();
    match vowels.as_slice() {
        [_, .., last] => {
            format!("{}{STRESS}{}", &sounds[..*last], &sounds[*last..])
        }
        _ => sounds.to_string(),
    }
}

/// l', d', j', qu'... said as the start of the next word.
fn elided(part: &str) -> String {
    match part {
        "c" => "s".into(),
        "j" => "ʒ".into(),
        "qu" => "k".into(),
        single if single.chars().count() == 1 => single.to_string(),
        longer => plain(&format!("{longer}e")),
    }
}

/// The phonemes of a French word, with a stress mark in front of its last vowel.
pub fn word(word: &str) -> String {
    // The model's alphabet has the script g (ɡ), not the plain one.
    spoken(word).replace('g', "ɡ")
}

fn spoken(word: &str) -> String {
    let lower = word.to_lowercase();
    if let Some(known) = lexicon::spelled(&lower) {
        return stressed(known);
    }
    let parts: Vec<&str> = lower.split('\'').filter(|p| !p.is_empty()).collect();
    let Some((last, before)) = parts.split_last() else {
        return String::new();
    };
    let prefix: String = before.iter().map(|p| elided(p)).collect();
    format!("{prefix}{}", stressed(&plain(last)))
}
