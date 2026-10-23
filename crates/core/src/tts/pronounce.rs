//! Picks the pronunciation rules for the language of a voice.

use std::path::Path;

use super::chinese::{Chinese, Dictionary as ChineseDictionary};
use super::french::French;
use super::hindi::Hindi;
use super::italian::Italian;
use super::language::Language;
use super::lexicon::Lexicon;
use super::phonemes::Phonemizer;
use super::portuguese::Portuguese;
use super::spanish::Spanish;
use crate::{Error, Result};

/// File names inside the model's folder; `catalog/models.json` lists the same names.
pub const GOLD_FILE: &str = "us_gold.json";
pub const SILVER_FILE: &str = "us_silver.json";
/// The British English dictionaries; voices of other languages need no dictionary file.
pub const BRITISH_GOLD_FILE: &str = "gb_gold.json";
pub const BRITISH_SILVER_FILE: &str = "gb_silver.json";
/// The Chinese character and word lists.
pub const CHINESE_CHARACTERS_FILE: &str = "zh_chars.txt";
pub const CHINESE_PHRASES_FILE: &str = "zh_phrases.txt";

/// The pronunciation of every language that is installed.
pub struct Pronouncer {
    american: Phonemizer,
    british: Option<Phonemizer>,
    chinese: Option<Chinese>,
}

impl Pronouncer {
    /// Reads the dictionaries and word lists in the model's folder `dir`. British English and
    /// Chinese are left out when their files are not installed.
    pub fn load(dir: &Path) -> Result<Pronouncer> {
        let read = |name: &str| std::fs::read_to_string(dir.join(name));
        let has = |name: &str| dir.join(name).is_file();
        let dictionary = |gold: &str, silver: &str| -> Result<Phonemizer> {
            Ok(Phonemizer::new(Lexicon::from_json(
                &read(gold)?,
                &read(silver)?,
            )?))
        };
        let british = if has(BRITISH_GOLD_FILE) && has(BRITISH_SILVER_FILE) {
            Some(dictionary(BRITISH_GOLD_FILE, BRITISH_SILVER_FILE)?)
        } else {
            None
        };
        let chinese = if has(CHINESE_CHARACTERS_FILE) && has(CHINESE_PHRASES_FILE) {
            let words = ChineseDictionary::parse(
                &read(CHINESE_CHARACTERS_FILE)?,
                &read(CHINESE_PHRASES_FILE)?,
            )?;
            Some(Chinese::new(words))
        } else {
            None
        };
        Ok(Pronouncer::new(
            dictionary(GOLD_FILE, SILVER_FILE)?,
            british,
            chinese,
        ))
    }

    pub fn new(
        american: Phonemizer,
        british: Option<Phonemizer>,
        chinese: Option<Chinese>,
    ) -> Pronouncer {
        Pronouncer {
            american,
            british,
            chinese,
        }
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
            Language::Chinese => self
                .chinese
                .as_ref()
                .map(|chinese| chinese.phonemize(text))
                .ok_or_else(|| Error::ModelMissing("kokoro-82m (Chinese)".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// The symbols in Kokoro's `tokenizer.json`; anything else is silently dropped by the model.
    const SYMBOLS: &str = ";:,.!?—…\"()“” ̃ʣʥʦʨᵝꭧAIOQSTWYᵊabcdefhijklmnopqrstuvwxyzɑɐɒæβɔɕçɖðʤəɚɛɜɟɡɥɨɪʝɯɰŋɳɲɴøɸθœɹɾɻʁɽʂʃʈʧʊʋʌɣɤχʎʒʔˈˌːʰʲ↓→↗↘ᵻ";

    const SENTENCES: [(&str, &str); 5] = [
        (
            "es",
            "Mañana tengo una reunión con mis compañeros, ¿vienes?",
        ),
        (
            "it",
            "Oggi ho comprato gnocchi e il pane, ci sono 2.500 persone.",
        ),
        (
            "pt",
            "O ônibus sai às 8 horas e a viagem dura três horas e meia.",
        ),
        (
            "fr",
            "Nous avons mangé du pain avec du fromage, il y a 22 livres.",
        ),
        ("hi", "मैं आज बाज़ार जा रहा हूँ। इस किताब में 150 पन्ने हैं।"),
    ];

    fn without_english_dictionary() -> Pronouncer {
        let empty =
            || Phonemizer::new(super::super::lexicon::Lexicon::from_json("{}", "{}").unwrap());
        Pronouncer::new(empty(), None, None)
    }

    #[test]
    fn every_language_writes_only_symbols_the_model_knows() {
        let known: HashSet<char> = SYMBOLS.chars().collect();
        let pronouncer = without_english_dictionary();
        for (code, text) in SENTENCES {
            let language = Language::of_code(code).unwrap();
            let phonemes = pronouncer.phonemize(language, text).unwrap();
            let unknown: Vec<char> = phonemes
                .chars()
                .filter(|c| !c.is_whitespace() && !known.contains(c))
                .collect();
            assert!(unknown.is_empty(), "{code}: {unknown:?} in {phonemes}");
        }
    }
}
