//! Finding the speech in a recording, so silence is never sent to the model. Whisper tends to
//! invent text for silence and can loop on long stretches of audio; transcribing short chunks of
//! speech avoids both. This is an energy detector, not a neural one: it needs no model file.

use std::ops::Range;

/// Length of one analysis frame.
const FRAME_MS: usize = 30;
/// A frame is speech when it is this much quieter than the loud parts of the recording or louder.
const RELATIVE_THRESHOLD: f32 = 0.1;
/// Below this level (RMS of samples in -1..1) a recording is treated as silence.
const MIN_RMS: f32 = 0.003;
/// Pauses shorter than this stay inside one stretch of speech.
const MERGE_GAP_MS: usize = 600;
/// Shorter bursts are clicks and breaths, not speech.
const MIN_SPEECH_MS: usize = 300;
/// Context kept before and after each stretch so word edges are not clipped.
const PAD_MS: usize = 250;
/// Whisper reads 30 s windows; chunks stay a little under that.
const MAX_CHUNK_MS: usize = 28_000;
/// Speech separated by a longer silence goes into different chunks.
const CHUNK_GAP_MS: usize = 2_000;
/// Whisper behaves badly on very short input, so chunks are at least this long.
const MIN_CHUNK_MS: usize = 1_000;
/// The loudness reference is this percentile of the sorted frame levels, so one loud click does
/// not raise the threshold above the speech.
const REFERENCE_PERCENTILE: usize = 95;

fn samples_per(ms: usize, rate: u32) -> usize {
    ms * rate as usize / 1000
}

fn frame_levels(samples: &[f32], frame: usize) -> Vec<f32> {
    samples
        .chunks(frame)
        .map(|chunk| {
            let (sum, count) = chunk.iter().fold((0.0_f32, 0.0_f32), |(sum, count), s| {
                (sum + s * s, count + 1.0)
            });
            (sum / count).sqrt()
        })
        .collect()
}

fn threshold(levels: &[f32]) -> f32 {
    let mut sorted = levels.to_vec();
    sorted.sort_by(f32::total_cmp);
    let index = (sorted.len() * REFERENCE_PERCENTILE / 100).min(sorted.len() - 1);
    (sorted[index] * RELATIVE_THRESHOLD).max(MIN_RMS)
}

/// Runs of loud frames as frame ranges, with short pauses bridged and short bursts dropped.
fn speech_frames(levels: &[f32], gap: usize, min_len: usize) -> Vec<Range<usize>> {
    if levels.is_empty() {
        return Vec::new();
    }
    let limit = threshold(levels);
    let mut runs: Vec<Range<usize>> = Vec::new();
    for (index, level) in levels.iter().enumerate() {
        if *level < limit {
            continue;
        }
        match runs.last_mut() {
            Some(last) if index - last.end <= gap => last.end = index + 1,
            _ => runs.push(index..index + 1),
        }
    }
    runs.retain(|run| run.len() >= min_len);
    runs
}

/// Splits `range` into pieces of at most `max` samples.
fn split_long(range: Range<usize>, max: usize) -> impl Iterator<Item = Range<usize>> {
    let pieces = (range.len()).div_ceil(max).max(1);
    (0..pieces).map(move |n| {
        let start = range.start + n * max;
        start..(start + max).min(range.end)
    })
}

/// Groups padded speech regions into chunks no longer than the model's window, starting a new
/// chunk after a long silence.
fn group(regions: Vec<Range<usize>>, max: usize, gap: usize) -> Vec<Range<usize>> {
    let mut chunks: Vec<Range<usize>> = Vec::new();
    for region in regions.into_iter().flat_map(|r| split_long(r, max)) {
        match chunks.last_mut() {
            Some(last) if region.start - last.end < gap && region.end - last.start <= max => {
                last.end = region.end;
            }
            _ => chunks.push(region),
        }
    }
    chunks
}

/// Sample ranges of `samples` that contain speech, each short enough for one model call. Empty
/// when the recording is silent.
pub fn speech_chunks(samples: &[f32], rate: u32) -> Vec<Range<usize>> {
    let frame = samples_per(FRAME_MS, rate).max(1);
    let levels = frame_levels(samples, frame);
    let frames = |ms: usize| ms.div_ceil(FRAME_MS);
    let pad = samples_per(PAD_MS, rate);
    let regions = speech_frames(&levels, frames(MERGE_GAP_MS), frames(MIN_SPEECH_MS))
        .into_iter()
        .map(|run| {
            let start = (run.start * frame).saturating_sub(pad);
            start..(run.end * frame + pad).min(samples.len())
        })
        .collect();
    let max = samples_per(MAX_CHUNK_MS, rate);
    let mut chunks = group(regions, max, samples_per(CHUNK_GAP_MS, rate));
    let min = samples_per(MIN_CHUNK_MS, rate).min(samples.len());
    for chunk in &mut chunks {
        chunk.end = chunk.end.max((chunk.start + min).min(samples.len()));
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::synthetic::{RATE, recording, silence, voice};

    fn chunks_of(parts: &[Vec<f32>]) -> Vec<Range<usize>> {
        speech_chunks(&recording(parts).samples, RATE)
    }

    /// A range of samples as `(start, end)` in milliseconds.
    fn millis(range: &Range<usize>) -> (usize, usize) {
        let ms = |samples: usize| samples * 1000 / RATE as usize;
        (ms(range.start), ms(range.end))
    }

    #[test]
    fn a_silent_recording_has_no_speech() {
        assert_eq!(chunks_of(&[silence(5000)]).len(), 0);
        assert_eq!(speech_chunks(&[], RATE).len(), 0);
    }

    #[test]
    fn speech_after_a_long_silence_becomes_its_own_chunk_with_padding() {
        let chunks = chunks_of(&[
            silence(1000),
            voice(2000),
            silence(4000),
            voice(1000),
            silence(1000),
        ]);

        assert_eq!(chunks.len(), 2);
        let (start, end) = millis(&chunks[0]);
        assert!((700..=800).contains(&start), "{start}");
        assert!((3200..=3300).contains(&end), "{end}");
        let (start, _) = millis(&chunks[1]);
        assert!((6700..=6800).contains(&start), "{start}");
    }

    #[test]
    fn a_short_pause_does_not_split_speech() {
        assert_eq!(
            chunks_of(&[voice(2000), silence(400), voice(2000)]).len(),
            1
        );
    }

    #[test]
    fn clicks_shorter_than_speech_are_ignored() {
        assert_eq!(
            chunks_of(&[silence(2000), voice(100), silence(2000)]).len(),
            0
        );
    }

    #[test]
    fn long_speech_is_cut_into_chunks_that_fit_the_model_and_cover_everything() {
        let audio = recording(&[voice(70_000)]);

        let chunks = speech_chunks(&audio.samples, RATE);

        assert!(chunks.len() >= 3);
        assert!(
            chunks
                .iter()
                .all(|c| c.len() <= samples_per(MAX_CHUNK_MS, RATE))
        );
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks.last().unwrap().end, audio.samples.len());
        assert!(chunks.windows(2).all(|w| w[0].end == w[1].start));
    }

    #[test]
    fn very_short_speech_is_extended_to_the_minimum_length() {
        let chunks = chunks_of(&[silence(3000), voice(350), silence(3000)]);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].len() >= samples_per(MIN_CHUNK_MS, RATE));
    }
}
