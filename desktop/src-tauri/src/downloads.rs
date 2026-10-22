//! Model installs started from the UI. Each install runs on its own thread and reports through
//! events; a shared flag per install lets the UI cancel it.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use vtt_core::engines::Control;
use vtt_core::jobs::Throttle;
use vtt_core::models::{ModelEntry, ModelStore};
use vtt_core::{Error, Result};

const PROGRESS_EVENT: &str = "model-progress";
const FINISHED_EVENT: &str = "model-finished";
const PROGRESS_EVERY: Duration = Duration::from_millis(100);

type CancelFlags = Mutex<HashMap<String, Arc<AtomicBool>>>;
static RUNNING: LazyLock<CancelFlags> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Mirrored by `ProgressEvent` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct ProgressEvent<'a> {
    id: &'a str,
    downloaded: u64,
    total: u64,
}

/// Mirrored by `FinishedEvent` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct FinishedEvent<'a> {
    id: &'a str,
    outcome: &'static str,
    message: Option<String>,
}

fn lock() -> std::sync::MutexGuard<'static, HashMap<String, Arc<AtomicBool>>> {
    RUNNING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Starts installing `entry` in the background. Fails if that model is already being installed.
pub fn start<R: Runtime>(app: AppHandle<R>, entry: ModelEntry, store: ModelStore) -> Result<()> {
    let cancel = Arc::new(AtomicBool::new(false));
    if lock()
        .insert(entry.id.clone(), Arc::clone(&cancel))
        .is_some()
    {
        return Err(Error::Input(format!(
            "{} is already being installed",
            entry.id
        )));
    }
    std::thread::spawn(move || {
        let mut throttle = Throttle::new(PROGRESS_EVERY);
        let result = store.install(&entry, &mut |p| {
            if cancel.load(Ordering::Relaxed) {
                return Control::Cancel;
            }
            if throttle.ready() || p.downloaded == p.total {
                let event = ProgressEvent {
                    id: &entry.id,
                    downloaded: p.downloaded,
                    total: p.total,
                };
                let _ = app.emit(PROGRESS_EVENT, event);
            }
            Control::Continue
        });
        lock().remove(&entry.id);
        let _ = app.emit(FINISHED_EVENT, finished(&entry.id, &result));
    });
    Ok(())
}

fn finished<'a>(id: &'a str, result: &Result<()>) -> FinishedEvent<'a> {
    match result {
        Ok(()) => FinishedEvent {
            id,
            outcome: "installed",
            message: None,
        },
        Err(Error::Cancelled) => FinishedEvent {
            id,
            outcome: "cancelled",
            message: None,
        },
        Err(error) => FinishedEvent {
            id,
            outcome: "failed",
            message: Some(error.to_string()),
        },
    }
}

/// Asks a running install to stop. The partial download is kept, so installing again resumes.
pub fn cancel(id: &str) {
    if let Some(flag) = lock().get(id) {
        flag.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{self, Receiver};

    use tauri::Listener;
    use vtt_core::models::{ModelFile, ModelKind};
    use vtt_core::paths::Paths;
    use vtt_core::testing::{TestServer, file_for, sample_data};

    use super::*;

    fn entry_with(id: &str, file: ModelFile) -> ModelEntry {
        ModelEntry {
            id: id.into(),
            name: id.into(),
            kind: ModelKind::Stt,
            description: String::new(),
            languages: vec![],
            license: "MIT".into(),
            files: vec![file],
        }
    }

    /// Starts an install on a mock app and returns the `(event name, JSON payload)` stream.
    fn run_install(entry: &ModelEntry, store: &ModelStore) -> Receiver<(&'static str, String)> {
        let app = tauri::test::mock_app();
        let (tx, rx) = mpsc::channel();
        for name in [PROGRESS_EVENT, FINISHED_EVENT] {
            let tx = tx.clone();
            app.listen(name, move |event| {
                let _ = tx.send((name, event.payload().to_string()));
            });
        }
        start(app.handle().clone(), entry.clone(), store.clone()).unwrap();
        // Keep the app alive until the install thread is done with it.
        std::mem::forget(app);
        rx
    }

    fn events_until_finished(rx: &Receiver<(&'static str, String)>) -> Vec<(&'static str, String)> {
        let mut events = Vec::new();
        while let Ok(event) = rx.recv_timeout(Duration::from_secs(10)) {
            let finished = event.0 == FINISHED_EVENT;
            events.push(event);
            if finished {
                break;
            }
        }
        events
    }

    #[test]
    fn an_install_reports_progress_then_finishes_with_the_model_installed() {
        let data = sample_data();
        let server = TestServer::start(data.clone());
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::new(&Paths::resolve(Some(dir.path().to_path_buf())));
        let entry = entry_with("e2e-ok", file_for(&server, &data));

        let events = events_until_finished(&run_install(&entry, &store));

        let (last_name, last_payload) = events.last().unwrap();
        assert_eq!(*last_name, FINISHED_EVENT);
        let finished: serde_json::Value = serde_json::from_str(last_payload).unwrap();
        assert_eq!(finished["id"], "e2e-ok");
        assert_eq!(finished["outcome"], "installed");
        let total = data.len() as u64;
        let progress: Vec<serde_json::Value> = events
            .iter()
            .filter(|(name, _)| *name == PROGRESS_EVENT)
            .map(|(_, payload)| serde_json::from_str(payload).unwrap())
            .collect();
        assert!(
            progress
                .iter()
                .any(|p| p["downloaded"] == total && p["total"] == total)
        );
        assert!(store.is_installed(&entry));
    }

    #[test]
    fn a_bad_checksum_finishes_as_failed_with_a_message() {
        let data = sample_data();
        let server = TestServer::start(data.clone());
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::new(&Paths::resolve(Some(dir.path().to_path_buf())));
        let mut file = file_for(&server, &data);
        file.sha256 = "0".repeat(64);
        let entry = entry_with("e2e-bad", file);

        let events = events_until_finished(&run_install(&entry, &store));

        let finished: serde_json::Value = serde_json::from_str(&events.last().unwrap().1).unwrap();
        assert_eq!(finished["outcome"], "failed");
        assert!(finished["message"].as_str().unwrap().contains("checksum"));
        assert!(!store.is_installed(&entry));
    }

    #[test]
    fn cancelling_sets_the_flag_of_a_running_install_only() {
        let flag = Arc::new(AtomicBool::new(false));
        lock().insert("running-model".into(), Arc::clone(&flag));
        cancel("some-other-model");
        assert!(!flag.load(Ordering::Relaxed));
        cancel("running-model");
        assert!(flag.load(Ordering::Relaxed));
        lock().remove("running-model");
    }

    #[test]
    fn outcomes_map_to_the_strings_the_ui_expects() {
        assert_eq!(finished("m", &Ok(())).outcome, "installed");
        assert_eq!(finished("m", &Err(Error::Cancelled)).outcome, "cancelled");
        let failed = finished("m", &Err(Error::Network("offline".into())));
        assert_eq!(failed.outcome, "failed");
        assert_eq!(failed.message.as_deref(), Some("offline"));
    }
}
