//! The thread behind a [`Queue`]: runs waiting jobs one at a time, stores the transcripts in the
//! library and reports every change to the front end.

use std::sync::atomic::Ordering;
use std::sync::{Arc, PoisonError};
use std::time::Duration;

use super::queue::{Inner, JobSpec, Shared};
use super::{JobInfo, JobRunner, JobState, Queue, Throttle};
use crate::engines::Control;
use crate::library::{Library, NewItem, title_for};
use crate::paths::Paths;
use crate::{Error, Result, Transcript};

/// The unfinished jobs, in the data directory.
const JOURNAL_FILE: &str = "queue.json";
const PROGRESS_EVERY: Duration = Duration::from_millis(100);

/// Called for every job that changes state or makes progress, on the worker thread.
pub type Notify = Arc<dyn Fn(&JobInfo) + Send + Sync>;

impl Queue {
    /// Starts the worker thread. It stops when the last clone of the queue is dropped.
    pub fn start(runner: impl JobRunner, paths: Paths, notify: Notify) -> Queue {
        let queue = Queue {
            shared: Arc::default(),
        };
        let shared = Arc::clone(&queue.shared);
        // A fresh queue has no notifier yet, so this cannot fail.
        let _ = shared.notify.set(Arc::clone(&notify));
        let journal = paths.data.join(JOURNAL_FILE);
        shared.restore(&journal);
        let _ = shared.journal.set(journal);
        std::thread::spawn(move || work(&shared, runner, &paths, &notify));
        queue
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        // The worker holds the other reference, so one left means every handle is gone.
        if Arc::strong_count(&self.shared) <= 2 {
            let mut inner = self.shared.lock();
            inner.shutdown = true;
            for entry in &inner.jobs {
                entry.cancel.store(true, Ordering::Relaxed);
            }
            drop(inner);
            self.shared.wake.notify_all();
        }
    }
}

fn work(shared: &Shared, mut runner: impl JobRunner, paths: &Paths, notify: &Notify) {
    while let Some((id, spec)) = next_job(shared, notify) {
        let cancel = shared
            .lock()
            .jobs
            .iter()
            .find(|e| e.info.id == id)
            .map(|e| e.cancel.clone());
        let mut throttle = Throttle::new(PROGRESS_EVERY);
        let result = runner.run(&spec, &mut |done, total| {
            if cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed)) {
                return Control::Cancel;
            }
            if throttle.ready() || done == total {
                update(shared, id, notify, |info| {
                    info.processed_ms = done;
                    info.total_ms = total;
                });
            }
            Control::Continue
        });
        let stored = result.and_then(|transcript| store(paths, &spec, &transcript));
        finish(shared, id, stored, notify);
    }
}

/// Blocks until a waiting job may start, marks it running and returns it. `None` means shut down.
fn next_job(shared: &Shared, notify: &Notify) -> Option<(u64, JobSpec)> {
    let mut inner = shared.lock();
    loop {
        if inner.shutdown {
            return None;
        }
        if let Some(started) = start_waiting(&mut inner) {
            let (info, spec) = started;
            // Saved before the work begins, so a crash during it counts as an attempt.
            shared.persist(&inner);
            drop(inner);
            notify(&info);
            return Some((info.id, spec));
        }
        inner = shared
            .wake
            .wait(inner)
            .unwrap_or_else(PoisonError::into_inner);
    }
}

fn start_waiting(inner: &mut Inner) -> Option<(JobInfo, JobSpec)> {
    if inner.paused {
        return None;
    }
    let entry = inner
        .jobs
        .iter_mut()
        .find(|e| e.info.state == JobState::Waiting)?;
    entry.info.state = JobState::Running;
    entry.attempts = entry.attempts.saturating_add(1);
    Some((entry.info.clone(), entry.spec.clone()))
}

fn store(paths: &Paths, spec: &JobSpec, transcript: &Transcript) -> Result<i64> {
    let title = title_for(&spec.file);
    let source = std::path::absolute(&spec.file)?.display().to_string();
    let new = NewItem {
        title: &title,
        source_path: Some(&source),
    };
    Library::open(paths)?.add(transcript, &new)
}

fn update(shared: &Shared, id: u64, notify: &Notify, change: impl FnOnce(&mut JobInfo)) {
    let mut inner = shared.lock();
    let Some(entry) = inner.jobs.iter_mut().find(|e| e.info.id == id) else {
        return;
    };
    change(&mut entry.info);
    let info = entry.info.clone();
    drop(inner);
    notify(&info);
}

fn finish(shared: &Shared, id: u64, outcome: Result<i64>, notify: &Notify) {
    update(shared, id, notify, |info| match outcome {
        Ok(item_id) => {
            info.state = JobState::Done;
            info.item_id = Some(item_id);
            info.processed_ms = info.total_ms;
        }
        Err(Error::Cancelled) => info.state = JobState::Cancelled,
        Err(error) => {
            info.state = JobState::Failed;
            info.error = Some(error.to_string());
        }
    });
    shared.persist(&shared.lock());
}
