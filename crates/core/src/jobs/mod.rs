//! Jobs: the events front ends subscribe to (the CLI prints them as NDJSON with `--json`) and the
//! queue that runs transcriptions one after another for the desktop app.

mod journal;
mod queue;
mod runner;
mod throttle;
mod worker;

#[cfg(test)]
mod queue_tests;
#[cfg(test)]
mod resume_tests;

pub use queue::{JobInfo, JobSpec, Queue};
pub use runner::{JobRunner, WhisperRunner};
pub use throttle::Throttle;
pub use worker::Notify;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Waiting,
    Running,
    Paused,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    Start {
        file: String,
        model: String,
        duration_ms: u64,
    },
    Progress {
        processed_ms: u64,
        total_ms: u64,
        speed: f32,
    },
    Segment {
        start_ms: u64,
        end_ms: u64,
        speaker: Option<String>,
        text: String,
    },
    Download {
        id: String,
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    Transcript {
        transcript: crate::Transcript,
    },
    Done {
        output: Option<String>,
        elapsed_ms: u64,
    },
    Error {
        code: u8,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_event_serializes_with_event_tag() {
        let e = Event::Progress {
            processed_ms: 10,
            total_ms: 100,
            speed: 4.0,
        };
        let v: serde_json::Value = serde_json::to_value(&e).unwrap();
        assert_eq!(v["event"], "progress");
        assert_eq!(v["total_ms"], 100);
    }
}
