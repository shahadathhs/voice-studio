//! The transcription queue as the UI sees it: one queue for the whole app, started with the window
//! and reporting every change as a `job-updated` event.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use tauri::{AppHandle, Emitter};
use vtt_core::Error;
use vtt_core::engines::{Device, DiarizeOptions};
use vtt_core::jobs::{JobInfo, JobSpec, Queue, WhisperRunner};
use vtt_core::models::{Catalog, ModelEntry, ModelStore, SPEAKER_MODEL_ID};
use vtt_core::paths::Paths;

use crate::commands::CommandError;

const UPDATED_EVENT: &str = "job-updated";

static QUEUE: OnceLock<Queue> = OnceLock::new();

/// Starts the queue worker. Called once when the app is set up.
pub fn start(app: &AppHandle) {
    QUEUE.get_or_init(|| {
        let paths = Paths::resolve(None);
        let handle = app.clone();
        let notify = Arc::new(move |info: &JobInfo| {
            let _ = handle.emit(UPDATED_EVENT, info);
        });
        Queue::start(WhisperRunner::new(&paths, Device::Auto), paths, notify)
    });
}

fn queue() -> vtt_core::Result<&'static Queue> {
    QUEUE
        .get()
        .ok_or_else(|| Error::Engine("the job queue is not running".into()))
}

/// Files to transcribe with one set of settings.
struct Batch<'a> {
    files: Vec<String>,
    model: &'a str,
    language: Option<String>,
    translate: bool,
    identify_speakers: bool,
}

/// Checks that a model can be used before anything is queued, so the UI hears about a missing or
/// unsuitable model now instead of from a failed job later.
fn check_model(store: &ModelStore, id: &str) -> vtt_core::Result<ModelEntry> {
    let catalog = Catalog::bundled()?;
    let entry = catalog
        .find(id)
        .ok_or_else(|| Error::Input(format!("unknown model: {id}")))?;
    if store.is_installed(entry) {
        Ok(entry.clone())
    } else {
        Err(Error::ModelMissing(id.to_string()))
    }
}

fn check_batch(store: &ModelStore, batch: &Batch<'_>) -> vtt_core::Result<()> {
    if let Some(file) = batch.files.iter().find(|f| !PathBuf::from(f).is_file()) {
        return Err(Error::Input(format!("cannot open {file}: no such file")));
    }
    let model = check_model(store, batch.model)?;
    if batch.translate && !model.can_translate {
        return Err(Error::Input(format!(
            "{} cannot translate; choose another model",
            model.name
        )));
    }
    if batch.identify_speakers {
        check_model(store, SPEAKER_MODEL_ID)?;
    }
    Ok(())
}

/// Queues every file, or none if any file or model is unusable.
fn add_all(queue: &Queue, store: &ModelStore, batch: Batch<'_>) -> vtt_core::Result<Vec<JobInfo>> {
    check_batch(store, &batch)?;
    Ok(batch
        .files
        .into_iter()
        .map(|file| {
            queue.add(JobSpec {
                file: file.into(),
                model_id: batch.model.to_string(),
                language: batch.language.clone(),
                translate: batch.translate,
                speakers: batch.identify_speakers.then(DiarizeOptions::default),
            })
        })
        .collect())
}

#[tauri::command]
pub fn list_jobs() -> Result<Vec<JobInfo>, CommandError> {
    Ok(queue()?.snapshot())
}

#[tauri::command]
pub fn add_jobs(
    files: Vec<String>,
    model: &str,
    language: Option<String>,
    translate: bool,
    identify_speakers: bool,
) -> Result<Vec<JobInfo>, CommandError> {
    let store = ModelStore::new(&Paths::resolve(None));
    let batch = Batch {
        files,
        model,
        language,
        translate,
        identify_speakers,
    };
    Ok(add_all(queue()?, &store, batch)?)
}

/// A waiting job is cancelled at once and announced; a running one stops shortly after.
#[tauri::command]
pub fn cancel_job(id: u64) -> Result<(), CommandError> {
    queue()?.cancel(id);
    Ok(())
}

#[tauri::command]
pub fn remove_job(id: u64) -> Result<bool, CommandError> {
    Ok(queue()?.remove(id))
}

#[tauri::command]
pub fn clear_finished_jobs() -> Result<(), CommandError> {
    queue()?.clear_finished();
    Ok(())
}

#[tauri::command]
pub fn move_job(id: u64, position: usize) -> Result<bool, CommandError> {
    Ok(queue()?.move_to(id, position))
}

