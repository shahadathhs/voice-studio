//! Summaries and translations of a library transcript, written by a language model on a
//! background thread. Progress and the result arrive as events. A translation is stored as a new
//! library item; a summary is shown and can be copied. The loaded model is kept between requests.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};
use vtt_core::engines::Control;
use vtt_core::library::{Library, NewItem, NewNote, Note, NoteKind};
use vtt_core::models::{Catalog, ModelEntry, ModelKind, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::text::{
    QwenEngine, action_items, chapters, chapters_to_json, languages, summarize, translate,
    translate_text,
};
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
    Chapters,
    Actions,
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
    /// The library item a translation was stored as.
    item_id: Option<i64>,
    message: Option<String>,
}

struct Done {
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

/// The loaded engine for `entry`, loading it first if another model was loaded before.
fn engine_for<'a>(
    loaded: &'a mut Option<(String, QwenEngine)>,
    paths: &Paths,
    entry: &ModelEntry,
) -> Result<&'a mut QwenEngine> {
    if loaded.as_ref().is_none_or(|(id, _)| *id != entry.id) {
        let dir = ModelStore::new(paths).model_dir(entry);
        *loaded = Some((entry.id.clone(), QwenEngine::load(&dir, &entry.id)?));
    }
    loaded
        .as_mut()
        .map(|(_, engine)| engine)
        .ok_or_else(|| Error::Engine("no language model loaded".into()))
}

/// Translates text for the Speak screen with the largest installed language model.
pub fn translate_for_speech(
    paths: &Paths,
    text: &str,
    code: &str,
    on_progress: &mut dyn FnMut(f32) -> Control,
) -> Result<String> {
    let catalog = Catalog::bundled()?;
    let store = ModelStore::new(paths);
    let entry = catalog
        .models
        .iter()
        .filter(|m| m.kind == ModelKind::Text && store.is_installed(m))
        .max_by_key(|m| m.size_bytes())
        .ok_or_else(|| Error::ModelMissing("a language model".into()))?;
    let mut loaded = lock(&ENGINE);
    let engine = engine_for(&mut loaded, paths, entry)?;
    translate_text(engine, text, code, on_progress)
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
    let engine = engine_for(&mut loaded, paths, entry)?;
    let language = request.language.as_deref();
    let kind = match request.task {
        Task::Summary => NoteKind::Summary,
        Task::Chapters => NoteKind::Chapters,
        Task::Actions => NoteKind::Actions,
        Task::Translation => return translated(&mut library, &item, engine, request, on_progress),
    };
    let text = match kind {
        NoteKind::Summary => summarize(engine, &transcript, language, on_progress)?,
        NoteKind::Actions => action_items(engine, &transcript, language, on_progress)?,
        NoteKind::Chapters => chapters_to_json(&chapters(engine, &transcript, on_progress)?)?,
    };
    let note = NewNote {
        kind,
        language: if kind == NoteKind::Chapters {
            None
        } else {
            language
        },
        model: &entry.id,
        text: &text,
    };
    library.add_note(request.item_id, &note)?;
    Ok(Done { item_id: None })
}

/// Translates the transcript and stores it as a new library item.
fn translated(
    library: &mut Library,
    item: &vtt_core::library::Item,
    engine: &mut QwenEngine,
    request: &Request,
    on_progress: &mut dyn FnMut(f32) -> Control,
) -> Result<Done> {
    let code = request
        .language
        .as_deref()
        .ok_or_else(|| Error::Input("choose a language".into()))?;
    let (_, transcript) = library.get(request.item_id)?;
    let result = translate(engine, &transcript, code, on_progress)?;
    let name = languages::name(code).unwrap_or(code);
    let title = format!("{} ({name})", item.title);
    let id = library.add(
        &result,
        &NewItem {
            title: &title,
            source_path: item.source_path.as_deref(),
        },
    )?;
    Ok(Done { item_id: Some(id) })
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

/// What was written about a transcript earlier, newest first.
#[tauri::command]
pub fn list_notes(item_id: i64) -> std::result::Result<Vec<Note>, CommandError> {
    Ok(Library::open(&Paths::resolve(None))?.notes(item_id)?)
}

#[tauri::command]
pub fn delete_note(note_id: i64) -> std::result::Result<(), CommandError> {
    Library::open(&Paths::resolve(None))?.delete_note(note_id)?;
    Ok(())
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
