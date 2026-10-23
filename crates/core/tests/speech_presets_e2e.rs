//! A custom blended voice is understood when spoken. Ignored by default because it needs two models; see `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_presets_e2e -- --ignored --nocapture
//! ```
#![allow(clippy::unwrap_used)]
mod speech_check;

use std::path::PathBuf;

use speech_check::{Case, check_with_presets};
use vtt_core::tts::{Preset, save_preset};

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn a_blended_voice_is_understood_by_a_recogniser() {
    let shipped = PathBuf::from(std::env::var_os("VTT_TEST_KOKORO").unwrap());
    let presets = tempfile::tempdir().unwrap();
    let parts = [("af_heart".to_string(), 0.6), ("af_bella".to_string(), 0.4)];
    let preset = Preset::new("Warm", &parts).unwrap();
    save_preset(presets.path(), &shipped, &preset).unwrap();
    let cases = [Case {
        voice: "af_my-warm",
        language: "en",
        text: "The quick brown fox jumps over the lazy dog.",
        allowed: 0.2,
    }];
    check_with_presets(&cases, Some(presets.path()));
}
