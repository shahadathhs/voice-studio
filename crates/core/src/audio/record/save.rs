//! Saving a recording as a file.

use std::path::Path;

use crate::Result;
use crate::audio::{encode_flac, encode_wav};
use crate::engines::AudioBuffer;

/// Writes `audio` to `path` as FLAC when the name ends in `.flac`, as WAV otherwise, creating the
/// folder if needed.
pub fn save_recording(audio: &AudioBuffer, path: &Path) -> Result<()> {
    if let Some(folder) = path.parent().filter(|f| !f.as_os_str().is_empty()) {
        std::fs::create_dir_all(folder)?;
    }
    let is_flac = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("flac"));
    let bytes = if is_flac {
        encode_flac(audio)?
    } else {
        encode_wav(audio)
    };
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::load_for_whisper;

    fn tone() -> AudioBuffer {
        AudioBuffer {
            samples: (0..16_000u16)
                .map(|n| 0.3 * (f32::from(n) / 20.0).sin())
                .collect(),
            sample_rate: 16_000,
        }
    }

    #[test]
    fn wav_and_flac_files_read_back_and_folders_are_created() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["a/b/note.wav", "note.FLAC"] {
            let path = dir.path().join(name);
            save_recording(&tone(), &path).unwrap();
            let back = load_for_whisper(&path).unwrap();
            assert!((950..=1050).contains(&back.duration_ms()), "{name}");
        }
    }
}
