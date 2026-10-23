//! Hindi sentences are understood when spoken. Ignored by default because it needs two models; see `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_hindi_e2e -- --ignored --nocapture
//! ```
mod speech_check;

use speech_check::{Case, check};

const CASES: &[Case] = &[
    // Add sentences here: Case { voice, language, text, allowed }.
];

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn spoken_sentences_are_understood_by_a_recogniser() {
    check(CASES);
}
