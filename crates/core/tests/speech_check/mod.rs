//! Shared by the `speech_*_e2e` tests: speak sentences with Kokoro, listen with Whisper, compare.
#![allow(clippy::unwrap_used, dead_code)]

use std::path::PathBuf;

use vtt_core::audio::encode_wav;
use vtt_core::engines::{Control, Device, SttOptions, TtsOptions, WhisperEngine};
use vtt_core::transcribe::transcribe_file;
use vtt_core::tts::KokoroEngine;

/// A sentence to say and how well it has to be understood.
pub struct Case {
    pub voice: &'static str,
    /// The language code Whisper is told to listen for.
    pub language: &'static str,
    pub text: &'static str,
    /// The largest share of letters that may differ from the text.
    pub allowed: f64,
}

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} to run this test")))
}

/// Letters only, lower case, so punctuation and spacing do not count.
fn letters(text: &str) -> Vec<char> {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Share of the longer text that differs (edit distance over length).
fn difference(a: &[char], b: &[char]) -> f64 {
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.iter().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let held = row[j + 1];
            row[j + 1] = (previous + usize::from(x != y))
                .min(row[j] + 1)
                .min(held + 1);
            previous = held;
        }
    }
    let count = |n: usize| f64::from(u32::try_from(n).unwrap_or(u32::MAX));
    count(row[b.len()]) / count(a.len().max(b.len()).max(1))
}

/// Speaks every case and fails if Whisper understood too little of any of them.
pub fn check(cases: &[Case]) {
    let mut kokoro = KokoroEngine::load(&env_path("VTT_TEST_KOKORO"), "kokoro-82m").unwrap();
    let mut whisper =
        WhisperEngine::load(&env_path("VTT_TEST_WHISPER"), "whisper", Device::Auto).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    for case in cases {
        let options = TtsOptions {
            voice: case.voice.into(),
            speed: 1.0,
        };
        let audio = kokoro
            .speak(case.text, &options, &mut |_, _| Control::Continue)
            .unwrap();
        let path = dir.path().join(format!("{}.wav", case.voice));
        std::fs::write(&path, encode_wav(&audio)).unwrap();
        let stt = SttOptions {
            language: Some(case.language.into()),
            ..SttOptions::default()
        };
        let heard = transcribe_file(&mut whisper, &path, &stt, None, &mut |_, _| {
            Control::Continue
        })
        .unwrap();
        let heard: String = heard
            .segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let off = difference(&letters(case.text), &letters(&heard));
        println!(
            "{}: {off:.2} off\n  said:  {}\n  heard: {heard}",
            case.voice, case.text
        );
        if off > case.allowed {
            failures.push(format!(
                "{}: {off:.2} > {}: {heard}",
                case.voice, case.allowed
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
