//! Model catalog, install state and downloads.
//!
//! The catalog ships with the app (`catalog/models.json`). Models are downloaded on demand into the
//! data directory, verified against their SHA-256, and only then count as installed.

mod catalog;
mod download;
mod store;

pub use catalog::{Catalog, ModelEntry, ModelFile, ModelKind};
pub use download::{Progress, download_file};
pub use store::{ModelStatus, ModelStore};
