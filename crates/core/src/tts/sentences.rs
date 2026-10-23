//! Cutting text into pieces the model can say naturally: one sentence at a time, with a longer
//! pause between paragraphs. Sentences that are too long are cut again at commas or spaces.

/// Words after which a full stop does not end a sentence.
const ABBREVIATIONS: [&str; 11] = [
    "mr", "mrs", "ms", "dr", "vs", "etc", "prof", "sr", "jr", "st", "no",
];

/// A piece of text and how long to pause after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub paragraph_end: bool,
}

fn ends_sentence(text: &str, at: usize, next: Option<char>) -> bool {
    let c = text[at..].chars().next().unwrap_or(' ');
    // Chinese, Japanese and Hindi marks end a sentence even without a space after them.
    if matches!(c, '。' | '！' | '？' | '।' | '॥') {
        return true;
    }
    if !matches!(c, '.' | '!' | '?' | '…')
        || next.is_some_and(|n| !n.is_whitespace() && !"\"”')".contains(n))
    {
        return false;
    }
    if c != '.' {
        return true;
    }
    let word: String = text[..at]
        .chars()
        .rev()
        .take_while(|w| w.is_alphabetic())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let single_capital = word.chars().count() == 1 && word.chars().all(char::is_uppercase);
    !(single_capital || ABBREVIATIONS.contains(&word.to_lowercase().as_str()))
}

/// Splits one paragraph into sentences.
fn sentences(paragraph: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut start = 0;
    for (at, c) in paragraph.char_indices() {
        let next = paragraph[at + c.len_utf8()..].chars().next();
        if ends_sentence(paragraph, at, next) {
            let mut end = at + c.len_utf8();
            // Closing quotes and brackets belong to the sentence they close.
            while let Some(q) = paragraph[end..]
                .chars()
                .next()
                .filter(|q| "\"”')".contains(*q))
            {
                end += q.len_utf8();
            }
            found.push(paragraph[start..end].trim().to_string());
            start = end;
        }
    }
    found.push(paragraph[start..].trim().to_string());
    found.retain(|s| !s.is_empty());
    found
}

/// The text as pieces in reading order.
pub fn pieces(text: &str) -> Vec<Piece> {
    let normalized = text.replace("\r\n", "\n");
    let mut pieces = Vec::new();
    for paragraph in normalized.split("\n\n") {
        let flat = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
        let found = sentences(&flat);
        let last = found.len().saturating_sub(1);
        pieces.extend(found.into_iter().enumerate().map(|(n, text)| Piece {
            text,
            paragraph_end: n == last,
        }));
    }
    pieces
}

/// Cuts `text` in two near its middle, preferably after a comma or semicolon, otherwise at a
/// space. `None` when there is nowhere to cut.
pub fn halve(text: &str) -> Option<(String, String)> {
    let middle = text.len() / 2;
    let candidates = |marks: &[char]| {
        text.char_indices()
            .filter(|(_, c)| marks.contains(c))
            .map(|(at, c)| at + c.len_utf8())
            .min_by_key(|at| at.abs_diff(middle))
    };
    let at = candidates(&[',', ';', ':', '，', '；', '：', '、']).or_else(|| candidates(&[' ']))?;
    let (head, tail) = text.split_at(at);
    let (head, tail) = (head.trim(), tail.trim());
    (!head.is_empty() && !tail.is_empty()).then(|| (head.to_string(), tail.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(text: &str) -> Vec<String> {
        pieces(text).into_iter().map(|p| p.text).collect()
    }

    #[test]
    fn text_is_cut_into_sentences() {
        assert_eq!(
            texts("Hello there. How are you? Fine!"),
            ["Hello there.", "How are you?", "Fine!"]
        );
    }

    #[test]
    fn abbreviations_and_initials_do_not_end_a_sentence() {
        assert_eq!(
            texts("Dr. Smith met J. K. Rowling. Then left."),
            ["Dr. Smith met J. K. Rowling.", "Then left."]
        );
        assert_eq!(
            texts("It cost 3.5 dollars. Fine."),
            ["It cost 3.5 dollars.", "Fine."]
        );
    }

    #[test]
    fn closing_quotes_stay_with_their_sentence() {
        assert_eq!(
            texts("He said \"stop.\" Then he ran."),
            ["He said \"stop.\"", "Then he ran."]
        );
    }

    #[test]
    fn blank_lines_mark_paragraph_ends_and_line_breaks_are_just_spaces() {
        let found = pieces("One two\nthree. Four.\n\nFive.");
        assert_eq!(found.len(), 3);
        assert_eq!(found[0].text, "One two three.");
        assert_eq!(
            found.iter().map(|p| p.paragraph_end).collect::<Vec<_>>(),
            [false, true, true]
        );
    }

    #[test]
    fn full_width_and_hindi_marks_end_sentences_without_a_space() {
        assert_eq!(texts("你好。今天很好！"), ["你好。", "今天很好！"]);
        assert_eq!(texts("यह घर है। वह स्कूल है।"), ["यह घर है।", "वह स्कूल है।"]);
    }

    #[test]
    fn empty_text_has_no_pieces() {
        assert_eq!(pieces("  \n\n ").len(), 0);
    }

    #[test]
    fn long_sentences_are_halved_at_a_comma_near_the_middle() {
        assert_eq!(
            halve("one two three, four five six").unwrap(),
            ("one two three,".to_string(), "four five six".to_string())
        );
        assert_eq!(halve("alpha beta gamma delta").unwrap().0, "alpha beta");
        assert_eq!(halve("unbroken"), None);
    }
}
