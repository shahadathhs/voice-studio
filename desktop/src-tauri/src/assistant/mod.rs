//! Summaries and translations of a library transcript, written by a language model on a
//! background thread. Progress and the result arrive as events. A translation is stored as a new
//! library item; a summary is shown and can be copied. The loaded model is kept between requests.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};
use vtt_core::engines::Control;
use vtt_core::library::{Library, NewItem};
use vtt_core::models::{Catalog, ModelEntry, ModelKind, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::text::{QwenEngine, languages, summarize, translate};
use vtt_core::{Error, Result};

use crate::commands::CommandError;

const PROGRESS_EVENT: &str = "text-progress";
const FINISHED_EVENT: &str = "text-finished";

static BUSY: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);
/// The model id with the engine loaded for it.
static ENGINE: Mutex<Option<(String, QwenEngine)>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    Summary,
    Translation,
}

/// What to do to which transcript.
#[derive(Debug, Clone)]
pub struct Request {
    pub item_id: i64,
    pub task: Task,
    /// The language to write in; required for a translation.
    pub language: Option<String>,
    pub model: String,
}

/// Mirrored by `TextFinished` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct FinishedEvent {
    outcome: &'static str,
    task: Task,
    /// The summary.
    text: Option<String>,
    /// The library item a translation was stored as.
    item_id: Option<i64>,
    message: Option<String>,
}

struct Done {
    text: Option<String>,
    item_id: Option<i64>,
}

fn lock<T>(mutex: &'static Mutex<T>) -> MutexGuard<'static, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The installed language model the request names.
fn model_entry(paths: &Paths, id: &str) -> Result<ModelEntry> {
    let catalog = Catalog::bundled()?;
    let entry = catalog
        .find(id)
        .filter(|entry| entry.kind == ModelKind::Text)
        .ok_or_else(|| Error::Input(format!("not a language model: {id}")))?;
    if ModelStore::new(paths).is_installed(entry) {
        Ok(entry.clone())
    } else {
        Err(Error::ModelMissing(id.to_string()))
    }
}

fn check(paths: &Paths, request: &Request) -> Result<ModelEntry> {
    if request.task == Task::Translation && request.language.is_none() {
        return Err(Error::Input("choose a language to translate into".into()));
    }
    if let Some(code) = &request.language
        && languages::name(code).is_none()
    {
        return Err(Error::Input(format!("unsupported language: {code}")));
    }
    model_entry(paths, &request.model)
}

fn work(
    paths: &Paths,
    entry: &ModelEntry,
    request: &Request,
    on_progress: &mut dyn FnMut(f32) -> Control,
) -> Result<Done> {
    let mut library = Library::open(paths)?;
    let (item, transcript) = library.get(request.item_id)?;
    let mut loaded = lock(&ENGINE);
    if loaded.as_ref().is_none_or(|(id, _)| *id != entry.id) {
        let dir = ModelStore::new(paths).model_dir(entry);
        *loaded = Some((entry.id.clone(), QwenEngine::load(&dir, &entry.id)?));
    }
    let (_, engine) = loaded
        .as_mut()
        .ok_or_else(|| Error::Engine("no language model loaded".into()))?;
    let language = request.language.as_deref();
    if request.task == Task::Summary {
        let text = summarize(engine, &transcript, language, on_progress)?;
        return Ok(Done {
            text: Some(text),
            item_id: None,
        });
    }
    let code = language.ok_or_else(|| Error::Input("choose a language".into()))?;
    let translated = translate(engine, &transcript, code, on_progress)?;
    let name = languages::name(code).unwrap_or(code);
    let title = format!("{} ({name})", item.title);
    let id = library.add(
        &translated,
        &NewItem {
            title: &title,
            source_path: item.source_path.as_deref(),
        },
    )?;
    Ok(Done {
        text: None,
        item_id: Some(id),
    })
}

fn finished(task: Task, result: Result<Done>) -> FinishedEvent {
    let (outcome, done, message) = match result {
        Ok(done) => ("done", Some(done), None),
        Err(Error::Cancelled) => ("cancelled", None, None),
        Err(error) => ("failed", None, Some(error.to_string())),
    };
    FinishedEvent {
        outcome,
        task,
        text: done.as_ref().and_then(|d| d.text.clone()),
        item_id: done.and_then(|d| d.item_id),
        message,
    }
}

/// Starts the task in the background. Fails at once for a bad request, a model that is not
/// installed, or when another task is running.
pub fn start<R: Runtime>(app: AppHandle<R>, paths: Paths, request: Request) -> Result<()> {
    let entry = check(&paths, &request)?;
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err(Error::Input(
            "another summary or translation is running".into(),
        ));
    }
    CANCEL.store(false, Ordering::SeqCst);
    std::thread::spawn(move || {
        let result = work(&paths, &entry, &request, &mut |fraction| {
            let _ = app.emit(PROGRESS_EVENT, fraction);
            if CANCEL.load(Ordering::SeqCst) {
                Control::Cancel
            } else {
                Control::Continue
            }
        });
        BUSY.store(false, Ordering::SeqCst);
        let _ = app.emit(FINISHED_EVENT, finished(request.task, result));
    });
    Ok(())
}

/// Starts a summary or translation; follow it with `text-progress` and `text-finished`.
#[tauri::command]
pub fn start_text_task(
    app: AppHandle,
    item_id: i64,
    task: Task,
    language: Option<String>,
    model: String,
) -> std::result::Result<(), CommandError> {
    let request = Request {
        item_id,
        task,
        language,
        model,
    };
    Ok(start(app, Paths::resolve(None), request)?)
}

/// The language codes a summary or translation can be written in.
#[tauri::command]
pub fn text_languages() -> Vec<&'static str> {
    languages::codes().collect()
}

#[tauri::command]
pub fn cancel_text_task() {
    CANCEL.store(true, Ordering::SeqCst);
}

#[cfg(test)]
mod tests;
