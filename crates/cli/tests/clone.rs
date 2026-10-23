//! Black-box tests for voice cloning that need no model: consent, bad input, listing, deleting and
//! the hint when the model is not installed.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

fn vtt(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("vtt").unwrap();
    command.arg("--data-dir").arg(dir.path());
    command
}

/// A cloned voice file with tiny tensors, written the way the documented format says.
fn fake_clone(data: &Path, id: &str) {
    let header = serde_json::json!({
        "meta": {
            "id": id,
            "name": "Anna",
            "cloned": true,
            "created_at": "2026-10-23T10:00:00Z",
            "consent": {
                "statement": "I own this voice or have the speaker's permission",
                "given_at": "2026-10-23T10:00:00Z"
            }
        },
        "shapes": [[1, 1, 1], [1, 1], [1, 1], [1, 1, 1]]
    })
    .to_string();
    let mut bytes = b"VSCV".to_vec();
    bytes.extend(u32::try_from(header.len()).unwrap().to_le_bytes());
    bytes.extend(header.as_bytes());
    bytes.extend(0.5_f32.to_le_bytes());
    bytes.extend(7_i64.to_le_bytes());
    bytes.extend(0.25_f32.to_le_bytes());
    bytes.extend(0.75_f32.to_le_bytes());
    let dir = data.join("voices").join("clones");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{id}.bin")), bytes).unwrap();
}

fn sample(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("sample.wav");
    std::fs::write(&path, b"not really audio").unwrap();
    path
}

#[test]
fn cloning_without_the_permission_flag_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = sample(&dir);
    vtt(&dir)
        .args(["voice", "clone", "Anna", "--from"])
        .arg(&file)
        .assert()
        .code(2)
        .stderr(contains("--i-have-permission"));
}

#[test]
fn a_missing_recording_is_an_input_error() {
    let dir = tempfile::tempdir().unwrap();
    vtt(&dir)
        .args([
            "voice",
            "clone",
            "Anna",
            "--i-have-permission",
            "--from",
            "/no/such/file.wav",
        ])
        .assert()
        .code(3)
        .stderr(contains("cannot read audio"));
}

#[test]
fn a_name_without_letters_is_an_input_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = sample(&dir);
    vtt(&dir)
        .args(["voice", "clone", "!!!", "--i-have-permission", "--from"])
        .arg(&file)
        .assert()
        .code(3);
}

#[test]
fn without_the_model_cloning_says_how_to_install_it() {
    let dir = tempfile::tempdir().unwrap();
    let file = sample(&dir);
    vtt(&dir)
        .args(["voice", "clone", "Anna", "--i-have-permission", "--from"])
        .arg(&file)
        .assert()
        .code(4)
        .stderr(contains("vtt models pull chatterbox-multilingual"));
}

#[test]
fn a_cloned_voice_is_listed_as_cloned_and_deleting_removes_its_file() {
    let dir = tempfile::tempdir().unwrap();
    fake_clone(dir.path(), "clone_anna");
    vtt(&dir)
        .args(["voice", "list"])
        .assert()
        .success()
        .stdout(contains("clone_anna").and(contains("[cloned]")));
    let json = vtt(&dir)
        .args(["--json", "voice", "list"])
        .output()
        .unwrap();
    let voices: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    let clone = voices
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == "clone_anna")
        .unwrap();
    assert_eq!(clone["cloned"], true);
    vtt(&dir)
        .args(["voice", "delete", "clone_anna"])
        .assert()
        .success();
    assert!(!dir.path().join("voices/clones/clone_anna.bin").exists());
    vtt(&dir)
        .args(["voice", "delete", "clone_anna"])
        .assert()
        .code(3);
}

#[test]
fn speaking_with_a_cloned_voice_needs_the_voice_and_the_model() {
    let dir = tempfile::tempdir().unwrap();
    vtt(&dir)
        .args(["speak", "--voice", "clone_nobody", "hello"])
        .assert()
        .code(3)
        .stderr(contains("unknown cloned voice"));
    fake_clone(dir.path(), "clone_anna");
    vtt(&dir)
        .args(["speak", "--voice", "clone_anna", "--language", "es", "hola"])
        .assert()
        .code(4)
        .stderr(contains("vtt models pull chatterbox-multilingual"));
}
