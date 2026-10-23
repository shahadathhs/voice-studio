//! Database layout and migrations, versioned with `SQLite`'s `user_version`.

use rusqlite::Connection;

use crate::Result;

/// Each entry upgrades the database by one version; never edit an entry that has shipped.
const MIGRATIONS: &[&str] = &[
    "
    CREATE TABLE item (
        id            INTEGER PRIMARY KEY,
        title         TEXT NOT NULL,
        created_at    INTEGER NOT NULL,
        duration_ms   INTEGER NOT NULL,
        source_path   TEXT,
        model         TEXT,
        language      TEXT,
        segment_count INTEGER NOT NULL,
        speaker_count INTEGER NOT NULL
    );
    CREATE TABLE segment (
        item_id  INTEGER NOT NULL REFERENCES item(id) ON DELETE CASCADE,
        idx      INTEGER NOT NULL,
        start_ms INTEGER NOT NULL,
        end_ms   INTEGER NOT NULL,
        speaker  TEXT,
        text     TEXT NOT NULL,
        PRIMARY KEY (item_id, idx)
    );
    CREATE VIRTUAL TABLE segment_fts USING fts5(text, content='segment', content_rowid='rowid');
    CREATE TRIGGER segment_added AFTER INSERT ON segment BEGIN
        INSERT INTO segment_fts(rowid, text) VALUES (new.rowid, new.text);
    END;
    CREATE TRIGGER segment_removed AFTER DELETE ON segment BEGIN
        INSERT INTO segment_fts(segment_fts, rowid, text) VALUES ('delete', old.rowid, old.text);
    END;
",
    // Editing a line keeps the search index in step: remove the old text, add the new.
    "
    CREATE TRIGGER segment_edited AFTER UPDATE OF text ON segment BEGIN
        INSERT INTO segment_fts(segment_fts, rowid, text) VALUES ('delete', old.rowid, old.text);
        INSERT INTO segment_fts(rowid, text) VALUES (new.rowid, new.text);
    END;
",
    // What a language model wrote about a transcript.
    "
    CREATE TABLE note (
        id         INTEGER PRIMARY KEY,
        item_id    INTEGER NOT NULL REFERENCES item(id) ON DELETE CASCADE,
        kind       TEXT NOT NULL,
        language   TEXT,
        model      TEXT NOT NULL,
        text       TEXT NOT NULL,
        created_at INTEGER NOT NULL
    );
    CREATE INDEX note_item ON note(item_id);
",
];

/// Brings `conn` up to the latest layout.
pub fn migrate(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let applied = usize::try_from(current).unwrap_or(0);
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(
            None,
            "user_version",
            i64::try_from(index + 1).unwrap_or(i64::MAX),
        )?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrating_twice_keeps_the_database_at_the_latest_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, i64::try_from(MIGRATIONS.len()).unwrap());
    }
}
