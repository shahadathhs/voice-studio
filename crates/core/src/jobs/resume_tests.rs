//! The queue after a restart: unfinished jobs come back, and one that keeps killing the app does not.

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::journal::{self, Saved};
use super::*;
use crate::engines::Control;
use crate::paths::Paths;
use crate::{Result, Transcript};

const WAIT: Duration = Duration::from_secs(10);

/// Remembers which files it was asked to transcribe and finishes each at once.
struct Recorder(Arc<Mutex<Vec<String>>>);

impl JobRunner for Recorder {
    fn run(
        &mut self,
        spec: &JobSpec,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript> {
        self.0.lock().unwrap().push(spec.file.display().to_string());
        on_progress(1000, 1000);
        Ok(Transcript::default())
    }
}

fn saved(name: &str, attempts: u8) -> Saved {
    Saved {
        spec: JobSpec {
            file: name.into(),
            model_id: "whisper-base".into(),
            language: None,
            translate: false,
            speakers: None,
        },
        attempts,
    }
}

struct Restart {
    queue: Queue,
    ran: Arc<Mutex<Vec<String>>>,
    events: Receiver<JobInfo>,
    paths: Paths,
    _dir: tempfile::TempDir,
}

/// Starts a queue in a data directory whose journal already holds `before`.
fn restart_with(before: &[Saved]) -> Restart {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    journal::save(&paths.data.join("queue.json"), before);
    let ran = Arc::new(Mutex::new(Vec::new()));
    let (sender, events) = mpsc::channel();
    let sender = Mutex::new(sender);
    let notify: Notify = Arc::new(move |info| {
        let _ = sender.lock().unwrap().send(info.clone());
    });
    let queue = Queue::start(Recorder(ran.clone()), paths.clone(), notify);
    Restart {
        queue,
        ran,
        events,
        paths,
        _dir: dir,
    }
}

/// Waits until `count` jobs have reached a final state.
fn wait_for_finished(restart: &Restart, count: usize) {
    let start = Instant::now();
    while restart
        .queue
        .snapshot()
        .iter()
        .filter(|j| !matches!(j.state, JobState::Waiting | JobState::Running))
        .count()
        < count
    {
        assert!(start.elapsed() < WAIT, "jobs did not finish");
        let _ = restart.events.recv_timeout(Duration::from_millis(50));
    }
}

#[test]
fn jobs_saved_before_a_restart_run_in_their_order() {
    let restart = restart_with(&[saved("first.wav", 1), saved("second.wav", 0)]);
    wait_for_finished(&restart, 2);
    assert_eq!(*restart.ran.lock().unwrap(), ["first.wav", "second.wav"]);
}

#[test]
fn a_job_that_was_started_twice_without_finishing_is_not_run_again() {
    let restart = restart_with(&[saved("crashes.wav", 2), saved("fine.wav", 0)]);
    wait_for_finished(&restart, 2);
    assert_eq!(*restart.ran.lock().unwrap(), ["fine.wav"]);
    let jobs = restart.queue.snapshot();
    let given_up = jobs.iter().find(|j| j.title == "crashes").unwrap();
    assert_eq!(given_up.state, JobState::Failed);
    assert!(given_up.error.as_deref().unwrap().contains("twice"));
}

#[test]
fn finished_jobs_leave_the_journal() {
    let restart = restart_with(&[saved("a.wav", 0)]);
    wait_for_finished(&restart, 1);
    let path = restart.paths.data.join("queue.json");
    let start = Instant::now();
    while !journal::load(&path).is_empty() {
        assert!(start.elapsed() < WAIT, "journal was not cleared");
        std::thread::sleep(Duration::from_millis(10));
    }
}
