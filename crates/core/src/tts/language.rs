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

    /// The language for a code such as `es` or `zh-CN`; `en` means American English.
    pub fn of_code(code: &str) -> Option<Language> {
        let main = code.split(['-', '_']).next().unwrap_or(code);
        Language::ALL
            .into_iter()
            .find(|l| l.code().eq_ignore_ascii_case(main))
    }

    /// The voice used when someone asks for the language without choosing one.
    pub fn default_voice(self) -> &'static str {
        match self {
            Language::AmericanEnglish => "af_heart",
            Language::BritishEnglish => "bf_emma",
            Language::Spanish => "ef_dora",
            Language::French => "ff_siwis",
            Language::Hindi => "hf_alpha",
            Language::Italian => "if_sara",
            Language::Portuguese => "pf_dora",
            Language::Chinese => "zf_xiaoxiao",
        }
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

    /// A short sentence in the language, for trying out a voice.
    pub fn sample_sentence(self) -> &'static str {
        match self {
            Language::AmericanEnglish | Language::BritishEnglish => {
                "Hello, this is how my new voice sounds."
            }
            Language::Spanish => "Hola, así suena mi nueva voz.",
            Language::French => "Bonjour, voici comment sonne ma nouvelle voix.",
            Language::Hindi => "नमस्ते, यह मेरी नई आवाज़ है।",
            Language::Italian => "Ciao, così suona la mia nuova voce.",
            Language::Portuguese => "Olá, é assim que soa a minha nova voz.",
            Language::Chinese => "你好，这是我新的声音。",
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
        Language::ALL.contains(&self)
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
    fn a_language_code_finds_the_language_and_its_default_voice() {
        assert_eq!(Language::of_code("zh-CN"), Some(Language::Chinese));
        assert_eq!(Language::of_code("en"), Some(Language::AmericanEnglish));
        assert_eq!(Language::of_code("bn"), None);
        for language in Language::ALL {
            assert_eq!(Language::of_voice(language.default_voice()), Some(language));
        }
    }

    #[test]
    fn every_language_has_a_sample_sentence() {
        for language in Language::ALL {
            assert_ne!(language.sample_sentence(), "");
        }
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
