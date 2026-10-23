//! Text to speech. Kokoro-82M speaks English in several voices; everything runs on this computer.
//! Pronunciation comes from a dictionary and a few rules, in pure Rust, so the app needs no
//! GPL speech-synthesis tools.

mod chinese;
pub mod clone;
mod digits;
mod flow;
mod french;
mod hindi;
mod italian;
mod kokoro;
mod language;
mod levels;
mod lexicon;
mod numbers;
mod phonemes;
mod portuguese;
mod presets;
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
pub use presets::{
    MAX_NAME_CHARS, MAX_PARTS, MIN_PARTS, Part, Preset, delete as delete_preset, is_preset_id,
    list as list_presets, load as load_preset, save as save_preset, voice_list,
};
pub use spanish::Spanish;
pub use voices::{VoiceInfo, describe, describe_preset};
