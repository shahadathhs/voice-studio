//! A record of the jobs that have not finished, kept on disk so a crash or a quit does not lose
//! the queue. It is rewritten (to a temporary file, then renamed, so it is never half written)
//! whenever the unfinished jobs change, and read when the queue starts.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::JobSpec;

/// A job that was started this many times without finishing is not started again: if it is what
/// brings the app down, restarting it would only bring it down again.
pub(super) const MAX_ATTEMPTS: u8 = 2;

/// One unfinished job as saved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct Saved {
    pub spec: JobSpec,
    /// How many times a worker has started it.
    pub attempts: u8,
}

/// Replaces the saved jobs. A failure to write is ignored: the queue still works, it just would
/// not survive a restart.
pub(super) fn save(path: &Path, jobs: &[Saved]) {
    let Ok(text) = serde_json::to_string(jobs) else {
        return;
    };
    let temporary = path.with_extension("json.tmp");
    if std::fs::write(&temporary, text).is_ok() {
        let _ = std::fs::rename(&temporary, path);
    }
}

/// The saved jobs, or none when there is no usable file.
pub(super) fn load(path: &Path) -> Vec<Saved> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(name: &str, attempts: u8) -> Saved {
        Saved {
            spec: JobSpec {
                file: name.into(),
                model_id: "whisper-base".into(),
                language: Some("en".into()),
                translate: false,
                speakers: None,
            },
            attempts,
        }
    }

    #[test]
    fn saved_jobs_are_read_back_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.json");
        let jobs = vec![saved("a.wav", 0), saved("b.wav", 1)];
        save(&path, &jobs);
        assert_eq!(load(&path), jobs);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn a_missing_or_damaged_file_is_an_empty_queue() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.json");
        assert_eq!(load(&path).len(), 0);
        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(load(&path).len(), 0);
    }
}
