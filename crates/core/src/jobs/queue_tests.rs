//! Queue behaviour, driven by a fake runner so no model is needed.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::*;
use crate::engines::Control;
use crate::library::Library;
use crate::paths::Paths;
use crate::{Error, Result, Segment, Transcript};

const WAIT: Duration = Duration::from_secs(10);

/// Finishes every job at once with one line named after the file, unless the name says otherwise.
struct FakeRunner {
    ran: Arc<Mutex<Vec<String>>>,
}

impl JobRunner for FakeRunner {
    fn run(
        &mut self,
        spec: &JobSpec,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript> {
        let name = spec.file.display().to_string();
        self.ran.lock().unwrap().push(name.clone());
        if name.contains("broken") {
            return Err(Error::Engine("decoder exploded".into()));
        }
        // A "slow" job reports progress until it is cancelled.
        while name.contains("slow") {
            if on_progress(1, 100) == Control::Cancel {
                return Err(Error::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        on_progress(1000, 1000);
        Ok(Transcript {
            segments: vec![Segment {
                start_ms: 0,
                end_ms: 1000,
                speaker: None,
                text: name,
            }],
            duration_ms: 1000,
            ..Transcript::default()
        })
    }
}

struct Fixture {
    queue: Queue,
    paths: Paths,
    ran: Arc<Mutex<Vec<String>>>,
    events: Receiver<JobInfo>,
    _dir: tempfile::TempDir,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    let ran = Arc::new(Mutex::new(Vec::new()));
    let (sender, events) = mpsc::channel();
    let sender = Mutex::new(sender);
    let notify: Notify = Arc::new(move |info| {
        let _ = sender.lock().unwrap().send(info.clone());
    });
    let runner = FakeRunner { ran: ran.clone() };
    let queue = Queue::start(runner, paths.clone(), notify);
    Fixture {
        queue,
        paths,
        ran,
        events,
        _dir: dir,
    }
}

fn spec(name: &str) -> JobSpec {
    JobSpec {
        file: PathBuf::from(name),
        model_id: "whisper-base".into(),
        language: None,
        translate: false,
    }
}

fn wait_until(queue: &Queue, done: impl Fn(&[JobInfo]) -> bool) -> Vec<JobInfo> {
    let deadline = Instant::now() + WAIT;
    loop {
        let jobs = queue.snapshot();
        if done(&jobs) || Instant::now() > deadline {
            return jobs;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn state_of(jobs: &[JobInfo], id: u64) -> JobState {
    jobs.iter().find(|j| j.id == id).unwrap().state
}

#[test]
fn a_finished_job_is_stored_in_the_library() {
    let f = fixture();
    let job = f.queue.add(spec("talks/standup.wav"));

    let jobs = wait_until(&f.queue, |jobs| jobs[0].state == JobState::Done);

    assert_eq!(jobs[0].title, "standup");
    let item_id = jobs[0].item_id.unwrap();
    let (item, transcript) = Library::open(&f.paths).unwrap().get(item_id).unwrap();
    assert_eq!(item.title, "standup");
    assert_eq!(transcript.segments[0].text, "talks/standup.wav");
    assert_eq!(job.state, JobState::Waiting);
}

#[test]
fn jobs_run_one_at_a_time_in_order() {
    let f = fixture();
    f.queue.set_paused(true);
    for name in ["a.wav", "b.wav", "c.wav"] {
        f.queue.add(spec(name));
    }
    f.queue.set_paused(false);

    wait_until(&f.queue, |jobs| {
        jobs.iter().all(|j| j.state == JobState::Done)
    });

    assert_eq!(*f.ran.lock().unwrap(), ["a.wav", "b.wav", "c.wav"]);
}

#[test]
fn a_paused_queue_starts_nothing_until_resumed() {
    let f = fixture();
    f.queue.set_paused(true);
    assert!(f.queue.is_paused());
    f.queue.add(spec("a.wav"));
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(f.queue.snapshot()[0].state, JobState::Waiting);

    f.queue.set_paused(false);
    wait_until(&f.queue, |jobs| jobs[0].state == JobState::Done);
}

#[test]
fn a_failing_job_reports_its_error_and_the_next_job_still_runs() {
    let f = fixture();
    f.queue.set_paused(true);
    f.queue.add(spec("broken.wav"));
    f.queue.add(spec("fine.wav"));
    f.queue.set_paused(false);

    let jobs = wait_until(&f.queue, |jobs| jobs[1].state == JobState::Done);

    assert_eq!(jobs[0].state, JobState::Failed);
    assert_eq!(jobs[0].error.as_deref(), Some("decoder exploded"));
    assert_eq!(Library::open(&f.paths).unwrap().list().unwrap().len(), 1);
}

#[test]
fn cancelling_a_running_job_stops_it_and_stores_nothing() {
    let f = fixture();
    let job = f.queue.add(spec("slow.wav"));
    wait_until(&f.queue, |jobs| jobs[0].state == JobState::Running);

    assert_eq!(f.queue.cancel(job.id), None);

    let jobs = wait_until(&f.queue, |jobs| jobs[0].state == JobState::Cancelled);
    assert_eq!(state_of(&jobs, job.id), JobState::Cancelled);
    assert_eq!(Library::open(&f.paths).unwrap().list().unwrap().len(), 0);
}

#[test]
fn cancelling_a_waiting_job_skips_it() {
    let f = fixture();
    f.queue.set_paused(true);
    let job = f.queue.add(spec("a.wav"));

    let changed = f.queue.cancel(job.id).unwrap();
    f.queue.set_paused(false);
    std::thread::sleep(Duration::from_millis(100));

    assert_eq!(changed.state, JobState::Cancelled);
    assert_eq!(f.ran.lock().unwrap().len(), 0);
    let announced = f.events.try_iter().last().unwrap();
    assert_eq!(announced.state, JobState::Cancelled);
}

#[test]
fn waiting_jobs_can_be_reordered_and_removed_but_a_running_one_cannot() {
    let f = fixture();
    let running = f.queue.add(spec("slow.wav"));
    wait_until(&f.queue, |jobs| jobs[0].state == JobState::Running);
    let b = f.queue.add(spec("b.wav"));
    let c = f.queue.add(spec("c.wav"));

    assert!(f.queue.move_to(c.id, 1));
    let order: Vec<u64> = f.queue.snapshot().iter().map(|j| j.id).collect();
    assert_eq!(order, [running.id, c.id, b.id]);
    assert!(!f.queue.move_to(running.id, 2));
    assert!(!f.queue.remove(running.id));
    assert!(f.queue.remove(b.id));

    f.queue.cancel(running.id);
    // The job behind it starts once the running one stops, so wait until nothing is active.
    wait_until(&f.queue, |jobs| {
        jobs.iter()
            .all(|j| !matches!(j.state, JobState::Waiting | JobState::Running))
    });
    f.queue.clear_finished();
    assert_eq!(f.queue.snapshot().len(), 0);
}

#[test]
fn every_change_is_announced_to_the_front_end() {
    let f = fixture();
    f.queue.add(spec("a.wav"));

    let mut states = Vec::new();
    while let Ok(info) = f.events.recv_timeout(WAIT) {
        states.push(info.state);
        if info.state == JobState::Done {
            break;
        }
    }

    assert_eq!(states.first(), Some(&JobState::Running));
    assert_eq!(states.last(), Some(&JobState::Done));
}
