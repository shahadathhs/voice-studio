//! Progress while a file is transcribed and then labelled with speakers.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use vtt_core::engines::{AudioBuffer, Control, DiarizeOptions, Diarizer, SttEngine, SttOptions};
use vtt_core::transcribe::{Speakers, transcribe_file};
use vtt_core::{Result, Transcript};

/// Finishes at once, reporting the whole audio as done.
struct QuickEngine;

impl SttEngine for QuickEngine {
    fn transcribe(
        &mut self,
        audio: &AudioBuffer,
        _options: &SttOptions,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript> {
        let total = audio.duration_ms();
        on_progress(total, total);
        Ok(Transcript {
            duration_ms: total,
            ..Transcript::default()
        })
    }
}

/// Reports a quarter, then all of its work.
struct QuickLabeller;

impl Diarizer for QuickLabeller {
    fn label_speakers(
        &mut self,
        _audio: &AudioBuffer,
        _transcript: &mut Transcript,
        _options: &DiarizeOptions,
        on_progress: &mut dyn FnMut(f32) -> Control,
    ) -> Result<()> {
        on_progress(0.25);
        on_progress(1.0);
        Ok(())
    }
}

fn run(with_speakers: bool) -> (u64, Vec<u64>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.wav");
    let mut seen = Vec::new();
    let mut total_seen = 0;
    let mut labeller = QuickLabeller;
    let options = DiarizeOptions::default();
    let speakers = with_speakers.then_some(Speakers {
        diarizer: &mut labeller,
        options: &options,
    });
    transcribe_file(
        &mut QuickEngine,
        &path,
        &SttOptions::default(),
        speakers,
        &mut |done, total| {
            seen.push(done);
            total_seen = total;
            Control::Continue
        },
    )
    .unwrap();
    (total_seen, seen)
}

#[test]
fn without_speakers_transcription_is_all_of_the_progress() {
    let (total, seen) = run(false);
    assert_eq!(seen.last().copied(), Some(total));
    assert!(seen.windows(2).all(|p| p[0] <= p[1]), "{seen:?}");
}

#[test]
fn with_speakers_transcription_is_the_first_half_and_labelling_the_second() {
    let (total, seen) = run(true);
    assert_eq!(seen.first().copied(), Some(total / 2));
    assert!(seen.windows(2).all(|p| p[0] <= p[1]), "{seen:?}");
    // A quarter of the labelling is 5/8 of the whole.
    assert!(
        seen.contains(&(total / 2 + total / 2 * 250 / 1000)),
        "{seen:?}"
    );
    assert_eq!(seen.last().copied(), Some(total));
}
