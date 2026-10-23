//! Log mel filterbank features in the style of Kaldi (`torchaudio.compliance.kaldi.fbank`), which
//! is what the `WeSpeaker` model was trained on: 80 mel bins, 25 ms Hamming windows every 10 ms,
//! pre-emphasis and no dither, followed by subtracting each bin's mean over the recording.

use std::f32::consts::TAU;

use super::fft::{PowerSpectrum, as_f32};

/// Number of mel bins; the model's input width.
pub const MEL_BINS: usize = 80;
const SAMPLE_RATE: f32 = 16_000.0;
const FRAME_LEN: usize = 400;
const FRAME_SHIFT: usize = 160;
const FFT_SIZE: usize = 512;
const PREEMPHASIS: f32 = 0.97;
const LOW_HZ: f32 = 20.0;
/// The model expects 16-bit PCM amplitudes, not samples in -1..1.
const PCM_SCALE: f32 = 32_768.0;

fn mel(hz: f32) -> f32 {
    1127.0 * (1.0 + hz / 700.0).ln()
}

/// Computes features for 16 kHz mono audio.
pub struct Fbank {
    window: Vec<f32>,
    spectrum: PowerSpectrum,
    /// For each mel bin, the first FFT bin it covers and its weights from there on.
    filters: Vec<(usize, Vec<f32>)>,
}

impl Fbank {
    pub fn new() -> Fbank {
        let window = (0..FRAME_LEN)
            .map(|n| 0.54 - 0.46 * (TAU * as_f32(n) / as_f32(FRAME_LEN - 1)).cos())
            .collect();
        Fbank {
            window,
            spectrum: PowerSpectrum::new(FFT_SIZE),
            filters: mel_filters(),
        }
    }

    /// Number of feature frames for `samples` audio samples: zero when shorter than one window,
    /// and at most 65535 (about 11 minutes), since the features are meant for short chunks.
    pub fn frame_count(samples: usize) -> usize {
        if samples < FRAME_LEN {
            0
        } else {
            (1 + (samples - FRAME_LEN) / FRAME_SHIFT).min(usize::from(u16::MAX))
        }
    }

    /// Features as `frames x MEL_BINS`, flattened row by row, with each column's mean removed.
    pub fn features(&self, samples: &[f32]) -> Vec<f32> {
        let frames = Fbank::frame_count(samples.len());
        let mut out = Vec::with_capacity(frames * MEL_BINS);
        for frame in 0..frames {
            let start = frame * FRAME_SHIFT;
            out.extend(self.frame(&samples[start..start + FRAME_LEN]));
        }
        subtract_column_means(&mut out, frames);
        out
    }

    fn frame(&self, raw: &[f32]) -> Vec<f32> {
        let mut x: Vec<f32> = raw.iter().map(|s| s * PCM_SCALE).collect();
        let mean = x.iter().sum::<f32>() / as_f32(FRAME_LEN);
        for s in &mut x {
            *s -= mean;
        }
        for n in (1..FRAME_LEN).rev() {
            x[n] -= PREEMPHASIS * x[n - 1];
        }
        x[0] -= PREEMPHASIS * x[0];
        x.iter_mut().zip(&self.window).for_each(|(s, w)| *s *= w);
        let power = self.spectrum.power(&x);
        self.filters
            .iter()
            .map(|(first, weights)| {
                let energy: f32 = weights
                    .iter()
                    .zip(&power[*first..])
                    .map(|(w, p)| w * p)
                    .sum();
                energy.max(f32::EPSILON).ln()
            })
            .collect()
    }
}

impl Default for Fbank {
    fn default() -> Self {
        Fbank::new()
    }
}

/// Triangular filters evenly spaced on the mel scale between 20 Hz and the Nyquist frequency.
fn mel_filters() -> Vec<(usize, Vec<f32>)> {
    let (low, high) = (mel(LOW_HZ), mel(SAMPLE_RATE / 2.0));
    let delta = (high - low) / as_f32(MEL_BINS + 1);
    let bin_width = SAMPLE_RATE / as_f32(FFT_SIZE);
    (0..MEL_BINS)
        .map(|bin| {
            let left = low + as_f32(bin) * delta;
            let (center, right) = (left + delta, left + 2.0 * delta);
            let weights: Vec<f32> = (0..FFT_SIZE / 2)
                .map(|k| {
                    let m = mel(as_f32(k) * bin_width);
                    if m > left && m <= center {
                        (m - left) / (center - left)
                    } else if m > center && m < right {
                        (right - m) / (right - center)
                    } else {
                        0.0
                    }
                })
                .collect();
            let first = weights.iter().position(|w| *w > 0.0).unwrap_or(0);
            let last = weights.iter().rposition(|w| *w > 0.0).unwrap_or(first);
            (first, weights[first..=last].to_vec())
        })
        .collect()
}

fn subtract_column_means(features: &mut [f32], frames: usize) {
    if frames == 0 {
        return;
    }
    for bin in 0..MEL_BINS {
        let mean = (0..frames)
            .map(|f| features[f * MEL_BINS + bin])
            .sum::<f32>()
            / as_f32(frames);
        for f in 0..frames {
            features[f * MEL_BINS + bin] -= mean;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(hz: f32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|n| 0.3 * (TAU * hz * as_f32(n) / SAMPLE_RATE).sin())
            .collect()
    }

    #[test]
    fn frames_follow_the_25_ms_window_and_10_ms_shift() {
        assert_eq!(Fbank::frame_count(399), 0);
        assert_eq!(Fbank::frame_count(400), 1);
        assert_eq!(Fbank::frame_count(560), 2);
        assert_eq!(Fbank::frame_count(24_000), 148);
        assert_eq!(
            Fbank::new().features(&tone(440.0, 24_000)).len(),
            148 * MEL_BINS
        );
    }

    #[test]
    fn every_column_is_centred_on_zero() {
        let mut audio = tone(300.0, 8_000);
        audio.extend(tone(2_000.0, 8_000));
        let frames = Fbank::frame_count(audio.len());
        let features = Fbank::new().features(&audio);
        for bin in 0..MEL_BINS {
            let mean: f32 = (0..frames)
                .map(|f| features[f * MEL_BINS + bin])
                .sum::<f32>()
                / as_f32(frames);
            assert!(mean.abs() < 1e-3, "bin {bin}: {mean}");
        }
    }

    #[test]
    fn higher_tones_light_up_higher_mel_bins() {
        let loudest_bin = |hz: f32| {
            let fbank = Fbank::new();
            // Without mean removal the tone's own bin dominates, so read one raw frame.
            let frame = fbank.frame(&tone(hz, FRAME_LEN));
            frame
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(bin, _)| bin)
                .unwrap()
        };
        let (low, mid, high) = (
            loudest_bin(300.0),
            loudest_bin(1_500.0),
            loudest_bin(5_000.0),
        );
        assert!(low < mid && mid < high, "{low} {mid} {high}");
    }

    #[test]
    fn silence_gives_the_floor_value_everywhere() {
        let frame = Fbank::new().frame(&vec![0.0; FRAME_LEN]);
        assert!(frame.iter().all(|v| (*v - f32::EPSILON.ln()).abs() < 1e-6));
    }
}
