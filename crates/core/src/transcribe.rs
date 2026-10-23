//! Turning an audio or video file into a transcript.

use std::ops::Range;
use std::path::Path;

use crate::engines::{AudioBuffer, Control, DiarizeOptions, Diarizer, SttEngine, SttOptions};
use crate::{Error, Result, Transcript};
use num_traits::ToPrimitive;

use crate::{audio, repetition};

/// Who-said-what labelling to run after the speech has been transcribed.
pub struct Speakers<'a> {
    pub diarizer: &'a mut dyn Diarizer,
    pub options: &'a DiarizeOptions,
}

/// Decodes `path`, runs `engine` on it and returns the transcript. `on_progress` receives
/// `(processed_ms, total_ms)` and can cancel by returning [`Control::Cancel`]. With
/// `options.vad`, only the stretches containing speech are transcribed. With `speakers`, every
/// line is then labelled with its speaker.
pub fn transcribe_file(
    engine: &mut dyn SttEngine,
    path: &Path,
    options: &SttOptions,
    speakers: Option<Speakers<'_>>,
    on_progress: &mut dyn FnMut(u64, u64) -> Control,
) -> Result<Transcript> {
    let audio = audio::load_for_whisper(path)?;
    let total = audio.duration_ms();
    // With speaker labels the work has two halves, which share the progress bar equally.
    let share = if speakers.is_some() { 2 } else { 1 };
    let mut transcript = {
        let mut progress = |done: u64, total: u64| on_progress(done / share, total);
        if options.vad {
            transcribe_speech(engine, &audio, options, &mut progress)?
        } else {
            repetition::transcribe_without_loops(engine, &audio, options, &mut progress)?
        }
    };
    if let Some(speakers) = speakers {
        speakers.diarizer.label_speakers(
            &audio,
            &mut transcript,
            speakers.options,
            &mut |part| {
                let done = total / 2 + (total / 2).saturating_mul(permille(part)) / 1000;
                on_progress(done, total)
            },
        )?;
        on_progress(total, total);
    }
    Ok(transcript)
}

/// A 0 to 1 fraction as thousandths (clamped), for scaling milliseconds without float casts.
fn permille(fraction: f32) -> u64 {
    (fraction.clamp(0.0, 1.0) * 1000.0)
        .round()
        .to_u64()
        .unwrap_or(0)
}

fn to_ms(samples: usize, rate: u32) -> u64 {
    samples as u64 * 1000 / u64::from(rate)
}

/// Transcribes each speech chunk on its own and puts the lines back at their place in the file.
fn transcribe_speech(
    engine: &mut dyn SttEngine,
    audio: &AudioBuffer,
    options: &SttOptions,
    on_progress: &mut dyn FnMut(u64, u64) -> Control,
) -> Result<Transcript> {
    let total = audio.duration_ms();
    if on_progress(0, total) == Control::Cancel {
        return Err(Error::Cancelled);
    }
    let mut merged = Transcript {
        duration_ms: total,
        ..Transcript::default()
    };
    for range in audio::speech_chunks(&audio.samples, audio.sample_rate) {
        let offset = to_ms(range.start, audio.sample_rate);
        let piece = transcribe_chunk(engine, audio, range, options, &mut |done| {
            on_progress(offset + done, total)
        })?;
        merged.language = merged.language.or(piece.language);
        merged.model = merged.model.or(piece.model);
        merged
            .segments
            .extend(piece.segments.into_iter().map(|mut line| {
                line.start_ms += offset;
                line.end_ms += offset;
                line
            }));
    }
    on_progress(total, total);
    Ok(merged)
}

