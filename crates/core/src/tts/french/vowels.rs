//! French vowel letters to sounds: nasal vowels, digraphs and the few single letters that
//! depend on their neighbours.

use super::stem::{Word, is_vowel};

/// Spellings of the nasal vowels; each ends in the `n` or `m` that is not said.
const NASALS: [(&str, &str); 18] = [
    ("oin", "wɛ̃"),
    ("ien", "jɛ̃"),
    ("ain", "ɛ̃"),
    ("aim", "ɛ̃"),
    ("ein", "ɛ̃"),
    ("eim", "ɛ̃"),
    ("un", "œ̃"),
    ("um", "œ̃"),
    ("yn", "ɛ̃"),
    ("ym", "ɛ̃"),
    ("in", "ɛ̃"),
    ("im", "ɛ̃"),
    ("on", "ɔ̃"),
    ("om", "ɔ̃"),
    ("an", "ɑ̃"),
    ("am", "ɑ̃"),
    ("en", "ɑ̃"),
    ("em", "ɑ̃"),
];

/// A nasal vowel stands where the `n` or `m` ends the word or is followed by another consonant.
fn nasal(w: &Word, i: usize) -> Option<(String, usize)> {
    NASALS.iter().find_map(|(letters, sound)| {
        let used = letters.chars().count();
        let free = w
            .raw(i + used)
            .is_none_or(|c| !is_vowel(c) && !matches!(c, 'n' | 'm'));
        (w.starts(i, letters) && free).then(|| ((*sound).to_string(), used))
    })
}

/// Whether an `ill` (or a final `il`) with a glide follows the letter at `i`.
pub fn ill_at(w: &Word, i: usize) -> bool {
    w.at(i) == Some('i') && w.at(i + 1) == Some('l') && (w.at(i + 2) == Some('l') || i + 2 == w.len)
}

/// `eu` is closed (ø) at the end of a word or before z, and open (œ) before other consonants.
fn eu(w: &Word, after: usize) -> &'static str {
    match (w.at(after), w.at(after + 1)) {
        (None | Some('z' | 'x'), _) | (Some('s'), None) => "ø",
        _ => "œ",
    }
}

fn digraph(w: &Word, i: usize) -> Option<(String, usize)> {
    let (first, second) = (w.at(i)?, w.at(i + 1)?);
    let third = w.at(i + 2);
    let glide_after = third.is_some_and(is_vowel);
    let ends_in_i = second == 'i' && ill_at(w, i + 1);
    let found: (&str, usize) = match (first, second) {
        ('e', 'a') if third == Some('u') => ("o", 3),
        ('o', 'e') if third == Some('u') => (eu(w, i + 3), 3),
        ('œ' | 'e', 'u') => (eu(w, i + 2), 2),
        ('a', 'u') => ("o", 2),
        ('o', 'u') if glide_after => ("w", 2),
        ('o', 'u') => ("u", 2),
        ('o', 'y') if glide_after => ("waj", 2),
        ('a', 'y') if glide_after => ("ɛj", 2),
        ('u', 'y') if glide_after => ("ɥij", 2),
        _ if ends_in_i => return None,
        ('o', 'i') => ("wa", 2),
        ('a', 'i') if third.is_none() && !w.closed => ("e", 2),
        ('a' | 'e', 'i') => ("ɛ", 2),
        ('u', 'i') => ("ɥi", 2),
        _ => return None,
    };
    Some((found.0.to_string(), found.1))
}

/// The vowel `e` is said ə in an open syllable and ɛ in a closed one.
fn plain_e(w: &Word, i: usize) -> &'static str {
    let rest: Vec<char> = (i + 1..w.len)
        .map_while(|k| w.at(k).filter(|c| !is_vowel(*c)))
        .collect();
    if w.vowel_at(i + 1) {
        return if ill_at(w, i + 1) { "ɛ" } else { "ə" };
    }
    let ends_word = i + 1 + rest.len() >= w.len;
    let stop_liquid = matches!(rest.as_slice(), [a, 'l' | 'r'] if "bcdfgkptv".contains(*a));
    match rest.len() {
        0 if ends_word => "ə",
        1 if !ends_word && rest[0] != 'x' => "ə",
        2 if stop_liquid && !ends_word => "ə",
        _ => "ɛ",
    }
}

fn plain_o(w: &Word, i: usize) -> &'static str {
    let before_z = matches!(w.raw(i + 1), Some('s' | 'z')) && w.raw(i + 2).is_some_and(is_vowel);
    if w.at(i + 1).is_none() || before_z {
        "o"
    } else {
        "ɔ"
    }
}

/// `i` and `u` before another vowel are glides (pied, nuage).
fn glide(w: &Word, i: usize, plain: &'static str, glide: &'static str) -> String {
    let next = w.vowel_at(i + 1);
    let cluster = i >= 2 && matches!(w.at(i - 1), Some('l' | 'r')) && w.consonant_at(i - 2);
    match (next, cluster) {
        (false, _) => plain.into(),
        (true, true) if plain == "i" => "ij".into(),
        _ => glide.into(),
    }
}

fn single(w: &Word, i: usize, c: char) -> String {
    match c {
        'a' | 'à' | 'â' | 'ä' => "a".into(),
        'e' => plain_e(w, i).into(),
        'é' | 'æ' => "e".into(),
        'è' | 'ê' | 'ë' => "ɛ".into(),
        'i' => glide(w, i, "i", "j"),
        'o' => plain_o(w, i).into(),
        'ô' | 'ö' => "o".into(),
        'u' | 'û' | 'ù' | 'ü' => glide(w, i, "y", "ɥ"),
        'œ' => "œ".into(),
        'y' if w.vowel_at(i + 1) && !w.vowel_at(i.wrapping_sub(1)) => "j".into(),
        _ => "i".into(),
    }
}

/// The sound of the vowel letter at `i` and how many letters it covers.
pub fn vowel(w: &Word, i: usize, c: char) -> (String, usize) {
    if ill_at(w, i) && i > 0 {
        let after_vowel = w.vowel_at(i - 1);
        let used = if w.at(i + 2) == Some('l') { 3 } else { 2 };
        return (if after_vowel { "j" } else { "ij" }.into(), used);
    }
    nasal(w, i)
        .or_else(|| digraph(w, i))
        .unwrap_or_else(|| (single(w, i, c), 1))
}
