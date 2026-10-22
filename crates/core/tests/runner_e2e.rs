//! The job runner with the real speech and speaker models. Ignored by default because it needs
//! models and audio that are not in the repository. Install the models into a scratch data
//! directory and run:
//!
//! ```text
//! vtt --data-dir /tmp/vtt-data models pull whisper-base
//! vtt --data-dir /tmp/vtt-data models pull wespeaker-resnet34
//! VTT_TEST_DATA_DIR=/tmp/vtt-data VTT_TEST_CONVERSATION=/path/to/two-voices.wav \
//!     cargo test -p vtt-core --release --test runner_e2e -- --ignored
//! ```
//!
//! `VTT_TEST_CONVERSATION` must contain at least two clearly different voices taking turns.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use vtt_core::engines::{Control, Device, DiarizeOptions};
use vtt_core::jobs::{JobRunner, JobSpec, WhisperRunner};
use vtt_core::paths::Paths;

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} to run this test")))
}

fn spec(file: &Path, speakers: Option<DiarizeOptions>) -> JobSpec {
    JobSpec {
        file: file.to_path_buf(),
        model_id: "whisper-base".into(),
        language: None,
        translate: false,
        speakers,
    }
}

#[test]
#[ignore = "needs VTT_TEST_DATA_DIR and VTT_TEST_CONVERSATION"]
fn a_job_labels_speakers_only_when_asked() {
    let paths = Paths::resolve(Some(env_path("VTT_TEST_DATA_DIR")));
    let file = env_path("VTT_TEST_CONVERSATION");
    let mut runner = WhisperRunner::new(&paths, Device::Auto);

    let plain = runner
        .run(&spec(&file, None), &mut |_, _| Control::Continue)
        .unwrap();
    assert!(plain.segments.iter().all(|s| s.speaker.is_none()));

    let labelled = runner
        .run(
            &spec(&file, Some(DiarizeOptions::default())),
            &mut |_, _| Control::Continue,
        )
        .unwrap();
    let mut names: Vec<_> = labelled
        .segments
        .iter()
        .map(|s| s.speaker.clone().unwrap())
        .collect();
    names.sort();
    names.dedup();
    println!("speakers found: {names:?}");
    assert!(names.len() >= 2, "{names:?}");
    assert_eq!(labelled.segments.len(), plain.segments.len());
}
