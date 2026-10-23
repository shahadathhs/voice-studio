//! Black-box tests for `vtt voice` with fake voice files, so no model is needed.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

fn fake_voice(data: &Path, name: &str, value: f32) {
    let dir = data.join("models").join("kokoro-82m");
    std::fs::create_dir_all(&dir).unwrap();
    let bytes: Vec<u8> = (0..256 * 2).flat_map(|_| value.to_le_bytes()).collect();
    std::fs::write(dir.join(format!("{name}.bin")), bytes).unwrap();
}

fn data() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fake_voice(dir.path(), "af_heart", 1.0);
    fake_voice(dir.path(), "af_bella", 3.0);
    fake_voice(dir.path(), "ef_dora", 2.0);
    dir
}

fn vtt(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("vtt").unwrap();
    command.arg("--data-dir").arg(dir.path());
    command
}

#[test]
fn a_voice_is_created_listed_and_deleted() {
    let dir = data();
    vtt(&dir)
        .args([
            "voice",
            "create",
            "Warm",
            "--blend",
            "af_heart:0.6,af_bella:0.4",
        ])
        .assert()
        .success()
        .stdout("af_my-warm\n");
    assert!(dir.path().join("voices/af_my-warm.json").is_file());
    vtt(&dir).args(["voice", "list"]).assert().success().stdout(
        contains("af_heart")
            .and(contains("af_my-warm"))
            .and(contains("[custom]")),
    );
    vtt(&dir)
        .args(["voice", "delete", "af_my-warm"])
        .assert()
        .success();
    assert!(!dir.path().join("voices/af_my-warm.json").exists());
    vtt(&dir)
        .args(["voice", "delete", "af_my-warm"])
        .assert()
        .code(3);
}

#[test]
fn json_list_marks_custom_voices() {
    let dir = data();
    vtt(&dir)
        .args(["voice", "create", "Mix", "--blend", "af_heart:1,af_bella:1"])
        .assert()
        .success();
    vtt(&dir)
        .args(["--json", "voice", "list"])
        .assert()
        .success()
        .stdout(contains("\"custom\":true").and(contains("\"custom\":false")));
}

#[test]
fn bad_blends_are_input_errors() {
    let dir = data();
    for blend in [
        "af_heart:1",
        "af_heart:1,ef_dora:1",
        "af_heart:1,af_bella:0",
        "af_heart:1,af_nobody:1",
        "af_heart,af_bella",
        "af_heart:1,af_bella:x",
    ] {
        vtt(&dir)
            .args(["voice", "create", "Bad", "--blend", blend])
            .assert()
            .code(3);
    }
    vtt(&dir)
        .args(["voice", "create", "", "--blend", "af_heart:1,af_bella:1"])
        .assert()
        .code(3);
}

#[test]
fn a_name_cannot_be_used_twice() {
    let dir = data();
    let create = |dir: &TempDir| {
        vtt(dir)
            .args([
                "voice",
                "create",
                "Same",
                "--blend",
                "af_heart:1,af_bella:1",
            ])
            .assert()
    };
    create(&dir).success();
    create(&dir).code(3).stderr(contains("exists"));
}

#[test]
fn without_the_voice_model_creating_a_voice_exits_4() {
    let dir = tempfile::tempdir().unwrap();
    vtt(&dir)
        .args(["voice", "create", "A", "--blend", "af_heart:1,af_bella:1"])
        .assert()
        .code(4)
        .stderr(contains("vtt models pull kokoro-82m"));
}
