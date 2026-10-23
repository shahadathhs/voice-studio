//! Brazilian Portuguese pronunciation from spelling. The letters give the sounds with a handful
//! of rules: nasal vowels, the sounds of `lh`, `nh`, `ch`, `rr`, `s` and `x` by position, the
//! soft `t` and `d` before `i`, and the reduced final vowels. The accent marks place the stress.
//! Kokoro's Portuguese voices speak the Brazilian standard.

mod consonants;
mod numbers;
mod sounds;
mod syllables;
mod units;
mod vowels;

use super::digits::NumberWords;
use super::flow::{self, Words};

const NUMBER_WORDS: NumberWords = NumberWords {
    cardinal: numbers::cardinal,
    digits: [
        "zero", "um", "dois", "três", "quatro", "cinco", "seis", "sete", "oito", "nove",
    ],
    point: "vírgula",
};

/// Turns Portuguese text into phonemes.
pub struct Portuguese;

impl Portuguese {
    pub fn phonemize(&self, text: &str) -> String {
        flow::phonemize(self, text)
    }
}

impl Words for Portuguese {
    fn number(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, ',')
    }

    fn ordinal(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, ',')
    }

    fn dollars(&self, amount: &str) -> String {
        format!("{} dólares", NUMBER_WORDS.spoken(amount, ','))
    }

    fn word(&self, word: &str) -> String {
        sounds::word(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_keep_punctuation_and_read_numbers() {
        let say = |text| Portuguese.phonemize(text).replace('\u{303}', "~");
        assert_eq!(say("Olá, mundo."), "olˈa, mˈu~du.");
        assert_eq!(say("3,5"), "tɾes vˈixɡulɐ sˈi~ku");
        assert_eq!(say("2.020"), "dojs miw i vˈi~ʧi");
    }
}
