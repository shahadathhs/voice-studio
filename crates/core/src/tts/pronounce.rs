//! Picks the pronunciation rules for the language of a voice.

use super::french::French;
use super::hindi::Hindi;
use super::italian::Italian;
use super::language::Language;
use super::phonemes::Phonemizer;
use super::portuguese::Portuguese;
use super::spanish::Spanish;
use crate::{Error, Result};

/// The pronunciation of every language that is installed.
pub struct Pronouncer {
    american: Phonemizer,
    british: Option<Phonemizer>,
}

impl Pronouncer {
    pub fn new(american: Phonemizer, british: Option<Phonemizer>) -> Pronouncer {
        Pronouncer { american, british }
    }

    /// The phonemes of `text` spoken in `language`.
    pub fn phonemize(&self, language: Language, text: &str) -> Result<String> {
        match language {
            Language::AmericanEnglish => Ok(self.american.phonemize(text)),
            Language::BritishEnglish => self
                .british
                .as_ref()
                .map(|british| british.phonemize(text))
                .ok_or_else(|| Error::ModelMissing("kokoro-82m (British English)".into())),
            Language::Spanish => Ok(Spanish.phonemize(text)),
            Language::French => Ok(French.phonemize(text)),
            Language::Hindi => Ok(Hindi.phonemize(text)),
            Language::Italian => Ok(Italian.phonemize(text)),
            Language::Portuguese => Ok(Portuguese.phonemize(text)),
            Language::Chinese => Err(Error::NotImplemented("speaking Chinese")),
        }
    }
}
