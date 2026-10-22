//! Black-box tests for the documented CLI contract (docs/product/cli.md).
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::str::contains;

fn vtt() -> Command {
    Command::cargo_bin("vtt").unwrap()
}

const TRANSCRIPT: &str = r#"{"language":"en","duration_ms":5000,"model":"test",
  "segments":[{"start_ms":0,"end_ms":4000,"speaker":"Maria Chen","text":"So if we ship the importer behind a flag first"}]}"#;

#[test]
fn export_srt_to_stdout() {
    vtt()
        .args(["export", "-", "-f", "srt"])
        .write_stdin(TRANSCRIPT)
        .assert()
        .success()
        .stdout("1\n00:00:00,000 --> 00:00:04,000\nMaria Chen: So if we ship the importer\nbehind a flag first\n\n");
}

#[test]
fn export_can_hide_speaker_names() {
    vtt()
        .args(["export", "-", "-f", "txt", "--no-speaker-names"])
        .write_stdin(TRANSCRIPT)
        .assert()
        .success()
        .stdout("So if we ship the importer behind a flag first\n");
}

#[test]
fn export_writes_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("t.vtt");
    vtt()
        .args(["export", "-", "-f", "vtt", "-o"])
        .arg(&out)
        .write_stdin(TRANSCRIPT)
        .assert()
        .success()
        .stdout("");
    assert!(std::fs::read_to_string(out).unwrap().starts_with("WEBVTT"));
}

#[test]
fn malformed_json_is_exit_3_with_error_on_stderr() {
    vtt()
        .args(["export", "-"])
        .write_stdin("not json")
        .assert()
        .code(3)
        .stdout("")
        .stderr(contains("error:"));
}

#[test]
fn missing_file_is_exit_3() {
    vtt()
        .args(["export", "/definitely/not/here.json"])
        .assert()
        .code(3);
}

#[test]
fn unknown_format_is_exit_3() {
    vtt()
        .args(["export", "-", "-f", "wat"])
        .write_stdin(TRANSCRIPT)
        .assert()
        .code(3);
}

#[test]
fn docx_is_reported_as_not_implemented() {
    vtt()
        .args(["export", "-", "-f", "docx"])
        .write_stdin(TRANSCRIPT)
        .assert()
        .code(70);
}

#[test]
fn engine_commands_are_not_implemented_yet() {
    vtt().args(["transcribe", "a.wav"]).assert().code(70);
    vtt().args(["speak", "hello"]).assert().code(70);
}

#[test]
fn bad_flags_are_a_usage_error() {
    vtt().args(["transcribe"]).assert().code(2);
    vtt().args(["nonsense"]).assert().code(2);
}

#[test]
fn json_errors_are_one_json_object_on_stderr() {
    let out = vtt()
        .args(["--json", "transcribe", "a.wav"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(70));
    assert!(out.stdout.is_empty());
    let v: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(v["event"], "error");
    assert_eq!(v["code"], 70);
}

#[test]
fn models_list_json_is_valid_and_data_dir_flag_is_honoured() {
    let dir = tempfile::tempdir().unwrap();
    let out = vtt()
        .args(["--json", "models", "list", "--data-dir"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v.is_array());
}

#[test]
fn config_path_reports_the_override() {
    let dir = tempfile::tempdir().unwrap();
    let out = vtt()
        .args(["--json", "config", "path", "--data-dir"])
        .arg(dir.path())
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["data"], dir.path().to_str().unwrap());
}

#[test]
fn completions_print_a_script() {
    vtt()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("vtt"));
}

#[test]
fn models_list_shows_the_real_catalog_with_install_state() {
    let dir = tempfile::tempdir().unwrap();
    let out = vtt()
        .args(["--json", "models", "list", "--data-dir"])
        .arg(dir.path())
        .output()
        .unwrap();
    let models: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let base = models
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "whisper-base")
        .unwrap();
    assert_eq!(base["kind"], "stt");
    assert_eq!(base["installed"], false);
    assert!(base["size_bytes"].as_u64().unwrap() > 100_000_000);
}

#[test]
fn unknown_model_ids_are_exit_3() {
    for sub in ["info", "pull", "remove"] {
        vtt()
            .args(["models", sub, "no-such-model"])
            .assert()
            .code(3);
    }
}

#[test]
fn removing_a_model_that_is_not_installed_is_exit_4() {
    let dir = tempfile::tempdir().unwrap();
    vtt()
        .args(["models", "remove", "whisper-tiny", "--data-dir"])
        .arg(dir.path())
        .assert()
        .code(4)
        .stderr(contains("not installed"));
}

#[test]
fn verify_with_nothing_installed_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    vtt()
        .args(["models", "verify", "--data-dir"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn list_installed_is_empty_on_a_fresh_data_dir() {
    let dir = tempfile::tempdir().unwrap();
    vtt()
        .args(["models", "list", "--installed", "--data-dir"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout("");
}
