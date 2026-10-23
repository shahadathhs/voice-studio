//! `vtt summarize` and `vtt translate` up to the point where a model is needed.
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::prelude::*;

const TRANSCRIPT: &str = r#"{"language":"en","duration_ms":5000,"model":"test",
  "segments":[{"start_ms":0,"end_ms":4000,"text":"Ship the importer first"}]}"#;

fn vtt(data: &tempfile::TempDir) -> Command {
    let mut command = Command::cargo_bin("vtt").unwrap();
    command.arg("--data-dir").arg(data.path());
    command
}

#[test]
fn a_missing_language_model_is_exit_4_with_the_command_to_install_it() {
    let data = tempfile::tempdir().unwrap();
    vtt(&data)
        .args(["summarize", "-"])
        .write_stdin(TRANSCRIPT)
        .assert()
        .code(4)
        .stderr(predicate::str::contains("vtt models pull qwen2.5-1.5b"));
}

#[test]
fn only_language_models_can_be_chosen() {
    let data = tempfile::tempdir().unwrap();
    vtt(&data)
        .args(["translate", "-", "--to", "es", "-m", "whisper-base"])
        .write_stdin(TRANSCRIPT)
        .assert()
        .code(3)
        .stderr(predicate::str::contains("not a language model"));
}

#[test]
fn a_transcript_file_that_does_not_exist_is_an_input_error() {
    let data = tempfile::tempdir().unwrap();
    vtt(&data)
        .args(["summarize", "nowhere.json", "-y"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("cannot read nowhere.json"));
}

#[test]
fn the_language_models_are_listed_as_text_models() {
    let data = tempfile::tempdir().unwrap();
    vtt(&data)
        .args(["models", "list", "--kind", "text"])
        .assert()
        .success()
        .stdout(predicate::str::contains("qwen2.5-1.5b"))
        .stdout(predicate::str::contains("qwen2.5-7b"));
}
