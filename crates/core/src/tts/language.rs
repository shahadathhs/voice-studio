//! The languages Kokoro's voices speak. A voice id starts with a letter for the language and one
//! for the gender: `ef_dora` is a Spanish female voice, `bm_george` a British male one.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Language {
    AmericanEnglish,
    BritishEnglish,
    Spanish,
    French,
    Hindi,
    Italian,
    Portuguese,
    Chinese,
}

impl Language {
    pub const ALL: [Language; 8] = [
        Language::AmericanEnglish,
        Language::BritishEnglish,
        Language::Spanish,
        Language::French,
        Language::Hindi,
        Language::Italian,
        Language::Portuguese,
        Language::Chinese,
    ];

    /// The language of a voice, from the first letter of its id.
    pub fn of_voice(id: &str) -> Option<Language> {
        let letter = id.chars().next()?;
        Language::ALL
            .into_iter()
            .find(|l| l.voice_letter() == letter)
    }

    fn voice_letter(self) -> char {
        match self {
            Language::AmericanEnglish => 'a',
            Language::BritishEnglish => 'b',
            Language::Spanish => 'e',
            Language::French => 'f',
            Language::Hindi => 'h',
            Language::Italian => 'i',
            Language::Portuguese => 'p',
            Language::Chinese => 'z',
        }
    }

    /// The language as people say it: "Spanish", "British English".
    pub fn name(self) -> &'static str {
        match self {
            Language::AmericanEnglish => "American English",
            Language::BritishEnglish => "British English",
            Language::Spanish => "Spanish",
            Language::French => "French",
            Language::Hindi => "Hindi",
            Language::Italian => "Italian",
            Language::Portuguese => "Brazilian Portuguese",
            Language::Chinese => "Mandarin Chinese",
        }
    }

    /// The two-letter code used for translation (`zh`, `pt`, `en`).
    pub fn code(self) -> &'static str {
        match self {
            Language::AmericanEnglish | Language::BritishEnglish => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::Hindi => "hi",
            Language::Italian => "it",
            Language::Portuguese => "pt",
            Language::Chinese => "zh",
        }
    }

    /// Whether this program can turn text of the language into phonemes.
    pub fn is_supported(self) -> bool {
        matches!(
            self,
            Language::AmericanEnglish | Language::BritishEnglish | Language::Spanish
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_letter_of_a_voice_names_its_language() {
        assert_eq!(Language::of_voice("ef_dora"), Some(Language::Spanish));
        assert_eq!(
            Language::of_voice("bm_george"),
            Some(Language::BritishEnglish)
        );
        assert_eq!(Language::of_voice("zf_xiaoxiao"), Some(Language::Chinese));
        assert_eq!(Language::of_voice("jf_alpha"), None);
        assert_eq!(Language::of_voice(""), None);
    }

    #[test]
    fn every_language_has_its_own_voice_letter() {
        for language in Language::ALL {
            assert_eq!(
                Language::ALL
                    .iter()
                    .filter(|l| l.voice_letter() == language.voice_letter())
                    .count(),
                1
            );
        }
    }
}
