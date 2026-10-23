//! What runs beside a recording: the level meter and timer, and the live captions.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use vtt_core::audio::record::Monitor;
use vtt_core::engines::{SttOptions, WhisperEngine};
use vtt_core::models::{Catalog, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::settings::Settings;
use vtt_core::transcribe_live::{Captions, LiveCaptions, PASS_INTERVAL, WINDOW_SECONDS};
use vtt_core::{Error, Result};

const LEVEL_EVENT: &str = "recording-level";
const CAPTIONS_EVENT: &str = "recording-captions";
const TICK: Duration = Duration::from_millis(66);

/// Mirrored by `RecordingLevel` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct LevelEvent {
    level: f32,
    elapsed_ms: u64,
    /// Set when the microphone stopped working (it was unplugged, say).
    error: Option<String>,
}

/// Mirrored by `RecordingCaptions` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct CaptionsEvent {
    #[serde(rename = "final")]
    final_text: String,
    draft: String,
    /// Set when live captions could not start or stopped.
    error: Option<String>,
}

/// The speech model for live captions: the one asked for, else the settings', else the default.
pub(crate) fn live_engine(paths: &Paths, model: Option<&str>) -> Result<WhisperEngine> {
    let settings = Settings::load(paths)?;
    let catalog = Catalog::bundled()?;
    let store = ModelStore::new(paths);
    let entry = match model.or(settings.model.as_deref()) {
        Some(id) => catalog
            .find(id)
            .ok_or_else(|| Error::Input(format!("unknown model: {id}")))?,
        None => store
            .default_stt(&catalog.models)
            .ok_or_else(|| Error::ModelMissing("whisper-base".into()))?,
    };
    if !store.is_installed(entry) {
        return Err(Error::ModelMissing(entry.id.clone()));
    }
    WhisperEngine::load(
        &store.file_path(entry, &entry.files[0]),
        &entry.id,
        settings.device,
    )
}

pub(super) fn tick_loop(app: &AppHandle, monitor: &Monitor, stop: &AtomicBool) {
    while !stop.load(Ordering::SeqCst) {
        let elapsed = u64::try_from(monitor.elapsed().as_millis()).unwrap_or(u64::MAX);
        let _ = app.emit(
            LEVEL_EVENT,
            LevelEvent {
                level: monitor.level(),
                elapsed_ms: elapsed,
                error: monitor.failure(),
            },
        );
        sleep(TICK);
    }
}

pub(super) fn caption_loop(
    app: &AppHandle,
    monitor: &Monitor,
    stop: &AtomicBool,
    setup: (WhisperEngine, SttOptions),
) {
    let (mut engine, options) = setup;
    let session = LiveCaptions {
        source: monitor,
        options: &options,
        stop,
        interval: PASS_INTERVAL,
        window_seconds: WINDOW_SECONDS,
    };
    let send = |captions: &Captions, error: Option<String>| {
        let _ = app.emit(
            CAPTIONS_EVENT,
            CaptionsEvent {
                final_text: captions.final_text.clone(),
                draft: captions.draft.clone(),
                error,
            },
        );
    };
    if let Err(error) = session.run(&mut engine, &mut |c| send(c, None)) {
        send(&Captions::default(), Some(error.to_string()));
    }
}
