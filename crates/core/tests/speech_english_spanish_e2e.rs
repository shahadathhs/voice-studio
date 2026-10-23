//! English and Spanish sentences are understood when spoken. Ignored by default because it needs two models; see `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_english_spanish_e2e -- --ignored --nocapture
//! ```
mod speech_check;

use speech_check::{Case, check};

const CASES: &[Case] = &[
    Case {
        voice: "af_heart",
        language: "en",
        text: "The quick brown fox jumps over the lazy dog.",
        allowed: 0.15,
    },
    Case {
        voice: "bf_emma",
        language: "en",
        text: "Good morning, the weather is lovely today.",
        allowed: 0.2,
    },
    Case {
        voice: "ef_dora",
        language: "es",
        text: "Hola, buenos días. Hoy vamos a hablar de la historia de México.",
        allowed: 0.3,
    },
    Case {
        voice: "em_alex",
        language: "es",
        text: "Mañana tengo una reunión importante con mis compañeros de trabajo.",
        allowed: 0.3,
    },
];

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn spoken_sentences_are_understood_by_a_recogniser() {
    check(CASES);
}
