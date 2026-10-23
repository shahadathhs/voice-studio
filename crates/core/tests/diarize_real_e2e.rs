//! Evaluates speaker identification on real people: every audio file in a directory is one speaker,
//! cut into pieces of about three seconds that are interleaved into one conversation with a known
//! speaker for every line. Ignored by default because it needs files that are not in the
//! repository (see docs/product/diarization.md for the set used):
//!
//! ```text
//! VTT_TEST_SPEAKER_MODEL=/path/to/voxceleb_campplus_lm.onnx \
//!     VTT_TEST_SEGMENTATION_MODEL=/path/to/segmentation-3.0.onnx VTT_TEST_REAL_AUDIO=/path/to/dir \
//!     cargo test -p vtt-core --release --test diarize_real_e2e -- --ignored --nocapture
//! ```
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use vtt_core::audio::load_for_whisper;
use vtt_core::diarize::{SegmentationModel, SpeakerDiarizer, SpeakerModel};
use vtt_core::engines::{AudioBuffer, DiarizeOptions, Diarizer};
use vtt_core::{Segment, Transcript};

const PIECE_SAMPLES: usize = 3 * 16_000;
const GAP_SAMPLES: usize = 4_800;
/// Fraction of line pairs that must be grouped correctly.
const MIN_PAIR_ACCURACY: f64 = 0.9;

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} to run this test")))
}

/// Pieces of one speaker; a last piece under half a piece joins the one before it.
fn pieces(samples: &[f32]) -> Vec<Vec<f32>> {
    let mut out: Vec<Vec<f32>> = samples.chunks(PIECE_SAMPLES).map(<[f32]>::to_vec).collect();
    if out.len() > 1 && out.last().unwrap().len() < PIECE_SAMPLES / 2 {
        let tail = out.pop().unwrap();
        out.last_mut().unwrap().extend(tail);
    }
    out
}

/// Speakers take turns in order, each with their next piece, until every piece is used.
fn conversation(dir: &std::path::Path) -> (AudioBuffer, Transcript, Vec<usize>) {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    let mut queues: Vec<std::collections::VecDeque<Vec<f32>>> = files
        .iter()
        .map(|f| pieces(&load_for_whisper(f).unwrap().samples).into())
        .collect();
    let (mut samples, mut segments, mut truth) = (Vec::new(), Vec::new(), Vec::new());
    while queues.iter().any(|q| !q.is_empty()) {
        for (speaker, queue) in queues.iter_mut().enumerate() {
            let Some(piece) = queue.pop_front() else {
                continue;
            };
            let ms = |n: usize| n as u64 * 1000 / 16_000;
            segments.push(Segment {
                start_ms: ms(samples.len()),
                end_ms: ms(samples.len() + piece.len()),
                speaker: None,
                text: "line".into(),
            });
            samples.extend(piece);
            samples.extend(std::iter::repeat_n(0.0, GAP_SAMPLES));
            truth.push(speaker);
        }
    }
    let duration_ms = segments.last().map_or(0, |s| s.end_ms);
    let audio = AudioBuffer {
        samples,
        sample_rate: 16_000,
    };
    let transcript = Transcript {
        segments,
        duration_ms,
        ..Transcript::default()
    };
    (audio, transcript, truth)
}

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

fn run(dir: &std::path::Path, options: &DiarizeOptions) -> (f64, usize) {
    let (audio, mut transcript, truth) = conversation(dir);
    let mut diarizer = SpeakerDiarizer::new(
        SpeakerModel::load(&env_path("VTT_TEST_SPEAKER_MODEL"), "test").unwrap(),
        SegmentationModel::load(&env_path("VTT_TEST_SEGMENTATION_MODEL"), "test").unwrap(),
    );
    diarizer
        .label_speakers(&audio, &mut transcript, options)
        .unwrap();
    let found: Vec<String> = transcript
        .segments
        .iter()
        .map(|s| s.speaker.clone().unwrap_or_default())
        .collect();
    let mut distinct = found.clone();
    distinct.sort();
    distinct.dedup();
    if std::env::var_os("VTT_TEST_VERBOSE").is_some() {
        let t: String = truth
            .iter()
            .map(|v| char::from_digit(u32::try_from(*v).unwrap(), 10).unwrap())
            .collect();
        let f: String = found
            .iter()
            .map(|v| v.chars().last().unwrap_or('-'))
            .collect();
        println!("truth {t}\nfound {f}");
    }
    (pair_accuracy(&truth, &found), distinct.len())
}

#[test]
#[ignore = "needs VTT_TEST_SPEAKER_MODEL, VTT_TEST_SEGMENTATION_MODEL and VTT_TEST_REAL_AUDIO"]
fn real_people_are_told_apart() {
    let dir = env_path("VTT_TEST_REAL_AUDIO");
    let speakers = std::fs::read_dir(&dir).unwrap().count();
    println!("{speakers} real speakers\nthreshold  speakers found  pair accuracy");
    for threshold in [0.2, 0.3, 0.4, 0.5, 0.6, 0.7] {
        let options = DiarizeOptions {
            speakers: None,
            threshold: Some(threshold),
        };
        let (accuracy, found) = run(&dir, &options);
        println!("{threshold:>9}  {found:>14}  {accuracy:>13.3}");
    }
    let (accuracy, found) = run(&dir, &DiarizeOptions::default());
    println!("automatic: {found} speakers, accuracy {accuracy:.3}");
    assert!(accuracy >= MIN_PAIR_ACCURACY, "automatic: {accuracy}");
    let told = DiarizeOptions {
        speakers: Some(speakers),
        threshold: None,
    };
    let (accuracy, found) = run(&dir, &told);
    println!("{speakers} speakers given: {found} speakers, accuracy {accuracy:.3}");
    assert!(accuracy >= MIN_PAIR_ACCURACY, "given count: {accuracy}");
}
