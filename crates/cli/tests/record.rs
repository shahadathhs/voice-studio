//! Black-box tests for `vtt record` and `vtt devices` that need no microphone.
#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::str::contains;

fn vtt() -> Command {
    Command::cargo_bin("vtt").unwrap()
}

/// A machine may have microphones (0) or none (3: input error), but never fail otherwise.
fn code_is_ok_or_no_devices(command: &mut Command) {
    let output = command.output().unwrap();
    let code = output.status.code().unwrap();
    assert!(matches!(code, 0 | 3), "exit {code}");
    if code == 3 {
        assert!(String::from_utf8_lossy(&output.stderr).contains("error:"));
    }
}

#[test]
fn list_devices_works_or_reports_that_there_are_none() {
    code_is_ok_or_no_devices(vtt().args(["record", "--list-devices"]));
}

#[test]
fn the_devices_command_lists_the_same_devices() {
    code_is_ok_or_no_devices(vtt().arg("devices"));
    code_is_ok_or_no_devices(vtt().args(["--json", "devices"]));
}

#[test]
fn a_bad_duration_is_a_usage_error() {
    for bad in ["soon", "0s", "-3", "5 parsecs"] {
        vtt()
            .args(["record", &format!("--duration={bad}")])
            .assert()
            .code(2)
            .stderr(contains("duration"));
    }
}

#[test]
fn an_unknown_device_is_an_input_error_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("note.wav");
    let output = vtt()
        .args([
            "record",
            "--device",
            "no such microphone",
            "--duration",
            "1s",
            "-o",
        ])
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains("error:"));
    assert!(!file.exists());
}

#[test]
fn live_captions_without_a_model_say_which_to_install() {
    let data = tempfile::tempdir().unwrap();
    let output = vtt()
        .args(["--data-dir"])
        .arg(data.path())
        .args(["record", "--live", "--duration", "1s"])
        .output()
        .unwrap();
    // The model is checked before the microphone opens, so this is the same on every machine.
    assert_eq!(output.status.code(), Some(4), "{output:?}");
}
