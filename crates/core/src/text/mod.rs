//! Language models: summaries and translation of transcripts. Runs on this computer with a small
//! Qwen model (see [`QwenEngine`]); the instructions given to it are in [`tasks`].

pub mod languages;

mod chunks;
mod qwen;
mod tasks;

pub use qwen::QwenEngine;
pub use tasks::{summarize, translate};
