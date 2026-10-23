//! Correcting a stored transcript: the words of a line, and who says it.

use rusqlite::params;

use super::{Library, refresh_speaker_count};
use crate::{Error, Result};

/// The stored position of a line; one too large to store matches no line.
fn row(line: usize) -> i64 {
    i64::try_from(line).unwrap_or(i64::MAX)
}

fn no_such_line(id: i64, line: usize) -> Error {
    Error::Input(format!("item {id} has no line {line}"))
}

impl Library {
    /// Replaces the words of one line (counting from 0). Search finds the new text, not the old.
    pub fn set_line_text(&mut self, id: i64, line: usize, text: &str) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Err(Error::Input("a line cannot be empty".into()));
        }
        let changed = self.conn.execute(
            "UPDATE segment SET text = ?3 WHERE item_id = ?1 AND idx = ?2",
            params![id, row(line), text],
        )?;
        if changed == 0 {
            return Err(no_such_line(id, line));
        }
        Ok(())
    }

    /// Gives one line to a speaker, who may be new. Use it to fix a line the labelling got wrong.
    pub fn set_line_speaker(&mut self, id: i64, line: usize, speaker: &str) -> Result<()> {
        let speaker = speaker.trim();
        if speaker.is_empty() {
            return Err(Error::Input("a speaker needs a name".into()));
        }
        let tx = self.conn.transaction()?;
        let changed = tx.execute(
            "UPDATE segment SET speaker = ?3 WHERE item_id = ?1 AND idx = ?2",
            params![id, row(line), speaker],
        )?;
        if changed == 0 {
            return Err(no_such_line(id, line));
        }
        refresh_speaker_count(&tx, id)?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{add_lines, library};

    #[test]
    fn an_edited_line_is_found_by_its_new_words_and_not_the_old() {
        let mut library = library();
        let id = add_lines(&mut library, "Call", &["hello wrold", "second line"]);
        library.set_line_text(id, 0, "  hello world ").unwrap();
        let (_, transcript) = library.get(id).unwrap();
        assert_eq!(transcript.segments[0].text, "hello world");
        assert_eq!(library.search("world", 10).unwrap().len(), 1);
        assert!(library.search("wrold", 10).unwrap().is_empty());
        assert_eq!(library.search("second", 10).unwrap().len(), 1);
    }

    #[test]
    fn a_line_can_move_to_another_or_a_new_speaker_and_counts_follow() {
        let mut library = library();
        let id = add_lines(&mut library, "Call", &["a", "b", "c"]);
        library.set_line_speaker(id, 0, "Maria").unwrap();
        library.set_line_speaker(id, 1, "Maria").unwrap();
        assert_eq!(library.get(id).unwrap().0.speaker_count, 1);
        library.set_line_speaker(id, 1, "Tom").unwrap();
        let (item, transcript) = library.get(id).unwrap();
        assert_eq!(item.speaker_count, 2);
        assert_eq!(transcript.segments[1].speaker.as_deref(), Some("Tom"));
        assert_eq!(transcript.segments[2].speaker, None);
    }

    #[test]
    fn empty_text_unknown_lines_and_nameless_speakers_are_input_errors() {
        let mut library = library();
        let id = add_lines(&mut library, "Call", &["a"]);
        assert!(library.set_line_text(id, 0, "  ").is_err());
        assert!(library.set_line_text(id, 5, "x").is_err());
        assert!(library.set_line_text(id + 1, 0, "x").is_err());
        assert!(library.set_line_speaker(id, 5, "Tom").is_err());
        assert!(library.set_line_speaker(id, 0, " ").is_err());
    }
}
