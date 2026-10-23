//! Tests of the speaker labelling with stand-in models.

use super::segmenter::{FIELD_SAMPLES, FRAME_SAMPLES, Frame, WINDOW_SAMPLES};
use super::*;
use crate::Segment;
use crate::engines::Control;

/// Hears one speaker wherever the audio is not silent.
struct ActivitySegmenter;

impl Segmenter for ActivitySegmenter {
    fn activity(&mut self, window: &[f32]) -> Result<Vec<Frame>> {
        let frames = (WINDOW_SAMPLES - FIELD_SAMPLES) / FRAME_SAMPLES + 1;
        Ok((0..frames)
            .map(|f| {
                let middle = f * FRAME_SAMPLES + FIELD_SAMPLES / 2;
                [
                    window.get(middle).is_some_and(|s| s.abs() > 0.01),
                    false,
                    false,
                ]
            })
            .collect())
    }
}

/// Finds nobody, as when the segmentation model cannot make out the voices.
struct DeafSegmenter;

impl Segmenter for DeafSegmenter {
    fn activity(&mut self, _window: &[f32]) -> Result<Vec<Frame>> {
        Ok(vec![[false; 3]; 589])
    }
}

/// "Hears" the level of the audio: quiet and loud stretches are two different voices.
struct LevelEmbedder;

impl SpeakerEmbedder for LevelEmbedder {
    fn embed(&mut self, samples: &[f32]) -> Result<Vec<f32>> {
        let (sum, count) = samples
            .iter()
            .fold((0.0, 0.0), |(sum, count), s| (sum + s, count + 1.0));
        let level = sum / count;
        Ok(vec![level, 1.0 - level])
    }
}

const QUIET: f32 = 0.1;
const LOUD: f32 = 0.9;

fn line(start_ms: u64, end_ms: u64) -> Segment {
    Segment {
        start_ms,
        end_ms,
        speaker: None,
        text: "words".into(),
    }
}

/// Audio where each `(seconds, level)` stretch is one steady level, and a transcript with a
/// line per stretch.
fn conversation(parts: &[(u64, f32)]) -> (AudioBuffer, Transcript) {
    let mut samples = Vec::new();
    let mut segments = Vec::new();
    let mut at = 0;
    for (millis, level) in parts {
        samples.extend(std::iter::repeat_n(
            *level,
            usize::try_from(*millis * 16).unwrap(),
        ));
        segments.push(line(at, at + millis));
        at += millis;
    }
    let audio = AudioBuffer {
        samples,
        sample_rate: 16_000,
    };
    (
        audio,
        Transcript {
            segments,
            duration_ms: at,
            ..Transcript::default()
        },
    )
}

fn speakers(transcript: &Transcript) -> Vec<Option<&str>> {
    transcript
        .segments
        .iter()
        .map(|s| s.speaker.as_deref())
        .collect()
}

fn run(parts: &[(u64, f32)], options: &DiarizeOptions) -> Transcript {
    let (audio, mut transcript) = conversation(parts);
    SpeakerDiarizer::new(LevelEmbedder, ActivitySegmenter)
        .label_speakers(&audio, &mut transcript, options, &mut |_| Control::Continue)
        .unwrap();
    transcript
}

#[test]
fn voices_are_told_apart_and_numbered_by_first_appearance() {
    let transcript = run(
        &[(2500, LOUD), (2500, QUIET), (2500, LOUD)],
        &DiarizeOptions::default(),
    );
    assert_eq!(
        speakers(&transcript),
        [Some("SPEAKER_00"), Some("SPEAKER_01"), Some("SPEAKER_00")]
    );
}

#[test]
fn a_given_speaker_count_overrides_the_distance_threshold() {
    let one = DiarizeOptions {
        speakers: Some(1),
        threshold: None,
    };
    let transcript = run(&[(2000, LOUD), (2000, QUIET)], &one);
    assert_eq!(
        speakers(&transcript),
        [Some("SPEAKER_00"), Some("SPEAKER_00")]
    );
}

#[test]
fn a_threshold_beyond_every_distance_merges_everyone() {
    let lax = DiarizeOptions {
        speakers: None,
        threshold: Some(1.5),
    };
    let transcript = run(&[(2000, LOUD), (2000, QUIET)], &lax);
    assert_eq!(
        speakers(&transcript),
        [Some("SPEAKER_00"), Some("SPEAKER_00")]
    );
}

