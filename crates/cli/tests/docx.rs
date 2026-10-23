//! Word export through the CLI: a real zip file, not text.
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;

const TRANSCRIPT: &str = r#"{"language":"en","duration_ms":5000,"model":"test",
  "segments":[{"start_ms":0,"end_ms":4000,"speaker":"Maria","text":"Ship the importer first"}]}"#;

#[test]
fn docx_export_writes_a_zip_with_the_document_inside() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("t.docx");
    Command::cargo_bin("vtt")
        .unwrap()
        .args(["export", "-", "-f", "docx", "-o"])
        .arg(&out)
        .write_stdin(TRANSCRIPT)
        .assert()
        .success();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..4], b"PK\x03\x04");
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("word/document.xml"));
    assert!(text.contains("Maria: "));
    assert!(text.contains("Ship the importer first"));
}
