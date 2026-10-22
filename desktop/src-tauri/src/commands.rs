//! Commands callable from the UI. Keep them thin; behavior belongs in `vtt-core`.

use serde::Serialize;
use tauri::AppHandle;
use vtt_core::models::{Catalog, ModelEntry, ModelStatus, ModelStore};
use vtt_core::paths::Paths;

use crate::downloads;

/// Mirrored by `AppInfo` in `desktop/src/lib/types.ts`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    core_version: &'static str,
    data_dir: String,
}

/// Failure sent to the UI as a plain message.
#[derive(Debug)]
pub struct CommandError(String);

impl Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl From<vtt_core::Error> for CommandError {
    fn from(error: vtt_core::Error) -> Self {
        CommandError(error.to_string())
    }
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Voice Studio",
        version: env!("CARGO_PKG_VERSION"),
        core_version: vtt_core::VERSION,
        data_dir: Paths::resolve(None).data.display().to_string(),
    }
}

fn store() -> ModelStore {
    ModelStore::new(&Paths::resolve(None))
}

fn entry(id: &str) -> Result<ModelEntry, CommandError> {
    Catalog::bundled()?
        .find(id)
        .cloned()
        .ok_or_else(|| vtt_core::Error::Input(format!("unknown model: {id}")).into())
}

#[tauri::command]
pub fn list_models() -> Result<Vec<ModelStatus>, CommandError> {
    let store = store();
    Ok(Catalog::bundled()?
        .models
        .iter()
        .map(|m| store.status(m))
        .collect())
}

/// Starts a background install; progress and the result arrive as `model-progress` and
/// `model-finished` events.
#[tauri::command]
pub fn install_model(app: AppHandle, id: &str) -> Result<(), CommandError> {
    Ok(downloads::start(app, entry(id)?, store())?)
}

#[tauri::command]
pub fn cancel_install(id: &str) {
    downloads::cancel(id);
}

#[tauri::command]
pub fn remove_model(id: &str) -> Result<(), CommandError> {
    Ok(store().remove(&entry(id)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_reports_versions_and_a_data_dir() {
        let info = app_info();
        assert_eq!(info.name, "Voice Studio");
        assert_ne!(info.version, "");
        assert_eq!(info.core_version, vtt_core::VERSION);
        assert_ne!(info.data_dir, "");
    }

    #[test]
    fn list_models_returns_every_catalog_entry() {
        let models = list_models().unwrap();
        assert_eq!(models.len(), Catalog::bundled().unwrap().models.len());
        assert!(models.iter().any(|m| m.id == "whisper-base"));
    }

    #[test]
    fn unknown_models_are_rejected_before_anything_starts() {
        let error = entry("no-such-model").unwrap_err();
        assert!(error.0.contains("unknown model"));
        assert!(remove_model("no-such-model").is_err());
    }

    #[test]
    fn core_errors_become_plain_messages() {
        let error = CommandError::from(vtt_core::Error::Input("bad file".into()));
        assert_eq!(serde_json::to_string(&error).unwrap(), "\"bad file\"");
    }
}
