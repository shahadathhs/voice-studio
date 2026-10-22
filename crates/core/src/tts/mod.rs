//! Text to speech. Kokoro-82M speaks English in several voices; everything runs on this computer.
//! Pronunciation comes from a dictionary and a few rules, in pure Rust, so the app needs no
//! GPL speech-synthesis tools.

mod kokoro;
mod lexicon;
mod numbers;
mod phonemes;
mod sentences;
mod spelling;
mod tokens;
mod vocab;

pub use kokoro::{KokoroEngine, MAX_SPEED, MIN_SPEED, SAMPLE_RATE};
