//! Translating the text of a speech request into the voice's language.

use vtt_core::engines::Control;
use vtt_core::paths::Paths;
use vtt_core::tts::Language;
use vtt_core::{Error, Result};

use super::Request;
use crate::assistant;

/// The text translated into the voice's language, when that was asked for. While it is being
/// translated, progress is reported with a total of 0.
pub(super) fn translated(
    paths: &Paths,
    request: &Request,
    progress: &dyn Fn(usize, usize) -> Control,
) -> Result<Option<String>> {
    if !request.translate {
        return Ok(None);
    }
    let language = Language::of_voice(&request.voice)
        .ok_or_else(|| Error::Input(format!("unknown voice: {}", request.voice)))?;
    progress(0, 0);
    let text = assistant::translate_for_speech(paths, &request.text, language.code(), &mut |_| {
        progress(0, 0)
    })?;
    Ok(Some(text))
}
