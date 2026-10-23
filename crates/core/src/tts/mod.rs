//! Text to speech. Kokoro-82M speaks English in several voices; everything runs on this computer.
//! Pronunciation comes from a dictionary and a few rules, in pure Rust, so the app needs no
//! GPL speech-synthesis tools.

mod chinese;
mod digits;
mod flow;
mod french;
mod hindi;
mod italian;
mod kokoro;
mod language;
mod lexicon;
mod numbers;
mod phonemes;
mod portuguese;
mod pronounce;
mod sentences;
mod spanish;
mod spelling;
mod tokens;
mod vocab;
mod voices;

pub use chinese::{Chinese, Dictionary as ChineseDictionary};
pub use french::French;
pub use hindi::Hindi;
pub use italian::Italian;
pub use kokoro::{KokoroEngine, MAX_SPEED, MIN_SPEED, SAMPLE_RATE};
pub use language::Language;
pub use portuguese::Portuguese;
pub use spanish::Spanish;
pub use voices::{VoiceInfo, describe};
