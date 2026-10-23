//! Speech behaviour with a fake installed model; the statics are shared, so tests run one at a time.

use std::sync::Mutex;

use super::*;

/// The statics are shared by every test, so tests that touch them run one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}

fn request(text: &str) -> Request {
    Request {
        text: text.into(),
        voice: "af_heart".into(),
        speed: 1.0,
        translate: false,
    }
}

/// Makes the speech model count as installed without downloading it: files of the right size.
fn fake_install(paths: &Paths) {
    let store = ModelStore::new(paths);
    let catalog = Catalog::bundled().unwrap();
    let entry = catalog.find(SPEECH_MODEL_ID).unwrap();
    std::fs::create_dir_all(store.model_dir(entry)).unwrap();
    for file in &entry.files {
        let handle = std::fs::File::create(store.file_path(entry, file)).unwrap();
        handle.set_len(file.size_bytes).unwrap();
    }
}

#[test]
fn nothing_to_say_and_a_missing_model_are_rejected_before_anything_starts() {
    let _guard = serial();
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    let app = tauri::test::mock_app();

    let empty = start(app.handle().clone(), paths.clone(), request("   "));
    assert!(matches!(empty, Err(Error::Input(_))));
    let missing = start(app.handle().clone(), paths, request("Hello"));
    assert!(matches!(missing, Err(Error::ModelMissing(id)) if id == SPEECH_MODEL_ID));
    assert!(!BUSY.load(Ordering::SeqCst));
}

#[test]
fn a_second_request_while_one_is_running_is_refused() {
    let _guard = serial();
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    fake_install(&paths);
    let app = tauri::test::mock_app();

    BUSY.store(true, Ordering::SeqCst);
    let result = start(app.handle().clone(), paths, request("Hello"));
    BUSY.store(false, Ordering::SeqCst);

    assert!(matches!(result, Err(Error::Input(m)) if m.contains("already")));
}

#[test]
fn voices_are_listed_only_when_the_model_is_installed() {
    let _guard = serial();
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    assert_eq!(voices(&paths).len(), 0);

    fake_install(&paths);
    let found = voices(&paths);
    assert!(
        found
            .iter()
            .any(|v| v.id == "af_heart" && v.name == "Heart")
    );
    assert!(found.iter().all(|v| !v.description.is_empty()));
}

#[test]
fn only_the_newest_speech_file_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["speech-1.wav", "speech-2.wav", "notes.txt"] {
        std::fs::write(dir.path().join(name), b"x").unwrap();
    }
    keep_only(dir.path(), &dir.path().join("speech-2.wav"));
    let mut left: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(left, ["notes.txt", "speech-2.wav"]);
}

#[test]
fn outcomes_map_to_the_strings_the_ui_expects() {
    let done = finished(&Ok((PathBuf::from("/a/b.wav"), 1500)));
    assert_eq!(
        (done.outcome, done.path.as_deref()),
        ("done", Some("/a/b.wav"))
    );
    assert_eq!(finished(&Err(Error::Cancelled)).outcome, "cancelled");
    let failed = finished(&Err(Error::Engine("boom".into())));
    assert_eq!(
        (failed.outcome, failed.message.as_deref()),
        ("failed", Some("boom"))
    );
}

#[test]
fn text_files_are_read_and_unreadable_or_huge_ones_are_input_errors() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");
    std::fs::write(&file, "Hello there").unwrap();
    assert_eq!(read_text(&file).unwrap(), "Hello there");
    assert!(matches!(
        read_text(&dir.path().join("none.txt")),
        Err(Error::Input(_))
    ));
    std::fs::write(dir.path().join("bin"), [0xff, 0xfe, 0xfd]).unwrap();
    assert!(matches!(
        read_text(&dir.path().join("bin")),
        Err(Error::Input(_))
    ));
    let big = std::fs::File::create(dir.path().join("big.txt")).unwrap();
    big.set_len(MAX_TEXT_BYTES + 1).unwrap();
    assert!(
        matches!(read_text(&dir.path().join("big.txt")), Err(Error::Input(m)) if m.contains("too large"))
    );
}

