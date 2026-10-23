//! The library: every transcript the user has made, stored in one `SQLite` file that the CLI and the
//! desktop app share. Lines are indexed for full-text search.

mod edit;
mod schema;
mod search;

pub use search::SearchHit;

use std::path::Path;
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

/// The title a file gets in the library: its name without the extension.
pub fn title_for(file: &Path) -> String {
    file.file_stem()
        .map_or_else(|| "Transcript".into(), |s| s.to_string_lossy().into_owned())
}

/// What the caller knows about a transcript that the transcript itself does not say.
#[derive(Debug, Clone, Copy)]
pub struct NewItem<'a> {
    pub title: &'a str,
    pub source_path: Option<&'a str>,
}

/// `SQLite` stores signed 64-bit integers; durations and counts are never negative.
fn unsigned(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    Ok(u64::try_from(row.get::<_, i64>(index)?).unwrap_or(0))
}

fn signed(value: impl TryInto<i64>) -> i64 {
    value.try_into().unwrap_or(i64::MAX)
}

fn signed_back(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
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

    /// Renames a speaker in one transcript. Giving a name that another speaker already has merges
    /// the two. Returns how many lines changed; fails if nobody has that name.
    pub fn rename_speaker(&mut self, id: i64, from: &str, to: &str) -> Result<u64> {
        let to = to.trim();
        if to.is_empty() {
            return Err(Error::Input("a speaker needs a name".into()));
        }
        let tx = self.conn.transaction()?;
        let changed = tx.execute(
            "UPDATE segment SET speaker = ?3 WHERE item_id = ?1 AND speaker = ?2",
            params![id, from, to],
        )?;
        if changed == 0 {
            return Err(Error::Input(format!(
                "no speaker named {from} in item {id}"
            )));
        }
        refresh_speaker_count(&tx, id)?;
        tx.commit()?;
        Ok(signed_back(changed))
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

/// Recounts the distinct speakers of an item after lines were relabelled.
fn refresh_speaker_count(tx: &rusqlite::Transaction<'_>, id: i64) -> Result<()> {
    tx.execute(
        "UPDATE item SET speaker_count = (SELECT COUNT(DISTINCT speaker) FROM segment
                                          WHERE item_id = ?1 AND speaker IS NOT NULL)
         WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
