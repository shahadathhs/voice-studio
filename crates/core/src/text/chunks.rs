//! Cutting a long transcript into parts that fit the language model.

use crate::Transcript;

/// Characters per part. About 1,500 words of English, and few enough tokens in Chinese or Bengali
/// that the part and its answer still fit the model's memory.
pub const PART_CHARS: usize = 6000;

/// One line per segment, with the speaker in front when there is one.
pub fn lines_of(transcript: &Transcript) -> Vec<String> {
    transcript
        .segments
        .iter()
        .filter(|s| !s.text.trim().is_empty())
        .map(|s| match &s.speaker {
            Some(speaker) => format!("{speaker}: {}", s.text.trim()),
            None => s.text.trim().to_string(),
        })
        .collect()
}

/// Joins lines into parts of at most `limit` characters, never splitting a line (a single longer
/// line is a part of its own).
pub fn parts(lines: &[String], limit: usize) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for line in lines {
        match parts.last_mut() {
            Some(last) if last.chars().count() + line.chars().count() < limit => {
                last.push('\n');
                last.push_str(line);
            }
            _ => parts.push(line.clone()),
        }
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Segment;

    fn segment(speaker: Option<&str>, text: &str) -> Segment {
        Segment {
            start_ms: 0,
            end_ms: 1,
            speaker: speaker.map(String::from),
            text: text.into(),
        }
    }

    #[test]
    fn lines_carry_speakers_and_skip_blank_text() {
        let transcript = Transcript {
            segments: vec![
                segment(Some("Ana"), " Hello "),
                segment(None, "  "),
                segment(None, "Bye"),
            ],
            ..Transcript::default()
        };
        assert_eq!(lines_of(&transcript), ["Ana: Hello", "Bye"]);
    }

    #[test]
    fn parts_keep_lines_whole_and_respect_the_limit() {
        let lines: Vec<String> = ["aaaa", "bbbb", "cccc"].map(String::from).to_vec();
        assert_eq!(parts(&lines, 10), ["aaaa\nbbbb", "cccc"]);
        assert_eq!(parts(&lines, 100), ["aaaa\nbbbb\ncccc"]);
        assert_eq!(parts(&lines, 2), ["aaaa", "bbbb", "cccc"]);
        assert_eq!(parts(&[], 10).len(), 0);
    }
}
