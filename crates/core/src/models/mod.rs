//! Model catalog, install state and downloads.
//!
//! The catalog ships with the app (`catalog/models.json`). Models are downloaded on demand into the
//! data directory, verified against their SHA-256, and only then count as installed.

mod catalog;
mod download;
mod store;

/// Catalog id of the model that tells speakers apart (see [`crate::diarize`]).
pub const SPEAKER_MODEL_ID: &str = "speaker-identification";

/// Catalog id of the text-to-speech model (see [`crate::tts`]).
pub const SPEECH_MODEL_ID: &str = "kokoro-82m";

/// Catalog id of the language model used for summaries and translation (see [`crate::text`]).
pub const TEXT_MODEL_ID: &str = "qwen2.5-1.5b";

pub use catalog::{Catalog, ModelEntry, ModelFile, ModelKind};
pub use download::{Progress, download_file};
pub use store::{ModelStatus, ModelStore};
