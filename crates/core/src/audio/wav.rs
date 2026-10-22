//! Writing audio as 16-bit mono WAV, the one format every player and editor understands.

use num_traits::ToPrimitive;

use crate::engines::AudioBuffer;

const PCM_HEADER_BYTES: u32 = 36;
const BITS: u16 = 16;
const BYTES_PER_SAMPLE: u32 = 2;

fn pcm(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX))
        .round()
        .to_i16()
        .unwrap_or(0)
}

/// The audio as the bytes of a WAV file. Samples outside -1..1 are clipped.
pub fn encode_wav(audio: &AudioBuffer) -> Vec<u8> {
    let data_len = u32::try_from(audio.samples.len()).unwrap_or(u32::MAX / BYTES_PER_SAMPLE)
        * BYTES_PER_SAMPLE;
    let mut wav = Vec::with_capacity(44 + audio.samples.len() * 2);
    wav.extend(b"RIFF");
    wav.extend((PCM_HEADER_BYTES + data_len).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(audio.sample_rate.to_le_bytes());
    wav.extend((audio.sample_rate * BYTES_PER_SAMPLE).to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(BITS.to_le_bytes());
    wav.extend(b"data");
    wav.extend(data_len.to_le_bytes());
    for sample in &audio.samples {
        wav.extend(pcm(*sample).to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_describes_16_bit_mono_audio_at_the_given_rate() {
        let audio = AudioBuffer {
            samples: vec![0.0; 100],
            sample_rate: 24_000,
        };
        let wav = encode_wav(&audio);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 24_000);
        assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16);
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 200);
        assert_eq!(wav.len(), 244);
    }

    #[test]
    fn loud_samples_are_clipped_not_wrapped() {
        let audio = AudioBuffer {
            samples: vec![2.0, -2.0, 0.5],
            sample_rate: 8_000,
        };
        let wav = encode_wav(&audio);
        let sample = |n: usize| i16::from_le_bytes(wav[44 + n * 2..46 + n * 2].try_into().unwrap());
        assert_eq!((sample(0), sample(1)), (i16::MAX, -i16::MAX));
        assert_eq!(sample(2), 16384);
    }

    #[test]
    fn a_file_we_wrote_can_be_decoded_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.wav");
        let tone: Vec<f32> = (0..8000u16)
            .map(|n| 0.3 * (f32::from(n) / 20.0).sin())
            .collect();
        std::fs::write(
            &path,
            encode_wav(&AudioBuffer {
                samples: tone,
                sample_rate: 16_000,
            }),
        )
        .unwrap();
        let decoded = crate::audio::load_for_whisper(&path).unwrap();
        assert!(
            (450..=550).contains(&decoded.duration_ms()),
            "{}",
            decoded.duration_ms()
        );
    }
}
