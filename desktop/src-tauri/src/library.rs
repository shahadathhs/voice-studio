//! Library and export commands. Each call opens the library, which is cheap, so the shell keeps
//! no database state of its own.

use std::path::Path;

use serde::{Deserialize, Serialize};
use vtt_core::Transcript;
use vtt_core::export::{self, ExportOptions, Format};
use vtt_core::library::{Item, Library, SearchHit};
use vtt_core::paths::Paths;

use crate::commands::CommandError;

const SEARCH_LIMIT: usize = 50;

/// Speaking time of one speaker. Mirrored by `SpeakerShare` in `desktop/src/lib/types.ts`.
#[derive(Debug, Serialize)]
pub struct SpeakerShare {
    name: String,
    talk_ms: u64,
}

/// One stored transcript with its lines. Mirrored by `TranscriptView` in `desktop/src/lib/types.ts`.
#[derive(Debug, Serialize)]
pub struct TranscriptView {
    item: Item,
    transcript: Transcript,
    word_count: usize,
    speakers: Vec<SpeakerShare>,
}

impl TranscriptView {
    fn new(item: Item, transcript: Transcript) -> TranscriptView {
        let speakers = transcript
            .talk_time_by_speaker()
            .into_iter()
            .map(|(name, talk_ms)| SpeakerShare { name, talk_ms })
            .collect();
        TranscriptView {
            word_count: transcript.word_count(),
            item,
            transcript,
            speakers,
        }
    }
}

/// The export dialog's options. Mirrored by `ExportSettings` in `desktop/src/lib/types.ts`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettings {
    speaker_names: bool,
    timestamps: bool,
    max_chars_per_line: usize,
    lines_per_cue: usize,
}

impl From<ExportSettings> for ExportOptions {
    fn from(settings: ExportSettings) -> Self {
        ExportOptions {
            speaker_names: settings.speaker_names,
            timestamps: settings.timestamps,
            max_chars_per_line: settings.max_chars_per_line,
            lines_per_cue: settings.lines_per_cue,
        }
    }
}

fn open() -> vtt_core::Result<Library> {
    Library::open(&Paths::resolve(None))
}

fn render_in(
    library: &Library,
    id: i64,
    format: &str,
    settings: ExportSettings,
) -> vtt_core::Result<String> {
    let format: Format = format.parse()?;
    let (_, transcript) = library.get(id)?;
    export::render(&transcript, format, &settings.into())
}

#[tauri::command]
pub fn list_library() -> Result<Vec<Item>, CommandError> {
    Ok(open()?.list()?)
}

#[tauri::command]
pub fn search_library(query: &str) -> Result<Vec<SearchHit>, CommandError> {
    Ok(open()?.search(query, SEARCH_LIMIT)?)
}

#[tauri::command]
pub fn get_transcript(id: i64) -> Result<TranscriptView, CommandError> {
    let (item, transcript) = open()?.get(id)?;
    Ok(TranscriptView::new(item, transcript))
}

#[tauri::command]
pub fn delete_library_item(id: i64) -> Result<bool, CommandError> {
    Ok(open()?.delete(id)?)
}

/// The exported text, for the dialog's preview and for Copy.
#[tauri::command]
pub fn render_export(
    id: i64,
    format: &str,
    settings: ExportSettings,
) -> Result<String, CommandError> {
    Ok(render_in(&open()?, id, format, settings)?)
}

#[tauri::command]
pub fn save_export(
    id: i64,
    format: &str,
    settings: ExportSettings,
    path: &str,
) -> Result<(), CommandError> {
    let text = render_in(&open()?, id, format, settings)?;
    std::fs::write(Path::new(path), text).map_err(vtt_core::Error::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use vtt_core::Segment;
    use vtt_core::library::NewItem;

    use super::*;

    fn settings() -> ExportSettings {
        ExportSettings {
            speaker_names: true,
            timestamps: false,
            max_chars_per_line: 42,
            lines_per_cue: 2,
        }
    }

    fn library_with_one_line() -> (Library, i64) {
        let dir = tempfile::tempdir().unwrap();
        let mut library = Library::open(&Paths::resolve(Some(dir.keep()))).unwrap();
        let transcript = Transcript {
            segments: vec![Segment {
                start_ms: 0,
                end_ms: 2000,
                speaker: None,
                text: "hello world".into(),
            }],
            ..Transcript::default()
        };
        let new = NewItem {
            title: "t",
            source_path: None,
        };
        let id = library.add(&transcript, &new).unwrap();
        (library, id)
    }

    #[test]
    fn export_renders_the_stored_transcript_in_the_chosen_format() {
        let (library, id) = library_with_one_line();
        assert_eq!(
            render_in(&library, id, "txt", settings()).unwrap(),
            "hello world\n"
        );
        let srt = render_in(&library, id, "srt", settings()).unwrap();
        assert!(srt.contains("00:00:00,000 --> 00:00:02,000"));
    }

    #[test]
    fn export_rejects_unknown_formats_and_missing_items() {
        let (library, id) = library_with_one_line();
        assert!(render_in(&library, id, "wat", settings()).is_err());
        assert!(render_in(&library, id + 1, "txt", settings()).is_err());
    }

    #[test]
    fn the_viewer_gets_word_count_and_speaker_shares_with_the_transcript() {
        let (library, id) = library_with_one_line();
        let (item, transcript) = library.get(id).unwrap();
        let view = TranscriptView::new(item, transcript);
        assert_eq!(view.word_count, 2);
        assert_eq!(view.speakers.len(), 0);
    }

    #[test]
    fn settings_from_the_dialog_arrive_in_camel_case() {
        let json =
            r#"{"speakerNames":false,"timestamps":true,"maxCharsPerLine":30,"linesPerCue":1}"#;
        let options: ExportOptions = serde_json::from_str::<ExportSettings>(json).unwrap().into();
        assert!(!options.speaker_names && options.timestamps);
        assert_eq!((options.max_chars_per_line, options.lines_per_cue), (30, 1));
    }
}
