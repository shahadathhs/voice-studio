//! Which pinyin a character or word has. The data is the MIT-licensed `pinyin-data` (one line per
//! character, most common reading first) and `phrase-pinyin-data` (words and idioms whose
//! reading differs from their characters' usual ones).

use std::collections::HashMap;

use super::pinyin::Syllable;
use crate::{Error, Result};

/// Particles that are nearly always said in the neutral tone.
const NEUTRAL: [(char, &str); 10] = [
    ('的', "de"),
    ('了', "le"),
    ('吗', "ma"),
    ('吧', "ba"),
    ('呢', "ne"),
    ('啊', "a"),
    ('么', "me"),
    ('们', "men"),
    ('着', "zhe"),
    ('嘛', "ma"),
];

pub struct Dictionary {
    characters: HashMap<char, String>,
    phrases: HashMap<String, Vec<String>>,
    longest: usize,
}

/// `U+4E00: yī,yí,yì  # 一` → the character and its first reading.
fn character_line(line: &str) -> Option<(char, String)> {
    let (code, rest) = line.strip_prefix("U+")?.split_once(':')?;
    let character = char::from_u32(u32::from_str_radix(code.trim(), 16).ok()?)?;
    let first = rest.split('#').next()?.split(',').next()?.trim();
    (!first.is_empty()).then(|| (character, first.to_string()))
}

impl Dictionary {
    /// Reads the two data files.
    pub fn parse(characters: &str, phrases: &str) -> Result<Dictionary> {
        let characters: HashMap<char, String> =
            characters.lines().filter_map(character_line).collect();
        if characters.is_empty() {
            return Err(Error::Engine("the Chinese character list is empty".into()));
        }
        let mut longest = 1;
        let phrases = phrases
            .lines()
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| {
                let (word, readings) = line.split_once(':')?;
                let readings: Vec<String> = readings.split_whitespace().map(String::from).collect();
                (readings.len() == word.chars().count())
                    .then(|| (word.trim().to_string(), readings))
            })
            .inspect(|(word, _)| longest = longest.max(word.chars().count()))
            .collect();
        Ok(Dictionary {
            characters,
            phrases,
            longest,
        })
    }

    /// The syllables of a run of Chinese characters: the longest known words first, then the
    /// usual reading of each character. Characters without a reading are left out.
    pub fn syllables(&self, run: &[char]) -> Vec<Syllable> {
        let mut found = Vec::new();
        let mut at = 0;
        while at < run.len() {
            let longest = self.longest.min(run.len() - at);
            let phrase = (2..=longest).rev().find_map(|len| {
                let word: String = run[at..at + len].iter().collect();
                self.phrases.get(&word).map(|readings| (len, readings))
            });
            if let Some((len, readings)) = phrase {
                found.extend(readings.iter().map(|r| Syllable::parse(r)));
                at += len;
                continue;
            }
            let c = run[at];
            let neutral = NEUTRAL
                .iter()
                .find(|(n, _)| *n == c)
                .map(|(_, r)| (*r).to_string());
            if let Some(reading) = neutral.or_else(|| self.characters.get(&c).cloned()) {
                found.push(Syllable::parse(&reading));
            }
            at += 1;
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary() -> Dictionary {
        Dictionary::parse(
            "# comment\nU+4F60: nǐ  # 你\nU+597D: hǎo,hào  # 好\nU+7684: de,dí,dì  # 的\nU+884C: xíng,háng  # 行\nU+94F6: yín  # 银\n",
            "银行: yín háng\n",
        )
        .unwrap()
    }

    fn spoken(run: &str) -> String {
        let chars: Vec<char> = run.chars().collect();
        dictionary()
            .syllables(&chars)
            .iter()
            .map(Syllable::phonemes)
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn a_character_has_its_first_reading() {
        assert_eq!(spoken("你好"), "ni↓ xau↓");
    }

    #[test]
    fn a_known_word_overrides_the_usual_reading() {
        assert_eq!(spoken("行"), "ɕiŋ↗");
        assert_eq!(spoken("银行"), "in↗ xaŋ↗");
    }

    #[test]
    fn particles_are_neutral_and_unknown_characters_are_left_out() {
        assert_eq!(spoken("你的"), "ni↓ tə");
        assert_eq!(spoken("你龘"), "ni↓");
    }

    #[test]
    fn an_empty_character_list_is_an_error() {
        assert!(Dictionary::parse("", "").is_err());
    }
}