#[test]
fn a_line_too_short_to_embed_joins_its_neighbour_in_time() {
    let transcript = run(
        &[(3000, LOUD), (400, LOUD), (3000, QUIET)],
        &DiarizeOptions::default(),
    );
    assert_eq!(speakers(&transcript)[1], Some("SPEAKER_00"));
}

#[test]
fn a_line_that_is_mostly_silence_is_not_embedded_on_its_own() {
    // Heard whole, the middle line would look like the quiet voice; its speech is only 0.5 s.
    let mut audio = conversation(&[(3000, LOUD), (1500, QUIET), (3000, QUIET)]).0;
    let start = 3000 * 16;
    audio.samples[start + 500 * 16..start + 1500 * 16].fill(0.0);
    audio.samples[start..start + 500 * 16].fill(LOUD);
    let (_, mut transcript) = conversation(&[(3000, LOUD), (1500, QUIET), (3000, QUIET)]);
    SpeakerDiarizer::new(LevelEmbedder, ActivitySegmenter)
        .label_speakers(
            &audio,
            &mut transcript,
            &DiarizeOptions::default(),
            &mut |_| Control::Continue,
        )
        .unwrap();
    assert_eq!(speakers(&transcript)[1], Some("SPEAKER_00"));
}

#[test]
fn audio_with_nothing_long_enough_leaves_the_transcript_unlabelled() {
    let transcript = run(&[(400, LOUD), (300, QUIET)], &DiarizeOptions::default());
    assert_eq!(speakers(&transcript), [None, None]);
}

#[test]
fn lines_the_segmentation_missed_are_told_apart_from_their_own_audio() {
    let (audio, mut transcript) = conversation(&[(2500, LOUD), (2500, QUIET), (2500, LOUD)]);
    SpeakerDiarizer::new(LevelEmbedder, DeafSegmenter)
        .label_speakers(
            &audio,
            &mut transcript,
            &DiarizeOptions::default(),
            &mut |_| Control::Continue,
        )
        .unwrap();
    assert_eq!(
        speakers(&transcript),
        [Some("SPEAKER_00"), Some("SPEAKER_01"), Some("SPEAKER_00")]
    );
}

#[test]
fn audio_at_another_sample_rate_is_rejected() {
    let (mut audio, mut transcript) = conversation(&[(2000, LOUD)]);
    audio.sample_rate = 44_100;
    let result = SpeakerDiarizer::new(LevelEmbedder, ActivitySegmenter).label_speakers(
        &audio,
        &mut transcript,
        &DiarizeOptions::default(),
        &mut |_| Control::Continue,
    );
    assert!(matches!(result, Err(Error::Engine(_))));
}

#[test]
fn progress_runs_from_zero_towards_one_in_order_and_ends_at_one() {
    let (audio, mut transcript) = conversation(&[(4000, LOUD), (4000, QUIET), (4000, LOUD)]);
    let mut seen = Vec::new();
    SpeakerDiarizer::new(LevelEmbedder, ActivitySegmenter)
        .label_speakers(
            &audio,
            &mut transcript,
            &DiarizeOptions::default(),
            &mut |p| {
                seen.push(p);
                Control::Continue
            },
        )
        .unwrap();
    assert!(seen.len() >= 2, "{seen:?}");
    assert!(seen.windows(2).all(|pair| pair[0] <= pair[1]), "{seen:?}");
    assert!(seen.iter().all(|p| (0.0..=1.0).contains(p)), "{seen:?}");
    assert!(
        (seen.last().copied().unwrap() - 1.0).abs() < f32::EPSILON,
        "{seen:?}"
    );
}

#[test]
fn a_cancel_during_labelling_stops_it() {
    let (audio, mut transcript) = conversation(&[(4000, LOUD), (4000, QUIET)]);
    let result = SpeakerDiarizer::new(LevelEmbedder, ActivitySegmenter).label_speakers(
        &audio,
        &mut transcript,
        &DiarizeOptions::default(),
        &mut |_| Control::Cancel,
    );
    assert!(matches!(result, Err(Error::Cancelled)));
    assert!(transcript.segments.iter().all(|s| s.speaker.is_none()));
}
