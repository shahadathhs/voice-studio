//! `vtt record` and `vtt devices`: the microphone to a file, with optional live captions and a
//! transcription when it ends.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use vtt_core::audio::record::{Recorder, list_input_devices, save_recording};
use vtt_core::engines::{AudioBuffer, SttOptions, WhisperEngine};
use vtt_core::models::{Catalog, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::settings::Settings;
use vtt_core::transcribe_live::{Captions, LiveCaptions, PASS_INTERVAL, WINDOW_SECONDS};
use vtt_core::{Error, Result};

use crate::cli::{Record, Transcribe};
use crate::models_cmd;
use crate::output::Output;
use crate::transcribe_cmd::{choose_model, load_engine, run as transcribe};

const POLL: Duration = Duration::from_millis(50);
const DEFAULT_FILE: &str = "recording.wav";
/// Exit code of a second Ctrl-C, or one that arrives while a model is loading or running.
const CANCELLED: i32 = 130;

/// Reads `30`, `30s`, `2m`, `1m30s`, `1.5s` or `1h` as a length of time. For clap, so a bad value
/// is a usage error (exit 2).
pub fn parse_duration(text: &str) -> std::result::Result<Duration, String> {
    let bad = || format!("'{text}' is not a duration; try 30s, 2m or 1m30s");
    let text = text.trim();
    if let Ok(seconds) = text.parse::<f64>() {
        return positive(seconds).ok_or_else(bad);
    }
    let mut total = 0.0;
    let mut rest = text;
    while !rest.is_empty() {
        let digits = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .ok_or_else(bad)?;
        let number: f64 = rest[..digits].parse().map_err(|_| bad())?;
        let unit = rest[digits..].trim_start_matches(|c: char| c.is_ascii_alphabetic());
        let name = &rest[digits..rest.len() - unit.len()];
        total += number
            * match name {
                "h" => 3600.0,
                "m" => 60.0,
                "s" => 1.0,
                _ => return Err(bad()),
            };
        rest = unit;
    }
    positive(total).ok_or_else(bad)
}

fn positive(seconds: f64) -> Option<Duration> {
    (seconds.is_finite() && seconds > 0.0).then(|| Duration::from_secs_f64(seconds))
}

/// Lists the input devices: one name per line (the default marked), or a JSON array.
pub fn list_devices(out: Output) -> Result<()> {
    let devices = list_input_devices()?;
    if devices.is_empty() {
        return Err(Error::Input(
            "no input devices found; connect a microphone".into(),
        ));
    }
    if out.json {
        println!("{}", serde_json::to_string(&devices)?);
    }
    for device in devices.iter().filter(|_| !out.json) {
        let mark = if device.is_default { " (default)" } else { "" };
        println!("{}{mark}", device.name);
    }
    Ok(())
}

/// What the first Ctrl-C does: end the recording normally. A second one, or one outside the
/// recording, cancels the process.
struct Interrupt {
    stop: AtomicBool,
    recording: AtomicBool,
}

fn catch_ctrl_c() -> Result<Arc<Interrupt>> {
    let interrupt = Arc::new(Interrupt {
        stop: AtomicBool::new(false),
        recording: AtomicBool::new(false),
    });
    let seen = Arc::clone(&interrupt);
    ctrlc::set_handler(move || {
        if !seen.recording.load(Ordering::SeqCst) || seen.stop.swap(true, Ordering::SeqCst) {
            std::process::exit(CANCELLED);
        }
    })
    .map_err(|e| Error::Engine(format!("cannot watch for Ctrl-C: {e}")))?;
    Ok(interrupt)
}

/// The speech model for live captions, loaded before the microphone opens.
struct Live {
    engine: WhisperEngine,
    options: SttOptions,
}

fn prepare_live(args: &Transcribe, paths: &Paths, out: Output) -> Result<Live> {
    let settings = Settings::load(paths)?;
    let catalog = Catalog::bundled()?;
    let store = ModelStore::new(paths);
    let entry = choose_model(&catalog, &store, args, &settings)?;
    models_cmd::ensure_installed(entry, &store, args.yes, out)?;
    Ok(Live {
        engine: load_engine(args, &settings, entry, &store)?,
        options: SttOptions {
            language: args.language.clone().or(settings.language),
            ..SttOptions::default()
        },
    })
}

pub fn run(args: &Record, paths: &Paths, out: Output) -> Result<()> {
    if args.list_devices {
        return list_devices(out);
    }
    let file = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_FILE));
    let plain = Transcribe::plain(
        file.clone(),
        args.recognition.model.clone(),
        args.recognition.language.clone(),
        args.recognition.yes,
    );
    let interrupt = catch_ctrl_c()?;
    let live = args
        .live
        .then(|| prepare_live(&plain, paths, out))
        .transpose()?;
    let recorder = Recorder::start(args.device.as_deref())?;
    if out.show_messages() {
        eprintln!("recording; press Ctrl-C to stop");
    }
    let (audio, captions) = capture(recorder, args.duration, live, &interrupt, out)?;
    if audio.samples.is_empty() {
        return Err(Error::Input("nothing was recorded".into()));
    }
    if audio.samples.iter().all(|s| *s == 0.0) {
        eprintln!(
            "warning: the recording is completely silent; if the microphone works, allow \
             microphone access for this terminal in your system's privacy settings"
        );
    }
    save_recording(&audio, &file)?;
    if out.show_messages() {
        eprintln!(
            "saved {} ({} s)",
            file.display(),
            audio.duration_ms() / 1000
        );
    }
    if args.transcribe {
        return transcribe(&plain, paths, out);
    }
    if let Some(text) = captions.map(|c| c.full_text()).filter(|t| !t.is_empty()) {
        println!("{text}");
    }
    Ok(())
}

