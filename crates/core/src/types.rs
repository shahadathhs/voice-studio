//! Data types shared across the core. The JSON form of [`Transcript`] is the
//! interchange format used by `vtt export`, `--json` output and the app's store.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    /// Display label, e.g. `SPEAKER_00` or a name the user chose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Transcript {
    pub segments: Vec<Segment>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub model: Option<String>,
}
