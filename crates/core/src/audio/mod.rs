//! Reading audio and video files into the 16 kHz mono samples that Whisper expects.

mod decode;
mod flac;
mod format;
pub mod record;
mod resample;
#[cfg(test)]
pub(crate) mod synthetic;
mod vad;
mod wav;
mod waveform;

pub use flac::{encode_flac, encode_flac_with_note};
pub use format::AudioFormat;
pub use resample::{resample_to, resample_to_whisper};
pub use vad::{speech_chunks, voiced_span};
pub use wav::{decode_wav, encode_wav, encode_wav_with_note, wav_note};
pub use waveform::{peaks, waveform_of};

use std::path::Path;

use crate::Result;
use crate::engines::AudioBuffer;

/// Whisper models are trained on 16 kHz mono audio.
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

/// Like [`load_for_whisper`], at `rate` instead of 16 kHz.
pub fn load_at_rate(path: &Path, rate: u32) -> Result<AudioBuffer> {
    decode::decode_to_rate(path, rate)
}

/// Decodes the first audio track of `path` (any supported audio or video file) and converts it to
/// 16 kHz mono.
pub fn load_for_whisper(path: &Path) -> Result<AudioBuffer> {
    decode::decode_to_whisper_input(path)
}
