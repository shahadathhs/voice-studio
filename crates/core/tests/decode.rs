//! Decodes real encoded files (a 1 s, 440 Hz, 44.1 kHz stereo tone in three containers).
#![allow(clippy::unwrap_used, clippy::cast_precision_loss)]

use std::path::PathBuf;

use vtt_core::audio::{WHISPER_SAMPLE_RATE, load_for_whisper};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Rising zero crossings per second, which for a sine is its frequency in Hz.
fn frequency(samples: &[f32]) -> f32 {
    let mid = &samples[samples.len() / 10..samples.len() * 9 / 10];
    let crossings = mid.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
    crossings as f32 / (mid.len() as f32 / WHISPER_SAMPLE_RATE as f32)
}

fn check_tone(name: &str) {
    let audio = load_for_whisper(&fixture(name)).unwrap();
    assert_eq!(audio.sample_rate, WHISPER_SAMPLE_RATE, "{name}");
    // AAC adds a little encoder padding, so allow a small difference in length.
    let seconds = audio.samples.len() as f32 / WHISPER_SAMPLE_RATE as f32;
    assert!((0.95..=1.15).contains(&seconds), "{name}: {seconds} s");
    let hz = frequency(&audio.samples);
    assert!((hz - 440.0).abs() < 8.0, "{name}: {hz} Hz");
    let peak = audio.samples.iter().fold(0.0f32, |p, s| p.max(s.abs()));
    assert!((0.7..=0.9).contains(&peak), "{name}: peak {peak}");
}

#[test]
fn decodes_wav() {
    check_tone("tone.wav");
}

#[test]
fn decodes_flac() {
    check_tone("tone.flac");
}

#[test]
fn decodes_aac_in_an_m4a_container() {
    check_tone("tone.m4a");
}
