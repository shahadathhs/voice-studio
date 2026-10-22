//! Black-box tests for speaker identification and translation options of `vtt transcribe`.
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use vtt_core::models::Catalog;
use vtt_core::paths::Paths;

const TONE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../core/tests/fixtures/tone.wav"
);

fn transcribe(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("vtt").unwrap();
    command
        .arg("--data-dir")
        .arg(dir.path())
        .args(["transcribe", TONE]);
    command
}

/// Makes a model count as installed without downloading it: files of the right size, all zeros.
fn fake_install(dir: &TempDir, id: &str) {
    let catalog = Catalog::bundled().unwrap();
    let entry = catalog.find(id).unwrap();
    let model_dir = Paths::resolve(Some(dir.path().to_path_buf()))
        .models_dir()
        .join(id);
    std::fs::create_dir_all(&model_dir).unwrap();
    for file in &entry.files {
        let handle = std::fs::File::create(model_dir.join(&file.name)).unwrap();
        handle.set_len(file.size_bytes).unwrap();
    }
}

#[test]
fn the_speaker_model_is_needed_for_diarize_and_the_hint_names_it() {
    let dir = tempfile::tempdir().unwrap();
    fake_install(&dir, "whisper-base");
    transcribe(&dir)
        .arg("--diarize")
        .assert()
        .code(4)
        .stderr(contains("vtt models pull wespeaker-resnet34"));
}

#[test]
fn speaker_count_and_threshold_only_make_sense_with_diarize() {
    let dir = tempfile::tempdir().unwrap();
    transcribe(&dir).args(["--speakers", "2"]).assert().code(2);
    transcribe(&dir)
        .args(["--diarize-threshold", "0.4"])
        .assert()
        .code(2);
}

#[test]
fn translating_with_a_turbo_model_is_refused_before_anything_else() {
    let dir = tempfile::tempdir().unwrap();
    transcribe(&dir)
        .args(["--translate", "-m", "whisper-large-v3-turbo"])
        .assert()
        .code(3)
        .stderr(contains("cannot translate"));
}

#[test]
fn translating_with_a_model_that_can_is_not_refused_for_that_reason() {
    let dir = tempfile::tempdir().unwrap();
    transcribe(&dir)
        .args(["--translate", "-m", "base"])
        .assert()
        .code(4)
        .stderr(contains("model not installed"));
}
