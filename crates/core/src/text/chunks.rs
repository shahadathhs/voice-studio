//! Cutting a long transcript into parts that fit the language model.

use crate::Transcript;

/// Characters per part. About 1,500 words of English, and few enough tokens in Chinese or Bengali
/// that the part and its answer still fit the model's memory.
pub const PART_CHARS: usize = 6000;

/// One line of the transcript: the segment's text with its speaker, and when it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub start_ms: u64,
    pub text: String,
}

/// A stretch of the transcript that is given to the model in one go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    /// Where its first line starts.
    pub start_ms: u64,
    pub text: String,
}

/// One line per segment, with the speaker in front when there is one; blank segments are left out.
pub fn lines_of(transcript: &Transcript) -> Vec<Line> {
    transcript
        .segments
        .iter()
        .filter(|s| !s.text.trim().is_empty())
        .map(|s| Line {
            start_ms: s.start_ms,
            text: match &s.speaker {
                Some(speaker) => format!("{speaker}: {}", s.text.trim()),
                None => s.text.trim().to_string(),
            },
        })
        .collect()
}

/// Joins lines into parts of at most `limit` characters, never splitting a line (a single longer
/// line is a part of its own).
pub fn parts(lines: &[Line], limit: usize) -> Vec<Part> {
    let mut parts: Vec<Part> = Vec::new();
    for line in lines {
        match parts.last_mut() {
            Some(last) if last.text.chars().count() + line.text.chars().count() < limit => {
                last.text.push('\n');
                last.text.push_str(&line.text);
            }
            _ => parts.push(Part {
                start_ms: line.start_ms,
                text: line.text.clone(),
            }),
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
        let texts: Vec<_> = lines_of(&transcript).into_iter().map(|l| l.text).collect();
        assert_eq!(texts, ["Ana: Hello", "Bye"]);
    }

    fn line(start_ms: u64, text: &str) -> Line {
        Line {
            start_ms,
            text: text.into(),
        }
    }

    #[test]
    fn parts_keep_lines_whole_and_respect_the_limit() {
        let lines = [line(0, "aaaa"), line(10, "bbbb"), line(20, "cccc")];
        let texts = |limit| -> Vec<(u64, String)> {
            parts(&lines, limit)
                .into_iter()
                .map(|p| (p.start_ms, p.text))
                .collect()
        };
        assert_eq!(texts(10), [(0, "aaaa\nbbbb".into()), (20, "cccc".into())]);
        assert_eq!(texts(100), [(0, "aaaa\nbbbb\ncccc".into())]);
        assert_eq!(texts(2).len(), 3);
        assert_eq!(parts(&[], 10).len(), 0);
    }
}
