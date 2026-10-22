//! Speech to text with whisper.cpp, through the `whisper-rs` bindings.

use std::fmt::Display;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Once};
use std::time::Duration;

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, get_lang_str,
};

use super::{AudioBuffer, Control, SttEngine, SttOptions};
use crate::audio::WHISPER_SAMPLE_RATE;
use crate::{Error, Result, Segment, Transcript};

const POLL_EVERY: Duration = Duration::from_millis(50);
const MAX_THREADS: usize = 8;
/// whisper.cpp reports timestamps in units of 10 ms.
const TIMESTAMP_UNIT_MS: i64 = 10;

static QUIET_LOGS: Once = Once::new();

fn engine_error(e: impl Display) -> Error {
    Error::Engine(format!("whisper: {e}"))
}

/// Which hardware runs the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Device {
    /// Use the GPU when this build supports one, otherwise the CPU.
    #[default]
    Auto,
    Cpu,
    Gpu,
}

pub struct WhisperEngine {
    context: WhisperContext,
    model_id: String,
}

impl WhisperEngine {
    /// Loads a ggml Whisper model. `model_id` is only used to label results and errors.
    pub fn load(model_path: &Path, model_id: &str, device: Device) -> Result<WhisperEngine> {
        // whisper.cpp is very chatty; set VTT_WHISPER_LOG=1 to see its own log when debugging.
        if std::env::var_os("VTT_WHISPER_LOG").is_none() {
            QUIET_LOGS.call_once(whisper_rs::install_logging_hooks);
        }
        if !model_path.exists() {
            return Err(Error::ModelMissing(model_id.to_string()));
        }
        let params = WhisperContextParameters {
            use_gpu: device != Device::Cpu,
            ..WhisperContextParameters::default()
        };
        let context = WhisperContext::new_with_params(model_path, params).map_err(engine_error)?;
        Ok(WhisperEngine {
            context,
            model_id: model_id.to_string(),
        })
    }
}

impl SttEngine for WhisperEngine {
    fn transcribe(
        &mut self,
        audio: &AudioBuffer,
        options: &SttOptions,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript> {
        if audio.sample_rate != WHISPER_SAMPLE_RATE {
            return Err(Error::Engine(format!(
                "whisper needs {WHISPER_SAMPLE_RATE} Hz audio, got {} Hz",
                audio.sample_rate
            )));
        }
        let total_ms = audio.duration_ms();
        if on_progress(0, total_ms) == Control::Cancel {
            return Err(Error::Cancelled);
        }
        let percent = Arc::new(AtomicI32::new(0));
        let abort = Arc::new(AtomicBool::new(false));
        let context = &self.context;

        // Inference blocks for a long time, so it runs on its own thread while this one reports
        // progress and relays a cancel request.
        let outcome = std::thread::scope(|scope| {
            let worker = scope.spawn(|| infer(context, &audio.samples, options, &percent, &abort));
            while !worker.is_finished() {
                let done =
                    total_ms * u64::try_from(percent.load(Ordering::Relaxed)).unwrap_or(0) / 100;
                if on_progress(done, total_ms) == Control::Cancel {
                    abort.store(true, Ordering::Relaxed);
                }
                std::thread::sleep(POLL_EVERY);
            }
            worker
                .join()
                .map_err(|_| Error::Engine("whisper thread panicked".into()))
        });

        // An aborted run makes whisper.cpp return an error code; that is a cancel, not a failure.
        if abort.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let outcome = outcome??;
        on_progress(total_ms, total_ms);
        Ok(Transcript {
            segments: outcome.segments,
            language: outcome.language,
            duration_ms: total_ms,
            model: Some(self.model_id.clone()),
        })
    }
}

struct Inference {
    segments: Vec<Segment>,
    language: Option<String>,
}

fn infer(
    context: &WhisperContext,
    samples: &[f32],
    options: &SttOptions,
    percent: &Arc<AtomicI32>,
    abort: &Arc<AtomicBool>,
) -> Result<Inference> {
    let mut state = context.create_state().map_err(engine_error)?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    let threads = std::thread::available_parallelism()
        .map_or(4, usize::from)
        .min(MAX_THREADS);
    params.set_n_threads(i32::try_from(threads).unwrap_or(4));
    params.set_translate(options.translate);
    params.set_language(Some(options.language.as_deref().unwrap_or("auto")));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    let progress = Arc::clone(percent);
    params.set_progress_callback_safe(move |p: i32| progress.store(p, Ordering::Relaxed));
    let should_abort = Arc::clone(abort);
    // whisper-rs 0.16 reads the abort closure through the wrong pointer type unless it is already
    // a boxed trait object (as its progress callback is), which made "abort?" randomly true.
    let abort_check: Box<dyn FnMut() -> bool> =
        Box::new(move || should_abort.load(Ordering::Relaxed));
    params.set_abort_callback_safe::<_, Box<dyn FnMut() -> bool>>(abort_check);

    state.full(params, samples).map_err(engine_error)?;

    let mut segments = Vec::new();
    for segment in state.as_iter() {
        let text = segment.to_str_lossy().map_err(engine_error)?;
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        segments.push(Segment {
            start_ms: to_ms(segment.start_timestamp()),
            end_ms: to_ms(segment.end_timestamp()),
            speaker: None,
            text: text.to_string(),
        });
    }
    let language = get_lang_str(state.full_lang_id_from_state()).map(str::to_string);
    Ok(Inference { segments, language })
}

fn to_ms(timestamp: i64) -> u64 {
    u64::try_from(timestamp * TIMESTAMP_UNIT_MS).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_model_file_is_reported_as_not_installed() {
        let error = WhisperEngine::load(
            Path::new("/no/such/model.bin"),
            "whisper-base",
            Device::Auto,
        );
        assert!(matches!(error, Err(Error::ModelMissing(id)) if id == "whisper-base"));
    }

    #[test]
    fn a_file_that_is_not_a_model_is_an_engine_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.bin");
        std::fs::write(&path, b"not a ggml model").unwrap();
        assert!(matches!(
            WhisperEngine::load(&path, "fake", Device::Cpu),
            Err(Error::Engine(_))
        ));
    }

    #[test]
    fn timestamps_are_converted_from_centiseconds() {
        assert_eq!(to_ms(150), 1500);
        assert_eq!(to_ms(-5), 0);
    }
}
