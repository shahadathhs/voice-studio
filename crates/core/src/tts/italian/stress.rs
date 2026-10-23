//! Finding the syllable nuclei of an Italian word and the one that is stressed. The stress mark
//! goes in front of the head vowel of the stressed syllable.

use super::letters::{Unit, Vowel};

/// Endings that put the stress on the third syllable from the end (possibile, amabile, utile).
const ANTEPENULT_ENDINGS: [&str; 3] = ["abile", "ibile", "evole"];

fn weak(v: &Vowel) -> bool {
    matches!(v.base, 'i' | 'u') && !v.stressed
}

/// Whether `next` joins the syllable of `last`: an unstressed i or u next to another vowel is a
/// glide; two strong vowels, or a doubled weak one, are separate syllables.
fn joins(last: &Vowel, next: &Vowel) -> bool {
    match (weak(last), weak(next)) {
        (false, false) => false,
        (true, true) => last.base != next.base,
        _ => true,
    }
}

fn vowel_at(units: &[Unit], index: usize) -> Option<&Vowel> {
    match units.get(index) {
        Some(Unit::Vowel(v)) => Some(v),
        _ => None,
    }
}

/// The unit indexes of the vowels of each syllable.
pub fn nuclei(units: &[Unit]) -> Vec<Vec<usize>> {
    let mut found: Vec<Vec<usize>> = Vec::new();
    for index in 0..units.len() {
        let Some(vowel) = vowel_at(units, index) else {
            continue;
        };
        let before = index.checked_sub(1).and_then(|i| vowel_at(units, i));
        match found.last_mut() {
            Some(last) if before.is_some_and(|b| joins(b, vowel)) => last.push(index),
            _ => found.push(vec![index]),
        }
    }
    found
}

/// The vowel of a nucleus that is said as a vowel; the others are glides (j or w).
pub fn head(units: &[Unit], nucleus: &[usize]) -> usize {
    let vowel = |i: &usize| vowel_at(units, *i);
    nucleus
        .iter()
        .copied()
        .find(|i| vowel(i).is_some_and(|v| v.stressed))
        .or_else(|| {
            nucleus
                .iter()
                .copied()
                .find(|i| vowel(i).is_some_and(|v| !weak(v)))
        })
        .or_else(|| nucleus.last().copied())
        .unwrap_or(0)
}

/// Which nucleus is stressed: the accented one, else the last but one (the last if the word ends
/// in a consonant, the last but two for a few endings).
pub fn stressed(units: &[Unit], nuclei: &[Vec<usize>], spelling: &str) -> usize {
    let accented = nuclei.iter().position(|n| {
        n.iter()
            .any(|i| vowel_at(units, *i).is_some_and(|v| v.stressed))
    });
    if let Some(n) = accented {
        return n;
    }
    let count = nuclei.len();
    if matches!(units.last(), Some(Unit::Consonant(_))) {
        count.saturating_sub(1)
    } else if count > 2 && ANTEPENULT_ENDINGS.iter().any(|e| spelling.ends_with(e)) {
        count - 3
    } else {
        count.saturating_sub(2)
    }
}