fn transcribe_chunk(
    engine: &mut dyn SttEngine,
    audio: &AudioBuffer,
    range: Range<usize>,
    options: &SttOptions,
    on_progress: &mut dyn FnMut(u64) -> Control,
) -> Result<Transcript> {
    let chunk = AudioBuffer {
        samples: audio.samples[range].to_vec(),
        sample_rate: audio.sample_rate,
    };
    repetition::transcribe_without_loops(engine, &chunk, options, &mut |done, _| on_progress(done))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::synthetic::{recording, silence, voice};
    use crate::engines::AudioBuffer;

    /// An engine that "hears" one segment per second of audio, so no model is needed.
    struct FakeEngine;

    impl SttEngine for FakeEngine {
        fn transcribe(
            &mut self,
            audio: &AudioBuffer,
            options: &SttOptions,
            on_progress: &mut dyn FnMut(u64, u64) -> Control,
        ) -> Result<Transcript> {
            let total = audio.duration_ms();
            on_progress(total, total);
            let language = options.language.clone();
            Ok(Transcript {
                duration_ms: total,
                language,
                ..Transcript::default()
            })
        }
    }

    #[test]
    fn decodes_the_file_and_hands_16khz_audio_to_the_engine() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.wav");
        let options = SttOptions {
            language: Some("en".into()),
            ..SttOptions::default()
        };
        let transcript = transcribe_file(&mut FakeEngine, &path, &options, None, &mut |_, _| {
            Control::Continue
        })
        .unwrap();
        assert!(
            (950..=1050).contains(&transcript.duration_ms),
            "{}",
            transcript.duration_ms
        );
        assert_eq!(transcript.language.as_deref(), Some("en"));
    }

    #[test]
    fn an_unreadable_file_never_reaches_the_engine() {
        let result = transcribe_file(
            &mut FakeEngine,
            Path::new("/no/such.wav"),
            &SttOptions::default(),
            None,
            &mut |_, _| Control::Continue,
        );
        assert!(matches!(result, Err(crate::Error::Input(_))));
    }

    /// Hears one line per chunk, as long as the chunk, and counts how often it was called.
    #[derive(Default)]
    struct LineEngine {
        calls: usize,
    }

    impl SttEngine for LineEngine {
        fn transcribe(
            &mut self,
            audio: &AudioBuffer,
            _options: &SttOptions,
            on_progress: &mut dyn FnMut(u64, u64) -> Control,
        ) -> Result<Transcript> {
            self.calls += 1;
            let total = audio.duration_ms();
            if on_progress(total, total) == Control::Cancel {
                return Err(Error::Cancelled);
            }
            Ok(Transcript {
                segments: vec![crate::Segment {
                    start_ms: 0,
                    end_ms: total,
                    speaker: None,
                    text: format!("line {}", self.calls),
                }],
                language: Some("en".into()),
                model: Some("fake".into()),
                ..Transcript::default()
            })
        }
    }

    fn speech_options() -> SttOptions {
        SttOptions {
            vad: true,
            ..SttOptions::default()
        }
    }

    #[test]
    fn speech_chunks_are_transcribed_separately_and_placed_back_in_the_file() {
        let audio = recording(&[
            silence(1000),
            voice(2000),
            silence(4000),
            voice(1000),
            silence(1000),
        ]);
        let mut engine = LineEngine::default();

        let transcript = transcribe_speech(&mut engine, &audio, &speech_options(), &mut |_, _| {
            Control::Continue
        })
        .unwrap();

        assert_eq!(engine.calls, 2);
        assert_eq!(transcript.duration_ms, 9000);
        assert_eq!(transcript.language.as_deref(), Some("en"));
        assert_eq!(transcript.model.as_deref(), Some("fake"));
        let starts: Vec<u64> = transcript.segments.iter().map(|s| s.start_ms).collect();
        assert!((700..=800).contains(&starts[0]), "{starts:?}");
        assert!((6700..=6800).contains(&starts[1]), "{starts:?}");
    }

    #[test]
    fn silence_never_reaches_the_engine() {
        let audio = recording(&[silence(5000)]);
        let mut engine = LineEngine::default();
        let transcript = transcribe_speech(&mut engine, &audio, &speech_options(), &mut |_, _| {
            Control::Continue
        })
        .unwrap();
        assert_eq!(engine.calls, 0);
        assert_eq!(transcript.segments.len(), 0);
        assert_eq!(transcript.duration_ms, 5000);
    }

    #[test]
    fn progress_covers_the_whole_file_and_can_cancel() {
        let audio = recording(&[silence(1000), voice(2000), silence(4000), voice(1000)]);
        let mut seen = Vec::new();
        transcribe_speech(
            &mut LineEngine::default(),
            &audio,
            &speech_options(),
            &mut |done, total| {
                seen.push((done, total));
                Control::Continue
            },
        )
        .unwrap();
        assert_eq!(seen.first(), Some(&(0, 8000)));
        assert_eq!(seen.last(), Some(&(8000, 8000)));
        assert!(seen.windows(2).all(|w| w[0].0 <= w[1].0), "{seen:?}");

        let cancelled = transcribe_speech(
            &mut LineEngine::default(),
            &audio,
            &speech_options(),
            &mut |_, _| Control::Cancel,
        );
        assert!(matches!(cancelled, Err(Error::Cancelled)));
    }
}
