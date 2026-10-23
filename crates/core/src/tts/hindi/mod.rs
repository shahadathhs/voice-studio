//! Hindi pronunciation. Not written yet: every word is silent.

use super::flow::{self, Words};

/// Turns Hindi text into phonemes.
pub struct Hindi;

impl Hindi {
    pub fn phonemize(&self, text: &str) -> String {
        flow::phonemize(self, text)
    }
}

impl Words for Hindi {
    fn number(&self, digits: &str) -> String {
        digits.to_string()
    }

    fn ordinal(&self, digits: &str) -> String {
        digits.to_string()
    }

    fn dollars(&self, amount: &str) -> String {
        amount.to_string()
    }

    fn word(&self, _word: &str) -> String {
        String::new()
    }
}
