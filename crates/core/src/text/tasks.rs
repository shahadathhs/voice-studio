//! What the language model is asked to do with a transcript: summarise it, or translate it.
//! Long transcripts are cut into parts, summarised one by one and the summaries merged.

use super::chunks;
use super::steps::{Steps, ask, never_cancel, target_language, translation_instruction};
use crate::engines::{Control, TextEngine};
use crate::{Error, Result, Segment, Transcript};

const SUMMARY_TOKENS: usize = 600;
const MIN_TRANSLATION_TOKENS: usize = 48;
const MAX_TRANSLATION_TOKENS: usize = 1024;

fn summary_instruction() -> &'static str {
    "You summarise transcripts of spoken recordings. Write a short overview in one paragraph, \
     then at most five key points, each on its own line starting with \"- \". Put the summary \
     in your own words and do not copy lines from the transcript. Use only what the transcript \
     says and never invent details."
}

fn merge_instruction() -> &'static str {
    "You are given summaries of consecutive parts of one recording. Merge them into one summary: \
     a short overview in one paragraph, then at most five key points, each on its own line \
     starting with \"- \". Use only what the summaries say."
}

/// A summary of the transcript: an overview and key points. `language` is a code such as `es`;
/// `None` writes it in the language of the transcript. `on_progress` receives 0 to 1.
pub fn summarize(
    engine: &mut dyn TextEngine,
    transcript: &Transcript,
    language: Option<&str>,
    on_progress: &mut dyn FnMut(f32) -> Control,
) -> Result<String> {
    let target = target_language(transcript, language)?;
    let parts = chunks::parts(&chunks::lines_of(transcript), chunks::PART_CHARS);
    if parts.is_empty() {
        return Err(Error::Input("there is no text to summarise".into()));
    }
    let merging = parts.len() > 1;
    let mut steps = Steps {
        done: 0,
        total: parts.len() + usize::from(merging) + usize::from(target.is_some()),
        on_progress,
    };
    let mut ask = |instruction: &str, input: &str, steps: &mut Steps| {
        ask(engine, steps, instruction, input, SUMMARY_TOKENS)
    };
    let mut summaries = Vec::with_capacity(parts.len());
    for part in &parts {
        summaries.push(ask(summary_instruction(), &part.text, &mut steps)?);
    }
    let mut summary = if merging {
        ask(merge_instruction(), &summaries.join("\n\n"), &mut steps)?
    } else {
        summaries.remove(0)
    };
    if let Some(code) = target {
        summary = ask(&translation_instruction(code)?, &summary, &mut steps)?;
    }
    Ok(summary)
}

/// The transcript with every line translated into `language` (a code such as `bn`); times and
/// speakers stay as they are.
pub fn translate(
    engine: &mut dyn TextEngine,
    transcript: &Transcript,
    language: &str,
    on_progress: &mut dyn FnMut(f32) -> Control,
) -> Result<Transcript> {
    let instruction = translation_instruction(language)?;
    let mut steps = Steps {
        done: 0,
        total: transcript.segments.len(),
        on_progress,
    };
    let mut segments = Vec::with_capacity(transcript.segments.len());
    for segment in &transcript.segments {
        let text = if segment.text.trim().is_empty() {
            String::new()
        } else {
            let budget = (segment.text.chars().count() * 3)
                .clamp(MIN_TRANSLATION_TOKENS, MAX_TRANSLATION_TOKENS);
            engine.generate(&instruction, segment.text.trim(), budget, &mut never_cancel)?
        };
        segments.push(Segment {
            text,
            ..segment.clone()
        });
        steps.finish_one()?;
    }
    Ok(Transcript {
        segments,
        language: Some(language.to_ascii_lowercase()),
        ..transcript.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::super::fake::{Echo, transcript};
    use super::*;

    #[test]
    fn a_short_transcript_is_summarised_in_one_step() {
        let mut engine = Echo::default();
        let summary = summarize(
            &mut engine,
            &transcript(&["hello there"]),
            Some("es"),
            &mut |_| Control::Continue,
        )
        .unwrap();
        assert_eq!(summary, "<<Ana:>>");
        assert_eq!(engine.calls.len(), 2, "the summary, then its translation");
        assert!(engine.calls[1].0.contains("into Spanish"));
    }

    #[test]
    fn a_summary_in_the_spoken_language_is_not_translated() {
        let mut engine = Echo::default();
        summarize(
            &mut engine,
            &transcript(&["hello"]),
            Some("en"),
            &mut |_| Control::Continue,
        )
        .unwrap();
        assert_eq!(engine.calls.len(), 1);
    }

    #[test]
    fn a_long_transcript_is_summarised_in_parts_and_merged() {
        let long = "word ".repeat(1000);
        let mut engine = Echo::default();
        let mut reported = Vec::new();
        summarize(&mut engine, &transcript(&[&long, &long]), None, &mut |f| {
            reported.push(f);
            Control::Continue
        })
        .unwrap();
        assert_eq!(engine.calls.len(), 3, "two parts and the merge");
        assert!(engine.calls[2].0.contains("Merge them"));
        assert_eq!(reported.last(), Some(&1.0));
    }

    #[test]
    fn translation_keeps_times_and_speakers() {
        let mut engine = Echo::default();
        let result = translate(
            &mut engine,
            &transcript(&["good morning", "", "bye now"]),
            "bn",
            &mut |_| Control::Continue,
        )
        .unwrap();
        assert_eq!(result.language.as_deref(), Some("bn"));
        assert_eq!(result.segments[0].text, "<good>");
        assert_eq!(result.segments[1].text, "");
        assert_eq!(result.segments[2].start_ms, 2000);
        assert_eq!(result.segments[2].speaker.as_deref(), Some("Ana"));
        assert!(engine.calls[0].0.contains("into Bengali"));
        assert_eq!(engine.calls.len(), 2, "the empty line is not sent");
    }

    #[test]
    fn unknown_languages_and_empty_transcripts_are_input_errors() {
        let mut engine = Echo::default();
        let go = &mut |_: f32| Control::Continue;
        assert!(translate(&mut engine, &transcript(&["a"]), "xx", go).is_err());
        assert!(summarize(&mut engine, &transcript(&["a"]), Some("xx"), go).is_err());
        assert!(summarize(&mut engine, &transcript(&[]), None, go).is_err());
    }

    #[test]
    fn cancelling_stops_the_work() {
        let mut engine = Echo::default();
        let result = translate(&mut engine, &transcript(&["a b", "c d"]), "es", &mut |_| {
            Control::Cancel
        });
        assert!(matches!(result, Err(Error::Cancelled)));
        assert_eq!(engine.calls.len(), 1);
    }
}