#[test]
fn saving_before_anything_was_spoken_is_an_input_error() {
    let _guard = serial();
    *lock(&LATEST) = None;
    assert!(matches!(
        save(Path::new("/tmp/x.wav")),
        Err(Error::Input(_))
    ));
}

#[test]
fn the_latest_speech_can_be_copied_to_a_chosen_file() {
    let _guard = serial();
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("speech-1.wav");
    std::fs::write(&source, b"audio").unwrap();
    *lock(&LATEST) = Some(source);
    let target = dir.path().join("saved.wav");

    save(&target).unwrap();

    assert_eq!(std::fs::read(target).unwrap(), b"audio");
    *lock(&LATEST) = None;
}

#[test]
fn speech_saved_under_a_flac_name_is_flac_and_keeps_its_sample_rate() {
    let _guard = serial();
    let dir = tempfile::tempdir().unwrap();
    let audio = vtt_core::engines::AudioBuffer {
        samples: (0..5000_u16).map(|n| f32::from(n % 50) / 100.0).collect(),
        sample_rate: SAMPLE_RATE,
    };
    let source = dir.path().join("speech-1.wav");
    std::fs::write(&source, encode_wav(&audio)).unwrap();
    *lock(&LATEST) = Some(source);
    let target = dir.path().join("saved.flac");

    save(&target).unwrap();

    let bytes = std::fs::read(target).unwrap();
    assert_eq!(&bytes[..4], b"fLaC");
    // STREAMINFO holds the sample rate in its 20 bits after the 4 byte header and two block sizes
    // and two frame sizes (bytes 18 to 20 of the file).
    let rate =
        (u32::from(bytes[18]) << 12) | (u32::from(bytes[19]) << 4) | (u32::from(bytes[20]) >> 4);
    assert_eq!(rate, SAMPLE_RATE);
    *lock(&LATEST) = None;
}

/// Needs the real voice model: `VTT_TEST_DATA_DIR=<data dir with kokoro-82m installed>`.
#[test]
#[ignore = "needs VTT_TEST_DATA_DIR with kokoro-82m installed"]
fn speaking_in_the_background_reports_progress_and_writes_a_wav() {
    use std::sync::mpsc;
    use std::time::Duration;

    use tauri::Listener;

    let _guard = serial();
    let data = std::env::var_os("VTT_TEST_DATA_DIR")
        .map(PathBuf::from)
        .unwrap();
    let app = tauri::test::mock_app();
    let (tx, rx) = mpsc::channel();
    for name in [PROGRESS_EVENT, FINISHED_EVENT] {
        let tx = tx.clone();
        app.listen(name, move |event| {
            let _ = tx.send((name, event.payload().to_string()));
        });
    }

    start(
        app.handle().clone(),
        Paths::resolve(Some(data)),
        request("First sentence. Second sentence."),
    )
    .unwrap();
    let mut events = Vec::new();
    while let Ok(event) = rx.recv_timeout(Duration::from_secs(60)) {
        let done = event.0 == FINISHED_EVENT;
        events.push(event);
        if done {
            break;
        }
    }

    let progress = events
        .iter()
        .filter(|(name, _)| *name == PROGRESS_EVENT)
        .count();
    assert_eq!(progress, 2, "{events:?}");
    let last: serde_json::Value = serde_json::from_str(&events.last().unwrap().1).unwrap();
    assert_eq!(last["outcome"], "done", "{last}");
    let path = last["path"].as_str().unwrap();
    assert_eq!(&std::fs::read(path).unwrap()[0..4], b"RIFF");
    assert!(last["duration_ms"].as_u64().unwrap() > 1000);
    assert!(!BUSY.load(Ordering::SeqCst));
}
