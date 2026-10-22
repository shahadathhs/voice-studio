//! Playing back the original recording of a library item. The webview reads the file through
//! Tauri's asset protocol, which only serves files that were explicitly allowed; only the file of
//! the transcript being viewed is ever allowed.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri::{AppHandle, Manager};
use vtt_core::library::Library;
use vtt_core::paths::Paths;
use vtt_core::{Error, Result};

use crate::commands::CommandError;

static APP: OnceLock<AppHandle> = OnceLock::new();

/// Remembers the app so commands can change the asset scope. Called once at setup.
pub fn init(app: &AppHandle) {
    let _ = APP.set(app.clone());
}

/// The recording a library item was made from, if it is still on disk.
fn source_file(library: &Library, id: i64) -> Result<PathBuf> {
    let (item, _) = library.get(id)?;
    let path = item
        .source_path
        .map(PathBuf::from)
        .ok_or_else(|| Error::Input("this transcript has no original recording".into()))?;
    if path.is_file() {
        Ok(path)
    } else {
        Err(Error::Input(format!(
            "the original recording was moved or deleted: {}",
            path.display()
        )))
    }
}

fn allow(path: &Path) -> Result<()> {
    let app = APP
        .get()
        .ok_or_else(|| Error::Engine("the app is not ready".into()))?;
    app.asset_protocol_scope()
        .allow_file(path)
        .map_err(|e| Error::Engine(e.to_string()))
}

/// Allows the webview to play the recording of item `id` and returns its path, which the UI turns
/// into a playable URL. Fails with a readable message when the file is gone.
#[tauri::command]
pub fn prepare_playback(id: i64) -> std::result::Result<String, CommandError> {
    let library = Library::open(&Paths::resolve(None))?;
    let path = source_file(&library, id)?;
    allow(&path)?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use vtt_core::Transcript;
    use vtt_core::library::NewItem;

    use super::*;

    fn library_with(source: Option<&str>) -> (Library, i64) {
        let dir = tempfile::tempdir().unwrap();
        let mut library = Library::open(&Paths::resolve(Some(dir.keep()))).unwrap();
        let new = NewItem {
            title: "t",
            source_path: source,
        };
        let id = library.add(&Transcript::default(), &new).unwrap();
        (library, id)
    }

    #[test]
    fn an_existing_recording_is_found() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().display().to_string();
        let (library, id) = library_with(Some(&path));
        assert_eq!(source_file(&library, id).unwrap(), file.path());
    }

    #[test]
    fn a_moved_recording_is_reported_with_its_path() {
        let (library, id) = library_with(Some("/no/such/recording.wav"));
        let error = source_file(&library, id).unwrap_err().to_string();
        assert!(error.contains("moved or deleted"), "{error}");
        assert!(error.contains("/no/such/recording.wav"), "{error}");
    }

    #[test]
    fn an_item_without_a_source_has_nothing_to_play() {
        let (library, id) = library_with(None);
        assert!(source_file(&library, id).is_err());
    }
}
