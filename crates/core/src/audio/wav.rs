//! Writing audio as 16-bit mono WAV, the one format every player and editor understands.

use num_traits::ToPrimitive;

use crate::engines::AudioBuffer;
use crate::{Error, Result};

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
    encode_wav_with_note(audio, None)
}

/// A `LIST`/`INFO` chunk holding `note` as the file's comment (`ICMT`), padded to an even size.
fn info_chunk(note: &str) -> Vec<u8> {
    let mut text = note.as_bytes().to_vec();
    text.push(0);
    let text_len = u32::try_from(text.len()).unwrap_or(u32::MAX / 2);
    if text.len() % 2 == 1 {
        text.push(0);
    }
    let mut chunk = Vec::with_capacity(text.len() + 20);
    chunk.extend(b"LIST");
    chunk.extend((12 + u32::try_from(text.len()).unwrap_or(0)).to_le_bytes());
    chunk.extend(b"INFOICMT");
    chunk.extend(text_len.to_le_bytes());
    chunk.extend(text);
    chunk
}

/// Like [`encode_wav`], with `note` stored in the file as its comment (a `LIST`/`INFO` chunk
/// that players and tag editors show). `None` writes the plain file.
pub fn encode_wav_with_note(audio: &AudioBuffer, note: Option<&str>) -> Vec<u8> {
    let data_len = u32::try_from(audio.samples.len()).unwrap_or(u32::MAX / BYTES_PER_SAMPLE)
        * BYTES_PER_SAMPLE;
    let info = note.map(info_chunk).unwrap_or_default();
    let info_len = u32::try_from(info.len()).unwrap_or(0);
    let mut wav = Vec::with_capacity(44 + audio.samples.len() * 2 + info.len());
    wav.extend(b"RIFF");
    wav.extend((PCM_HEADER_BYTES + data_len + info_len).to_le_bytes());
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
    wav.extend(info);
    wav
}

/// The comment stored in a WAV file by [`encode_wav_with_note`], if it has one.
pub fn wav_note(bytes: &[u8]) -> Option<String> {
    const TAG: &[u8] = b"INFOICMT";
    let at = bytes.windows(TAG.len()).position(|window| window == TAG)? + TAG.len();
    let (length, rest) = bytes.get(at..)?.split_first_chunk::<4>()?;
    let text = rest.get(..u32::from_le_bytes(*length) as usize)?;
    Some(
        String::from_utf8_lossy(text)
            .trim_end_matches('\0')
            .to_string(),
    )
}

/// The audio of a WAV file written by [`encode_wav`] (16-bit mono PCM, a 44-byte header), at its own
/// sample rate. Other WAV layouts are an error; this is for converting our own output.
pub fn decode_wav(bytes: &[u8]) -> Result<AudioBuffer> {
    let header_ok = bytes.len() >= 44
        && &bytes[..4] == b"RIFF"
        && &bytes[8..16] == b"WAVEfmt "
        && bytes[20..22] == 1u16.to_le_bytes()
        && bytes[22..24] == 1u16.to_le_bytes()
        && bytes[34..36] == BITS.to_le_bytes()
        && &bytes[36..40] == b"data";
    if !header_ok {
        return Err(Error::Input("not a 16-bit mono WAV file".into()));
    }
    let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let declared = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;
    let data = &bytes[44..];
    let (pairs, _) = data[..declared.min(data.len())].as_chunks::<2>();
    let samples = pairs
        .iter()
        .map(|pair| f32::from(i16::from_le_bytes(*pair)) / f32::from(i16::MAX))
        .collect();
    Ok(AudioBuffer {
        samples,
        sample_rate,
    })
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

    #[test]
    fn a_wav_we_wrote_reads_back_at_its_own_rate() {
        let audio = AudioBuffer {
            samples: vec![0.0, 0.5, -0.5, 1.0],
            sample_rate: 24_000,
        };
        let back = decode_wav(&encode_wav(&audio)).unwrap();
        assert_eq!(back.sample_rate, 24_000);
        assert_eq!(back.samples.len(), 4);
        assert!(
            back.samples
                .iter()
                .zip(&audio.samples)
                .all(|(a, b)| (a - b).abs() < 0.001)
        );
    }

    #[test]
    fn anything_else_is_an_input_error() {
        assert!(decode_wav(b"not a wav file at all, definitely not one of ours....").is_err());
        assert!(decode_wav(&[]).is_err());
    }

    #[test]
    fn a_note_is_stored_in_an_info_chunk_and_the_audio_still_reads_back() {
        let audio = AudioBuffer {
            samples: vec![0.25; 101],
            sample_rate: 24_000,
        };
        let wav = encode_wav_with_note(&audio, Some("synthetic voice"));
        assert!(wav.windows(8).any(|w| w == b"INFOICMT"));
        assert!(wav.windows(15).any(|w| w == b"synthetic voice"));
        let riff = u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize;
        assert_eq!(riff + 8, wav.len());
        assert_eq!(wav.len() % 2, 0);
        let back = decode_wav(&wav).unwrap();
        assert_eq!(back.samples.len(), 101);
        assert_eq!(wav_note(&wav).as_deref(), Some("synthetic voice"));
        assert_eq!(wav_note(&encode_wav(&audio)), None);
        let plain = encode_wav(&audio);
        assert_eq!(plain.len(), 44 + 202);
    }
}
