//! Finding the syllables of a Portuguese word and the one that is stressed.

use super::units::{Mark, Unit, is_vowel};

fn is_weak(unit: &Unit) -> bool {
    matches!(unit.base, 'i' | 'u') && unit.mark == Mark::Plain && !unit.nasal
}

fn accented(unit: &Unit) -> bool {
    matches!(unit.mark, Mark::Acute | Mark::Circumflex)
}

fn accented_weak(unit: &Unit) -> bool {
    matches!(unit.base, 'i' | 'u') && accented(unit)
}

/// Whether `next` joins the syllable of `last`: a weak vowel after another vowel (ai, eu, ui),
/// a weak vowel before a strong one inside a word (the `ia` of `diante`), or the e/o glide after
/// a nasal `ã`/`õ` (mãe, ão, põe). Word-final `ia`, `io`, `ua` are two syllables.
fn joins(last: &Unit, next: &Unit, word_end: bool) -> bool {
    if accented_weak(last) || accented_weak(next) {
        return false;
    }
    if last.mark == Mark::Tilde && matches!(next.base, 'e' | 'o') && next.mark == Mark::Plain {
        return true;
    }
    match (is_weak(last), is_weak(next)) {
        (_, true) => !(is_weak(last) && last.base == next.base),
        (true, false) => !word_end,
        (false, false) => false,
    }
}

/// The vowels (indexes into `units`) that make up each syllable.
pub fn nuclei(units: &[Unit]) -> Vec<Vec<usize>> {
    let end = units.len();
    let mut found: Vec<Vec<usize>> = Vec::new();
    for (index, unit) in units.iter().enumerate() {
        if !is_vowel(unit) {
            continue;
        }
        let word_end = index + 1 == end || (index + 2 == end && units[end - 1].text == "s");
        let adjacent = index > 0 && is_vowel(&units[index - 1]);
        let joined = match found.last() {
            Some(last) if adjacent => joins(&units[last[last.len() - 1]], unit, word_end),
            _ => false,
        };
        match found.last_mut() {
            Some(last) if joined => last.push(index),
            _ => found.push(vec![index]),
        }
    }
    found
}

/// The vowel of a nucleus that is said as a vowel; the others are glides.
pub fn head(units: &[Unit], nucleus: &[usize]) -> usize {
    let find = |test: &dyn Fn(&Unit) -> bool| nucleus.iter().copied().find(|i| test(&units[*i]));
    find(&accented)
        .or_else(|| find(&|u| u.mark == Mark::Tilde))
        .or_else(|| find(&|u| !is_weak(u)))
        .or_else(|| nucleus.first().copied())
        .unwrap_or(0)
}

/// The word's letters without accents or the silent parts, for reading its ending.
fn spelling(units: &[Unit]) -> String {
    let mut out = String::new();
    for unit in units {
        if is_vowel(unit) {
            out.push(unit.base);
            out.extend(unit.consumed);
        } else {
            out.push_str(&unit.text);
        }
    }
    out
}

/// Which nucleus is stressed: one with an accent (acute or circumflex, then tilde), else the
/// last for words ending in r, l, z, x, i, u, im, um (with or without s), else the last but one.
pub fn stressed(units: &[Unit], nuclei: &[Vec<usize>]) -> usize {
    let marked = |test: &dyn Fn(&Unit) -> bool| {
        nuclei
            .iter()
            .position(|n| n.iter().any(|i| test(&units[*i])))
    };
    if let Some(n) = marked(&accented).or_else(|| marked(&|u| u.mark == Mark::Tilde)) {
        return n;
    }
    let spelled = spelling(units);
    let stem = match spelled.strip_suffix('s') {
        Some(rest) if !rest.is_empty() => rest,
        _ => &spelled,
    };
    let last = ["r", "l", "z", "x", "i", "u", "im", "um", "in", "un"]
        .iter()
        .any(|e| stem.ends_with(e));
    if last || nuclei.len() < 2 {
        nuclei.len().saturating_sub(1)
    } else {
        nuclei.len() - 2
    }
}
