//! The outline of a transcript: chapters with the time they start, and the action items.

use serde::{Deserialize, Serialize};

use super::chunks;
use super::steps::{Steps, ask, target_language, translation_instruction};
use crate::engines::{Control, TextEngine};
use crate::{Error, Result, Transcript};

/// Characters per chapter: short enough that a long talk gets many chapters.
const CHAPTER_CHARS: usize = 2500;
const TITLE_TOKENS: usize = 40;
const ACTION_TOKENS: usize = 400;
const MAX_TITLE_CHARS: usize = 80;
const NOTHING: &str = "NONE";

/// A stretch of the recording with a title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chapter {
    pub start_ms: u64,
    pub title: String,
}

const TITLE_INSTRUCTION: &str = "You give titles to parts of a recording. Reply with one short \
    title of at most eight words for the text you are given, in the same language as the text, \
    with no quotes and no explanation.";

const ACTION_INSTRUCTION: &str = "You find the tasks, follow-ups and commitments in a transcript. \
    List each on its own line starting with \"- \", saying who is responsible when the transcript \
    says. Use only what the transcript says. If there are none, reply exactly NONE.";

/// Chapters as stored in a library note.
pub fn chapters_to_json(chapters: &[Chapter]) -> Result<String> {
    Ok(serde_json::to_string(chapters)?)
}

/// Chapters from a stored library note.
pub fn chapters_from_json(json: &str) -> Result<Vec<Chapter>> {
    Ok(serde_json::from_str(json)?)
}

/// The first line of the model's answer, without the decoration small models like to add.
fn clean_title(raw: &str) -> String {
    let line = raw
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let line = line.trim_start_matches(['#', '*', '-', ' ']);
    let line = ["title:", "Title:", "TITLE:"]
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
        .unwrap_or(line);
    let line = line
        .trim()
        .trim_matches(['"', '\u{201c}', '\u{201d}', '*', '\''])
        .trim();
    line.chars().take(MAX_TITLE_CHARS).collect()
}

/// A title for each stretch of the recording, in order.
pub fn chapters(
    engine: &mut dyn TextEngine,
    transcript: &Transcript,
    on_progress: &mut dyn FnMut(f32) -> Control,
) -> Result<Vec<Chapter>> {
    let parts = chunks::parts(&chunks::lines_of(transcript), CHAPTER_CHARS);
    if parts.is_empty() {
        return Err(Error::Input(
            "there is no text to divide into chapters".into(),
        ));
    }
    let mut steps = Steps {
        done: 0,
        total: parts.len(),
        on_progress,
    };
    let mut chapters = Vec::with_capacity(parts.len());
    for part in &parts {
        let raw = ask(
            engine,
            &mut steps,
            TITLE_INSTRUCTION,
            &part.text,
            TITLE_TOKENS,
        )?;
        let title = clean_title(&raw);
        if !title.is_empty() {
            chapters.push(Chapter {
                start_ms: part.start_ms,
                title,
            });
        }
    }
    Ok(chapters)
}

/// The list lines of an answer, each as `- text`; nothing for an answer of `NONE`.
fn list_lines(answer: &str) -> Vec<String> {
    answer
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix(['-', '*', '\u{2022}']))
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty() && !item.eq_ignore_ascii_case(NOTHING))
        .collect()
}

/// The tasks and commitments in the transcript as lines starting with `- `, or an empty string
/// when there are none. `language` works as for [`super::summarize`].
pub fn action_items(
    engine: &mut dyn TextEngine,
    transcript: &Transcript,
    language: Option<&str>,
    on_progress: &mut dyn FnMut(f32) -> Control,
) -> Result<String> {
    let target = target_language(transcript, language)?;
    let parts = chunks::parts(&chunks::lines_of(transcript), chunks::PART_CHARS);
    if parts.is_empty() {
        return Err(Error::Input("there is no text to look for tasks in".into()));
    }
    let mut steps = Steps {
        done: 0,
        total: parts.len() + usize::from(target.is_some()),
        on_progress,
    };
    let mut items: Vec<String> = Vec::new();
    for part in &parts {
        let answer = ask(
            engine,
            &mut steps,
            ACTION_INSTRUCTION,
            &part.text,
            ACTION_TOKENS,
        )?;
        for item in list_lines(&answer) {
            if !items.iter().any(|known| known.eq_ignore_ascii_case(&item)) {
                items.push(item);
            }
        }
    }
    if items.is_empty() {
        return Ok(String::new());
    }
    let list = items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n");
    match target {
        Some(code) => ask(
            engine,
            &mut steps,
            &translation_instruction(code)?,
            &list,
            ACTION_TOKENS,
        ),
        None => Ok(list),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fake::{Echo, transcript};
    use super::*;

    /// Answers each request from a list, in order.
    struct Script(Vec<&'static str>);

    impl TextEngine for Script {
        fn generate(
            &mut self,
            _instruction: &str,
            _input: &str,
            _max_tokens: usize,
            _on_progress: &mut dyn FnMut(usize) -> Control,
        ) -> Result<String> {
            Ok(self.0.remove(0).to_string())
        }
    }

    fn go(_: f32) -> Control {
        Control::Continue
    }

    #[test]
    fn titles_lose_quotes_labels_and_extra_lines() {
        assert_eq!(clean_title("\"Launch plan\""), "Launch plan");
        assert_eq!(clean_title("Title: **Budget**\nBecause..."), "Budget");
        assert_eq!(clean_title("# Intro"), "Intro");
        assert_eq!(clean_title("  \n  "), "");
    }

    #[test]
    fn every_part_gets_a_chapter_starting_where_the_part_starts() {
        let long = "word ".repeat(450);
        let mut script = Script(vec!["First", "Second"]);
        let result = chapters(&mut script, &transcript(&[&long, &long]), &mut go).unwrap();
        assert_eq!(
            result,
            [
                Chapter {
                    start_ms: 0,
                    title: "First".into()
                },
                Chapter {
                    start_ms: 1000,
                    title: "Second".into()
                },
            ]
        );
    }

    #[test]
    fn action_items_are_collected_without_duplicates() {
        let long = "word ".repeat(1000);
        let mut script = Script(vec![
            "- Ben fixes the installer\n- Ana writes it",
            "* ben fixes the installer\nNONE",
        ]);
        let items = action_items(&mut script, &transcript(&[&long, &long]), None, &mut go).unwrap();
        assert_eq!(items, "- Ben fixes the installer\n- Ana writes it");
    }

    #[test]
    fn no_tasks_gives_an_empty_list_and_no_translation() {
        let mut engine = Echo::default();
        let items = action_items(&mut engine, &transcript(&["hi"]), Some("es"), &mut go).unwrap();
        assert_eq!(items, "", "an answer without list lines means no tasks");
        assert_eq!(engine.calls.len(), 1, "nothing to translate");
    }

    #[test]
    fn an_empty_transcript_is_an_input_error() {
        let mut engine = Echo::default();
        assert!(chapters(&mut engine, &transcript(&[]), &mut go).is_err());
        assert!(action_items(&mut engine, &transcript(&[]), None, &mut go).is_err());
    }
}
