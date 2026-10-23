//! Italian words to phonemes.

use super::letters::{Unit, Vowel, units};
use super::lexicon::respell;
use super::stress::{head, nuclei, stressed};

const STRESS: char = 'ˈ';

fn vowel_sound(vowel: &Vowel, glide: bool) -> &'static str {
    match (vowel.base, glide, vowel.open) {
        ('i', true, _) => "j",
        ('u', true, _) => "w",
        ('e', _, true) => "ɛ",
        ('o', _, true) => "ɔ",
        ('a', ..) => "a",
        ('e', ..) => "e",
        ('i', ..) => "i",
        ('o', ..) => "o",
        _ => "u",
    }
}

/// The phonemes of a lower-case word without apostrophes.
fn spoken(word: &str) -> String {
    let units = units(word);
    let nuclei = nuclei(&units);
    let heads: Vec<usize> = nuclei.iter().map(|n| head(&units, n)).collect();
    // A word of one syllable carries no mark.
    let stress_at = (nuclei.len() > 1).then(|| heads[stressed(&units, &nuclei, word)]);
    let mut out = String::new();
    for (index, unit) in units.iter().enumerate() {
        if stress_at == Some(index) {
            out.push(STRESS);
        }
        match unit {
            Unit::Vowel(v) => out.push_str(vowel_sound(v, !heads.contains(&index))),
            Unit::Consonant(c) => out.push_str(c),
        }
    }
    out
}

/// "l'amico": the elided word is said as part of the next one, without a stress of its own.
fn elided(prefix: &str, rest: &str) -> String {
    let front = rest.starts_with(['e', 'i', 'è', 'é', 'ì']);
    let sound = match prefix {
        "c" if front => "ʧ".to_string(),
        "c" => "k".to_string(),
        _ => spoken(prefix),
    };
    format!("{}{}", sound.replace(STRESS, ""), word(rest))
}

/// The phonemes of an Italian word, with a stress mark in front of the stressed syllable.
pub fn word(text: &str) -> String {
    let lower = text.to_lowercase();
    match lower.split_once('\'') {
        Some((prefix, rest)) if !prefix.is_empty() && !rest.is_empty() => elided(prefix, rest),
        _ => spoken(respell(&lower.replace('\'', ""))),
    }
}

#[cfg(test)]
mod tests {
    use super::word;

    #[test]
    fn stress_follows_the_accent_or_the_ending() {
        assert_eq!(word("casa"), "kˈaza");
        assert_eq!(word("città"), "ʧittˈa");
        assert_eq!(word("perché"), "perkˈe");
        assert_eq!(word("caffè"), "kaffˈɛ");
        assert_eq!(word("parlano"), "pˈarlano");
        assert_eq!(word("possibile"), "possˈibile");
        assert_eq!(word("e"), "e");
    }

    #[test]
    fn glides_and_silent_letters() {
        assert_eq!(word("giorno"), "ʤˈorno");
        assert_eq!(word("buongiorno"), "bwonʤˈorno");
        assert_eq!(word("storia"), "stˈorja");
        assert_eq!(word("hanno"), "ˈanno");
    }

    #[test]
    fn elided_words_join_the_next_word() {
        assert_eq!(word("l'amico"), "lamˈiko");
        assert_eq!(word("c'è"), "ʧɛ");
    }
}
