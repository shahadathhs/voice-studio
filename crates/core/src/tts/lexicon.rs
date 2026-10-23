//! The pronunciation dictionary: English words to the phoneme symbols the Kokoro model reads.
//! The data is the `misaki` project's American English "gold" and "silver" dictionaries
//! (Apache-2.0); this file only looks words up and builds plurals, past tenses and `-ing` forms of
//! known words, following the same rules as `misaki`.

use std::collections::HashMap;

use serde::Deserialize;

use crate::{Error, Result};

/// A word's pronunciation: one for every use, or one per part of speech.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Entry {
    Plain(String),
    /// Some tags are `null` in the data, meaning "no pronunciation for this use".
    ByTag(HashMap<String, Option<String>>),
}

impl Entry {
    /// Without a part-of-speech tagger the default pronunciation is used.
    fn phonemes(&self) -> Option<&str> {
        match self {
            Entry::Plain(phonemes) => Some(phonemes),
            Entry::ByTag(tags) => tags.get("DEFAULT")?.as_deref(),
        }
    }
}

/// Phonemes after which "-s" is pronounced "s".
const VOICELESS: &str = "ptkfθ";
/// Phonemes after which "-s" or "-ed" needs an extra vowel.
const SIBILANTS: &str = "szʃʒʧʤ";
/// Phonemes after which a "t" becomes a flap ("ɾ") before "-ed" or "-ing" in American English.
const FLAP_AFTER: &str = "AIOWYiuæɑəɛɪɹʊʌ";
/// Phonemes after which "-ed" is pronounced "t".
const ED_AS_T: &str = "pkfθʃsʧ";

/// Words and their phonemes, with the second list consulted only when the first has no entry.
pub struct Lexicon {
    gold: HashMap<String, Entry>,
    silver: HashMap<String, Entry>,
}

fn parse(json: &str, name: &str) -> Result<HashMap<String, Entry>> {
    serde_json::from_str(json)
        .map_err(|e| Error::Engine(format!("pronunciation dictionary {name}: {e}")))
}

impl Lexicon {
    /// Reads the two dictionaries from their JSON text.
    pub fn from_json(gold: &str, silver: &str) -> Result<Lexicon> {
        Ok(Lexicon {
            gold: parse(gold, "gold")?,
            silver: parse(silver, "silver")?,
        })
    }

    fn direct(&self, word: &str) -> Option<&str> {
        self.gold
            .get(word)
            .or_else(|| self.silver.get(word))
            .and_then(Entry::phonemes)
    }

    fn known(&self, word: &str) -> bool {
        self.direct(word).is_some()
    }

    /// The phonemes of `word` as written (any capitalisation), or `None` when the dictionaries
    /// have neither the word nor a stem it is built from.
    pub fn lookup(&self, word: &str) -> Option<String> {
        if let Some(found) = self.gold.get(word).and_then(Entry::phonemes) {
            return Some(found.to_string());
        }
        let lower = word.to_lowercase();
        if let Some(found) = self.direct(&lower) {
            return Some(found.to_string());
        }
        self.stem_s(&lower)
            .or_else(|| self.stem_ed(&lower))
            .or_else(|| self.stem_ing(&lower))
    }

    fn stem_s(&self, word: &str) -> Option<String> {
        if word.len() < 3 || !word.ends_with('s') {
            return None;
        }
        let stem = if !word.ends_with("ss") && self.known(&word[..word.len() - 1]) {
            word[..word.len() - 1].to_string()
        } else if (word.ends_with("'s")
            || (word.len() > 4 && word.ends_with("es") && !word.ends_with("ies")))
            && self.known(&word[..word.len() - 2])
        {
            word[..word.len() - 2].to_string()
        } else if word.len() > 4
            && word.ends_with("ies")
            && self.known(&format!("{}y", &word[..word.len() - 3]))
        {
            format!("{}y", &word[..word.len() - 3])
        } else {
            return None;
        };
        self.direct(&stem).map(with_s)
    }

    fn stem_ed(&self, word: &str) -> Option<String> {
        if !word.ends_with('d') {
            return None;
        }
        let stem = if !word.ends_with("dd") && self.known(&word[..word.len() - 1]) {
            &word[..word.len() - 1]
        } else if word.len() > 4
            && word.ends_with("ed")
            && !word.ends_with("eed")
            && self.known(&word[..word.len() - 2])
        {
            &word[..word.len() - 2]
        } else {
            return None;
        };
        self.direct(stem).map(with_ed)
    }

    fn stem_ing(&self, word: &str) -> Option<String> {
        if word.len() < 5 || !word.ends_with("ing") {
            return None;
        }
        let base = &word[..word.len() - 3];
        let doubled = word.len() > 5 && base.chars().rev().nth(1) == base.chars().last();
        let stem = if self.known(base) {
            base.to_string()
        } else if word.len() > 5 && self.known(&format!("{base}e")) {
            format!("{base}e")
        } else if doubled && self.known(&base[..base.len() - 1]) {
            base[..base.len() - 1].to_string()
        } else {
            return None;
        };
        self.direct(&stem).map(with_ing)
    }
}

fn last(phonemes: &str) -> Option<char> {
    phonemes.chars().last()
}

fn second_last(phonemes: &str) -> Option<char> {
    phonemes.chars().rev().nth(1)
}

