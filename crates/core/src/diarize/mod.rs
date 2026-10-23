//! Speaker identification: who says each line of a transcript. A segmentation model finds, in
//! overlapping 10 s windows, when each of up to three local speakers talks (overlaps included).
//! What each of them says in a window is embedded as one voiceprint, the voiceprints of the whole
//! recording are clustered into speakers, and the clusters vote back onto the timeline. A line takes
//! the speaker who talks most during it.

mod cluster;
mod embedder;
mod fbank;
mod fft;
mod group;
mod load;
mod segmenter;
mod timeline;
mod units;
mod windows;

pub use embedder::SpeakerModel;
pub use load::{SpeakerLabeller, load};
pub use segmenter::{SegmentationModel, Segmenter};

use crate::audio::WHISPER_SAMPLE_RATE;
use crate::engines::{AudioBuffer, DiarizeOptions, Diarizer};
use crate::{Error, Result, Transcript};
use cluster::Stop;
use group::group;
use timeline::Timeline;
use units::Unit;

/// Samples between the starts of two segmentation windows (2.5 s: each moment is seen four times).
const WINDOW_SHIFT: usize = 80_000;

/// Turns a stretch of 16 kHz mono audio into a vector that is similar for the same voice.
pub trait SpeakerEmbedder: Send {
    fn embed(&mut self, samples: &[f32]) -> Result<Vec<f32>>;
}

/// Labels the lines of a transcript with `SPEAKER_00`, `SPEAKER_01`, ... in order of appearance.
pub struct SpeakerDiarizer<E: SpeakerEmbedder, S: Segmenter> {
    embedder: E,
    segmenter: S,
}

impl<E: SpeakerEmbedder, S: Segmenter> SpeakerDiarizer<E, S> {
    pub fn new(embedder: E, segmenter: S) -> SpeakerDiarizer<E, S> {
        SpeakerDiarizer {
            embedder,
            segmenter,
        }
    }

    fn voiceprints(&mut self, samples: &[f32], units: &[Unit]) -> Result<Vec<Vec<f32>>> {
        units
            .iter()
            .map(|unit| self.embedder.embed(&units::audio_of(samples, &unit.listen)))
            .collect()
    }
}

impl<E: SpeakerEmbedder, S: Segmenter> Diarizer for SpeakerDiarizer<E, S> {
    fn label_speakers(
        &mut self,
        audio: &AudioBuffer,
        transcript: &mut Transcript,
        options: &DiarizeOptions,
    ) -> Result<()> {
        if audio.sample_rate != WHISPER_SAMPLE_RATE {
            return Err(Error::Engine(format!(
                "speaker identification needs {WHISPER_SAMPLE_RATE} Hz audio, got {} Hz",
                audio.sample_rate
            )));
        }
        let windows = windows::analyse(&audio.samples, WINDOW_SHIFT, &mut self.segmenter)?;
        let edges = line_edges(transcript, audio.sample_rate);
        let mut found = units::from_windows(&windows, &windows::local_turns(&windows, &edges));
        let whole = units::from_uncovered_lines(
            &audio.samples,
            audio.sample_rate,
            &transcript.segments,
            &found,
        );
        found.extend(whole);
        if found.is_empty() {
            return Ok(());
        }
        let voiceprints = self.voiceprints(&audio.samples, &found)?;
        let stop = match options.speakers {
            Some(count) if count >= 1 => Stop::Clusters(count),
            _ => options.threshold.map_or(Stop::Auto, Stop::Distance),
        };
        let clusters = group(&voiceprints, &found, stop);
        label_lines(transcript, &timeline_of(audio, &found, &clusters));
        Ok(())
    }
}

/// Gives every line the speaker who talks most during it, numbered by first appearance.
fn label_lines(transcript: &mut Transcript, timeline: &Timeline) {
    let per_line: Vec<Option<usize>> = transcript
        .segments
        .iter()
        .map(|s| timeline.speaker_of(s.start_ms, s.end_ms))
        .collect();
    let known: Vec<usize> = per_line.iter().flatten().copied().collect();
    let mut numbers = cluster::renumber(&known).into_iter();
    for (segment, label) in transcript.segments.iter_mut().zip(&per_line) {
        segment.speaker = label
            .and_then(|_| numbers.next())
            .map(|n| format!("SPEAKER_{n:02}"));
    }
}

/// Where the lines of the transcript start and end, as sample positions in ascending order.
fn line_edges(transcript: &Transcript, sample_rate: u32) -> Vec<usize> {
    let at = |ms: u64| usize::try_from(ms * u64::from(sample_rate) / 1000).unwrap_or(usize::MAX);
    let mut edges: Vec<usize> = transcript
        .segments
        .iter()
        .flat_map(|s| [at(s.start_ms), at(s.end_ms)])
        .collect();
    edges.sort_unstable();
    edges.dedup();
    edges
}

fn timeline_of(audio: &AudioBuffer, units: &[Unit], clusters: &[usize]) -> Timeline {
    let speakers = clusters.iter().max().map_or(0, |m| m + 1);
    let mut timeline = Timeline::new(audio.samples.len(), speakers, audio.sample_rate);
    for (unit, cluster) in units.iter().zip(clusters) {
        for span in &unit.voted {
            timeline.vote(span, *cluster);
        }
    }
    timeline
}

#[cfg(test)]
mod tests;
