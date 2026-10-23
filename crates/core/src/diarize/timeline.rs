//! Who talks when, for the whole recording: every window votes, per 10 ms, for the speakers it heard,
//! and a transcript line takes the speaker with the most votes over its span.

use std::ops::Range;

/// Samples per timeline bin (10 ms at 16 kHz).
const BIN_SAMPLES: usize = 160;
/// A span needs this many votes (50 ms of one voice) to count: a frame at the edge of a neighbouring
/// line spills a few milliseconds over, which is no evidence.
const MIN_VOTES: f32 = 5.0;

pub struct Timeline {
    speakers: usize,
    /// `speakers` votes per bin, bin after bin.
    votes: Vec<f32>,
    sample_rate: usize,
}

impl Timeline {
    /// An empty timeline for `samples` samples of audio and `speakers` speakers.
    pub fn new(samples: usize, speakers: usize, sample_rate: u32) -> Timeline {
        let bins = samples.div_ceil(BIN_SAMPLES) + 1;
        Timeline {
            speakers,
            votes: vec![0.0; bins * speakers],
            sample_rate: sample_rate as usize,
        }
    }

    fn bins(&self) -> usize {
        self.votes.len() / self.speakers.max(1)
    }

    /// Adds a vote for `speaker` over the samples in `span`.
    pub fn vote(&mut self, span: &Range<usize>, speaker: usize) {
        let to = span.end.div_ceil(BIN_SAMPLES).min(self.bins());
        for bin in (span.start / BIN_SAMPLES).min(to)..to {
            self.votes[bin * self.speakers + speaker] += 1.0;
        }
    }

    fn bin_at(&self, ms: u64) -> usize {
        let samples = usize::try_from(ms)
            .unwrap_or(usize::MAX)
            .saturating_mul(self.sample_rate)
            / 1000;
        (samples / BIN_SAMPLES).min(self.bins().saturating_sub(1))
    }

    fn votes_in(&self, bin: usize) -> &[f32] {
        &self.votes[bin * self.speakers..(bin + 1) * self.speakers]
    }

    /// The speaker with the most votes between `start_ms` and `end_ms`; for a span nobody voted in,
    /// the speaker of the nearest bin that has votes. `None` when the timeline has no votes at all.
    pub fn speaker_of(&self, start_ms: u64, end_ms: u64) -> Option<usize> {
        if self.speakers == 0 {
            return None;
        }
        let first = self.bin_at(start_ms);
        let last = self.bin_at(end_ms.saturating_sub(1)).max(first);
        let mut totals = vec![0.0; self.speakers];
        for bin in first..=last {
            for (total, vote) in totals.iter_mut().zip(self.votes_in(bin)) {
                *total += vote;
            }
        }
        strongest(&totals)
            .filter(|speaker| totals[*speaker] >= MIN_VOTES)
            .or_else(|| self.nearest_voted(first, last))
    }

    /// The speaker of the voted bin closest to the middle of bins `first..=last`; the earlier one on
    /// a tie.
    fn nearest_voted(&self, first: usize, last: usize) -> Option<usize> {
        let middle = first.midpoint(last);
        (0..self.bins())
            .filter(|b| self.votes_in(*b).iter().any(|v| *v > 0.0))
            .min_by_key(|b| b.abs_diff(middle))
            .and_then(|b| strongest(self.votes_in(b)))
    }
}

/// Index of the largest positive value; the first wins a tie.
fn strongest(values: &[f32]) -> Option<usize> {
    values
        .iter()
        .enumerate()
        .filter(|(_, v)| **v > 0.0)
        .fold(None, |best: Option<(usize, f32)>, (i, v)| match best {
            Some((_, b)) if b >= *v => best,
            _ => Some((i, *v)),
        })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One frame of window 0 starts at 360 samples, so frame `n` is around `n * 270 + 360`.
    fn timeline_with(votes: &[(usize, usize)]) -> Timeline {
        let mut timeline = Timeline::new(160_000, 2, 16_000);
        for (frame, speaker) in votes {
            timeline.vote(&(frame * 270 + 360..frame * 270 + 630), *speaker);
        }
        timeline
    }

    #[test]
    fn a_span_takes_the_speaker_with_the_most_votes_in_it() {
        // Frames 40..80 are about 0.7 s to 1.4 s; frame 100 is about 1.7 s.
        let mut votes: Vec<(usize, usize)> = (40..80).map(|f| (f, 0)).collect();
        votes.extend((100..140).map(|f| (f, 1)));
        let timeline = timeline_with(&votes);
        assert_eq!(timeline.speaker_of(800, 1300), Some(0));
        assert_eq!(timeline.speaker_of(1800, 2400), Some(1));
        // A span with both: speaker 1 has more frames inside it.
        assert_eq!(timeline.speaker_of(1200, 2400), Some(1));
    }

    #[test]
    fn a_span_without_votes_takes_the_nearest_speaker() {
        let timeline = timeline_with(&[(40, 0), (41, 0), (200, 1), (201, 1)]);
        assert_eq!(timeline.speaker_of(300, 400), Some(0));
        assert_eq!(timeline.speaker_of(3500, 3600), Some(1));
    }

    #[test]
    fn a_timeline_nobody_voted_in_has_no_speaker() {
        assert_eq!(timeline_with(&[]).speaker_of(0, 1000), None);
        assert_eq!(Timeline::new(1000, 0, 16_000).speaker_of(0, 10), None);
    }

    #[test]
    fn votes_past_the_end_of_the_audio_are_ignored() {
        let mut timeline = Timeline::new(1000, 1, 16_000);
        timeline.vote(&(5_000_000..5_000_270), 0);
        assert_eq!(timeline.speaker_of(0, 50), None);
    }
}
