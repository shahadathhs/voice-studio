//! What a language model wrote about a transcript: a summary, chapters, action items. Kept with
//! the transcript so it is there the next time it is opened.

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::{Library, now_seconds};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteKind {
    Summary,
    /// A JSON list of `{"start_ms", "title"}`; see `text::Chapter`.
    Chapters,
    Actions,
}

impl NoteKind {
    fn as_str(self) -> &'static str {
        match self {
            NoteKind::Summary => "summary",
            NoteKind::Chapters => "chapters",
            NoteKind::Actions => "actions",
        }
    }

    fn parse(name: &str) -> Option<NoteKind> {
        [NoteKind::Summary, NoteKind::Chapters, NoteKind::Actions]
            .into_iter()
            .find(|kind| kind.as_str() == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub item_id: i64,
    pub kind: NoteKind,
    /// The language it is written in, when it was asked for one.
    pub language: Option<String>,
    /// The language model that wrote it.
    pub model: String,
    pub text: String,
    /// Seconds since the Unix epoch.
    pub created_at: i64,
}

/// A note to keep.
#[derive(Debug, Clone, Copy)]
pub struct NewNote<'a> {
    pub kind: NoteKind,
    pub language: Option<&'a str>,
    pub model: &'a str,
    pub text: &'a str,
}

impl Library {
    /// Keeps a note. Writing the same kind in the same language again replaces the earlier one.
    pub fn add_note(&mut self, item_id: i64, note: &NewNote<'_>) -> Result<i64> {
        let NewNote {
            kind,
            language,
            model,
            text,
        } = *note;
        let tx = self.conn.transaction()?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM item WHERE id = ?1)",
            [item_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(Error::Input(format!("no library item with id {item_id}")));
        }
        tx.execute(
            "DELETE FROM note WHERE item_id = ?1 AND kind = ?2 AND language IS ?3",
            params![item_id, kind.as_str(), language],
        )?;
        tx.execute(
            "INSERT INTO note (item_id, kind, language, model, text, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![item_id, kind.as_str(), language, model, text, now_seconds()],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    /// The notes of one item, newest first.
    pub fn notes(&self, item_id: i64) -> Result<Vec<Note>> {
        let mut statement = self.conn.prepare(
            "SELECT id, kind, language, model, text, created_at FROM note
             WHERE item_id = ?1 ORDER BY created_at DESC, id DESC",
        )?;
        let rows = statement.query_map([item_id], |row| {
            let kind: String = row.get(1)?;
            Ok((
                Note {
                    id: row.get(0)?,
                    item_id,
                    kind: NoteKind::Summary,
                    language: row.get(2)?,
                    model: row.get(3)?,
                    text: row.get(4)?,
                    created_at: row.get(5)?,
                },
                kind,
            ))
        })?;
        let mut notes = Vec::new();
        for row in rows {
            let (mut note, kind) = row?;
            if let Some(kind) = NoteKind::parse(&kind) {
                note.kind = kind;
                notes.push(note);
            }
        }
        Ok(notes)
    }

    /// Removes a note. Returns whether it existed.
    pub fn delete_note(&mut self, note_id: i64) -> Result<bool> {
        Ok(self
            .conn
            .execute("DELETE FROM note WHERE id = ?1", [note_id])?
            > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{add_lines, library};
    use super::*;

    fn note<'a>(kind: NoteKind, language: Option<&'a str>, text: &'a str) -> NewNote<'a> {
        NewNote {
            kind,
            language,
            model: "m",
            text,
        }
    }

    #[test]
    fn notes_are_kept_per_item_and_replaced_by_the_same_kind_and_language() {
        let mut library = library();
        let item = add_lines(&mut library, "Talk", &["hello"]);
        library
            .add_note(item, &note(NoteKind::Summary, None, "first"))
            .unwrap();
        library
            .add_note(item, &note(NoteKind::Summary, Some("es"), "resumen"))
            .unwrap();
        library
            .add_note(item, &note(NoteKind::Summary, None, "second"))
            .unwrap();
        let notes = library.notes(item).unwrap();
        let texts: Vec<_> = notes.iter().map(|n| n.text.as_str()).collect();
        assert_eq!(texts, ["second", "resumen"]);
        assert_eq!(notes[0].kind, NoteKind::Summary);
    }

    #[test]
    fn notes_go_away_with_their_item_or_on_request() {
        let mut library = library();
        let item = add_lines(&mut library, "Talk", &["hello"]);
        let saved = library
            .add_note(item, &note(NoteKind::Actions, None, "- call Ana"))
            .unwrap();
        assert!(library.delete_note(saved).unwrap());
        assert!(!library.delete_note(saved).unwrap());
        library
            .add_note(item, &note(NoteKind::Actions, None, "- again"))
            .unwrap();
        library.delete(item).unwrap();
        assert_eq!(library.notes(item).unwrap().len(), 0);
    }

    #[test]
    fn a_note_needs_an_item() {
        let mut library = library();
        let error = library
            .add_note(99, &note(NoteKind::Summary, None, "x"))
            .unwrap_err();
        assert!(matches!(error, Error::Input(_)));
    }
}
