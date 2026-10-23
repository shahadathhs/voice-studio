//! Microphone capture: list the input devices, record, and read the audio while it runs.
//!
//! ```no_run
//! # fn main() -> vtt_core::Result<()> {
//! use vtt_core::audio::record::Recorder;
//! let recorder = Recorder::start(None)?;          // the default microphone
//! std::thread::sleep(std::time::Duration::from_secs(2));
//! let meter = recorder.level();                   // 0..1, for a level bar
//! let recent = recorder.snapshot(15.0)?;          // last 15 s at 16 kHz, for live captions
//! let whole = recorder.stop()?;                   // everything, at the device's sample rate
//! # let _ = (meter, recent, whole); Ok(()) }
//! ```
//!
//! Needs no UI. A denied microphone permission or a missing device is an `Error::Input` with a
//! message fit for the user.

mod device;
mod save;
mod store;

use std::time::Duration;

use self::device::{Capture, Shared, lock};
pub use self::save::save_recording;
use super::resample_to_whisper;
use crate::engines::AudioBuffer;
use crate::{Error, Result};

/// A microphone the system offers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InputDevice {
    /// The name to pass to [`Recorder::start`].
    pub name: String,
    /// Whether the system uses this one unless told otherwise.
    pub is_default: bool,
}

/// The input devices of this machine, the default one flagged. Empty when there is none.
pub fn list_input_devices() -> Result<Vec<InputDevice>> {
    device::list()
}

/// A cheap, cloneable, thread-safe view of a running recording: everything a meter, a timer or
/// live captions need, without owning the stream.
#[derive(Debug, Clone)]
pub struct Monitor {
    store: Shared,
}

impl Monitor {
    /// The recent peak, 0 to 1; falls off a little on each call.
    pub fn level(&self) -> f32 {
        lock(&self.store).level()
    }

    /// How much audio has been captured.
    pub fn elapsed(&self) -> Duration {
        lock(&self.store).elapsed()
    }

    /// The last `last_seconds` captured (or all of it if shorter), mono at 16 kHz.
    pub fn snapshot(&self, last_seconds: f32) -> Result<AudioBuffer> {
        // Copy under the lock, resample outside it so the audio callback is never kept waiting.
        let recent = lock(&self.store).tail(last_seconds);
        resample_to_whisper(&recent)
    }

    /// The reason the stream died (the microphone was unplugged, say), if it did.
    pub fn failure(&self) -> Option<String> {
        lock(&self.store).failure().map(str::to_string)
    }
}

/// A recording in progress.
pub struct Recorder {
    capture: Capture,
    monitor: Monitor,
}

impl Recorder {
    /// Starts capturing from the named device (see [`list_input_devices`]), or the default one.
    pub fn start(device: Option<&str>) -> Result<Recorder> {
        let (capture, store) = device::start(device)?;
        Ok(Recorder {
            capture,
            monitor: Monitor { store },
        })
    }

    /// A view that can be handed to another thread (live captions, a level meter).
    pub fn monitor(&self) -> Monitor {
        self.monitor.clone()
    }

    /// See [`Monitor::level`].
    pub fn level(&self) -> f32 {
        self.monitor.level()
    }

    /// See [`Monitor::elapsed`].
    pub fn elapsed(&self) -> Duration {
        self.monitor.elapsed()
    }

    /// See [`Monitor::snapshot`].
    pub fn snapshot(&self, last_seconds: f32) -> Result<AudioBuffer> {
        self.monitor.snapshot(last_seconds)
    }

    /// Stops the stream and returns everything captured: mono, at the device's sample rate.
    /// Fails only if the stream broke before any audio arrived.
    pub fn stop(self) -> Result<AudioBuffer> {
        self.capture.stop();
        let store = lock(&self.monitor.store);
        match store.failure() {
            Some(reason) if store.is_empty() => Err(Error::Input(reason.to_string())),
            _ => Ok(store.all()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::store::Store;
    use super::*;

    fn monitor_with(rate: u32, samples: &[f32]) -> Monitor {
        let mut store = Store::new(rate);
        store.push(samples, 1);
        Monitor {
            store: Arc::new(Mutex::new(store)),
        }
    }

    #[test]
    fn a_snapshot_is_the_recent_audio_at_the_speech_rate() {
        let monitor = monitor_with(48_000, &vec![0.25; 48_000 * 3]);
        let snapshot = monitor.snapshot(2.0).unwrap();
        assert_eq!(snapshot.sample_rate, 16_000);
        assert_eq!(snapshot.samples.len(), 32_000);
        assert_eq!(monitor.elapsed(), Duration::from_secs(3));
    }

    #[test]
    fn the_monitor_reports_level_and_failure() {
        let monitor = monitor_with(16_000, &[0.0, 0.5]);
        assert!((monitor.level() - 0.5).abs() < 1e-6);
        assert!(monitor.failure().is_none());
        lock(&monitor.store).fail("unplugged".into());
        assert_eq!(monitor.failure().as_deref(), Some("unplugged"));
    }
}
