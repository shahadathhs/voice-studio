//! The stretches of audio that get a voiceprint: what a local speaker says in a window, and any line
//! of the transcript the segmentation did not cover well, taken as it is.

use std::ops::Range;

use super::segmenter::FRAME_SAMPLES;
use super::windows::{LocalTurn, Window, frame_start};
use crate::Segment;
use crate::audio::voiced_span;

/// Samples per bin of the coverage map (10 ms at 16 kHz).
const BIN_SAMPLES: usize = 160;
/// A line is taken as it is when the segmentation covers less than this share of it.
const MIN_COVERAGE_PERCENT: usize = 60;
/// A line with less speech than this (1 s) gives an unreliable voiceprint and is skipped.
const MIN_LINE_SAMPLES: usize = 16_000;

/// One voiceprint's worth of audio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    /// Where to listen to make the voiceprint.
    pub listen: Vec<Range<usize>>,
    /// The span of the recording the voiceprint speaks for.
    pub voted: Vec<Range<usize>>,
}

impl Unit {
    /// Samples of speech the voiceprint is made from.
    pub fn listening(&self) -> usize {
        self.listen.iter().map(ExactSizeIterator::len).sum()
    }
}

/// Joins ascending frames of a window into sample ranges, one per run of adjacent frames.
fn frame_ranges(window_start: usize, frames: &[usize]) -> Vec<Range<usize>> {
    let mut out: Vec<(usize, Range<usize>)> = Vec::new();
    for frame in frames {
        let start = frame_start(window_start, *frame);
        let end = start + FRAME_SAMPLES;
        match out.last_mut() {
            Some((last, range)) if *frame == *last + 1 => {
                range.end = end;
                *last = *frame;
            }
            _ => out.push((*frame, start..end)),
        }
    }
    out.into_iter().map(|(_, range)| range).collect()
}

/// The units for the local speakers found by the segmentation.
pub fn from_windows(windows: &[Window], turns: &[LocalTurn]) -> Vec<Unit> {
    turns
        .iter()
        .map(|turn| {
            let start = windows[turn.window].start;
            Unit {
                listen: frame_ranges(start, &turn.listen),
                voted: frame_ranges(start, &turn.active),
            }
        })
        .collect()
}

/// A map of which 10 ms of the recording some unit speaks for.
fn coverage(units: &[Unit], length: usize) -> Vec<bool> {
    let mut covered = vec![false; length.div_ceil(BIN_SAMPLES) + 1];
    for range in units.iter().flat_map(|u| &u.voted) {
        let end = range.end.div_ceil(BIN_SAMPLES).min(covered.len());
        covered[(range.start / BIN_SAMPLES).min(end)..end].fill(true);
    }
    covered
}

/// Whether at least `MIN_COVERAGE_PERCENT` of the line's bins are covered. A line with no bins
/// counts as covered, so there is nothing to add for it.
fn is_covered(covered: &[bool], line: &Range<usize>) -> bool {
    let end = (line.end / BIN_SAMPLES).min(covered.len());
    let bins = &covered[(line.start / BIN_SAMPLES).min(end)..end];
    bins.iter().filter(|c| **c).count() * 100 >= bins.len() * MIN_COVERAGE_PERCENT
}

/// Units for the lines of the transcript that `covering` leaves mostly unheard: the speech of the
/// line, whole.
pub fn from_uncovered_lines(
    samples: &[f32],
    sample_rate: u32,
    lines: &[Segment],
    covering: &[Unit],
) -> Vec<Unit> {
    let covered = coverage(covering, samples.len());
    let at = |ms: u64| usize::try_from(ms * u64::from(sample_rate) / 1000).unwrap_or(usize::MAX);
    lines
        .iter()
        .filter_map(|line| {
            let end = at(line.end_ms).min(samples.len());
            let whole = at(line.start_ms).min(end)..end;
            if is_covered(&covered, &whole) {
                return None;
            }
            let speech = voiced_span(&samples[whole.clone()], sample_rate)?;
            let span = whole.start + speech.start..whole.start + speech.end;
            (span.len() >= MIN_LINE_SAMPLES).then(|| Unit {
                listen: vec![span.clone()],
                voted: vec![span],
            })
        })
        .collect()
}

/// The audio of `ranges`, joined.
pub fn audio_of(samples: &[f32], ranges: &[Range<usize>]) -> Vec<f32> {
    let mut out = Vec::with_capacity(ranges.iter().map(ExactSizeIterator::len).sum());
    for range in ranges {
        let end = range.end.min(samples.len());
        out.extend_from_slice(&samples[range.start.min(end)..end]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(start_ms: u64, end_ms: u64) -> Segment {
        Segment {
            start_ms,
            end_ms,
            speaker: None,
            text: "words".into(),
        }
    }

    fn unit(span: Range<usize>) -> Unit {
        Unit {
            listen: vec![span.clone()],
            voted: vec![span],
        }
    }

    /// 16 kHz audio of steady tone, so that every stretch counts as speech.
    fn tone(seconds: usize) -> Vec<f32> {
        (0..seconds * 16_000)
            .map(|n| 0.3 * (f32::from(u8::try_from(n % 157).unwrap()) * 0.2).sin())
            .collect()
    }

    #[test]
    fn adjacent_frames_become_one_range() {
        let ranges = frame_ranges(0, &[0, 1, 2, 5]);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].len(), 3 * FRAME_SAMPLES);
        assert_eq!(ranges[1].len(), FRAME_SAMPLES);
    }

    #[test]
    fn a_line_the_segmentation_covered_is_left_alone() {
        let samples = tone(5);
        let covering = [unit(0..40_000)];
        assert_eq!(
            from_uncovered_lines(&samples, 16_000, &[line(0, 2500)], &covering).len(),
            0
        );
    }

    #[test]
    fn a_line_nobody_covered_becomes_a_unit_of_its_own() {
        let samples = tone(5);
        let units = from_uncovered_lines(&samples, 16_000, &[line(1000, 4000)], &[]);
        assert_eq!(units.len(), 1);
        assert!(units[0].listening() >= 40_000);
        assert!(units[0].voted[0].start >= 16_000);
    }

    #[test]
    fn a_line_with_under_a_second_of_speech_is_skipped() {
        let samples = tone(5);
        assert_eq!(
            from_uncovered_lines(&samples, 16_000, &[line(0, 600)], &[]).len(),
            0
        );
    }

    #[test]
    fn audio_is_cut_from_its_place_and_clipped_to_the_recording() {
        let samples: Vec<f32> = (0..10_u8).map(f32::from).collect();
        assert_eq!(audio_of(&samples, &[2..4, 8..20]), [2.0, 3.0, 8.0, 9.0]);
        assert_eq!(audio_of(&samples, std::slice::from_ref(&(30..40))).len(), 0);
    }
}