#[tauri::command]
pub fn set_queue_paused(paused: bool) -> Result<(), CommandError> {
    queue()?.set_paused(paused);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use vtt_core::jobs::JobState;

    use super::*;

    fn idle_queue(dir: &std::path::Path) -> (Queue, ModelStore) {
        let paths = Paths::resolve(Some(dir.to_path_buf()));
        let runner = WhisperRunner::new(&paths, Device::Cpu);
        let queue = Queue::start(runner, paths.clone(), Arc::new(|_| {}));
        queue.set_paused(true);
        (queue, ModelStore::new(&paths))
    }

    fn batch(files: Vec<String>) -> Batch<'static> {
        Batch {
            files,
            model: "whisper-base",
            language: None,
            translate: false,
            identify_speakers: false,
        }
    }

    #[test]
    fn a_missing_file_is_rejected_before_anything_is_queued() {
        let dir = tempfile::tempdir().unwrap();
        let (queue, store) = idle_queue(dir.path());
        let result = add_all(&queue, &store, batch(vec!["/no/such.wav".into()]));
        assert!(matches!(result, Err(Error::Input(_))));
        assert_eq!(queue.snapshot().len(), 0);
    }

    #[test]
    fn a_model_that_is_not_installed_is_rejected_with_its_id() {
        let dir = tempfile::tempdir().unwrap();
        let (queue, store) = idle_queue(dir.path());
        let file = dir.path().join("a.wav");
        std::fs::write(&file, b"x").unwrap();
        let files = vec![file.display().to_string()];

        let result = add_all(&queue, &store, batch(files));

        assert!(matches!(result, Err(Error::ModelMissing(id)) if id == "whisper-base"));
        assert_eq!(queue.snapshot().len(), 0);
    }

    #[test]
    fn unknown_models_are_an_input_error() {
        let dir = tempfile::tempdir().unwrap();
        let (_, store) = idle_queue(dir.path());
        let error = check_model(&store, "nope").unwrap_err();
        assert!(matches!(error, Error::Input(_)));
    }

    /// Makes `id` count as installed without downloading it: files of the right size, all zeros.
    fn fake_install(store: &ModelStore, id: &str) {
        let catalog = Catalog::bundled().unwrap();
        let entry = catalog.find(id).unwrap();
        std::fs::create_dir_all(store.model_dir(entry)).unwrap();
        for file in &entry.files {
            let path = store.file_path(entry, file);
            std::fs::File::create(path)
                .unwrap()
                .set_len(file.size_bytes)
                .unwrap();
        }
    }

    fn audio_file(dir: &std::path::Path) -> String {
        let file = dir.join("a.wav");
        std::fs::write(&file, b"x").unwrap();
        file.display().to_string()
    }

    #[test]
    fn translating_with_a_model_that_cannot_translate_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (queue, store) = idle_queue(dir.path());
        fake_install(&store, "whisper-large-v3-turbo");
        let mut batch = batch(vec![audio_file(dir.path())]);
        batch.model = "whisper-large-v3-turbo";
        batch.translate = true;

        let error = add_all(&queue, &store, batch).unwrap_err().to_string();

        assert!(error.contains("cannot translate"), "{error}");
        assert_eq!(queue.snapshot().len(), 0);
    }

    #[test]
    fn identifying_speakers_needs_the_speaker_model_to_be_installed() {
        let dir = tempfile::tempdir().unwrap();
        let (queue, store) = idle_queue(dir.path());
        fake_install(&store, "whisper-base");
        let mut batch = batch(vec![audio_file(dir.path())]);
        batch.identify_speakers = true;

        let result = add_all(&queue, &store, batch);
        assert!(matches!(result, Err(Error::ModelMissing(id)) if id == SPEAKER_MODEL_ID));

        fake_install(&store, SPEAKER_MODEL_ID);
        let mut batch = self::batch(vec![audio_file(dir.path())]);
        batch.identify_speakers = true;
        let jobs = add_all(&queue, &store, batch).unwrap();
        assert_eq!(jobs.len(), 1);
    }

    #[test]
    fn valid_jobs_are_queued_in_order_while_the_queue_is_paused() {
        let dir = tempfile::tempdir().unwrap();
        let (queue, _) = idle_queue(dir.path());
        let first = queue.add(JobSpec {
            file: "a.wav".into(),
            model_id: "whisper-base".into(),
            language: None,
            translate: false,
            speakers: None,
        });
        assert_eq!(first.state, JobState::Waiting);
        assert_eq!(queue.snapshot().len(), 1);
    }
}
