//! Made-up audio for tests: silence and a steady tone standing in for speech.

use super::WHISPER_SAMPLE_RATE;
use crate::engines::AudioBuffer;

/// Samples in one period of the tone (200 Hz at 16 kHz).
const PERIOD: u16 = 80;
const LOUDNESS: f32 = 0.3;

pub const RATE: u32 = WHISPER_SAMPLE_RATE;

fn samples(ms: usize) -> usize {
    ms * RATE as usize / 1000
}

pub fn silence(ms: usize) -> Vec<f32> {
    vec![0.0; samples(ms)]
}

pub fn voice(ms: usize) -> Vec<f32> {
    (0..samples(ms))
        .map(|n| {
            let step = f32::from(u16::try_from(n % usize::from(PERIOD)).unwrap_or(0));
            LOUDNESS * (std::f32::consts::TAU * step / f32::from(PERIOD)).sin()
        })
        .collect()
}

/// Joins pieces of audio into one buffer.
pub fn recording(parts: &[Vec<f32>]) -> AudioBuffer {
    AudioBuffer {
        samples: parts.concat(),
        sample_rate: RATE,
    }
}