fn with_s(stem: &str) -> String {
    match last(stem) {
        Some(c) if VOICELESS.contains(c) => format!("{stem}s"),
        Some(c) if SIBILANTS.contains(c) => format!("{stem}ᵻz"),
        _ => format!("{stem}z"),
    }
}

fn with_ed(stem: &str) -> String {
    match last(stem) {
        Some(c) if ED_AS_T.contains(c) => format!("{stem}t"),
        Some('d') => format!("{stem}ᵻd"),
        Some('t') => match second_last(stem) {
            Some(before) if FLAP_AFTER.contains(before) => {
                format!("{}ɾᵻd", &stem[..stem.len() - 't'.len_utf8()])
            }
            _ => format!("{stem}ᵻd"),
        },
        _ => format!("{stem}d"),
    }
}

fn with_ing(stem: &str) -> String {
    if last(stem) == Some('t') && second_last(stem).is_some_and(|c| FLAP_AFTER.contains(c)) {
        format!("{}ɾɪŋ", &stem[..stem.len() - 't'.len_utf8()])
    } else {
        format!("{stem}ɪŋ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lexicon() -> Lexicon {
        let gold = r#"{"hello":"həlˈO","read":{"DEFAULT":"ɹˈid","VBD":"ɹˈɛd"},"Hello":"hɛlˈO",
            "chocolate":"ʧˈɑkləᵻt","walk":"wˈɔk","plant":"plˈænt","wish":"wˈɪʃ","play":"plˈA",
            "bake":"bˈAk","run":"ɹˈʌn","carry":"kˈæɹi","need":"nˈid","start":"stˈɑɹt","wait":"wˈAt","dog":"dˈɔɡ","glass":"ɡlˈæs"}"#;
        let silver = r#"{"zebra":"zˈibɹə"}"#;
        Lexicon::from_json(gold, silver).unwrap()
    }

    #[test]
    fn words_are_found_in_either_dictionary_and_use_the_default_pronunciation() {
        let lexicon = lexicon();
        assert_eq!(lexicon.lookup("hello").as_deref(), Some("həlˈO"));
        assert_eq!(lexicon.lookup("zebra").as_deref(), Some("zˈibɹə"));
        assert_eq!(lexicon.lookup("read").as_deref(), Some("ɹˈid"));
        assert_eq!(lexicon.lookup("unknownword"), None);
    }

    #[test]
    fn tags_that_are_null_in_the_data_are_accepted() {
        let gold = r#"{"AA":{"DEFAULT":"ˈɑˌɑ","NOUN":null},"odd":{"NOUN":"ˈɑd"}}"#;
        let lexicon = Lexicon::from_json(gold, "{}").unwrap();
        assert_eq!(lexicon.lookup("AA").as_deref(), Some("ˈɑˌɑ"));
        assert_eq!(lexicon.lookup("odd"), None);
    }

    #[test]
    fn a_capitalised_entry_wins_and_other_capitals_fall_back_to_lowercase() {
        let lexicon = lexicon();
        assert_eq!(lexicon.lookup("Hello").as_deref(), Some("hɛlˈO"));
        assert_eq!(lexicon.lookup("Zebra").as_deref(), Some("zˈibɹə"));
    }

    #[test]
    fn plurals_choose_s_z_or_iz_by_the_last_sound() {
        let lexicon = lexicon();
        assert_eq!(lexicon.lookup("chocolates").as_deref(), Some("ʧˈɑkləᵻts"));
        assert_eq!(lexicon.lookup("dogs").as_deref(), Some("dˈɔɡz"));
        assert_eq!(lexicon.lookup("wishes").as_deref(), Some("wˈɪʃᵻz"));
        assert_eq!(lexicon.lookup("glasses").as_deref(), Some("ɡlˈæsᵻz"));
        assert_eq!(lexicon.lookup("carries").as_deref(), Some("kˈæɹiz"));
        assert_eq!(lexicon.lookup("dog's").as_deref(), Some("dˈɔɡz"));
        assert_eq!(lexicon.lookup("glass"), Some("ɡlˈæs".to_string()));
    }

    #[test]
    fn past_tenses_choose_t_d_or_id() {
        let lexicon = lexicon();
        assert_eq!(lexicon.lookup("walked").as_deref(), Some("wˈɔkt"));
        assert_eq!(lexicon.lookup("played").as_deref(), Some("plˈAd"));
        assert_eq!(lexicon.lookup("needed").as_deref(), Some("nˈidᵻd"));
        assert_eq!(lexicon.lookup("baked").as_deref(), Some("bˈAkt"));
        assert_eq!(lexicon.lookup("waited").as_deref(), Some("wˈAɾᵻd"));
    }

    #[test]
    fn ing_forms_handle_dropped_e_and_doubled_letters() {
        let lexicon = lexicon();
        assert_eq!(lexicon.lookup("walking").as_deref(), Some("wˈɔkɪŋ"));
        assert_eq!(lexicon.lookup("baking").as_deref(), Some("bˈAkɪŋ"));
        assert_eq!(lexicon.lookup("running").as_deref(), Some("ɹˈʌnɪŋ"));
        assert_eq!(lexicon.lookup("starting").as_deref(), Some("stˈɑɹɾɪŋ"));
    }

    #[test]
    fn a_broken_dictionary_is_an_engine_error() {
        assert!(matches!(
            Lexicon::from_json("not json", "{}"),
            Err(Error::Engine(_))
        ));
    }
}
