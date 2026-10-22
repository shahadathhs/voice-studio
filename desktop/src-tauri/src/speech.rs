//! Speech generation for the Speak screen. One piece of text is spoken at a time on a background
//! thread; progress and the result arrive as events, and the finished WAV is made playable by the
//! webview (see `playback`). The loaded model is kept between requests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use vtt_core::audio::encode_wav;
use vtt_core::engines::{Control, TtsOptions};
use vtt_core::models::{Catalog, ModelEntry, ModelStore, SPEECH_MODEL_ID};
use vtt_core::paths::Paths;
use vtt_core::tts::{KokoroEngine, SAMPLE_RATE, VoiceInfo, describe};
use vtt_core::{Error, Result};

use crate::commands::CommandError;

const PROGRESS_EVENT: &str = "speech-progress";
const FINISHED_EVENT: &str = "speech-finished";
const SPEECH_DIR: &str = "speech";
const FILE_PREFIX: &str = "speech-";

static BUSY: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);
static ENGINE: Mutex<Option<KokoroEngine>> = Mutex::new(None);
static LATEST: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Mirrored by `SpeechProgress` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct ProgressEvent {
    done: usize,
    total: usize,
}

/// Mirrored by `SpeechFinished` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct FinishedEvent {
    outcome: &'static str,
    path: Option<String>,
    duration_ms: u64,
    message: Option<String>,
}

/// What to say and how.
pub struct Request {
    pub text: String,
    pub voice: String,
    pub speed: f32,
}

