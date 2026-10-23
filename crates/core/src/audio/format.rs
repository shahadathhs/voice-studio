//! The audio file formats speech can be saved as.

use std::path::Path;
use std::str::FromStr;

use super::{encode_flac_with_note, encode_wav_with_note};
use crate::engines::AudioBuffer;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioFormat {
    /// 16-bit mono PCM: every player and editor reads it.
    #[default]
    Wav,
    /// The same samples, losslessly compressed to about half the size.
    Flac,
}

impl AudioFormat {
    pub fn extension(self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Flac => "flac",
        }
    }

    /// The format a file name asks for (`speech.flac`), if it is one we write.
    pub fn from_path(path: &Path) -> Option<AudioFormat> {
        path.extension()?.to_str()?.parse().ok()
    }

    /// The audio as the bytes of a file in this format.
    pub fn encode(self, audio: &AudioBuffer) -> Result<Vec<u8>> {
        self.encode_with_note(audio, None)
    }

    /// Like [`AudioFormat::encode`], storing `note` in the file as its comment.
    pub fn encode_with_note(self, audio: &AudioBuffer, note: Option<&str>) -> Result<Vec<u8>> {
        match self {
            AudioFormat::Wav => Ok(encode_wav_with_note(audio, note)),
            AudioFormat::Flac => encode_flac_with_note(audio, note),
        }
    }
}

impl FromStr for AudioFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<AudioFormat> {
        match s.to_ascii_lowercase().as_str() {
            "wav" => Ok(AudioFormat::Wav),
            "flac" => Ok(AudioFormat::Flac),
            // The common MP3 encoder is LGPL, which this project does not link.
            "mp3" => Err(Error::NotImplemented("mp3 output")),
            other => Err(Error::Input(format!("unknown audio format: {other}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_paths_pick_the_format() {
        assert_eq!("WAV".parse::<AudioFormat>().unwrap(), AudioFormat::Wav);
        assert_eq!("flac".parse::<AudioFormat>().unwrap(), AudioFormat::Flac);
        assert_eq!(
            AudioFormat::from_path(Path::new("a/b/speech.FLAC")),
            Some(AudioFormat::Flac)
        );
        assert_eq!(AudioFormat::from_path(Path::new("speech")), None);
        assert_eq!(AudioFormat::from_path(Path::new("speech.txt")), None);
    }

    #[test]
    fn mp3_is_not_implemented_and_other_names_are_input_errors() {
        assert!(matches!(
            "mp3".parse::<AudioFormat>(),
            Err(Error::NotImplemented(_))
        ));
        assert!(matches!("ogg".parse::<AudioFormat>(), Err(Error::Input(_))));
    }

    #[test]
    fn each_format_writes_its_own_magic_bytes() {
        let audio = AudioBuffer {
            samples: vec![0.0; 100],
            sample_rate: 24_000,
        };
        assert_eq!(&AudioFormat::Wav.encode(&audio).unwrap()[..4], b"RIFF");
        assert_eq!(&AudioFormat::Flac.encode(&audio).unwrap()[..4], b"fLaC");
    }
}
