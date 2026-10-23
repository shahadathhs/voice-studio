//! Italian pronunciation from spelling. Italian is written close to how it is said, so rules are
//! enough: the letters give the sounds, an accent or the penultimate syllable gives the stress.
//! The vowels e and o are not told open from close unless an accent says so (è, ò).

mod letters;
mod lexicon;
mod numbers;
mod sounds;
mod stress;

use super::digits::NumberWords;
use super::flow::{self, Words};

const NUMBER_WORDS: NumberWords = NumberWords {
    cardinal: numbers::cardinal,
    digits: [
        "zero", "uno", "due", "tre", "quattro", "cinque", "sei", "sette", "otto", "nove",
    ],
    point: "virgola",
};

/// Turns Italian text into phonemes.
pub struct Italian;

impl Italian {
    pub fn phonemize(&self, text: &str) -> String {
        flow::phonemize(self, text)
    }
}

impl Words for Italian {
    fn number(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, ',')
    }

    fn ordinal(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, ',')
    }

    fn dollars(&self, amount: &str) -> String {
        format!("{} dollari", NUMBER_WORDS.spoken(amount, ','))
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
        assert_eq!(Italian.phonemize("Ciao, mondo."), "ʧˈao, mˈondo.");
        assert_eq!(Italian.phonemize("3,5"), "tre virɡˈola ʧˈiŋkwe");
    }
}
