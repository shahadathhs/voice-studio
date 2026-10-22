//! Engine interfaces. Implementations (whisper.cpp / sherpa-onnx, Kokoro, Piper, diarization)
//! are added behind these traits so front ends never depend on a specific backend.

mod whisper;

pub use whisper::{Device, WhisperEngine};

use crate::{Result, Transcript};

/// Mono PCM audio.
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

// Printing every sample would flood a failing test or a log line.
impl std::fmt::Debug for AudioBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBuffer")
            .field("frames", &self.samples.len())
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}

impl AudioBuffer {
    pub fn duration_ms(&self) -> u64 {
        if self.sample_rate == 0 {
            return 0;
        }
        self.samples.len() as u64 * 1000 / u64::from(self.sample_rate)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SttOptions {
    /// `None` means auto-detect.
    pub language: Option<String>,
    pub translate: bool,
    /// Transcribe only the stretches that contain speech, in chunks (see `audio::speech_chunks`).
    pub vad: bool,
}

/// Returned by progress callbacks so a front end can cancel a running job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Continue,
    Cancel,
}

pub trait SttEngine: Send {
    fn transcribe(
        &mut self,
        audio: &AudioBuffer,
        options: &SttOptions,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript>;
}

#[derive(Debug, Clone)]
pub struct TtsOptions {
    pub voice: String,
    pub speed: f32,
}

pub trait TtsEngine: Send {
    /// Synthesize one sentence/chunk; callers stream chunks so playback can start early.
    fn synthesize(&mut self, text: &str, options: &TtsOptions) -> Result<AudioBuffer>;
}

#[derive(Debug, Clone, Default)]
pub struct DiarizeOptions {
    pub speakers: Option<usize>,
    pub threshold: Option<f32>,
}

pub trait Diarizer: Send {
    /// Assigns `speaker` labels to the segments of `transcript` using `audio`.
    fn label_speakers(
        &mut self,
        audio: &AudioBuffer,
        transcript: &mut Transcript,
        options: &DiarizeOptions,
    ) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_from_samples() {
        let a = AudioBuffer {
            samples: vec![0.0; 32_000],
            sample_rate: 16_000,
        };
        assert_eq!(a.duration_ms(), 2000);
    }
}
