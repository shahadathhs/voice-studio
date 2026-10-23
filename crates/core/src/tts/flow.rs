//! The part of turning text into phonemes that every language shares: cutting the text into
//! words, numbers and punctuation, and putting the phonemes back together. What a word sounds
//! like, and how a number is said, is up to the language ([`Words`]).

use super::tokens::{Token, tokenize};

/// Quotes and brackets that open a phrase when they follow a gap.
const OPENERS: [char; 3] = ['(', '“', '"'];

/// What differs between languages.
pub trait Words {
    /// A whole number or decimal written with digits, in words.
    fn number(&self, digits: &str) -> String;
    /// An ordinal such as "21st", in words.
    fn ordinal(&self, digits: &str) -> String;
    /// A dollar amount such as "5.50", in words.
    fn dollars(&self, amount: &str) -> String;
    /// The phonemes of one word; empty when there is nothing to say.
    fn word(&self, word: &str) -> String;
}

/// Phonemes under construction: where the next word goes.
#[derive(Default)]
struct Output {
    text: String,
    /// Set after an opening quote or bracket, which the next word hugs.
    glue_next: bool,
}

impl Output {
    fn word(&mut self, phonemes: &str) {
        if !self.text.is_empty() && !self.glue_next {
            self.text.push(' ');
        }
        self.text.push_str(phonemes);
        self.glue_next = false;
    }

    fn punctuation(&mut self, c: char, after_gap: bool) {
        let opening = OPENERS.contains(&c) && (self.text.is_empty() || after_gap);
        if opening && !self.text.is_empty() {
            self.text.push(' ');
        }
        self.text.push(c);
        self.glue_next = opening;
    }
}

/// The phonemes of `text`: words separated by spaces and punctuation attached as written.
pub fn phonemize(language: &impl Words, text: &str) -> String {
    let mut out = Output::default();
    let mut after_gap = false;
    for token in tokenize(text) {
        let words = match token {
            Token::Gap => {
                after_gap = true;
                continue;
            }
            Token::Punct(c) => {
                out.punctuation(c, after_gap);
                after_gap = false;
                continue;
            }
            Token::Word(word) => word,
            Token::Number(number) => language.number(&number),
            Token::Ordinal(number) => language.ordinal(&number),
            Token::Dollars(amount) => language.dollars(&amount),
        };
        after_gap = false;
        for word in words.split_whitespace() {
            let phonemes = language.word(word);
            if !phonemes.is_empty() {
                out.word(&phonemes);
            }
        }
    }
    out.text
}
