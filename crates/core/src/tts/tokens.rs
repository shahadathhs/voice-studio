//! Cutting text into words, numbers, punctuation and gaps.

/// Punctuation the model understands; anything else that is not a letter or digit is dropped.
const PUNCTUATION: &str = ";:,.!?—…\"()“”";
const ORDINAL_SUFFIXES: [&str; 4] = ["st", "nd", "rd", "th"];

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    Number(String),
    Ordinal(String),
    Dollars(String),
    Punct(char),
    Gap,
}

fn symbol_word(c: char) -> Option<&'static str> {
    match c {
        '&' => Some("and"),
        '@' => Some("at"),
        '+' => Some("plus"),
        '=' => Some("equals"),
        '%' => Some("percent"),
        _ => None,
    }
}

/// The run of characters from `start` that satisfy `keep`, which also sees the next character.
fn run(chars: &[char], start: usize, keep: impl Fn(char, Option<char>) -> bool) -> usize {
    let mut end = start;
    while end < chars.len() && keep(chars[end], chars.get(end + 1).copied()) {
        end += 1;
    }
    end
}

fn word_char(c: char, next: Option<char>) -> bool {
    c.is_alphabetic() || (matches!(c, '\'' | '’') && next.is_some_and(char::is_alphabetic))
}

fn number_char(c: char, next: Option<char>) -> bool {
    c.is_ascii_digit() || (matches!(c, ',' | '.') && next.is_some_and(|n| n.is_ascii_digit()))
}

/// An ordinal suffix ("st", "nd", "rd", "th") right after a number, if there is one.
fn ordinal_suffix(chars: &[char], at: usize) -> bool {
    let end = run(chars, at, |c, _| c.is_alphabetic());
    let suffix: String = chars[at..end].iter().collect::<String>().to_lowercase();
    ORDINAL_SUFFIXES.contains(&suffix.as_str())
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if word_char(c, chars.get(i + 1).copied()) {
            let end = run(&chars, i, word_char);
            tokens.push(Token::Word(
                chars[i..end].iter().collect::<String>().replace('’', "'"),
            ));
            i = end;
        } else if c.is_ascii_digit()
            || (c == '$' && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            let dollars = c == '$';
            let start = if dollars { i + 1 } else { i };
            let end = run(&chars, start, number_char);
            let number: String = chars[start..end].iter().collect();
            i = end;
            if dollars {
                tokens.push(Token::Dollars(number));
            } else if ordinal_suffix(&chars, end) {
                tokens.push(Token::Ordinal(number));
                i = run(&chars, end, |c, _| c.is_alphabetic());
            } else {
                tokens.push(Token::Number(number));
            }
        } else {
            tokens.extend(other(c));
            i += 1;
        }
    }
    tokens
}

/// The token for a character that is neither part of a word nor of a number.
fn other(c: char) -> Option<Token> {
    if PUNCTUATION.contains(c) {
        Some(Token::Punct(c))
    } else if c == '–' {
        Some(Token::Punct('—'))
    } else if let Some(word) = symbol_word(c) {
        Some(Token::Word(word.to_string()))
    } else if c.is_whitespace() || c == '-' || c == '/' {
        Some(Token::Gap)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contractions_and_curly_apostrophes_are_one_word() {
        assert_eq!(tokenize("don’t stop")[0], Token::Word("don't".into()));
    }

    #[test]
    fn numbers_ordinals_and_money_are_recognised() {
        assert_eq!(tokenize("1,250"), [Token::Number("1,250".into())]);
        assert_eq!(tokenize("21st"), [Token::Ordinal("21".into())]);
        assert_eq!(tokenize("$5.50"), [Token::Dollars("5.50".into())]);
        assert_eq!(
            tokenize("3.5."),
            [Token::Number("3.5".into()), Token::Punct('.')]
        );
    }

    #[test]
    fn spaces_and_hyphens_are_gaps_and_unknown_symbols_vanish() {
        assert_eq!(
            tokenize("a-b ~"),
            [
                Token::Word("a".into()),
                Token::Gap,
                Token::Word("b".into()),
                Token::Gap
            ]
        );
    }
}
