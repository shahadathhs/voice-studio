//! Black-box tests for `vtt speak` that need no model.
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn speak(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("vtt").unwrap();
    command.arg("--data-dir").arg(dir.path()).arg("speak");
    command
}

#[test]
fn without_the_voice_model_speak_exits_4_and_names_the_pull_command() {
    let dir = tempfile::tempdir().unwrap();
    speak(&dir)
        .arg("Hello there")
        .assert()
        .code(4)
        .stderr(contains("vtt models pull kokoro-82m"));
}

#[test]
fn text_is_needed_and_empty_text_is_an_input_error() {
    let dir = tempfile::tempdir().unwrap();
    speak(&dir)
        .arg("   ")
        .assert()
        .code(3)
        .stderr(contains("no text"));
    speak(&dir).write_stdin("").assert().code(3);
}

#[test]
fn the_text_can_come_from_a_missing_file_which_is_an_input_error() {
    let dir = tempfile::tempdir().unwrap();
    speak(&dir)
        .args(["--file", "/definitely/not/here.txt"])
        .assert()
        .code(3)
        .stderr(contains("cannot read"));
}

#[test]
fn speed_outside_the_supported_range_is_an_input_error() {
    let dir = tempfile::tempdir().unwrap();
    speak(&dir).args(["hi", "--speed", "5"]).assert().code(3);
    speak(&dir).args(["hi", "--speed", "0.1"]).assert().code(3);
}

#[test]
fn only_wav_is_written_so_far() {
    let dir = tempfile::tempdir().unwrap();
    speak(&dir)
        .args(["hi", "--format", "mp3"])
        .assert()
        .code(70);
    speak(&dir).args(["hi", "--format", "ogg"]).assert().code(3);
    speak(&dir).args(["hi", "--play"]).assert().code(70);
}
