//! Library behaviour: storing, listing, renaming speakers and deleting.

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

fn add_spoken(library: &mut Library, speakers: &[&str]) -> i64 {
    let segments = speakers
        .iter()
        .enumerate()
        .map(|(n, speaker)| Segment {
            start_ms: n as u64 * 1000,
            end_ms: n as u64 * 1000 + 900,
            speaker: Some((*speaker).to_string()),
            text: format!("line {n}"),
        })
        .collect();
    let transcript = Transcript {
        segments,
        ..Transcript::default()
    };
    library
        .add(
            &transcript,
            &NewItem {
                title: "talk",
                source_path: None,
            },
        )
        .unwrap()
}

#[test]
fn renaming_a_speaker_changes_every_line_and_keeps_the_count() {
    let mut library = library();
    let id = add_spoken(&mut library, &["SPEAKER_00", "SPEAKER_01", "SPEAKER_00"]);

    assert_eq!(
        library
            .rename_speaker(id, "SPEAKER_00", "  Maria ")
            .unwrap(),
        2
    );

    let (item, transcript) = library.get(id).unwrap();
    let names: Vec<_> = transcript
        .segments
        .iter()
        .map(|s| s.speaker.clone().unwrap())
        .collect();
    assert_eq!(names, ["Maria", "SPEAKER_01", "Maria"]);
    assert_eq!(item.speaker_count, 2);
}

#[test]
fn renaming_to_an_existing_name_merges_two_speakers() {
    let mut library = library();
    let id = add_spoken(&mut library, &["SPEAKER_00", "SPEAKER_01", "SPEAKER_02"]);

    library
        .rename_speaker(id, "SPEAKER_02", "SPEAKER_00")
        .unwrap();

    assert_eq!(library.get(id).unwrap().0.speaker_count, 2);
}

#[test]
fn renaming_needs_a_real_speaker_and_a_real_name() {
    let mut library = library();
    let id = add_spoken(&mut library, &["SPEAKER_00"]);
    assert!(matches!(
        library.rename_speaker(id, "SPEAKER_00", "  "),
        Err(Error::Input(_))
    ));
    assert!(matches!(
        library.rename_speaker(id, "nobody", "x"),
        Err(Error::Input(_))
    ));
    assert!(matches!(
        library.rename_speaker(id + 1, "SPEAKER_00", "x"),
        Err(Error::Input(_))
    ));
    assert_eq!(
        library.get(id).unwrap().1.segments[0].speaker.as_deref(),
        Some("SPEAKER_00")
    );
}

#[test]
fn the_database_lives_in_the_data_directory_and_persists() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    let id = add_lines(&mut Library::open(&paths).unwrap(), "kept", &["x"]);
    let reopened = Library::open(&paths).unwrap();
    assert_eq!(reopened.get(id).unwrap().0.title, "kept");
}