fn lock<T>(mutex: &'static Mutex<T>) -> MutexGuard<'static, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn speech_entry(store: &ModelStore) -> Result<ModelEntry> {
    let catalog = Catalog::bundled()?;
    let entry = catalog
        .find(SPEECH_MODEL_ID)
        .ok_or_else(|| Error::ModelMissing(SPEECH_MODEL_ID.into()))?;
    if store.is_installed(entry) {
        Ok(entry.clone())
    } else {
        Err(Error::ModelMissing(SPEECH_MODEL_ID.into()))
    }
}

/// The installed voices; empty when the voice model is not installed.
pub fn voices(paths: &Paths) -> Vec<VoiceInfo> {
    let store = ModelStore::new(paths);
    match speech_entry(&store) {
        Ok(entry) => KokoroEngine::voices_in(&store.model_dir(&entry))
            .iter()
            .map(|id| describe(id))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Deletes earlier speech files so the folder never holds more than the latest one.
fn keep_only(dir: &Path, keep: &Path) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let is_speech = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(FILE_PREFIX));
        if is_speech && path != keep {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Speaks the request into a new WAV in the data directory and returns its path and length.
fn synthesize(
    paths: &Paths,
    request: &Request,
    on_progress: &mut dyn FnMut(usize, usize) -> Control,
) -> Result<(PathBuf, u64)> {
    let store = ModelStore::new(paths);
    let entry = speech_entry(&store)?;
    let mut engine = lock(&ENGINE);
    if engine.is_none() {
        *engine = Some(KokoroEngine::load(&store.model_dir(&entry), &entry.id)?);
    }
    let engine = engine
        .as_mut()
        .ok_or_else(|| Error::Engine("no voice model loaded".into()))?;
    let options = TtsOptions {
        voice: request.voice.clone(),
        speed: request.speed,
    };
    let audio = engine.speak(&request.text, &options, on_progress)?;
    let dir = paths.data.join(SPEECH_DIR);
    std::fs::create_dir_all(&dir)?;
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let path = dir.join(format!("{FILE_PREFIX}{millis}.wav"));
    std::fs::write(&path, encode_wav(&audio))?;
    keep_only(&dir, &path);
    let duration_ms =
        u64::try_from(audio.samples.len()).unwrap_or(u64::MAX) * 1000 / u64::from(SAMPLE_RATE);
    Ok((path, duration_ms))
}

fn finished(result: &Result<(PathBuf, u64)>) -> FinishedEvent {
    match result {
        Ok((path, duration_ms)) => FinishedEvent {
            outcome: "done",
            path: Some(path.display().to_string()),
            duration_ms: *duration_ms,
            message: None,
        },
        Err(Error::Cancelled) => FinishedEvent {
            outcome: "cancelled",
            path: None,
            duration_ms: 0,
            message: None,
        },
        Err(error) => FinishedEvent {
            outcome: "failed",
            path: None,
            duration_ms: 0,
            message: Some(error.to_string()),
        },
    }
}

/// Starts speaking in the background. Fails at once when there is nothing to say, the voice model
/// is not installed, or something is already being spoken.
pub fn start<R: Runtime>(app: AppHandle<R>, paths: Paths, request: Request) -> Result<()> {
    if request.text.trim().is_empty() {
        return Err(Error::Input("there is no text to speak".into()));
    }
    speech_entry(&ModelStore::new(&paths))?;
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err(Error::Input("something is already being spoken".into()));
    }
    CANCEL.store(false, Ordering::SeqCst);
    std::thread::spawn(move || {
        let result = synthesize(&paths, &request, &mut |done, total| {
            let _ = app.emit(PROGRESS_EVENT, ProgressEvent { done, total });
            if CANCEL.load(Ordering::SeqCst) {
                Control::Cancel
            } else {
                Control::Continue
            }
        });
        let event = match result.as_ref().map(|(path, _)| allow(&app, path)) {
            Ok(Err(error)) => finished(&Err(error)),
            _ => finished(&result),
        };
        *lock(&LATEST) = result.ok().map(|(path, _)| path);
        BUSY.store(false, Ordering::SeqCst);
        let _ = app.emit(FINISHED_EVENT, event);
    });
    Ok(())
}

/// Lets the webview play `path` through the asset protocol.
fn allow<R: Runtime>(app: &AppHandle<R>, path: &Path) -> Result<()> {
    app.asset_protocol_scope()
        .allow_file(path)
        .map_err(|e| Error::Engine(e.to_string()))
}

/// Copies the latest speech to `destination`.
fn save(destination: &Path) -> Result<()> {
    let source = lock(&LATEST)
        .clone()
        .ok_or_else(|| Error::Input("nothing has been spoken yet".into()))?;
    std::fs::copy(source, destination)?;
    Ok(())
}

#[tauri::command]
pub fn list_voices() -> Vec<VoiceInfo> {
    voices(&Paths::resolve(None))
}

/// Starts speaking; follow it with the `speech-progress` and `speech-finished` events.
#[tauri::command]
pub fn start_speech(
    app: AppHandle,
    text: String,
    voice: &str,
    speed: f32,
) -> std::result::Result<(), CommandError> {
    let request = Request {
        text,
        voice: voice.to_string(),
        speed,
    };
    Ok(start(app, Paths::resolve(None), request)?)
}

#[tauri::command]
pub fn cancel_speech() {
    CANCEL.store(true, Ordering::SeqCst);
}

/// The most text a file may hold to be read into the text box.
const MAX_TEXT_BYTES: u64 = 2_000_000;

/// Reads a plain-text file for the Speak screen's text box.
fn read_text(path: &Path) -> Result<String> {
    let size = std::fs::metadata(path)
        .map_err(|e| Error::Input(format!("cannot read {}: {e}", path.display())))?
        .len();
    if size > MAX_TEXT_BYTES {
        return Err(Error::Input(format!(
            "{} is too large to read aloud in one go",
            path.display()
        )));
    }
    std::fs::read_to_string(path)
        .map_err(|e| Error::Input(format!("cannot read {} as text: {e}", path.display())))
}

#[tauri::command]
pub fn read_text_file(path: &str) -> std::result::Result<String, CommandError> {
    Ok(read_text(Path::new(path))?)
}

/// Saves the latest speech to a file the user chose.
#[tauri::command]
pub fn save_speech(path: &str) -> std::result::Result<(), CommandError> {
    Ok(save(Path::new(path))?)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// The statics are shared by every test, so tests that touch them run one at a time.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn serial() -> MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn request(text: &str) -> Request {
        Request {
            text: text.into(),
            voice: "af_heart".into(),
            speed: 1.0,
        }
    }

    /// Makes the speech model count as installed without downloading it: files of the right size.
    fn fake_install(paths: &Paths) {
        let store = ModelStore::new(paths);
        let catalog = Catalog::bundled().unwrap();
        let entry = catalog.find(SPEECH_MODEL_ID).unwrap();
        std::fs::create_dir_all(store.model_dir(entry)).unwrap();
        for file in &entry.files {
            let handle = std::fs::File::create(store.file_path(entry, file)).unwrap();
            handle.set_len(file.size_bytes).unwrap();
        }
    }

    #[test]
    fn nothing_to_say_and_a_missing_model_are_rejected_before_anything_starts() {
        let _guard = serial();
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::resolve(Some(dir.path().to_path_buf()));
        let app = tauri::test::mock_app();

        let empty = start(app.handle().clone(), paths.clone(), request("   "));
        assert!(matches!(empty, Err(Error::Input(_))));
        let missing = start(app.handle().clone(), paths, request("Hello"));
        assert!(matches!(missing, Err(Error::ModelMissing(id)) if id == SPEECH_MODEL_ID));
        assert!(!BUSY.load(Ordering::SeqCst));
    }

    #[test]
    fn a_second_request_while_one_is_running_is_refused() {
        let _guard = serial();
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::resolve(Some(dir.path().to_path_buf()));
        fake_install(&paths);
        let app = tauri::test::mock_app();

        BUSY.store(true, Ordering::SeqCst);
        let result = start(app.handle().clone(), paths, request("Hello"));
        BUSY.store(false, Ordering::SeqCst);

        assert!(matches!(result, Err(Error::Input(m)) if m.contains("already")));
    }

    #[test]
    fn voices_are_listed_only_when_the_model_is_installed() {
        let _guard = serial();
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::resolve(Some(dir.path().to_path_buf()));
        assert_eq!(voices(&paths).len(), 0);

        fake_install(&paths);
        let found = voices(&paths);
        assert!(
            found
                .iter()
                .any(|v| v.id == "af_heart" && v.name == "Heart")
        );
        assert!(found.iter().all(|v| !v.description.is_empty()));
    }

    #[test]
    fn only_the_newest_speech_file_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["speech-1.wav", "speech-2.wav", "notes.txt"] {
            std::fs::write(dir.path().join(name), b"x").unwrap();
        }
        keep_only(dir.path(), &dir.path().join("speech-2.wav"));
        let mut left: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, ["notes.txt", "speech-2.wav"]);
    }

    #[test]
    fn outcomes_map_to_the_strings_the_ui_expects() {
        let done = finished(&Ok((PathBuf::from("/a/b.wav"), 1500)));
        assert_eq!(
            (done.outcome, done.path.as_deref()),
            ("done", Some("/a/b.wav"))
        );
        assert_eq!(finished(&Err(Error::Cancelled)).outcome, "cancelled");
        let failed = finished(&Err(Error::Engine("boom".into())));
        assert_eq!(
            (failed.outcome, failed.message.as_deref()),
            ("failed", Some("boom"))
        );
    }

    #[test]
    fn text_files_are_read_and_unreadable_or_huge_ones_are_input_errors() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "Hello there").unwrap();
        assert_eq!(read_text(&file).unwrap(), "Hello there");
        assert!(matches!(
            read_text(&dir.path().join("none.txt")),
            Err(Error::Input(_))
        ));
        std::fs::write(dir.path().join("bin"), [0xff, 0xfe, 0xfd]).unwrap();
        assert!(matches!(
            read_text(&dir.path().join("bin")),
            Err(Error::Input(_))
        ));
        let big = std::fs::File::create(dir.path().join("big.txt")).unwrap();
        big.set_len(MAX_TEXT_BYTES + 1).unwrap();
        assert!(
            matches!(read_text(&dir.path().join("big.txt")), Err(Error::Input(m)) if m.contains("too large"))
        );
    }

    #[test]
    fn saving_before_anything_was_spoken_is_an_input_error() {
        let _guard = serial();
        *lock(&LATEST) = None;
        assert!(matches!(
            save(Path::new("/tmp/x.wav")),
            Err(Error::Input(_))
        ));
    }

    #[test]
    fn the_latest_speech_can_be_copied_to_a_chosen_file() {
        let _guard = serial();
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("speech-1.wav");
        std::fs::write(&source, b"audio").unwrap();
        *lock(&LATEST) = Some(source);
        let target = dir.path().join("saved.wav");

        save(&target).unwrap();

        assert_eq!(std::fs::read(target).unwrap(), b"audio");
        *lock(&LATEST) = None;
    }

    /// Needs the real voice model: `VTT_TEST_DATA_DIR=<data dir with kokoro-82m installed>`.
    #[test]
    #[ignore = "needs VTT_TEST_DATA_DIR with kokoro-82m installed"]
    fn speaking_in_the_background_reports_progress_and_writes_a_wav() {
        use std::sync::mpsc;
        use std::time::Duration;

        use tauri::Listener;

        let _guard = serial();
        let data = std::env::var_os("VTT_TEST_DATA_DIR")
            .map(PathBuf::from)
            .unwrap();
        let app = tauri::test::mock_app();
        let (tx, rx) = mpsc::channel();
        for name in [PROGRESS_EVENT, FINISHED_EVENT] {
            let tx = tx.clone();
            app.listen(name, move |event| {
                let _ = tx.send((name, event.payload().to_string()));
            });
        }

        start(
            app.handle().clone(),
            Paths::resolve(Some(data)),
            request("First sentence. Second sentence."),
        )
        .unwrap();
        let mut events = Vec::new();
        while let Ok(event) = rx.recv_timeout(Duration::from_secs(60)) {
            let done = event.0 == FINISHED_EVENT;
            events.push(event);
            if done {
                break;
            }
        }

        let progress = events
            .iter()
            .filter(|(name, _)| *name == PROGRESS_EVENT)
            .count();
        assert_eq!(progress, 2, "{events:?}");
        let last: serde_json::Value = serde_json::from_str(&events.last().unwrap().1).unwrap();
        assert_eq!(last["outcome"], "done", "{last}");
        let path = last["path"].as_str().unwrap();
        assert_eq!(&std::fs::read(path).unwrap()[0..4], b"RIFF");
        assert!(last["duration_ms"].as_u64().unwrap() > 1000);
        assert!(!BUSY.load(Ordering::SeqCst));
    }
}
