//! Text to the phoneme string the Kokoro model reads: numbers and abbreviations become words,
//! each word is looked up in the dictionary (or guessed from its spelling), and punctuation is
//! kept because the model uses it for pauses and intonation.

use super::lexicon::Lexicon;
use super::tokens::{Token, tokenize};
use super::{numbers, spelling};

/// Quotes and brackets that open a phrase when they follow a gap.
const OPENERS: [char; 3] = ['(', '“', '"'];
/// Unknown words of this many letters or fewer, written in capitals, are spelled out as letters.
const MAX_ACRONYM: usize = 4;
/// Shortest part either side of a split when an unknown word is two known words joined.
const MIN_PART: usize = 3;

const ABBREVIATIONS: [(&str, &str); 7] = [
    ("mr", "mister"),
    ("mrs", "missus"),
    ("ms", "miss"),
    ("dr", "doctor"),
    ("vs", "versus"),
    ("etc", "et cetera"),
    ("approx", "approximately"),
];

/// Turns text into phonemes.
pub struct Phonemizer {
    lexicon: Lexicon,
}

/// Words for a dollar amount: "5" → "five dollars", "5.25" → "five dollars twenty five cents".
fn dollars(amount: &str) -> String {
    let (whole, cents) = amount.split_once('.').unwrap_or((amount, ""));
    let mut words = format!("{} dollars", numbers::spoken(whole));
    if let Some(cents) = cents
        .get(..2)
        .and_then(|c| c.parse::<u64>().ok())
        .filter(|c| *c > 0)
    {
        words = format!("{words} {} cents", numbers::cardinal(cents));
    }
    words
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

impl Phonemizer {
    pub fn new(lexicon: Lexicon) -> Phonemizer {
        Phonemizer { lexicon }
    }

    /// The phonemes of `text`, words separated by spaces and punctuation attached as written.
    pub fn phonemize(&self, text: &str) -> String {
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
                Token::Number(number) => numbers::spoken(&number),
                Token::Ordinal(number) => numbers::ordinal(number.parse().unwrap_or(0)),
                Token::Dollars(amount) => dollars(&amount),
            };
            after_gap = false;
            for word in words.split_whitespace() {
                let phonemes = self.word(word);
                if !phonemes.is_empty() {
                    out.word(&phonemes);
                }
            }
        }
        out.text
    }

    /// One word: abbreviation, dictionary, letter names for acronyms, two joined words, or spelling.
    fn word(&self, word: &str) -> String {
        let lower = word.to_lowercase();
        if let Some((_, full)) = ABBREVIATIONS.iter().find(|(short, _)| *short == lower) {
            return full
                .split(' ')
                .map(|w| self.word(w))
                .collect::<Vec<_>>()
                .join(" ");
        }
        if let Some(found) = self.lexicon.lookup(word) {
            return found;
        }
        let letters = word.chars().count();
        let all_capitals = word.chars().all(char::is_uppercase);
        if (letters == 1 || (all_capitals && letters <= MAX_ACRONYM)) && word.is_ascii() {
            return self.spell_out(word);
        }
        if let Some(joined) = self.split_compound(&lower) {
            return joined;
        }
        let plain = lower.replace('\'', "");
        self.lexicon
            .lookup(&plain)
            .unwrap_or_else(|| spelling::pronounce(&plain))
    }

    fn spell_out(&self, word: &str) -> String {
        word.chars()
            .filter_map(|c| self.lexicon.lookup(&c.to_ascii_uppercase().to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// An unknown word that is two known words written together ("firefight").
    fn split_compound(&self, word: &str) -> Option<String> {
        word.char_indices()
            .map(|(at, _)| at)
            .filter(|at| *at >= MIN_PART && word.len() - at >= MIN_PART)
            .find_map(|at| {
                let (head, tail) = word.split_at(at);
                let (a, b) = (self.lexicon.lookup(head)?, self.lexicon.lookup(tail)?);
                Some(format!("{a}{}", b.replace('ˈ', "ˌ")))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phonemizer() -> Phonemizer {
        let gold = r#"{"hello":"həlˈO","world":"wˈɜɹld","the":"ðə","cat":"kˈæt","fire":"fˈIɚ","fight":"fˈIt",
            "one":"wˈʌn","two":"tˈu","hundred":"hˈʌndɹəd","twenty":"twˈɛnti","four":"fˈɔɹ","point":"pˈYnt",
            "five":"fˈIv","percent":"pəɹsˈɛnt","mister":"mˈɪstəɹ","smith":"smˈɪθ","A":"A","B":"bˈi","C":"sˈi",
            "and":"ænd","dollars":"dˈɑləɹz","cents":"sˈɛnts","first":"fˈɜɹst","third":"θˈɜɹd","fifty":"fˈɪfti"}"#;
        Phonemizer::new(Lexicon::from_json(gold, "{}").unwrap())
    }

    #[test]
    fn words_are_separated_by_spaces_and_punctuation_stays_attached() {
        assert_eq!(phonemizer().phonemize("Hello, world!"), "həlˈO, wˈɜɹld!");
        assert_eq!(
            phonemizer().phonemize("the  cat\nthe cat."),
            "ðə kˈæt ðə kˈæt."
        );
    }

    #[test]
    fn numbers_ordinals_percentages_and_money_become_words() {
        assert_eq!(phonemizer().phonemize("124"), "wˈʌn hˈʌndɹəd twˈɛnti fˈɔɹ");
        assert_eq!(phonemizer().phonemize("5%"), "fˈIv pəɹsˈɛnt");
        assert_eq!(phonemizer().phonemize("1st 3rd"), "fˈɜɹst θˈɜɹd");
        assert_eq!(phonemizer().phonemize("$5"), "fˈIv dˈɑləɹz");
        assert_eq!(
            phonemizer().phonemize("$5.50"),
            "fˈIv dˈɑləɹz fˈɪfti sˈɛnts"
        );
        assert_eq!(phonemizer().phonemize("2.5").split(' ').next(), Some("tˈu"));
    }

    #[test]
    fn abbreviations_acronyms_and_symbols() {
        assert_eq!(phonemizer().phonemize("Mr. Smith"), "mˈɪstəɹ. smˈɪθ");
        assert_eq!(phonemizer().phonemize("ABC"), "A bˈi sˈi");
        assert_eq!(phonemizer().phonemize("cat & cat"), "kˈæt ænd kˈæt");
    }

    #[test]
    fn quotes_and_brackets_hug_the_words_inside_but_not_the_words_around() {
        assert_eq!(
            phonemizer().phonemize("the \"cat\" (hello) the"),
            "ðə \"kˈæt\" (həlˈO) ðə"
        );
    }

    #[test]
    fn two_known_words_joined_are_said_as_one_word() {
        assert_eq!(phonemizer().phonemize("firefight"), "fˈIɚfˌIt");
    }

    #[test]
    fn an_unknown_word_is_guessed_from_its_spelling() {
        assert_eq!(phonemizer().phonemize("zork"), spelling::pronounce("zork"));
        assert_ne!(phonemizer().phonemize("zork"), "");
    }

    #[test]
    fn nothing_but_symbols_gives_nothing() {
        assert_eq!(phonemizer().phonemize("***"), "");
        assert_eq!(phonemizer().phonemize(""), "");
    }
}
