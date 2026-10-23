//! French pronunciation from spelling. Rules turn the letters into sounds (nasal vowels, silent
//! final consonants and `e`, digraphs), a short lexicon covers the exceptions, and the stress
//! falls on the last syllable. Liaison between words is not made.

mod consonants;
mod lexicon;
mod numbers;
mod sounds;
mod stem;
mod vowels;

use super::digits::NumberWords;
use super::flow::{self, Words};

const NUMBER_WORDS: NumberWords = NumberWords {
    cardinal: numbers::cardinal,
    digits: [
        "zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf",
    ],
    point: "virgule",
};

/// Turns French text into phonemes.
pub struct French;

impl French {
    pub fn phonemize(&self, text: &str) -> String {
        flow::phonemize(self, text)
    }
}

impl Words for French {
    fn number(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, ',')
    }

    fn ordinal(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, ',')
    }

    fn dollars(&self, amount: &str) -> String {
        format!("{} dollars", NUMBER_WORDS.spoken(amount, ','))
    }

    fn word(&self, word: &str) -> String {
        sounds::word(word)
    }
}

#[cfg(test)]
mod tests {
    use super::sounds::word;
    use super::*;

    #[test]
    fn sentences_keep_punctuation() {
        assert_eq!(French.phonemize("Bonjour, monde."), "bɔ̃ʒˈuʁ, mɔ̃d.");
    }

    #[test]
    fn final_letters_and_e_are_silent() {
        assert_eq!(word("petit"), "pətˈi");
        assert_eq!(word("table"), "tabl");
        assert_eq!(word("parlent"), "paʁl");
        assert_eq!(word("parler"), "paʁlˈe");
        assert_eq!(word("grand"), "ɡʁɑ̃");
        assert_eq!(word("pour"), "puʁ");
    }

    #[test]
    fn nasal_vowels_and_digraphs() {
        assert_eq!(word("bon"), "bɔ̃");
        assert_eq!(word("matin"), "matˈɛ̃");
        assert_eq!(word("bonne"), "bɔn");
        assert_eq!(word("eau"), "o");
        assert_eq!(word("oiseau"), "wazˈo");
        assert_eq!(word("nation"), "nasjˈɔ̃");
        assert_eq!(word("question"), "kɛstjˈɔ̃");
    }

    #[test]
    fn consonant_rules() {
        assert_eq!(word("rose"), "ʁoz");
        assert_eq!(word("classe"), "klas");
        assert_eq!(word("garçon"), "ɡaʁsˈɔ̃");
        assert_eq!(word("rouge"), "ʁuʒ");
        assert_eq!(word("montagne"), "mɔ̃tˈaɲ");
        assert_eq!(word("famille"), "famˈij");
        assert_eq!(word("travail"), "tʁavˈaj");
    }

    #[test]
    fn g_is_the_script_letter_of_the_model_alphabet() {
        assert_eq!(word("gare"), "ɡaʁ");
        assert_eq!(word("longue"), "lɔ̃ɡ");
        assert_eq!(word("virgule"), "viʁɡˈyl");
    }

    #[test]
    fn apostrophes_join_the_next_word() {
        assert_eq!(word("l'été"), "letˈe");
        assert_eq!(word("j'aime"), "ʒɛm");
        assert_eq!(word("qu'il"), "kil");
    }

    #[test]
    fn numbers_and_dollars_are_spoken() {
        assert_eq!(French.number("21"), "vingt et un");
        assert_eq!(French.dollars("5"), "cinq dollars");
    }
}
