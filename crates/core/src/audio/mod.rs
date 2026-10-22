//! Reading audio and video files into the 16 kHz mono samples that Whisper expects.

mod decode;
mod resample;
#[cfg(test)]
pub(crate) mod synthetic;
mod vad;

pub use vad::speech_chunks;

use std::path::Path;

use crate::Result;
use crate::engines::AudioBuffer;

/// Whisper models are trained on 16 kHz mono audio.
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

/// Decodes the first audio track of `path` (any supported audio or video file) and converts it to
/// 16 kHz mono.
pub fn load_for_whisper(path: &Path) -> Result<AudioBuffer> {
    decode::decode_to_whisper_input(path)
}
