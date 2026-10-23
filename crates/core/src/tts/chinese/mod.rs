//! Mandarin Chinese: characters to pinyin to sounds. The pinyin comes from two word lists (see
//! [`Dictionary`]), tones change the way they do in speech (see [`sandhi`]), and each syllable is
//! written as the sounds Kokoro knows with an arrow for its tone (see [`pinyin`]).

mod dictionary;
mod numbers;
mod pinyin;
mod sandhi;

pub use dictionary::Dictionary;

use pinyin::Syllable;

/// Turns Chinese text into phonemes.
pub struct Chinese {
    dictionary: Dictionary,
}

/// Punctuation as the model knows it; the full-width marks of Chinese become the usual ones.
fn punctuation(c: char) -> Option<char> {
    match c {
        '，' | '、' => Some(','),
        '。' => Some('.'),
        '！' => Some('!'),
        '？' => Some('?'),
        '；' => Some(';'),
        '：' => Some(':'),
        '（' => Some('('),
        '）' => Some(')'),
        '…' | '—' | '“' | '”' | ',' | '.' | '!' | '?' | ';' | ':' | '(' | ')' | '"' => {
            Some(c)
        }
        _ => None,
    }
}

fn is_character(c: char) -> bool {
    matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '〇')
}

fn is_number_char(c: char) -> bool {
    c.is_ascii_digit() || c == '.'
}

impl Chinese {
    pub fn new(dictionary: Dictionary) -> Chinese {
        Chinese { dictionary }
    }

    /// The phonemes of a run of characters, syllables written one after another.
    fn run(&self, run: &[char]) -> String {
        let mut syllables = self.dictionary.syllables(run);
        sandhi::apply(&mut syllables, run);
        syllables.iter().map(Syllable::phonemes).collect()
    }

    /// The phonemes of `text`.
    pub fn phonemize(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::new();
        let mut at = 0;
        while at < chars.len() {
            let c = chars[at];
            let end =
                |keep: fn(char) -> bool| at + chars[at..].iter().take_while(|c| keep(**c)).count();
            if is_character(c) {
                let stop = end(is_character);
                out.push_str(&self.run(&chars[at..stop]));
                at = stop;
            } else if c.is_ascii_digit() {
                let stop = end(is_number_char);
                let number: String = chars[at..stop].iter().collect();
                let spoken = number_words(&number, chars.get(stop));
                out.push_str(&self.run(&spoken.chars().collect::<Vec<_>>()));
                at = if chars.get(stop) == Some(&'%') {
                    stop + 1
                } else {
                    stop
                };
            } else {
                if let Some(mark) = punctuation(c) {
                    out.push(mark);
                }
                at += 1;
            }
        }
        out
    }
}

/// A number as characters; a year before 年 is read digit by digit and a percentage is
/// "百分之" in front.
fn number_words(digits: &str, next: Option<&char>) -> String {
    match next {
        Some('年') if digits.len() == 4 => numbers::by_digit(digits),
        Some('%') => format!("百分之{}", numbers::spoken(digits)),
        _ => numbers::spoken(digits),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chinese() -> Chinese {
        let characters = "U+4F60: nǐ  # 你\nU+597D: hǎo  # 好\nU+5E74: nián  # 年\nU+4E8C: èr  # 二\nU+96F6: líng  # 零\nU+56DB: sì  # 四\nU+4E94: wǔ  # 五\nU+767E: bǎi  # 百\nU+5206: fēn  # 分\nU+4E4B: zhī  # 之\nU+5341: shí  # 十\n";
        Chinese::new(Dictionary::parse(characters, "").unwrap())
    }

    #[test]
    fn characters_become_syllables_and_full_width_marks_become_ordinary_ones() {
        assert_eq!(chinese().phonemize("你好。"), "ni↗xau↓.");
        assert_eq!(chinese().phonemize("好，好！"), "xau↓,xau↓!");
    }

    #[test]
    fn years_are_read_digit_by_digit_and_percentages_have_their_word() {
        assert_eq!(chinese().phonemize("2024年"), "ɚ↘liŋ↗ɚ↘sɨ↘njɛn↗");
        assert!(chinese().phonemize("50%").starts_with("pai"));
    }
}
