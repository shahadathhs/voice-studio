//! Downloader behavior against a real HTTP server on the loopback interface (no internet needed).
#![allow(clippy::unwrap_used)]

use vtt_core::Error;
use vtt_core::engines::Control;
use vtt_core::models::{ModelFile, download_file};
use vtt_core::testing::{TestServer, file_for, sample_data};

#[test]
fn downloads_and_verifies_a_file() {
    let data = sample_data();
    let server = TestServer::start(data.clone());
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("m").join("model.bin");

    let mut last = 0;
    download_file(&file_for(&server, &data), &dest, &mut |p| {
        assert!(p.downloaded >= last, "progress never goes backwards");
        last = p.downloaded;
        Control::Continue
    })
    .unwrap();

    assert_eq!(std::fs::read(&dest).unwrap(), data);
    assert_eq!(
        last,
        data.len() as u64,
        "final progress report is the full size"
    );
    assert!(!dir.path().join("m").join("model.bin.part").exists());
}

#[test]
fn resumes_a_partial_download_with_a_range_request() {
    let data = sample_data();
    let server = TestServer::start(data.clone());
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("model.bin");
    std::fs::write(dir.path().join("model.bin.part"), &data[..250_000]).unwrap();

    download_file(&file_for(&server, &data), &dest, &mut |_| Control::Continue).unwrap();

    assert_eq!(std::fs::read(&dest).unwrap(), data);
    assert_eq!(*server.range_starts.lock().unwrap(), vec![Some(250_000)]);
}

#[test]
fn a_checksum_mismatch_fails_and_leaves_no_files_behind() {
    let data = sample_data();
    let server = TestServer::start(data.clone());
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("model.bin");
    let mut file = file_for(&server, &data);
    file.sha256 = "0".repeat(64);

    let error = download_file(&file, &dest, &mut |_| Control::Continue).unwrap_err();

    assert!(
        matches!(error, Error::Network(ref m) if m.contains("checksum")),
        "{error}"
    );
    assert!(!dest.exists());
    assert!(!dir.path().join("model.bin.part").exists());
}

#[test]
fn cancelling_keeps_the_partial_file_so_the_next_attempt_can_resume() {
    let data = sample_data();
    let server = TestServer::start(data.clone());
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("model.bin");
    let file = file_for(&server, &data);

    let error = download_file(&file, &dest, &mut |p| {
        if p.downloaded > 0 {
            Control::Cancel
        } else {
            Control::Continue
        }
    })
    .unwrap_err();
    assert!(matches!(error, Error::Cancelled));
    assert!(!dest.exists());
    let partial = std::fs::metadata(dir.path().join("model.bin.part"))
        .unwrap()
        .len();
    assert!(partial > 0 && partial < data.len() as u64);

    download_file(&file, &dest, &mut |_| Control::Continue).unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
}

#[test]
fn an_unreachable_server_is_a_network_error() {
    let dir = tempfile::tempdir().unwrap();
    let file = ModelFile {
        name: "model.bin".into(),
        url: "http://127.0.0.1:9/model.bin".into(),
        sha256: "0".repeat(64),
        size_bytes: 10,
    };
    let error = download_file(&file, &dir.path().join("model.bin"), &mut |_| {
        Control::Continue
    })
    .unwrap_err();
    assert!(matches!(error, Error::Network(_)), "{error}");
}
