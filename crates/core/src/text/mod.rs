//! Language models: summaries and translation of transcripts. Runs on this computer with a small
//! Qwen model (see [`QwenEngine`]); the instructions given to it are in [`tasks`].

pub mod languages;

mod chunks;
#[cfg(test)]
mod fake;
mod outline;
mod qwen;
mod steps;
mod tasks;

pub use outline::{Chapter, action_items, chapters, chapters_from_json, chapters_to_json};
pub use qwen::QwenEngine;
pub use tasks::{summarize, translate};
