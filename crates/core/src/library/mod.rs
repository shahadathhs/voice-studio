//! The library: every transcript the user has made, stored in one SQLite file that the CLI and the
//! desktop app share. Lines are indexed for full-text search.

mod schema;
mod search;

pub use search::SearchHit;

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

use crate::paths::Paths;
use crate::{Error, Result, Segment, Transcript};

/// Columns of [`Item`], in order, selected from a table aliased `i`.
const ITEM_COLUMNS: &str = "i.id, i.title, i.created_at, i.duration_ms, i.source_path, i.model, \
                            i.language, i.segment_count, i.speaker_count";

/// What a front end lists for one stored transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub title: String,
    /// Seconds since the Unix epoch.
    pub created_at: i64,
    pub duration_ms: u64,
    pub source_path: Option<String>,
    pub model: Option<String>,
    pub language: Option<String>,
    pub segment_count: u64,
    pub speaker_count: u64,
}

/// What the caller knows about a transcript that the transcript itself does not say.
#[derive(Debug, Clone, Copy)]
pub struct NewItem<'a> {
    pub title: &'a str,
    pub source_path: Option<&'a str>,
}

/// SQLite stores signed 64-bit integers; durations and counts are never negative.
fn unsigned(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    Ok(u64::try_from(row.get::<_, i64>(index)?).unwrap_or(0))
}

fn signed(value: impl TryInto<i64>) -> i64 {
    value.try_into().unwrap_or(i64::MAX)
}

fn item_from_row(row: &Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        duration_ms: unsigned(row, 3)?,
        source_path: row.get(4)?,
        model: row.get(5)?,
        language: row.get(6)?,
        segment_count: unsigned(row, 7)?,
        speaker_count: unsigned(row, 8)?,
    })
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .unwrap_or(0)
}

/// An open library database.
#[derive(Debug)]
pub struct Library {
    conn: Connection,
}

impl Library {
    /// Opens (creating and upgrading as needed) the library in the data directory.
    pub fn open(paths: &Paths) -> Result<Library> {
        std::fs::create_dir_all(&paths.data)?;
        Library::from_connection(Connection::open(paths.library_db())?)
    }

    fn from_connection(mut conn: Connection) -> Result<Library> {
        schema::migrate(&mut conn)?;
        Ok(Library { conn })
    }

    /// Stores a transcript and returns its id.
    pub fn add(&mut self, transcript: &Transcript, new: &NewItem<'_>) -> Result<i64> {
        let speakers = transcript
            .segments
            .iter()
            .filter_map(|s| s.speaker.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len();
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO item (title, created_at, duration_ms, source_path, model, language,
                               segment_count, speaker_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                new.title,
                now_seconds(),
                signed(transcript.duration_ms),
                new.source_path,
                transcript.model,
                transcript.language,
                signed(transcript.segments.len()),
                signed(speakers)
            ],
        )?;
        let id = tx.last_insert_rowid();
        let mut insert = tx.prepare(
            "INSERT INTO segment (item_id, idx, start_ms, end_ms, speaker, text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for (idx, s) in transcript.segments.iter().enumerate() {
            insert.execute(params![
                id,
                signed(idx),
                signed(s.start_ms),
                signed(s.end_ms),
                s.speaker,
                s.text
            ])?;
        }
        drop(insert);
        tx.commit()?;
        Ok(id)
    }

    /// Every item, newest first.
    pub fn list(&self) -> Result<Vec<Item>> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT {ITEM_COLUMNS} FROM item i ORDER BY i.created_at DESC, i.id DESC"
        ))?;
        let items = statement
            .query_map([], item_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(items)
    }

    /// One item with its full transcript.
    pub fn get(&self, id: i64) -> Result<(Item, Transcript)> {
        let item = self
            .conn
            .query_row(
                &format!("SELECT {ITEM_COLUMNS} FROM item i WHERE i.id = ?1"),
                [id],
                item_from_row,
            )
            .optional()?
            .ok_or_else(|| Error::Input(format!("no library item with id {id}")))?;
        let mut statement = self.conn.prepare(
            "SELECT start_ms, end_ms, speaker, text FROM segment WHERE item_id = ?1 ORDER BY idx",
        )?;
        let segments = statement
            .query_map([id], |row| {
                Ok(Segment {
                    start_ms: unsigned(row, 0)?,
                    end_ms: unsigned(row, 1)?,
                    speaker: row.get(2)?,
                    text: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let transcript = Transcript {
            segments,
            language: item.language.clone(),
            duration_ms: item.duration_ms,
            model: item.model.clone(),
        };
        Ok((item, transcript))
    }

    /// Removes an item and its lines. Returns whether it existed.
    pub fn delete(&mut self, id: i64) -> Result<bool> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM segment WHERE item_id = ?1", [id])?;
        let removed = tx.execute("DELETE FROM item WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(removed > 0)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn library() -> Library {
        Library::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    /// Stores a transcript with one line per second, so line `n` starts at `n * 1000` ms.
    pub fn add_lines(library: &mut Library, title: &str, lines: &[&str]) -> i64 {
        let segments = lines
            .iter()
            .enumerate()
            .map(|(n, text)| Segment {
                start_ms: n as u64 * 1000,
                end_ms: n as u64 * 1000 + 900,
                speaker: None,
                text: (*text).to_string(),
            })
            .collect();
        let transcript = Transcript {
            segments,
            ..Transcript::default()
        };
        let new = NewItem {
            title,
            source_path: None,
        };
        library.add(&transcript, &new).unwrap()
    }

    #[test]
    fn a_stored_transcript_comes_back_unchanged() {
        let mut library = library();
        let transcript = Transcript {
            segments: vec![
                Segment {
                    start_ms: 0,
                    end_ms: 1500,
                    speaker: Some("Maria".into()),
                    text: "Hello there".into(),
                },
                Segment {
                    start_ms: 1500,
                    end_ms: 3000,
                    speaker: Some("Lee".into()),
                    text: "Hi".into(),
                },
            ],
            language: Some("en".into()),
            duration_ms: 3000,
            model: Some("whisper-base".into()),
        };
        let new = NewItem {
            title: "Call",
            source_path: Some("/tmp/call.wav"),
        };

        let id = library.add(&transcript, &new).unwrap();
        let (item, loaded) = library.get(id).unwrap();

        assert_eq!(loaded, transcript);
        assert_eq!(item.title, "Call");
        assert_eq!(item.source_path.as_deref(), Some("/tmp/call.wav"));
        assert_eq!((item.segment_count, item.speaker_count), (2, 2));
        assert!(item.created_at > 0);
    }

    #[test]
    fn list_is_newest_first_and_delete_removes_items() {
        let mut library = library();
        let first = add_lines(&mut library, "first", &["a"]);
        let second = add_lines(&mut library, "second", &["b"]);

        let ids: Vec<i64> = library.list().unwrap().iter().map(|i| i.id).collect();
        assert_eq!(ids, vec![second, first]);

        assert!(library.delete(first).unwrap());
        assert!(!library.delete(first).unwrap());
        assert_eq!(library.list().unwrap().len(), 1);
        assert!(matches!(library.get(first), Err(Error::Input(_))));
    }

    #[test]
    fn the_database_lives_in_the_data_directory_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::resolve(Some(dir.path().to_path_buf()));
        let id = add_lines(&mut Library::open(&paths).unwrap(), "kept", &["x"]);
        let reopened = Library::open(&paths).unwrap();
        assert_eq!(reopened.get(id).unwrap().0.title, "kept");
    }
}
