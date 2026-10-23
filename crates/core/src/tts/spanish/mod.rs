//! Spanish pronunciation from spelling. Spanish is written almost as it is said, so rules are
//! enough: the letters give the sounds, and a few rules give the stress. The sounds are those of
//! Castilian Spanish, which Kokoro's Spanish voices were trained on.

mod numbers;
mod sounds;
mod syllables;

use super::digits::NumberWords;
use super::flow::{self, Words};

const NUMBER_WORDS: NumberWords = NumberWords {
    cardinal: numbers::cardinal,
    digits: [
        "cero", "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve",
    ],
    point: "coma",
};

/// Turns Spanish text into phonemes.
pub struct Spanish;

impl Spanish {
    pub fn phonemize(&self, text: &str) -> String {
        flow::phonemize(self, text)
    }
}

impl Words for Spanish {
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
        sounds::word(&word.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_keep_punctuation_and_read_numbers() {
        assert_eq!(Spanish.phonemize("Hola, mundo."), "ˈola, mˈundo.");
        assert_eq!(Spanish.phonemize("3,5"), "tɾes kˈoma θˈinko");
    }
}
