//! Hindi pronunciation from the Devanagari script. Hindi is written almost as it is said once
//! the inherent vowels that are not pronounced are removed, so rules are enough.

mod letters;
mod numbers;
mod sounds;

use super::digits::NumberWords;
use super::flow::{self, Words};
use letters::{NUKTA, VIRAMA};

const NUMBER_WORDS: NumberWords = NumberWords {
    cardinal: numbers::cardinal,
    digits: numbers::DIGITS,
    point: "दशमलव",
};

/// Turns Hindi text into phonemes.
pub struct Hindi;

impl Hindi {
    pub fn phonemize(&self, text: &str) -> String {
        flow::phonemize(self, &prepare(text))
    }
}

/// Makes the text safe for the shared tokenizer: it would cut words at the virama and nukta and
/// ignore Devanagari digits and the danda.
fn prepare(text: &str) -> String {
    text.chars().flat_map(prepared).collect()
}

fn prepared(c: char) -> Vec<char> {
    match c {
        // The letters with a nukta that have a code point of their own.
        '\u{0958}'..='\u{095f}' => {
            let base = ['क', 'ख', 'ग', 'ज', 'ड', 'ढ', 'फ', 'य'];
            let index = usize::try_from(u32::from(c) - 0x0958).unwrap_or(0);
            vec![base[index.min(base.len() - 1)], NUKTA]
        }
        '्' => vec![VIRAMA],
        '़' => vec![NUKTA],
        '०'..='९' => char::from_digit(u32::from(c) - u32::from('०'), 10)
            .into_iter()
            .collect(),
        '।' | '॥' => vec!['.'],
        '\u{200c}' | '\u{200d}' => Vec::new(),
        _ => vec![c],
    }
}

impl Words for Hindi {
    fn number(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, '.')
    }

    fn ordinal(&self, digits: &str) -> String {
        NUMBER_WORDS.spoken(digits, '.')
    }

    fn dollars(&self, amount: &str) -> String {
        format!("{} डॉलर", NUMBER_WORDS.spoken(amount, '.'))
    }

    fn word(&self, word: &str) -> String {
        sounds::word(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_keep_punctuation_and_end_at_the_danda() {
        assert_eq!(Hindi.phonemize("नमस्ते, दुनिया।"), "nəməsteː, dʊnɪjaː.");
    }

    #[test]
    fn numbers_are_read_in_hindi_also_in_devanagari_digits() {
        assert_eq!(Hindi.phonemize("२५"), Hindi.phonemize("पच्चीस"));
        assert_eq!(Hindi.phonemize("3.5"), Hindi.phonemize("तीन दशमलव पाँच"));
        assert_eq!(Hindi.phonemize("$2"), Hindi.phonemize("दो डॉलर"));
    }
}
