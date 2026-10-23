//! A stand-in language model for tests.

use crate::engines::{Control, TextEngine};
use crate::{Result, Segment, Transcript};

/// Answers with the first word of the input in angle brackets, and records what it was asked.
#[derive(Default)]
pub struct Echo {
    pub calls: Vec<(String, String)>,
}

impl TextEngine for Echo {
    fn generate(
        &mut self,
        instruction: &str,
        input: &str,
        _max_tokens: usize,
        _on_progress: &mut dyn FnMut(usize) -> Control,
    ) -> Result<String> {
        self.calls.push((instruction.into(), input.into()));
        Ok(format!(
            "<{}>",
            input.split_whitespace().next().unwrap_or("")
        ))
    }
}

/// One segment per text, a second apart, all spoken by Ana, in English.
pub fn transcript(lines: &[&str]) -> Transcript {
    Transcript {
        segments: lines
            .iter()
            .enumerate()
            .map(|(i, text)| Segment {
                start_ms: i as u64 * 1000,
                end_ms: i as u64 * 1000 + 900,
                speaker: Some("Ana".into()),
                text: (*text).into(),
            })
            .collect(),
        language: Some("en".into()),
        duration_ms: 5000,
        model: Some("whisper-base".into()),
    }
}
