//! The outline of a recording for drawing: the loudest sample of each slice of it.

use std::path::Path;

use super::load_for_whisper;
use crate::Result;

/// Loudness of each of `bars` equal slices of `samples`, from 0 (silence) to 1 (the loudest slice).
/// Fewer samples than bars gives fewer bars; nothing gives none.
pub fn peaks(samples: &[f32], bars: usize) -> Vec<f32> {
    if samples.is_empty() || bars == 0 {
        return Vec::new();
    }
    let bars = bars.min(samples.len());
    let mut heights: Vec<f32> = (0..bars)
        .map(|n| {
            let from = n * samples.len() / bars;
            let to = (n + 1) * samples.len() / bars;
            samples[from..to]
                .iter()
                .fold(0.0_f32, |m, s| m.max(s.abs()))
        })
        .collect();
    let loudest = heights.iter().copied().fold(0.0_f32, f32::max);
    if loudest > 0.0 {
        for height in &mut heights {
            *height /= loudest;
        }
    }
    heights
}

/// Decodes `path` and returns its outline in `bars` slices.
pub fn waveform_of(path: &Path, bars: usize) -> Result<Vec<f32>> {
    Ok(peaks(&load_for_whisper(path)?.samples, bars))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_bar_is_the_loudest_sample_of_its_slice_scaled_to_the_loudest_bar() {
        let samples = [0.1, -0.2, 0.0, 0.4, -0.8, 0.2];
        assert_eq!(peaks(&samples, 3), [0.25, 0.5, 1.0]);
    }

    #[test]
    fn silence_stays_flat_and_short_audio_gives_fewer_bars() {
        assert_eq!(peaks(&[0.0; 10], 4), [0.0; 4]);
        assert_eq!(peaks(&[0.5, 0.25], 10).len(), 2);
        assert_eq!(peaks(&[], 5).len(), 0);
        assert_eq!(peaks(&[0.5], 0).len(), 0);
    }

    #[test]
    fn a_file_is_decoded_and_outlined() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.wav");
        let bars = waveform_of(&path, 20).unwrap();
        assert_eq!(bars.len(), 20);
        assert!(bars.iter().any(|b| (*b - 1.0).abs() < f32::EPSILON));
        assert!(waveform_of(Path::new("/no/such.wav"), 20).is_err());
    }
}
