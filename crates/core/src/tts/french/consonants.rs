//! French consonant letters to sounds.

use super::stem::{Word, is_vowel};

/// Whether the letter at `i` makes a preceding `c`, `g` or `s` soft (a silent final `e` counts).
fn soft(w: &Word, i: usize) -> bool {
    matches!(
        w.raw(i),
        Some('e' | 'i' | 'y' | 'é' | 'è' | 'ê' | 'î' | 'ï')
    )
}

fn c_sound(w: &Word, i: usize) -> (&'static str, usize) {
    match w.at(i + 1) {
        Some('h') => ("ʃ", 2),
        Some('c') if soft(w, i + 2) => ("ks", 2),
        Some('c' | 'k') => ("k", 2),
        _ if soft(w, i + 1) => ("s", 1),
        _ => ("k", 1),
    }
}

fn g_sound(w: &Word, i: usize) -> (&'static str, usize) {
    match (w.at(i + 1), w.at(i + 2)) {
        (Some('n'), _) => ("ɲ", 2),
        (Some('u'), _) if soft(w, i + 2) => ("g", 2),
        (Some('e'), Some('a' | 'o' | 'u' | 'à' | 'â' | 'ô')) => ("ʒ", 2),
        _ if soft(w, i + 1) => ("ʒ", 1),
        _ => ("g", 1),
    }
}

fn s_sound(w: &Word, i: usize) -> (&'static str, usize) {
    let between = w.before(i).is_some_and(is_vowel) && w.raw(i + 1).is_some_and(is_vowel);
    match w.at(i + 1) {
        Some('s') => ("s", 2),
        Some('h') => ("ʃ", 2),
        Some('c') if soft(w, i + 2) => ("s", 2),
        _ if between => ("z", 1),
        _ => ("s", 1),
    }
}

/// `tion` is said sjɔ̃ except after s or x (question).
fn t_sound(w: &Word, i: usize) -> (&'static str, usize) {
    let tion = w.starts(i, "tion") && !matches!(w.before(i), Some('s' | 'x'));
    let nasal = w
        .raw(i + 4)
        .is_none_or(|c| !is_vowel(c) && !matches!(c, 'n' | 'm'));
    match w.at(i + 1) {
        _ if tion && nasal => ("sjɔ̃", 4),
        Some('h' | 't') => ("t", 2),
        _ => ("t", 1),
    }
}

fn x_sound(w: &Word, i: usize) -> (&'static str, usize) {
    let after_initial_e = i == 1 && w.at(0) == Some('e');
    if after_initial_e && w.raw(i + 1).is_some_and(|c| is_vowel(c) || c == 'h') {
        ("gz", 1)
    } else {
        ("ks", 1)
    }
}

fn q_sound(w: &Word, i: usize) -> (&'static str, usize) {
    ("k", if w.at(i + 1) == Some('u') { 2 } else { 1 })
}

/// The sound of the consonant letter at `i` and how many letters it covers.
pub fn consonant(w: &Word, i: usize, c: char) -> (String, usize) {
    let doubled = w.at(i + 1) == Some(c) && !matches!(c, 'c' | 'g');
    let (sound, used) = match c {
        'c' => c_sound(w, i),
        'ç' => ("s", 1),
        'g' => g_sound(w, i),
        's' => s_sound(w, i),
        't' => t_sound(w, i),
        'x' => x_sound(w, i),
        'q' => q_sound(w, i),
        'p' if w.at(i + 1) == Some('h') => ("f", 2),
        'h' => ("", 1),
        'j' => ("ʒ", 1),
        'r' => ("ʁ", if doubled { 2 } else { 1 }),
        'w' => ("v", 1),
        'k' => ("k", 1),
        _ => return (c.to_string(), if doubled { 2 } else { 1 }),
    };
    (sound.to_string(), used)
}
