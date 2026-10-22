//! Speaker identification: who says each line of a transcript. Each line (or sliding windows of a
//! long line) is turned into a speaker embedding, the embeddings are clustered, and every line
//! gets the label of its cluster. The method is the one of the earlier Python prototype.

mod cluster;
mod embedder;
mod fbank;
mod fft;
mod labels;

pub use embedder::SpeakerModel;

use crate::audio::{WHISPER_SAMPLE_RATE, voiced_span};
use crate::engines::{AudioBuffer, DiarizeOptions, Diarizer};
use crate::{Error, Result, Transcript};
use cluster::Stop;

/// Cosine distance below which two voices count as the same speaker, when the number of speakers
/// is not given. Measured on the evaluation set described in docs/product/diarization.md.
pub const DEFAULT_THRESHOLD: f32 = 0.45;

/// A chunk is only embedded when at least this much of it is speech; a single short word gives an
/// unreliable voiceprint, so such lines take the label of the nearest line in time instead.
const MIN_VOICED_MS: u64 = 800;

/// Turns a stretch of 16 kHz mono audio into a vector that is similar for the same voice.
pub trait SpeakerEmbedder: Send {
    fn embed(&mut self, samples: &[f32]) -> Result<Vec<f32>>;
}

/// Labels the lines of a transcript with `SPEAKER_00`, `SPEAKER_01`, ... in order of appearance.
pub struct SpeakerDiarizer<E: SpeakerEmbedder> {
    embedder: E,
}

impl<E: SpeakerEmbedder> SpeakerDiarizer<E> {
    pub fn new(embedder: E) -> SpeakerDiarizer<E> {
        SpeakerDiarizer { embedder }
    }
}

fn sample_range(chunk: &labels::Chunk, audio: &AudioBuffer) -> std::ops::Range<usize> {
    let at =
        |ms: u64| usize::try_from(ms * u64::from(audio.sample_rate) / 1000).unwrap_or(usize::MAX);
    let end = at(chunk.end_ms).min(audio.samples.len());
    at(chunk.start_ms).min(end)..end
}

impl<E: SpeakerEmbedder> Diarizer for SpeakerDiarizer<E> {
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
        let min_samples = MIN_VOICED_MS * u64::from(audio.sample_rate) / 1000;
        let mut kept = Vec::new();
        let mut embeddings = Vec::new();
        for chunk in labels::plan_chunks(&transcript.segments) {
            let samples = &audio.samples[sample_range(&chunk, audio)];
            let Some(voiced) = voiced_span(samples, audio.sample_rate) else {
                continue;
            };
            let voiced = &samples[voiced];
            if (voiced.len() as u64) < min_samples {
                continue;
            }
            embeddings.push(self.embedder.embed(voiced)?);
            kept.push(chunk);
        }
        if embeddings.is_empty() {
            return Ok(());
        }
        let stop = match options.speakers {
            Some(count) if count >= 1 => Stop::Clusters(count),
            _ => Stop::Distance(options.threshold.unwrap_or(DEFAULT_THRESHOLD)),
        };
        let clusters = cluster::agglomerate(&embeddings, stop);
        let mut per_line = labels::label_lines(&transcript.segments, &kept, &clusters);
        labels::smooth(&transcript.segments, &mut per_line);
        for (segment, label) in transcript
            .segments
            .iter_mut()
            .zip(cluster::renumber(&per_line))
        {
            segment.speaker = Some(format!("SPEAKER_{label:02}"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Segment;

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
        SpeakerDiarizer::new(LevelEmbedder)
            .label_speakers(&audio, &mut transcript, options)
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
        SpeakerDiarizer::new(LevelEmbedder)
            .label_speakers(&audio, &mut transcript, &DiarizeOptions::default())
            .unwrap();
        assert_eq!(speakers(&transcript)[1], Some("SPEAKER_00"));
    }

    #[test]
    fn audio_with_nothing_long_enough_leaves_the_transcript_unlabelled() {
        let transcript = run(&[(400, LOUD), (300, QUIET)], &DiarizeOptions::default());
        assert_eq!(speakers(&transcript), [None, None]);
    }

    #[test]
    fn audio_at_another_sample_rate_is_rejected() {
        let (mut audio, mut transcript) = conversation(&[(2000, LOUD)]);
        audio.sample_rate = 44_100;
        let result = SpeakerDiarizer::new(LevelEmbedder).label_speakers(
            &audio,
            &mut transcript,
            &DiarizeOptions::default(),
        );
        assert!(matches!(result, Err(Error::Engine(_))));
    }
}
