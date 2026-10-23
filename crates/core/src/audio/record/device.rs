//! Talking to the operating system: listing microphones and running a capture stream.
//!
//! The stream lives on its own thread because some platforms' streams cannot move between
//! threads; that also keeps `Recorder` freely sendable.

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ErrorKind, FromSample, SampleFormat, SizedSample, Stream, SupportedStreamConfig};

use super::InputDevice;
use super::store::Store;
use crate::{Error, Result};

pub(super) type Shared = Arc<Mutex<Store>>;

pub(super) fn lock(store: &Shared) -> std::sync::MutexGuard<'_, Store> {
    store.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What a failed cpal call means for the person using the app.
pub(super) fn input_error(error: &cpal::Error) -> Error {
    let message = match error.kind() {
        ErrorKind::PermissionDenied => "microphone access was denied; allow it for this app in \
            the system privacy settings (on macOS: System Settings > Privacy & Security > Microphone)"
            .to_string(),
        ErrorKind::DeviceNotAvailable | ErrorKind::HostUnavailable => {
            format!("the microphone is not available: {error}")
        }
        ErrorKind::DeviceBusy => format!("the microphone is busy in another app: {error}"),
        _ => format!("cannot use the microphone: {error}"),
    };
    Error::Input(message)
}

fn name_of(device: &cpal::Device) -> Option<String> {
    device.description().ok().map(|d| d.name().to_string())
}

pub(super) fn list() -> Result<Vec<InputDevice>> {
    let host = cpal::default_host();
    let default = host.default_input_device().and_then(|d| name_of(&d));
    let devices = host.input_devices().map_err(|e| input_error(&e))?;
    Ok(devices
        .filter_map(|device| name_of(&device))
        .map(|name| InputDevice {
            is_default: default.as_deref() == Some(name.as_str()),
            name,
        })
        .collect())
}

fn pick(name: Option<&str>) -> Result<cpal::Device> {
    let host = cpal::default_host();
    let Some(wanted) = name else {
        return host
            .default_input_device()
            .ok_or_else(|| Error::Input("no microphone found; connect one and try again".into()));
    };
    let mut devices = host.input_devices().map_err(|e| input_error(&e))?;
    devices
        .find(|d| name_of(d).is_some_and(|n| n.eq_ignore_ascii_case(wanted)))
        .ok_or_else(|| Error::Input(format!("no input device named \"{wanted}\"")))
}

/// A running capture; stopping or dropping it ends the stream.
pub(super) struct Capture {
    stop: Sender<()>,
    thread: JoinHandle<()>,
}

impl Capture {
    pub(super) fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.thread.join();
    }
}

pub(super) fn start(name: Option<&str>) -> Result<(Capture, Shared)> {
    let name = name.map(str::to_string);
    let (ready_tx, ready_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let thread = thread::spawn(move || match open(name.as_deref()) {
        Ok((stream, store)) => {
            let _ = ready_tx.send(Ok(store));
            // Returns when `stop` is sent or the `Capture` is dropped.
            let _ = stop_rx.recv();
            drop(stream);
        }
        Err(e) => {
            let _ = ready_tx.send(Err(e));
        }
    });
    let store = ready_rx
        .recv()
        .map_err(|_| Error::Engine("the audio thread stopped unexpectedly".into()))??;
    Ok((
        Capture {
            stop: stop_tx,
            thread,
        },
        store,
    ))
}

fn open(name: Option<&str>) -> Result<(Stream, Shared)> {
    let device = pick(name)?;
    let config = device.default_input_config().map_err(|e| input_error(&e))?;
    let store: Shared = Arc::new(Mutex::new(Store::new(config.sample_rate())));
    let stream = build(&device, &config, &store).map_err(|e| input_error(&e))?;
    stream.play().map_err(|e| input_error(&e))?;
    Ok((stream, store))
}

fn build(
    device: &cpal::Device,
    config: &SupportedStreamConfig,
    store: &Shared,
) -> std::result::Result<Stream, cpal::Error> {
    match config.sample_format() {
        SampleFormat::F32 => stream_of::<f32>(device, config, store),
        SampleFormat::I16 => stream_of::<i16>(device, config, store),
        SampleFormat::I32 => stream_of::<i32>(device, config, store),
        SampleFormat::U16 => stream_of::<u16>(device, config, store),
        _ => Err(ErrorKind::UnsupportedConfig.into()),
    }
}

fn stream_of<T>(
    device: &cpal::Device,
    config: &SupportedStreamConfig,
    store: &Shared,
) -> std::result::Result<Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels());
    let (on_data, on_error) = (Arc::clone(store), Arc::clone(store));
    device.build_input_stream(
        (*config).into(),
        move |data: &[T], _: &_| {
            let floats: Vec<f32> = data.iter().map(|s| s.to_sample::<f32>()).collect();
            lock(&on_data).push(&floats, channels);
        },
        move |error: cpal::Error| {
            // Rerouting and under-runs are glitches, not the end of the recording.
            if !matches!(
                error.kind(),
                ErrorKind::DeviceChanged | ErrorKind::Xrun | ErrorKind::RealtimeDenied
            ) {
                lock(&on_error).fail(input_error(&error).to_string());
            }
        },
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_denied_microphone_explains_how_to_allow_it() {
        let message = input_error(&ErrorKind::PermissionDenied.into()).to_string();
        assert!(message.contains("Microphone"), "{message}");
    }

    #[test]
    fn other_failures_are_input_errors_too() {
        let error = input_error(&ErrorKind::DeviceNotAvailable.into());
        assert!(matches!(error, Error::Input(m) if m.contains("not available")));
    }
}
