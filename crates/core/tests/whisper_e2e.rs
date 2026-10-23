//! End-to-end transcription with a real model and real speech. Ignored by default because it needs
//! files that are not in the repository. Run with:
//!
//! ```text
//! VTT_TEST_MODEL=/path/to/ggml-base.bin VTT_TEST_SPEECH=/path/to/speech.m4a \
//!     cargo test -p vtt-core --test whisper_e2e -- --ignored
//! ```
//!
//! `VTT_TEST_SPEECH` must contain the phrase "quick brown fox".
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use vtt_core::engines::{Control, Device, SttOptions, WhisperEngine};
use vtt_core::transcribe::transcribe_file;

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} to run this test")))
}

#[test]
#[ignore = "needs VTT_TEST_MODEL and VTT_TEST_SPEECH"]
fn transcribes_real_speech_on_cpu_and_gpu() {
    let model = env_path("VTT_TEST_MODEL");
    let speech = env_path("VTT_TEST_SPEECH");
    for device in [Device::Cpu, Device::Auto] {
        let mut engine = WhisperEngine::load(&model, "test-model", device).unwrap();
        let mut last = (0, 0);
        let transcript = transcribe_file(
            &mut engine,
            &speech,
            &SttOptions::default(),
            None,
            &mut |done, total| {
                last = (done, total);
                Control::Continue
            },
        )
        .unwrap();
        let text = transcript
            .segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            text.to_lowercase().contains("quick brown fox"),
            "{device:?}: {text}"
        );
        assert_eq!(transcript.language.as_deref(), Some("en"));
        assert_eq!(last.0, last.1, "progress ends at 100%");
        assert!(
            transcript
                .segments
                .windows(2)
                .all(|w| w[0].end_ms <= w[1].start_ms + 1000)
        );
    }
}

#[test]
#[ignore = "needs VTT_TEST_MODEL and VTT_TEST_SPEECH"]
fn cancelling_stops_transcription() {
    let model = env_path("VTT_TEST_MODEL");
    let speech = env_path("VTT_TEST_SPEECH");
    let mut engine = WhisperEngine::load(&model, "test-model", Device::Cpu).unwrap();
    let result = transcribe_file(
        &mut engine,
        &speech,
        &SttOptions::default(),
        None,
        &mut |_, _| Control::Cancel,
    );
    assert!(
        matches!(result, Err(vtt_core::Error::Cancelled)),
        "{result:?}"
    );
}
