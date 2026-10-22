//! Model catalog, install state and downloads.
//!
//! The catalog ships with the app (`catalog/models.json`). Models are downloaded on demand into the
//! data directory, verified against their SHA-256, and only then count as installed.

mod catalog;
mod download;
mod store;

/// Catalog id of the model that tells speakers apart (see [`crate::diarize`]).
pub const SPEAKER_MODEL_ID: &str = "wespeaker-resnet34";

/// Catalog id of the text-to-speech model (see [`crate::tts`]).
pub const SPEECH_MODEL_ID: &str = "kokoro-82m";

pub use catalog::{Catalog, ModelEntry, ModelFile, ModelKind};
pub use download::{Progress, download_file};
pub use store::{ModelStatus, ModelStore};
