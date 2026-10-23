//! `vtt config` and the way saved settings become defaults (docs/product/cli.md).
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

fn vtt(data: &tempfile::TempDir) -> Command {
    let mut command = Command::cargo_bin("vtt").unwrap();
    command.arg(format!("--data-dir={}", data.path().display()));
    command
}

#[test]
fn a_new_setting_is_listed_and_can_be_read_back() {
    let data = tempfile::tempdir().unwrap();
    vtt(&data)
        .args(["config", "set", "language", "es"])
        .assert()
        .success();
    vtt(&data)
        .args(["config", "get", "language"])
        .assert()
        .success()
        .stdout("es\n");
    vtt(&data)
        .args(["config", "list"])
        .assert()
        .success()
        .stdout(contains("language = es"))
        .stdout(contains("device = auto"))
        .stdout(contains("identify-speakers = false"));
}

#[test]
fn list_as_json_is_one_object() {
    let data = tempfile::tempdir().unwrap();
    let out = vtt(&data)
        .args(["--json", "config", "list"])
        .output()
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["model"], "auto");
}

#[test]
fn unknown_keys_and_bad_values_are_input_errors() {
    let data = tempfile::tempdir().unwrap();
    let cases: [&[&str]; 3] = [
        &["config", "set", "colour", "red"],
        &["config", "set", "device", "tpu"],
        &["config", "get", "colour"],
    ];
    for args in cases {
        vtt(&data).args(args).assert().code(3);
    }
}

#[test]
fn the_saved_model_is_used_when_no_flag_names_one() {
    let data = tempfile::tempdir().unwrap();
    let audio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/tests/fixtures/tone.wav");
    vtt(&data)
        .args(["config", "set", "model", "nonsense"])
        .assert()
        .success();
    vtt(&data)
        .args(["--quiet", "transcribe"])
        .arg(&audio)
        .assert()
        .code(3)
        .stderr(contains("unknown model: nonsense"));
}

#[test]
fn a_damaged_settings_file_is_reported_with_its_name() {
    let data = tempfile::tempdir().unwrap();
    std::fs::write(data.path().join("config.toml"), "device = 7").unwrap();
    vtt(&data)
        .args(["config", "list"])
        .assert()
        .code(3)
        .stderr(contains("config.toml"));
}
