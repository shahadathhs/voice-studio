//! The model's phoneme alphabet: which number stands for which symbol.

use std::collections::HashMap;

use crate::{Error, Result};

/// Symbol numbers, as listed in the model's `tokenizer.json`.
pub struct Vocab {
    ids: HashMap<char, i64>,
}

impl Vocab {
    /// Reads `model.vocab` of a Hugging Face `tokenizer.json`; entries that are not a single
    /// character are ignored.
    pub fn from_tokenizer_json(json: &str) -> Result<Vocab> {
        let parsed: serde_json::Value = serde_json::from_str(json)?;
        let entries = parsed["model"]["vocab"]
            .as_object()
            .ok_or_else(|| Error::Engine("tokenizer.json has no model.vocab".into()))?;
        let ids = entries
            .iter()
            .filter_map(|(symbol, id)| {
                let mut chars = symbol.chars();
                let (c, rest) = (chars.next()?, chars.next());
                rest.is_none().then_some((c, id.as_i64()?))
            })
            .collect();
        Ok(Vocab { ids })
    }

    /// The numbers for a phoneme string. Symbols the model does not know are left out, the same
    /// as the model's own tokenizer does.
    pub fn encode(&self, phonemes: &str) -> Vec<i64> {
        phonemes
            .chars()
            .filter_map(|c| self.ids.get(&c).copied())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{"model":{"vocab":{"$":0,".":4," ":16,"a":43,"ˈ":156,"long":99}}}"#;

    #[test]
    fn symbols_become_their_numbers_and_unknown_ones_are_dropped() {
        let vocab = Vocab::from_tokenizer_json(JSON).unwrap();
        assert_eq!(vocab.encode("a ˈa."), [43, 16, 156, 43, 4]);
        assert_eq!(vocab.encode("☃?"), Vec::<i64>::new());
    }

    #[test]
    fn a_file_without_a_vocabulary_is_an_engine_error() {
        assert!(matches!(
            Vocab::from_tokenizer_json("{}"),
            Err(Error::Engine(_))
        ));
    }
}
