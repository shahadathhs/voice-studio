use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, TrackType};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::default::{get_codecs, get_probe};

use super::resample::StreamResampler;
use crate::engines::AudioBuffer;
use crate::{Error, Result};

use super::WHISPER_SAMPLE_RATE;

fn unreadable(path: &Path, why: impl std::fmt::Display) -> Error {
    Error::Input(format!("cannot read audio from {}: {why}", path.display()))
}

pub fn decode_to_whisper_input(path: &Path) -> Result<AudioBuffer> {
    let (mut format, track_id, mut decoder) = open(path)?;
    let mut resampler: Option<StreamResampler> = None;
    let mut interleaved: Vec<f32> = Vec::new();
    let mut mono: Vec<f32> = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(e) => return Err(unreadable(path, e)),
        };
        if packet.track_id != track_id {
            continue;
        }
        let buffer = match decoder.decode(&packet) {
            Ok(buffer) => buffer,
            // A damaged packet is skipped; the rest of the file is still usable.
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(unreadable(path, e)),
        };
        let channels = buffer.spec().channels().count().max(1);
        buffer.copy_to_vec_interleaved::<f32>(&mut interleaved);
        downmix(&interleaved, channels, &mut mono);
        if resampler.is_none() {
            resampler = Some(StreamResampler::new(buffer.spec().rate())?);
        }
        if let Some(r) = resampler.as_mut() {
            r.push(&mono)?;
        }
    }
    let resampler = resampler.ok_or_else(|| unreadable(path, "the audio track is empty"))?;
    Ok(AudioBuffer {
        samples: resampler.finish()?,
        sample_rate: WHISPER_SAMPLE_RATE,
    })
}

type OpenedAudio = (Box<dyn FormatReader>, u32, Box<dyn AudioDecoder>);

/// Opens a file, finds its first audio track and prepares a decoder for it.
fn open(path: &Path) -> Result<OpenedAudio> {
    let file = File::open(path)
        .map_err(|e| Error::Input(format!("cannot open {}: {e}", path.display())))?;
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(extension);
    }
    let stream = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());
    let format = get_probe()
        .probe(
            &hint,
            stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| unreadable(path, format!("unrecognized format ({e})")))?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| unreadable(path, "no audio track"))?;
    let track_id = track.id;
    let Some(CodecParameters::Audio(params)) = track.codec_params.clone() else {
        return Err(unreadable(path, "the audio track has no codec information"));
    };
    let decoder = get_codecs()
        .make_audio_decoder(&params, &AudioDecoderOptions::default())
        .map_err(|e| unreadable(path, format!("unsupported audio codec ({e})")))?;
    Ok((format, track_id, decoder))
}

/// Averages the channels of interleaved audio into one.
#[allow(clippy::cast_precision_loss)]
fn downmix(interleaved: &[f32], channels: usize, out: &mut Vec<f32>) {
    out.clear();
    out.extend(
        interleaved
            .chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downmix_averages_channels() {
        let mut out = Vec::new();
        downmix(&[1.0, 0.0, 0.5, 0.5, -1.0, 1.0], 2, &mut out);
        assert_eq!(out, vec![0.5, 0.5, 0.0]);
        downmix(&[0.25, 0.75], 1, &mut out);
        assert_eq!(out, vec![0.25, 0.75]);
    }

    #[test]
    fn a_missing_file_is_an_input_error() {
        let error = decode_to_whisper_input(Path::new("/no/such/file.mp3")).unwrap_err();
        assert!(matches!(error, Error::Input(_)));
    }

    #[test]
    fn a_file_that_is_not_audio_is_an_input_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.mp3");
        std::fs::write(&path, b"this is plainly not an audio file").unwrap();
        assert!(matches!(
            decode_to_whisper_input(&path),
            Err(Error::Input(_))
        ));
    }
}
