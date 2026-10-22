//! Speaks real sentences with the real Kokoro model and checks them by transcribing the result
//! with Whisper. Ignored by default because it needs models that are not in the repository.
//! Install them into a scratch data directory and run:
//!
//! ```text
//! vtt --data-dir /tmp/vtt-data models pull kokoro-82m
//! vtt --data-dir /tmp/vtt-data models pull whisper-base
//! VTT_TEST_DATA_DIR=/tmp/vtt-data \
//!     cargo test -p vtt-core --release --test tts_e2e -- --ignored --nocapture
//! ```
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use vtt_core::audio::encode_wav;
use vtt_core::engines::{Control, Device, SttOptions, TtsOptions, WhisperEngine};
use vtt_core::models::{Catalog, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::transcribe::transcribe_file;
use vtt_core::tts::KokoroEngine;

fn data_dir() -> Paths {
    let dir = std::env::var_os("VTT_TEST_DATA_DIR")
        .unwrap_or_else(|| panic!("set VTT_TEST_DATA_DIR to run this test"));
    Paths::resolve(Some(PathBuf::from(dir)))
}

/// Speaks `text` and returns what Whisper hears, in lower case without punctuation.
fn heard(
    engine: &mut KokoroEngine,
    whisper: &mut WhisperEngine,
    text: &str,
    voice: &str,
) -> String {
    let options = TtsOptions {
        voice: voice.into(),
        speed: 1.0,
    };
    let audio = engine
        .speak(text, &options, &mut |_, _| Control::Continue)
        .unwrap();
    assert_eq!(audio.sample_rate, 24_000);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("speech.wav");
    std::fs::write(&path, encode_wav(&audio)).unwrap();
    let transcript = transcribe_file(whisper, &path, &SttOptions::default(), None, &mut |_, _| {
        Control::Continue
    })
    .unwrap();
    let text: Vec<String> = transcript.segments.iter().map(|s| s.text.clone()).collect();
    text.join(" ")
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

#[test]
#[ignore = "needs VTT_TEST_DATA_DIR with kokoro-82m and whisper-base installed"]
fn speech_is_understood_by_whisper() {
    let paths = data_dir();
    let catalog = Catalog::bundled().unwrap();
    let store = ModelStore::new(&paths);
    let kokoro = catalog.find("kokoro-82m").unwrap();
    let whisper_entry = catalog.find("whisper-base").unwrap();
    let mut engine = KokoroEngine::load(&store.model_dir(kokoro), "kokoro-82m").unwrap();
    let whisper_path = store.file_path(whisper_entry, &whisper_entry.files[0]);
    let mut whisper = WhisperEngine::load(&whisper_path, "whisper-base", Device::Auto).unwrap();

    let simple = heard(
        &mut engine,
        &mut whisper,
        "The quick brown fox jumps over the lazy dog.",
        "af_heart",
    );
    println!("simple:  {simple}");
    assert!(simple.contains("quick brown fox"), "{simple}");
    assert!(simple.contains("lazy dog"), "{simple}");

    let numbers = heard(
        &mut engine,
        &mut whisper,
        "On March 3rd, 2024, we sold 1,250 tickets for 25 dollars each. That is 90% of the room!",
        "am_adam",
    );
    println!("numbers: {numbers}");
    // Whisper writes numbers as digits, so spoken words come back as "3rd", "2024", "1250".
    for expected in ["march", "3rd", "2024", "1250", "tickets", "25", "90"] {
        assert!(
            numbers.contains(expected),
            "missing '{expected}' in: {numbers}"
        );
    }

    let long = "Kokoro speaks one sentence at a time. It pauses between sentences, and a little longer between paragraphs.\n\nThis is the second paragraph.";
    let paragraphs = heard(&mut engine, &mut whisper, long, "af_bella");
    println!("long:    {paragraphs}");
    assert!(paragraphs.contains("second paragraph"), "{paragraphs}");
}
