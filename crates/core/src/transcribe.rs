//! Turning an audio or video file into a transcript.

use std::path::Path;

use crate::audio;
use crate::engines::{Control, SttEngine, SttOptions};
use crate::{Result, Transcript};

/// Decodes `path`, runs `engine` on it and returns the transcript. `on_progress` receives
/// `(processed_ms, total_ms)` and can cancel by returning [`Control::Cancel`].
pub fn transcribe_file(
    engine: &mut dyn SttEngine,
    path: &Path,
    options: &SttOptions,
    on_progress: &mut dyn FnMut(u64, u64) -> Control,
) -> Result<Transcript> {
    let audio = audio::load_for_whisper(path)?;
    engine.transcribe(&audio, options, on_progress)
}

#[cfg(test)]
mod tests {
    use super::*;
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
            translate: false,
        };
        let transcript = transcribe_file(&mut FakeEngine, &path, &options, &mut |_, _| {
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
            &mut |_, _| Control::Continue,
        );
        assert!(matches!(result, Err(crate::Error::Input(_))));
    }
}
