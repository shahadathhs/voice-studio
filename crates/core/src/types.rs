//! Data types shared across the core. The JSON form of [`Transcript`] is the
//! interchange format used by `vtt export`, `--json` output and the app's store.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    /// Display label, e.g. `SPEAKER_00` or a name the user chose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Transcript {
    pub segments: Vec<Segment>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub model: Option<String>,
}

impl Transcript {
    /// Words in the whole transcript, counted as whitespace-separated runs.
    pub fn word_count(&self) -> usize {
        self.segments
            .iter()
            .map(|s| s.text.split_whitespace().count())
            .sum()
    }

    /// Speaking time per speaker, longest first (ties keep the order of first appearance).
    /// Lines without a speaker are not counted.
    pub fn talk_time_by_speaker(&self) -> Vec<(String, u64)> {
        let mut totals: Vec<(String, u64)> = Vec::new();
        for segment in &self.segments {
            let Some(name) = segment.speaker.as_deref() else {
                continue;
            };
            let spoken = segment.end_ms.saturating_sub(segment.start_ms);
            match totals.iter_mut().find(|(known, _)| known == name) {
                Some((_, total)) => *total += spoken,
                None => totals.push((name.to_string(), spoken)),
            }
        }
        totals.sort_by_key(|(_, total)| std::cmp::Reverse(*total));
        totals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(start_ms: u64, end_ms: u64, speaker: Option<&str>, text: &str) -> Segment {
        Segment {
            start_ms,
            end_ms,
            speaker: speaker.map(str::to_string),
            text: text.into(),
        }
    }

    #[test]
    fn words_are_counted_across_lines() {
        let transcript = Transcript {
            segments: vec![
                line(0, 1, None, "one two  three"),
                line(1, 2, None, " four "),
            ],
            ..Transcript::default()
        };
        assert_eq!(transcript.word_count(), 4);
    }

    #[test]
    fn talk_time_is_summed_per_speaker_and_sorted_longest_first() {
        let transcript = Transcript {
            segments: vec![
                line(0, 1000, Some("Lee"), "a"),
                line(1000, 4000, Some("Maria"), "b"),
                line(4000, 5000, Some("Lee"), "c"),
                line(5000, 9000, None, "d"),
            ],
            ..Transcript::default()
        };
        assert_eq!(
            transcript.talk_time_by_speaker(),
            vec![("Maria".to_string(), 3000), ("Lee".to_string(), 2000)]
        );
    }
}
