//! Black-box tests for `vtt library` (docs/product/cli.md).
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use vtt_core::library::{Library, NewItem};
use vtt_core::paths::Paths;
use vtt_core::{Segment, Transcript};

fn vtt(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("vtt").unwrap();
    command.arg("--data-dir").arg(dir.path());
    command
}

/// A data directory holding one stored transcript with id 1.
fn seeded() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let transcript = Transcript {
        segments: vec![Segment {
            start_ms: 0,
            end_ms: 2000,
            speaker: None,
            text: "ship the importer behind a flag".into(),
        }],
        language: Some("en".into()),
        duration_ms: 2000,
        model: Some("whisper-base".into()),
    };
    let new = NewItem {
        title: "Standup",
        source_path: None,
    };
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    Library::open(&paths)
        .unwrap()
        .add(&transcript, &new)
        .unwrap();
    dir
}

#[test]
fn list_shows_stored_transcripts_and_json_is_an_array() {
    let dir = seeded();
    vtt(&dir)
        .args(["library", "list"])
        .assert()
        .success()
        .stdout(contains("Standup"));
    let out = vtt(&dir)
        .args(["--json", "library", "list"])
        .output()
        .unwrap();
    let items: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(items[0]["title"], "Standup");
    assert_eq!(items[0]["segment_count"], 1);
}

#[test]
fn an_empty_library_lists_nothing() {
    let dir = tempfile::tempdir().unwrap();
    vtt(&dir)
        .args(["library", "list"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn search_finds_text_and_shows_the_match() {
    let dir = seeded();
    vtt(&dir)
        .args(["library", "search", "importer"])
        .assert()
        .success()
        .stdout(contains("[importer]"));
    vtt(&dir)
        .args(["library", "search", "nothing-like-this"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn show_prints_the_transcript_in_the_requested_format() {
    let dir = seeded();
    vtt(&dir)
        .args(["library", "show", "1"])
        .assert()
        .success()
        .stdout("ship the importer behind a flag\n");
    vtt(&dir)
        .args(["library", "show", "1", "-f", "srt"])
        .assert()
        .success()
        .stdout(contains("00:00:00,000 --> 00:00:02,000"));
}

#[test]
fn export_writes_one_file_per_transcript_into_a_directory() {
    let dir = seeded();
    let out = tempfile::tempdir().unwrap();
    vtt(&dir)
        .args(["library", "export", "1", "-f", "vtt", "-o"])
        .arg(out.path())
        .assert()
        .success();
    let written = std::fs::read_to_string(out.path().join("1-Standup.vtt")).unwrap();
    assert!(written.starts_with("WEBVTT"));
}

#[test]
fn delete_removes_the_item_and_unknown_ids_are_input_errors() {
    let dir = seeded();
    vtt(&dir)
        .args(["library", "delete", "1"])
        .assert()
        .success();
    vtt(&dir)
        .args(["library", "list"])
        .assert()
        .success()
        .stdout("");
    vtt(&dir).args(["library", "delete", "1"]).assert().code(3);
    vtt(&dir).args(["library", "show", "abc"]).assert().code(3);
    vtt(&dir).args(["library", "show", "99"]).assert().code(3);
}
