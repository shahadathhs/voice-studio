//! A Devanagari word as sounds. Each consonant letter carries an inherent schwa unless a vowel
//! sign or a virama says otherwise; in speech many of those schwas are not pronounced, so the
//! word is first cut into units and the schwas are then deleted by rule.

use super::letters::{self, NUKTA, VIRAMA};

const NASAL_MARK: char = '\u{0303}';

#[derive(Clone, PartialEq)]
enum Vowel {
    /// The consonant's own schwa; it may be dropped.
    Inherent,
    /// A vowel that is always said.
    Sign(&'static str),
    /// No vowel (a virama follows, or the unit is a closing sound).
    None,
}

#[derive(Clone, Copy, PartialEq)]
enum Nasal {
    /// The dot above (anusvara): a nasal that agrees with the next consonant.
    Dot,
    /// The moon-dot (chandrabindu): always a nasal vowel.
    Moon,
}

struct Unit {
    onset: String,
    vowel: Vowel,
    nasal: Option<Nasal>,
}

impl Unit {
    fn new(onset: &str, vowel: Vowel) -> Self {
        Self {
            onset: onset.to_string(),
            vowel,
            nasal: None,
        }
    }
}

fn units(chars: &[char]) -> Vec<Unit> {
    let mut units: Vec<Unit> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        if let Some(base) = chars
            .get(i)
            .filter(|&&n| n == NUKTA)
            .and_then(|_| letters::with_nukta(c))
        {
            units.push(Unit::new(base, Vowel::Inherent));
            i += 1;
        } else if let Some(sound) = letters::consonant(c) {
            units.push(Unit::new(sound, Vowel::Inherent));
        } else if let Some(sound) = letters::independent(c) {
            units.push(Unit::new("", Vowel::Sign(sound)));
        } else if let Some(sound) = letters::latin_name(c) {
            units.push(Unit::new("", Vowel::Sign(sound)));
        } else {
            modify(&mut units, c);
        }
    }
    units
}

/// Marks that change the unit before them.
fn modify(units: &mut Vec<Unit>, c: char) {
    if c == 'ः' {
        units.push(Unit::new("h", Vowel::None));
        return;
    }
    if c == 'ॐ' {
        units.push(Unit::new("", Vowel::Sign("oːm")));
        return;
    }
    let Some(last) = units.last_mut() else { return };
    match c {
        VIRAMA => last.vowel = Vowel::None,
        'ं' => last.nasal = Some(Nasal::Dot),
        'ँ' => last.nasal = Some(Nasal::Moon),
        _ => {
            if let Some(sound) = letters::matra(c) {
                last.vowel = Vowel::Sign(sound);
            }
        }
    }
}

/// Whether the schwa of unit `i` is said. Words end in a consonant sound; between two syllables
/// that are said, a schwa after a vowel is dropped (कमल is kəməl, करना is kərnaː).
fn schwa_kept(units: &[Unit], dropped: &[bool], i: usize) -> bool {
    let unit = &units[i];
    if unit.vowel != Vowel::Inherent || unit.nasal.is_some() || units.len() == 1 {
        return true;
    }
    let Some(next) = units.get(i + 1) else {
        return false;
    };
    let said_after = match next.vowel {
        Vowel::Sign(_) => true,
        Vowel::Inherent => !dropped[i + 1],
        Vowel::None => false,
    };
    let said_before = i > 0
        && units[i - 1].nasal != Some(Nasal::Dot)
        && matches!(units[i - 1].vowel, Vowel::Sign(_) | Vowel::Inherent);
    !(said_before && said_after)
}

/// The nasal that fits the consonant after an anusvara, if it is a stop.
fn homorganic(next: Option<&Unit>) -> Option<&'static str> {
    let first = next?.onset.chars().next()?;
    Some(match first {
        'k' | 'ɡ' | 'q' | 'x' | 'ɣ' => "ŋ",
        'ʧ' | 'ʤ' => "ɲ",
        'ʈ' | 'ɖ' => "ɳ",
        't' | 'd' => "n",
        'p' | 'b' => "m",
        _ => return None,
    })
}

/// Makes the vowel nasal: the tilde goes on the vowel letter, before any length mark.
fn nasalise(vowel: &str) -> String {
    let mut out = String::new();
    let mut chars = vowel.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if chars.peek().is_none_or(|&n| n == 'ː') && !out.contains(NASAL_MARK) {
            out.push(NASAL_MARK);
        }
    }
    out
}

fn render(units: &[Unit]) -> String {
    let mut dropped = vec![false; units.len()];
    for i in (0..units.len()).rev() {
        dropped[i] = !schwa_kept(units, &dropped, i);
    }
    let mut out = String::new();
    for (i, unit) in units.iter().enumerate() {
        out.push_str(&unit.onset);
        let vowel = match unit.vowel {
            Vowel::Sign(sound) => sound,
            Vowel::Inherent if !dropped[i] => "ə",
            _ => "",
        };
        match unit.nasal {
            Some(Nasal::Dot) if homorganic(units.get(i + 1)).is_some() => {
                out.push_str(vowel);
                out.push_str(homorganic(units.get(i + 1)).unwrap_or_default());
            }
            Some(_) => out.push_str(&nasalise(vowel)),
            None => out.push_str(vowel),
        }
    }
    out
}

/// The phonemes of one word written in Devanagari (or spelt out, if it is Latin).
pub fn word(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    render(&units(&chars))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn say(word: &str) -> String {
        let marked = word
            .replace('्', &VIRAMA.to_string())
            .replace('़', &NUKTA.to_string());
        super::word(&marked)
    }

    #[test]
    fn final_and_medial_schwas_are_dropped() {
        assert_eq!(say("कमल"), "kəməl");
        assert_eq!(say("करना"), "kəɾnaː");
        assert_eq!(say("समझना"), "səməʤʰnaː");
        assert_eq!(say("मित्र"), "mɪtɾ");
    }

    #[test]
    fn vowels_and_nasals() {
        assert_eq!(say("हिंदी"), "hɪndiː");
        assert_eq!(say("में"), "me\u{303}ː");
        assert_eq!(say("नहीं"), "nəhi\u{303}ː");
        assert_eq!(say("ऐसा"), "ɛːsaː");
        assert_eq!(say("ऋषि"), "ɾɪʃɪ");
        assert_eq!(say("ज़रूर"), "zəɾuːɾ");
    }
}