/// Records until Ctrl-C or `limit`, running live captions beside it when there is a model.
fn capture(
    recorder: Recorder,
    limit: Option<Duration>,
    live: Option<Live>,
    interrupt: &Interrupt,
    out: Output,
) -> Result<(AudioBuffer, Option<Captions>)> {
    let captions_stop = AtomicBool::new(false);
    let captions = std::thread::scope(|scope| {
        let worker = live.map(|mut live| {
            let monitor = recorder.monitor();
            let (stop, show) = (&captions_stop, out.show_messages());
            scope.spawn(move || {
                let session = LiveCaptions {
                    source: &monitor,
                    options: &live.options,
                    stop,
                    interval: PASS_INTERVAL,
                    window_seconds: WINDOW_SECONDS,
                };
                session.run(&mut live.engine, &mut |c| show_draft(c, show))
            })
        });
        wait(&recorder, limit, interrupt);
        captions_stop.store(true, Ordering::SeqCst);
        worker.map(std::thread::ScopedJoinHandle::join)
    });
    let captions = match captions {
        Some(Ok(Ok(captions))) => Some(captions),
        Some(Ok(Err(e))) => {
            eprintln!("warning: live captions stopped: {e}");
            None
        }
        Some(Err(_)) => {
            eprintln!("warning: live captions stopped unexpectedly");
            None
        }
        None => None,
    };
    if std::io::stderr().is_terminal() && captions.is_some() && out.show_messages() {
        eprintln!();
    }
    Ok((recorder.stop()?, captions))
}

fn wait(recorder: &Recorder, limit: Option<Duration>, interrupt: &Interrupt) {
    let monitor = recorder.monitor();
    interrupt.recording.store(true, Ordering::SeqCst);
    while !interrupt.stop.load(Ordering::SeqCst)
        && limit.is_none_or(|limit| recorder.elapsed() < limit)
        && monitor.failure().is_none()
    {
        sleep(POLL);
    }
    interrupt.recording.store(false, Ordering::SeqCst);
}

/// Draft captions on stderr: one rewritten line on a terminal, one line per update otherwise.
fn show_draft(captions: &Captions, show: bool) {
    if !show {
        return;
    }
    if std::io::stderr().is_terminal() {
        eprint!("\r\x1b[2K{}", captions.full_text());
    } else {
        eprintln!("{}", captions.full_text());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_accept_units_and_plain_seconds() {
        let secs = Duration::from_secs;
        assert_eq!(parse_duration("30s"), Ok(secs(30)));
        assert_eq!(parse_duration("30"), Ok(secs(30)));
        assert_eq!(parse_duration("2m"), Ok(secs(120)));
        assert_eq!(parse_duration("1m30s"), Ok(secs(90)));
        assert_eq!(parse_duration("1h"), Ok(secs(3600)));
        assert_eq!(parse_duration("1.5s"), Ok(Duration::from_millis(1500)));
    }

    #[test]
    fn nonsense_and_zero_are_rejected() {
        for bad in ["", "abc", "0", "0s", "-5", "5x", "m", "1m2", "10 parsecs"] {
            assert!(parse_duration(bad).is_err(), "{bad}");
        }
    }
}
