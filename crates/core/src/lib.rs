//! Core engine for Voice Studio: shared by the `vtt` CLI and the desktop app.
//!
//! Nothing in here knows about a UI. Front ends call these modules directly, in-process.

pub mod audio;
pub mod engines;
pub mod error;
pub mod export;
pub mod jobs;
pub mod models;
pub mod paths;
#[cfg(feature = "testing")]
pub mod testing;
pub mod transcribe;
pub mod types;

/// Version of the core crate, shown in the app's About section.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::{Error, Result};
pub use types::{Segment, Transcript};
