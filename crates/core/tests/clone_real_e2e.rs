//! Clones a voice from a recording, speaks English and Spanish with it and has Whisper listen.
//! Ignored by default because it needs the Chatterbox model (about 1.5 GB), Whisper and a
//! reference recording of a voice you may use (a Kokoro-made file works):
//!
//! ```text
//! VTT_TEST_CLONE=<data dir>/models/chatterbox-multilingual VTT_TEST_REFERENCE=reference.wav \
//!     VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test clone_real_e2e -- --ignored --nocapture
//! ```
#![allow(clippy::unwrap_used)]
mod speech_check;

use std::path::PathBuf;
use std::time::Instant;

use speech_check::{listen, off_by};
use vtt_core::audio::{encode_wav_with_note, load_for_whisper};
use vtt_core::engines::{Control, Device, WhisperEngine};
use vtt_core::tts::clone::{CloneEngine, Consent, DISCLOSURE};

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} to run this test")))
}

const CASES: &[(&str, &str, f64)] = &[
    ("en", "The quick brown fox jumps over the lazy dog.", 0.25),
    (
        "es",
        "Mañana tengo una reunión importante con mis compañeros de trabajo.",
        0.35,
    ),
];

#[test]
#[ignore = "needs the Chatterbox and Whisper models and a reference recording"]
fn a_cloned_voice_says_what_it_is_told() {
    let started = Instant::now();
    let mut engine = CloneEngine::load(&env_path("VTT_TEST_CLONE")).unwrap();
    println!("loaded in {:.1} s", started.elapsed().as_secs_f64());
    let reference = load_for_whisper(&env_path("VTT_TEST_REFERENCE")).unwrap();
    let started = Instant::now();
    let voice = engine
        .voice_from_audio(&reference, "Test voice", Consent::confirm(true).unwrap())
        .unwrap();
    println!("cloned in {:.1} s", started.elapsed().as_secs_f64());
    let mut whisper =
        WhisperEngine::load(&env_path("VTT_TEST_WHISPER"), "whisper", Device::Auto).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    for (language, text, allowed) in CASES {
        let started = Instant::now();
        let audio = engine
            .speak(&voice, text, language, &mut |_, _| Control::Continue)
            .unwrap();
        let compute = started.elapsed().as_secs_f64();
        let samples = f64::from(u32::try_from(audio.samples.len()).unwrap());
        let seconds = samples / f64::from(audio.sample_rate);
        println!(
            "{language}: {seconds:.1} s of audio in {compute:.1} s, RTF {:.2}",
            compute / seconds
        );
        let path = dir.path().join(format!("{language}.wav"));
        std::fs::write(&path, encode_wav_with_note(&audio, Some(DISCLOSURE))).unwrap();
        let heard = listen(&mut whisper, &path, language);
        let off = off_by(text, &heard);
        println!("  said:  {text}\n  heard: {heard}\n  {off:.2} off");
        if off > *allowed {
            failures.push(format!("{language}: {off:.2} > {allowed}: {heard}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
