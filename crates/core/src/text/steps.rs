//! Pieces shared by the things the model is asked to do: counting steps for progress, asking
//! once, and deciding whether a result must be translated.

use super::languages;
use crate::engines::{Control, TextEngine};
use crate::{Error, Result, Transcript};

pub(super) fn translation_instruction(language: &str) -> Result<String> {
    let name = languages::name(language)
        .ok_or_else(|| Error::Input(format!("unsupported language: {language}")))?;
    Ok(format!(
        "Translate the user's text into {name}. Reply with only the translation."
    ))
}

/// Runs the model for one step of `total`, reporting `done/total` of the whole job.
pub(super) struct Steps<'a> {
    pub done: usize,
    pub total: usize,
    pub on_progress: &'a mut dyn FnMut(f32) -> Control,
}

impl Steps<'_> {
    pub fn finish_one(&mut self) -> Result<()> {
        self.done += 1;
        self.report()
    }

    fn report(&mut self) -> Result<()> {
        let count = |n: usize| f32::from(u16::try_from(n).unwrap_or(u16::MAX));
        let fraction = count(self.done) / count(self.total).max(1.0);
        match (self.on_progress)(fraction.min(1.0)) {
            Control::Continue => Ok(()),
            Control::Cancel => Err(Error::Cancelled),
        }
    }
}

pub(super) fn never_cancel(_: usize) -> Control {
    Control::Continue
}

/// Summaries are written in the language of the transcript, then translated if another language
/// is wanted: a small model follows "write in X" far less reliably than "translate into X".
pub(super) fn target_language<'a>(
    transcript: &Transcript,
    language: Option<&'a str>,
) -> Result<Option<&'a str>> {
    let Some(code) = language else {
        return Ok(None);
    };
    translation_instruction(code)?;
    let same = transcript
        .language
        .as_deref()
        .and_then(languages::name)
        .is_some_and(|spoken| Some(spoken) == languages::name(code));
    Ok(if same { None } else { Some(code) })
}

/// Asks the model once and counts the step as done.
pub(super) fn ask(
    engine: &mut dyn TextEngine,
    steps: &mut Steps,
    instruction: &str,
    input: &str,
    max_tokens: usize,
) -> Result<String> {
    let answer = engine.generate(instruction, input, max_tokens, &mut never_cancel);
    steps.finish_one().and(answer)
}
