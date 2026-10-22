//! Evaluates speaker identification with the real model on a small labelled set. Ignored by
//! default because it needs files that are not in the repository. Build the set (macOS) and run:
//!
//! ```text
//! scripts/make-diarization-eval.sh /tmp/diarization-eval
//! VTT_TEST_SPEAKER_MODEL=/path/to/voxceleb_resnet34.onnx VTT_TEST_SPEAKER_AUDIO=/tmp/diarization-eval \
//!     cargo test -p vtt-core --release --test diarize_e2e -- --ignored --nocapture
//! ```
//!
//! The set is six voices saying four sentences each. They are joined into one conversation, with a
//! known speaker for every line, and the result is scored on how well pairs of lines are grouped.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use vtt_core::audio::load_for_whisper;
use vtt_core::diarize::{DEFAULT_THRESHOLD, SpeakerDiarizer, SpeakerModel};
use vtt_core::engines::{AudioBuffer, DiarizeOptions, Diarizer};
use vtt_core::{Segment, Transcript};

const VOICES: [&str; 6] = ["Daniel", "Karen", "Moira", "Rishi", "Samantha", "Tessa"];
const SENTENCES: usize = 4;
const GAP_SAMPLES: usize = 6_400;
/// Fraction of line pairs that must be grouped correctly.
const MIN_PAIR_ACCURACY: f64 = 0.9;

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} to run this test")))
}

/// The conversation, and the true speaker of every line. Voices take turns in a rotating order so
/// the same two voices never alternate.
fn conversation(dir: &std::path::Path) -> (AudioBuffer, Transcript, Vec<usize>) {
    let mut samples = Vec::new();
    let mut segments = Vec::new();
    let mut truth = Vec::new();
    for sentence in 1..=SENTENCES {
        for turn in 0..VOICES.len() {
            let voice = (turn + sentence) % VOICES.len();
            let clip =
                load_for_whisper(&dir.join(format!("{}-{sentence}.wav", VOICES[voice]))).unwrap();
            let start = samples.len();
            samples.extend_from_slice(&clip.samples);
            let ms = |n: usize| n as u64 * 1000 / 16_000;
            segments.push(Segment {
                start_ms: ms(start),
                end_ms: ms(samples.len()),
                speaker: None,
                text: "line".into(),
            });
            truth.push(voice);
            samples.extend(std::iter::repeat_n(0.0, GAP_SAMPLES));
        }
    }
    let duration_ms = segments.last().map_or(0, |s| s.end_ms);
    let audio = AudioBuffer {
        samples,
        sample_rate: 16_000,
    };
    (
        audio,
        Transcript {
            segments,
            duration_ms,
            ..Transcript::default()
        },
        truth,
    )
}

/// Fraction of pairs of lines that are grouped together exactly when they share a true speaker.
fn pair_accuracy(truth: &[usize], found: &[String]) -> f64 {
    let (mut right, mut total) = (0, 0);
    for i in 0..truth.len() {
        for j in (i + 1)..truth.len() {
            total += 1;
            if (truth[i] == truth[j]) == (found[i] == found[j]) {
                right += 1;
            }
        }
    }
    f64::from(right) / f64::from(total)
}

fn run(model: &std::path::Path, dir: &std::path::Path, options: &DiarizeOptions) -> (f64, usize) {
    let (audio, mut transcript, truth) = conversation(dir);
    let mut diarizer = SpeakerDiarizer::new(SpeakerModel::load(model, "test").unwrap());
    diarizer
        .label_speakers(&audio, &mut transcript, options)
        .unwrap();
    let found: Vec<String> = transcript
        .segments
        .iter()
        .map(|s| s.speaker.clone().unwrap())
        .collect();
    let mut distinct = found.clone();
    distinct.sort();
    distinct.dedup();
    (pair_accuracy(&truth, &found), distinct.len())
}

#[test]
#[ignore = "needs VTT_TEST_SPEAKER_MODEL and VTT_TEST_SPEAKER_AUDIO"]
fn six_voices_are_told_apart() {
    let model = env_path("VTT_TEST_SPEAKER_MODEL");
    let dir = env_path("VTT_TEST_SPEAKER_AUDIO");

    println!("threshold  speakers found  pair accuracy");
    for threshold in [0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9] {
        let options = DiarizeOptions {
            speakers: None,
            threshold: Some(threshold),
        };
        let (accuracy, found) = run(&model, &dir, &options);
        println!("{threshold:>9}  {found:>14}  {accuracy:>13.3}");
    }

    let by_threshold = DiarizeOptions {
        speakers: None,
        threshold: Some(DEFAULT_THRESHOLD),
    };
    let (accuracy, found) = run(&model, &dir, &by_threshold);
    println!("default threshold {DEFAULT_THRESHOLD}: {found} speakers, accuracy {accuracy:.3}");
    assert!(
        accuracy >= MIN_PAIR_ACCURACY,
        "default threshold: {accuracy}"
    );

    let told = DiarizeOptions {
        speakers: Some(VOICES.len()),
        threshold: None,
    };
    let (accuracy, found) = run(&model, &dir, &told);
    println!("6 speakers given: {found} speakers, accuracy {accuracy:.3}");
    assert_eq!(found, VOICES.len());
    assert!(
        accuracy >= MIN_PAIR_ACCURACY,
        "given speaker count: {accuracy}"
    );
}
