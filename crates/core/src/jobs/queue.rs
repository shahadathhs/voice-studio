//! The job queue: files waiting to be transcribed, one at a time, in order. The queue owns the
//! list and its states; [`super::worker`] is the thread that works through it.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use super::JobState;
use crate::library::title_for;

/// What to transcribe and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSpec {
    pub file: PathBuf,
    pub model_id: String,
    /// `None` means auto-detect.
    pub language: Option<String>,
    pub translate: bool,
}

/// What a front end shows for one job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JobInfo {
    pub id: u64,
    pub file: String,
    /// File name without its extension; also the title of the library item.
    pub title: String,
    pub model: String,
    pub state: JobState,
    pub processed_ms: u64,
    pub total_ms: u64,
    pub error: Option<String>,
    /// Library id of the finished transcript.
    pub item_id: Option<i64>,
}

pub(super) struct Entry {
    pub info: JobInfo,
    pub spec: JobSpec,
    pub cancel: Arc<AtomicBool>,
}

#[derive(Default)]
pub(super) struct Inner {
    pub jobs: Vec<Entry>,
    pub next_id: u64,
    pub paused: bool,
    pub shutdown: bool,
}

/// State shared between the [`Queue`] handle and its worker thread.
#[derive(Default)]
pub(super) struct Shared {
    pub inner: Mutex<Inner>,
    pub wake: Condvar,
}

impl Shared {
    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Inner {
    fn entry(&mut self, id: u64) -> Option<&mut Entry> {
        self.jobs.iter_mut().find(|e| e.info.id == id)
    }
}

/// Handle to the queue. Cheap to clone; every clone controls the same jobs.
#[derive(Clone)]
pub struct Queue {
    pub(super) shared: Arc<Shared>,
}

impl Queue {
    /// Appends a job and returns it; the worker picks it up when it is its turn.
    pub fn add(&self, spec: JobSpec) -> JobInfo {
        let mut inner = self.shared.lock();
        inner.next_id += 1;
        let info = JobInfo {
            id: inner.next_id,
            file: spec.file.display().to_string(),
            title: title_for(&spec.file),
            model: spec.model_id.clone(),
            state: JobState::Waiting,
            processed_ms: 0,
            total_ms: 0,
            error: None,
            item_id: None,
        };
        inner.jobs.push(Entry {
            info: info.clone(),
            spec,
            cancel: Arc::default(),
        });
        drop(inner);
        self.shared.wake.notify_all();
        info
    }

    /// All jobs in queue order.
    pub fn snapshot(&self) -> Vec<JobInfo> {
        self.shared
            .lock()
            .jobs
            .iter()
            .map(|e| e.info.clone())
            .collect()
    }

    /// Stops a job: a waiting job is cancelled at once, a running one at its next progress check.
    /// Returns the job if its state changed immediately.
    pub fn cancel(&self, id: u64) -> Option<JobInfo> {
        let mut inner = self.shared.lock();
        let entry = inner.entry(id)?;
        entry.cancel.store(true, Ordering::Relaxed);
        if entry.info.state != JobState::Waiting {
            return None;
        }
        entry.info.state = JobState::Cancelled;
        Some(entry.info.clone())
    }

    /// Takes a job off the list. A running job cannot be removed; cancel it first.
    pub fn remove(&self, id: u64) -> bool {
        let mut inner = self.shared.lock();
        let before = inner.jobs.len();
        inner
            .jobs
            .retain(|e| e.info.id != id || e.info.state == JobState::Running);
        inner.jobs.len() < before
    }

    /// Drops every job that has finished, failed or been cancelled.
    pub fn clear_finished(&self) {
        let active = |e: &Entry| matches!(e.info.state, JobState::Waiting | JobState::Running);
        self.shared.lock().jobs.retain(active);
    }

    /// Moves a waiting job to `position` in the list (clamped). Returns whether it moved.
    pub fn move_to(&self, id: u64, position: usize) -> bool {
        let mut inner = self.shared.lock();
        let Some(from) = inner.jobs.iter().position(|e| e.info.id == id) else {
            return false;
        };
        if inner.jobs[from].info.state != JobState::Waiting {
            return false;
        }
        let entry = inner.jobs.remove(from);
        let to = position.min(inner.jobs.len());
        inner.jobs.insert(to, entry);
        true
    }

    /// While paused, the running job finishes but no waiting job starts.
    pub fn set_paused(&self, paused: bool) {
        self.shared.lock().paused = paused;
        self.shared.wake.notify_all();
    }

    pub fn is_paused(&self) -> bool {
        self.shared.lock().paused
    }
}
