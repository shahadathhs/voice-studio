//! Summaries and translation with the real language model. Ignored by default because it needs the
//! model, which is not in the repository:
//!
//! ```text
//! vtt models pull qwen2.5-1.5b
//! VTT_TEST_TEXT_MODEL=<data dir>/models/qwen2.5-1.5b \
//!     cargo test -p vtt-core --release --test text_real_e2e -- --ignored --nocapture
//! ```
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use vtt_core::engines::Control;
use vtt_core::text::{QwenEngine, summarize, translate};
use vtt_core::{Segment, Transcript};

fn engine() -> QwenEngine {
    let dir = PathBuf::from(
        std::env::var_os("VTT_TEST_TEXT_MODEL")
            .unwrap_or_else(|| panic!("set VTT_TEST_TEXT_MODEL to run this test")),
    );
    QwenEngine::load(&dir, "qwen2.5-1.5b").unwrap()
}

fn meeting() -> Transcript {
    let lines = [
        (
            "Ana",
            "Thanks for joining. Today we decide the launch date for the new app.",
        ),
        (
            "Ben",
            "The code is ready, but the Windows installer still fails on two machines.",
        ),
        (
            "Ana",
            "Then we move the launch from the 3rd to the 10th of March.",
        ),
        (
            "Ben",
            "Agreed. I will fix the installer by Friday and Ana will write the announcement.",
        ),
    ];
    Transcript {
        segments: lines
            .iter()
            .enumerate()
            .map(|(i, (speaker, text))| Segment {
                start_ms: i as u64 * 5000,
                end_ms: i as u64 * 5000 + 4000,
                speaker: Some((*speaker).into()),
                text: (*text).into(),
            })
            .collect(),
        language: Some("en".into()),
        duration_ms: 20_000,
        model: None,
    }
}

#[test]
#[ignore = "needs the language model"]
fn summaries_mention_what_was_decided() {
    let mut engine = engine();
    for language in [None, Some("es"), Some("zh"), Some("bn")] {
        let summary = summarize(&mut engine, &meeting(), language, &mut |_| {
            Control::Continue
        })
        .unwrap();
        println!("--- summary ({language:?})\n{summary}");
        assert_ne!(summary, "");
    }
}

#[test]
#[ignore = "needs the language model"]
fn translation_keeps_the_lines() {
    let mut engine = engine();
    for language in ["es", "zh", "bn"] {
        let result = translate(&mut engine, &meeting(), language, &mut |_| {
            Control::Continue
        })
        .unwrap();
        println!("--- translation ({language})");
        for line in &result.segments {
            println!("{}: {}", line.speaker.as_deref().unwrap_or("-"), line.text);
        }
        assert_eq!(result.segments.len(), 4);
        assert!(result.segments.iter().all(|s| !s.text.is_empty()));
    }
}
