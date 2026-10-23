//! The sound of each vowel unit: quality from the accent, stress and position.

use super::units::{Mark, Unit, is_vowel};

pub const TILDE: char = '\u{303}';

/// What the vowel rules need to know about the word.
pub struct Stress {
    /// Unit index of each syllable's head vowel.
    pub heads: Vec<usize>,
    /// Unit index of the stressed head vowel.
    pub at: usize,
    /// A small unstressed word (a, o, de, que…): its vowel is reduced as after the stress.
    pub reduced: bool,
}

fn nasal_of(base: char, mark: Mark) -> String {
    let vowel = match (base, mark) {
        ('a', _) => 'ɐ',
        ('e', Mark::Acute) => 'ɛ',
        ('o', Mark::Acute) => 'ɔ',
        (other, _) => other,
    };
    format!("{vowel}{TILDE}")
}

/// Nothing or just a plural s comes after unit `i`.
fn word_final(units: &[Unit], i: usize) -> bool {
    let rest = &units[i + 1..];
    rest.is_empty() || (rest.len() == 1 && rest[0].text == "s")
}

fn nasal(units: &[Unit], i: usize) -> String {
    let u = &units[i];
    let ending = word_final(units, i) && u.consumed == Some('m');
    match u.base {
        'e' if ending => format!("e{TILDE}j{TILDE}"),
        'a' if ending && u.mark == Mark::Plain => format!("ɐ{TILDE}w{TILDE}"),
        base => nasal_of(base, u.mark),
    }
}

fn glide(units: &[Unit], i: usize) -> String {
    let u = &units[i];
    let sound = match u.base {
        'i' | 'e' => "j".to_string(),
        'u' | 'o' => "w".to_string(),
        other => other.to_string(),
    };
    let after_nasal = i > 0 && is_vowel(&units[i - 1]) && units[i - 1].nasal;
    if after_nasal {
        format!("{sound}{TILDE}")
    } else {
        sound
    }
}

/// A stressed unmarked e or o is open before r, rr and l (terra, porta, sol, vez).
fn open(units: &[Unit], i: usize) -> bool {
    units.get(i + 1).is_some_and(|n| {
        matches!(n.text.as_str(), "r" | "rr" | "l") || (n.text == "z" && i + 2 == units.len())
    })
}

fn oral(units: &[Unit], i: usize, stress: &Stress) -> String {
    let u = &units[i];
    let after = stress.reduced || i > stress.at;
    let stressed = i == stress.at && !stress.reduced;
    let es = i == 0 && i < stress.at && units.get(1).is_some_and(|n| n.text == "s");
    match (u.base, u.mark) {
        ('a', Mark::Circumflex) => "ɐ",
        ('a', Mark::Plain) if after => "ɐ",
        ('e', Mark::Acute) => "ɛ",
        ('e', Mark::Plain) if after || es => "i",
        ('e', Mark::Plain) if stressed && open(units, i) => "ɛ",
        ('o', Mark::Acute) => "ɔ",
        ('o', Mark::Plain) if after => "u",
        ('o', Mark::Plain) if stressed && open(units, i) => "ɔ",
        (other, _) => return other.to_string(),
    }
    .to_string()
}

/// The sound of vowel unit `i`.
pub fn sound(units: &[Unit], i: usize, stress: &Stress) -> String {
    let u = &units[i];
    if !stress.heads.contains(&i) {
        return glide(units, i);
    }
    // A weak vowel after the stress and before another vowel (história, rio) is a glide.
    let hiatus = matches!(u.base, 'i' | 'u')
        && u.mark == Mark::Plain
        && !u.nasal
        && i > stress.at
        && units.get(i + 1).is_some_and(is_vowel);
    if hiatus {
        return glide(units, i);
    }
    if u.nasal {
        nasal(units, i)
    } else {
        oral(units, i, stress)
    }
}
