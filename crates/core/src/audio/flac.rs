//! Writing audio as 16-bit mono FLAC: lossless like WAV, about half the size.

use flacenc::component::BitRepr;
use flacenc::error::Verify;

use num_traits::ToPrimitive;

use crate::engines::AudioBuffer;
use crate::{Error, Result};

const CHANNELS: usize = 1;
const BITS: usize = 16;
/// Samples per FLAC block; the usual value for 16-bit audio at speech sample rates.
const BLOCK_SIZE: usize = 4096;

fn encode_error(e: impl std::fmt::Display) -> Error {
    Error::Engine(format!("flac: {e}"))
}

/// The audio as the bytes of a FLAC file. Samples outside -1..1 are clipped. The end is padded
/// with silence (less than one block, under 0.1 s at speech sample rates) because some decoders,
/// this crate's own among them, refuse a file whose last block is short.
pub fn encode_flac(audio: &AudioBuffer) -> Result<Vec<u8>> {
    let mut samples: Vec<i32> = audio
        .samples
        .iter()
        .map(|s| {
            (s.clamp(-1.0, 1.0) * f32::from(i16::MAX))
                .round()
                .to_i32()
                .unwrap_or(0)
        })
        .collect();
    let padding = (BLOCK_SIZE - samples.len() % BLOCK_SIZE) % BLOCK_SIZE;
    samples.resize(samples.len() + padding, 0);
    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|(_, e)| encode_error(format!("{e:?}")))?;
    let sample_rate = usize::try_from(audio.sample_rate).unwrap_or(usize::MAX);
    let source = flacenc::source::MemSource::from_samples(&samples, CHANNELS, BITS, sample_rate);
    let stream = flacenc::encode_with_fixed_block_size(&config, source, BLOCK_SIZE)
        .map_err(|e| encode_error(format!("{e:?}")))?;
    let mut sink = flacenc::bitsink::ByteSink::new();
    stream.write(&mut sink).map_err(encode_error)?;
    Ok(sink.as_slice().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::load_for_whisper;

    #[test]
    fn flac_decodes_back_to_the_same_audio() {
        let samples: Vec<f32> = (0..16_000_u16)
            .map(|n| 0.5 * (f32::from(n % 100) * 0.0628).sin())
            .collect();
        let audio = AudioBuffer {
            samples: samples.clone(),
            sample_rate: 16_000,
        };
        let bytes = encode_flac(&audio).unwrap();
        assert_eq!(&bytes[..4], b"fLaC");
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.flac");
        std::fs::write(&path, bytes).unwrap();
        let back = load_for_whisper(&path).unwrap();
        // Whole blocks: 16 000 samples become four blocks of 4096.
        assert_eq!(back.samples.len(), 16_384);
        let tail = back.samples[16_000..]
            .iter()
            .fold(0.0_f32, |m, s| m.max(s.abs()));
        assert!(tail < 0.001, "{tail}");
        let worst = back
            .samples
            .iter()
            .zip(&samples)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(worst < 0.001, "{worst}");
    }

    #[test]
    fn empty_audio_still_makes_a_valid_file() {
        let audio = AudioBuffer {
            samples: Vec::new(),
            sample_rate: 24_000,
        };
        assert_eq!(&encode_flac(&audio).unwrap()[..4], b"fLaC");
    }
}
