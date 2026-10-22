//! Job queue types. The queue itself (pause, cancel, checkpoint/resume) lands in phase 1;
//! these are the events front ends will subscribe to. The CLI prints them as NDJSON with `--json`.

mod throttle;

pub use throttle::Throttle;

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
