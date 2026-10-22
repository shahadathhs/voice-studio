//! Full-text search over transcript text (SQLite FTS5).

use serde::Serialize;

use super::{Item, Library, item_from_row, unsigned};
use crate::Result;

const SNIPPET_WORDS: u32 = 12;

/// One library item that matched a search, with the best matching line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHit {
    pub item: Item,
    /// Start of the matching line, so a viewer can jump to it.
    pub start_ms: u64,
    /// The matching line with matches wrapped in `[` and `]`.
    pub snippet: String,
}

/// Turns what the user typed into an FTS5 query: every word must appear, the last one as a prefix
/// so results update while typing. Quoting each word keeps FTS5 operators in user text harmless.
fn fts_query(text: &str) -> Option<String> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
        .collect();
    let (last, rest) = words.split_last()?;
    let mut query = rest.join(" ");
    if !query.is_empty() {
        query.push(' ');
    }
    query.push_str(last);
    query.push('*');
    Some(query)
}

impl Library {
    /// Items whose text contains every word of `text`, best match first, one hit per item.
    pub fn search(&self, text: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let Some(query) = fts_query(text).filter(|_| limit > 0) else {
            return Ok(Vec::new());
        };
        // snippet() only works on plain rows, so lines are read best first and the first line of
        // each item is kept; grouping in SQL is not possible.
        let mut statement = self.conn.prepare(&format!(
            "SELECT {ITEM_COLUMNS}, s.start_ms,
                    snippet(segment_fts, 0, '[', ']', '…', {SNIPPET_WORDS})
             FROM segment_fts
             JOIN segment s ON s.rowid = segment_fts.rowid
             JOIN item i ON i.id = s.item_id
             WHERE segment_fts MATCH ?1
             ORDER BY rank",
            ITEM_COLUMNS = super::ITEM_COLUMNS,
        ))?;
        let rows = statement.query_map([query], |row| {
            Ok(SearchHit {
                item: item_from_row(row)?,
                start_ms: unsigned(row, 9)?,
                snippet: row.get(10)?,
            })
        })?;
        let mut hits: Vec<SearchHit> = Vec::new();
        for hit in rows {
            let hit = hit?;
            if hits.iter().all(|seen| seen.item.id != hit.item.id) {
                hits.push(hit);
            }
            if hits.len() == limit {
                break;
            }
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::tests::{add_lines, library};

    #[test]
    fn queries_are_quoted_and_the_last_word_matches_a_prefix() {
        assert_eq!(
            fts_query("ship importer").unwrap(),
            "\"ship\" \"importer\"*"
        );
        assert_eq!(fts_query("say \"hi\"").unwrap(), "\"say\" \"\"\"hi\"\"\"*");
        assert_eq!(fts_query("   "), None);
    }

    #[test]
    fn search_finds_the_matching_line_and_where_it_starts() {
        let mut library = library();
        add_lines(
            &mut library,
            "Standup",
            &["good morning", "ship the importer"],
        );
        add_lines(&mut library, "Other", &["nothing relevant"]);

        let hits = library.search("importer", 10).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].item.title, "Standup");
        assert_eq!(hits[0].start_ms, 1000);
        assert_eq!(hits[0].snippet, "ship the [importer]");
    }

    #[test]
    fn search_reports_one_hit_per_item_and_ignores_operators() {
        let mut library = library();
        add_lines(&mut library, "A", &["red fish", "red cat"]);
        assert_eq!(library.search("red", 10).unwrap().len(), 1);
        assert_eq!(library.search("red AND OR NOT (", 10).unwrap().len(), 0);
        assert_eq!(library.search("", 10).unwrap().len(), 0);
    }

    #[test]
    fn deleted_items_are_no_longer_found() {
        let mut library = library();
        let id = add_lines(&mut library, "A", &["findable words"]);
        library.delete(id).unwrap();
        assert_eq!(library.search("findable", 10).unwrap().len(), 0);
    }
}
