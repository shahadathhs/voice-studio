//! The samples captured so far, shared between the audio callback and the code that reads them.
//! Pure data and arithmetic, so it is tested with synthetic input instead of a microphone.

use std::time::Duration;

use super::super::decode::downmix;
use crate::engines::AudioBuffer;

/// How much of the meter's peak is kept each time it is read, so the bar falls smoothly.
const LEVEL_DECAY: f32 = 0.6;

/// Mono audio at the device's own rate, plus the meter state and the last stream failure.
#[derive(Debug)]
pub(super) struct Store {
    samples: Vec<f32>,
    sample_rate: u32,
    peak: f32,
    failure: Option<String>,
    mono: Vec<f32>,
}

impl Store {
    pub(super) fn new(sample_rate: u32) -> Store {
        Store {
            samples: Vec::new(),
            sample_rate,
            peak: 0.0,
            failure: None,
            mono: Vec::new(),
        }
    }

    /// Appends interleaved frames from the device, mixed down to mono.
    pub(super) fn push(&mut self, interleaved: &[f32], channels: usize) {
        downmix(interleaved, channels.max(1), &mut self.mono);
        let loudest = self.mono.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));
        self.peak = self.peak.max(loudest);
        self.samples.extend_from_slice(&self.mono);
    }

    /// The recent peak, 0 to 1. Each read lets the meter fall a little, so a quiet stretch
    /// fades out instead of freezing.
    pub(super) fn level(&mut self) -> f32 {
        let level = self.peak.min(1.0);
        self.peak *= LEVEL_DECAY;
        level
    }

    #[allow(clippy::cast_precision_loss)]
    pub(super) fn elapsed(&self) -> Duration {
        Duration::from_secs_f64(self.samples.len() as f64 / f64::from(self.sample_rate.max(1)))
    }

    /// The last `seconds` of audio, or everything if less has been captured.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub(super) fn tail(&self, seconds: f32) -> AudioBuffer {
        let wanted = (f64::from(seconds.max(0.0)) * f64::from(self.sample_rate)) as usize;
        let start = self.samples.len().saturating_sub(wanted);
        AudioBuffer {
            samples: self.samples[start..].to_vec(),
            sample_rate: self.sample_rate,
        }
    }

    pub(super) fn all(&self) -> AudioBuffer {
        AudioBuffer {
            samples: self.samples.clone(),
            sample_rate: self.sample_rate,
        }
    }

    pub(super) fn fail(&mut self, message: String) {
        self.failure = Some(message);
    }

    pub(super) fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_frames_are_mixed_down_to_mono() {
        let mut store = Store::new(10);
        store.push(&[1.0, 0.0, 0.5, 0.5], 2);
        assert_eq!(store.all().samples, vec![0.5, 0.5]);
    }

    #[test]
    fn elapsed_follows_the_sample_count() {
        let mut store = Store::new(1000);
        store.push(&vec![0.0; 1500], 1);
        assert_eq!(store.elapsed(), Duration::from_millis(1500));
    }

    #[test]
    fn the_level_is_the_recent_peak_and_decays() {
        let mut store = Store::new(1000);
        store.push(&[0.1, -0.8, 0.2], 1);
        assert!((store.level() - 0.8).abs() < 1e-6);
        let second = store.level();
        assert!(second < 0.8 && second > 0.0, "{second}");
        store.push(&[3.0], 1);
        assert!(
            (store.level() - 1.0).abs() < 1e-6,
            "clipped input stays at 1"
        );
    }

    #[test]
    fn tail_returns_only_the_last_seconds() {
        let mut store = Store::new(100);
        let ramp: Vec<f32> = (0..300u16).map(|n| f32::from(n) / 300.0).collect();
        store.push(&ramp, 1);
        let tail = store.tail(1.0);
        assert_eq!(tail.samples.len(), 100);
        assert_eq!(tail.samples[0], ramp[200]);
        assert_eq!(store.tail(10.0).samples.len(), 300);
        assert_eq!(store.tail(0.0).samples.len(), 0);
    }

    #[test]
    fn a_failure_is_remembered() {
        let mut store = Store::new(100);
        assert!(store.failure().is_none() && store.is_empty());
        store.fail("unplugged".into());
        assert_eq!(store.failure(), Some("unplugged"));
    }
}
