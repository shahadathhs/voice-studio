//! Finding the syllables of a Spanish word and the one that is stressed. The stress mark goes in
//! front of the head vowel of the stressed syllable, after its first consonants and any glide.

use super::sounds::{Unit, UnitKind};

const WEAK: [char; 2] = ['i', 'u'];
pub fn is_vowel(unit: &Unit) -> bool {
    unit.kind == UnitKind::Vowel
}

fn is_weak(unit: &Unit) -> bool {
    WEAK.contains(&unit.base) && !unit.accented
}

/// Whether `next` joins the syllable that `current` belongs to: a weak vowel next to another
/// vowel makes a diphthong; two strong vowels, an accented weak one, or a doubled weak one do not.
fn joins(current: &[&Unit], next: &Unit) -> bool {
    let Some(last) = current.last() else {
        return false;
    };
    let accented_weak = |u: &Unit| WEAK.contains(&u.base) && u.accented;
    if accented_weak(last) || accented_weak(next) {
        return false;
    }
    match (is_weak(last), is_weak(next)) {
        (false, false) => false,
        (true, true) => last.base != next.base,
        _ => true,
    }
}

/// The vowels (as indexes into `units`) that make up each syllable's nucleus, with the index of
/// the vowel that carries the stress if it is the head of that syllable.
pub fn nuclei(units: &[Unit]) -> Vec<Vec<usize>> {
    let mut found: Vec<Vec<usize>> = Vec::new();
    for (index, unit) in units.iter().enumerate() {
        if !is_vowel(unit) {
            continue;
        }
        let adjacent = index > 0 && is_vowel(&units[index - 1]);
        let current: Vec<&Unit> = found
            .last()
            .map(|n| n.iter().map(|i| &units[*i]).collect())
            .unwrap_or_default();
        match found.last_mut() {
            Some(last) if adjacent && joins(&current, unit) => last.push(index),
            _ => found.push(vec![index]),
        }
    }
    found
}

/// The vowel of a nucleus that is said as a vowel; the others are glides (j or w).
pub fn head(units: &[Unit], nucleus: &[usize]) -> usize {
    nucleus
        .iter()
        .copied()
        .find(|i| units[*i].accented)
        .or_else(|| nucleus.iter().copied().find(|i| !is_weak(&units[*i])))
        .or_else(|| nucleus.last().copied())
        .unwrap_or(0)
}

/// Which nucleus is stressed: the accented one, else the last but one if the word ends in a
/// vowel, n or s, else the last.
pub fn stressed(units: &[Unit], nuclei: &[Vec<usize>]) -> usize {
    if let Some(n) = nuclei
        .iter()
        .position(|nucleus| nucleus.iter().any(|i| units[*i].accented))
    {
        return n;
    }
    let ends_light = units.last().is_some_and(|u| {
        is_vowel(u) || (u.kind == UnitKind::Consonant && ["n", "s"].contains(&u.text.as_str()))
    });
    if ends_light && nuclei.len() > 1 {
        nuclei.len() - 2
    } else {
        nuclei.len().saturating_sub(1)
    }
}
