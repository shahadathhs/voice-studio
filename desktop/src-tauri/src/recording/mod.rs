//! Microphone recording for the Record page. One recording runs at a time; a ticker thread sends
//! the level meter and timer, and (when asked) a second thread sends live captions. Stopping saves
//! a WAV into the data directory's `recordings/` folder, which the UI then transcribes, copies or
//! deletes.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::AppHandle;
use vtt_core::audio::record::{InputDevice, Recorder, save_recording};
use vtt_core::engines::SttOptions;
use vtt_core::paths::Paths;
use vtt_core::{Error, Result};

use crate::commands::CommandError;

mod live;

pub(crate) use live::live_engine;

const RECORDINGS_DIR: &str = "recordings";
const FILE_PREFIX: &str = "recording-";

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// Mirrored by `RecordingResult` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
pub struct RecordingResult {
    path: String,
    duration_ms: u64,
}

struct Session {
    recorder: Recorder,
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

fn session() -> MutexGuard<'static, Option<Session>> {
    SESSION.lock().unwrap_or_else(PoisonError::into_inner)
}

fn recordings_dir(paths: &Paths) -> PathBuf {
    paths.data.join(RECORDINGS_DIR)
}

/// A file name for a recording started at `millis` since the epoch.
fn recording_path(dir: &Path, millis: u128) -> PathBuf {
    dir.join(format!("{FILE_PREFIX}{millis}.wav"))
}

/// Only files this module made may be copied or deleted by the UI.
fn check_recording(paths: &Paths, file: &str) -> Result<PathBuf> {
    let path = PathBuf::from(file);
    let inside = path.parent() == Some(recordings_dir(paths).as_path());
    if inside && path.is_file() {
        Ok(path)
    } else {
        Err(Error::Input(
            "that is not one of the app's recordings".into(),
        ))
    }
}

fn start(app: AppHandle, request: &Request) -> Result<()> {
    let paths = Paths::resolve(None);
    let mut slot = session();
    if slot.is_some() {
        return Err(Error::Input("a recording is already running".into()));
    }
    let engine = request
        .live
        .then(|| live::live_engine(&paths, request.model.as_deref()))
        .transpose()?;
    let recorder = Recorder::start(request.device.as_deref())?;
    let stop = Arc::new(AtomicBool::new(false));
    let mut threads = Vec::new();
    if let Some(engine) = engine {
        let (handle, monitor, flag) = (app.clone(), recorder.monitor(), Arc::clone(&stop));
        let options = SttOptions {
            language: request.language.clone(),
            ..SttOptions::default()
        };
        threads.push(std::thread::spawn(move || {
            live::caption_loop(&handle, &monitor, &flag, (engine, options));
        }));
    }
    let (monitor, flag) = (recorder.monitor(), Arc::clone(&stop));
    threads.push(std::thread::spawn(move || {
        live::tick_loop(&app, &monitor, &flag);
    }));
    *slot = Some(Session {
        recorder,
        stop,
        threads,
    });
    Ok(())
}

/// Ends the running recording and returns its audio.
fn finish() -> Result<vtt_core::engines::AudioBuffer> {
    let Some(running) = session().take() else {
        return Err(Error::Input("nothing is being recorded".into()));
    };
    running.stop.store(true, Ordering::SeqCst);
    for thread in running.threads {
        let _ = thread.join();
    }
    running.recorder.stop()
}

fn save_finished(paths: &Paths) -> Result<RecordingResult> {
    let audio = finish()?;
    if audio.samples.is_empty() {
        return Err(Error::Input("nothing was recorded".into()));
    }
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let path = recording_path(&recordings_dir(paths), millis);
    save_recording(&audio, &path)?;
    Ok(RecordingResult {
        path: path.display().to_string(),
        duration_ms: audio.duration_ms(),
    })
}

/// What to record and whether to caption it live.
pub struct Request {
    device: Option<String>,
    live: bool,
    model: Option<String>,
    language: Option<String>,
}

/// The microphones, the default one flagged. Empty when there is none.
#[tauri::command]
pub fn list_input_devices() -> std::result::Result<Vec<InputDevice>, CommandError> {
    Ok(vtt_core::audio::record::list_input_devices()?)
}

/// Starts recording. With `live`, captions arrive as `recording-captions` events; that needs an
/// installed speech model (`model`, else the settings', else the default).
#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    device: Option<String>,
    live: bool,
    model: Option<String>,
    language: Option<String>,
) -> std::result::Result<(), CommandError> {
    let request = Request {
        device,
        live,
        model,
        language,
    };
    Ok(start(app, &request)?)
}

/// Stops, saves the recording into the data directory and returns where.
#[tauri::command]
pub fn stop_recording() -> std::result::Result<RecordingResult, CommandError> {
    Ok(save_finished(&Paths::resolve(None))?)
}

/// Stops and throws the audio away (the user left the page mid-recording).
#[tauri::command]
pub fn cancel_recording() {
    let _ = finish();
}

/// Copies a finished recording to a place the user chose.
#[tauri::command]
pub fn save_recording_as(file: &str, destination: &str) -> std::result::Result<(), CommandError> {
    let source = check_recording(&Paths::resolve(None), file)?;
    std::fs::copy(source, destination).map_err(Error::Io)?;
    Ok(())
}

/// Deletes a finished recording the user does not want.
#[tauri::command]
pub fn discard_recording(file: &str) -> std::result::Result<(), CommandError> {
    let source = check_recording(&Paths::resolve(None), file)?;
    std::fs::remove_file(source).map_err(Error::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recordings_get_unique_wav_names_in_the_recordings_folder() {
        let path = recording_path(Path::new("/data/recordings"), 1234);
        assert_eq!(path, Path::new("/data/recordings/recording-1234.wav"));
    }

    #[test]
    fn only_files_in_the_recordings_folder_can_be_touched() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::resolve(Some(dir.path().to_path_buf()));
        let folder = recordings_dir(&paths);
        std::fs::create_dir_all(&folder).unwrap();
        let mine = folder.join("recording-1.wav");
        let other = dir.path().join("secret.txt");
        std::fs::write(&mine, b"x").unwrap();
        std::fs::write(&other, b"x").unwrap();

        assert_eq!(
            check_recording(&paths, &mine.display().to_string()).unwrap(),
            mine
        );
        assert!(check_recording(&paths, &other.display().to_string()).is_err());
        assert!(
            check_recording(&paths, &folder.join("missing.wav").display().to_string()).is_err()
        );
    }

    #[test]
    fn stopping_without_a_recording_is_an_error() {
        assert!(matches!(finish(), Err(Error::Input(_))));
    }
}
