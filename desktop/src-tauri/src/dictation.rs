//! Dictation: press a shortcut anywhere, speak, press it again, and the words are copied to the
//! clipboard, ready to paste. A small floating window shows what is happening. Nothing is typed
//! into other programs for you: that needs accessibility permissions on every system, so the
//! text is left on the clipboard instead.

use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use vtt_core::audio::record::Recorder;
use vtt_core::engines::{Control, SttOptions, WhisperEngine};
use vtt_core::paths::Paths;
use vtt_core::repetition::transcribe_without_loops;
use vtt_core::settings::Settings;
use vtt_core::{Error, Result, Transcript};

use crate::recording::live_engine;

const STATE_EVENT: &str = "dictation-state";
const WINDOW: &str = "dictation";
/// Anything shorter than this is a mistaken key press, not speech.
const MIN_MS: u64 = 400;

static SESSION: Mutex<Option<Recorder>> = Mutex::new(None);
static ENGINE: Mutex<Option<WhisperEngine>> = Mutex::new(None);

/// Mirrored by `DictationState` in `desktop/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
struct StateEvent {
    /// `listening`, `working`, `done` or `failed`.
    state: &'static str,
    text: Option<String>,
    message: Option<String>,
}

fn lock<T>(mutex: &'static Mutex<T>) -> MutexGuard<'static, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn send(app: &AppHandle, state: &'static str, text: Option<String>, message: Option<String>) {
    let _ = app.emit(
        STATE_EVENT,
        StateEvent {
            state,
            text,
            message,
        },
    );
}

/// All the words of a transcript on one line.
pub(crate) fn plain_text(transcript: &Transcript) -> String {
    transcript
        .segments
        .iter()
        .map(|s| s.text.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Shows the floating window, creating it the first time.
fn show_pill(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.show();
        return;
    }
    let built =
        WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("index.html#dictation".into()))
            .title("Dictation")
            .inner_size(300.0, 64.0)
            .decorations(false)
            .always_on_top(true)
            .resizable(false)
            .skip_taskbar(true)
            .focused(false)
            .build();
    if let Err(error) = built {
        send(app, "failed", None, Some(error.to_string()));
    }
}

fn hide_pill(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.hide();
    }
}

fn transcribe(paths: &Paths, audio: &vtt_core::engines::AudioBuffer) -> Result<String> {
    let settings = Settings::load(paths)?;
    let mut engine = lock(&ENGINE);
    if engine.is_none() {
        *engine = Some(live_engine(paths, None)?);
    }
    let engine = engine
        .as_mut()
        .ok_or_else(|| Error::Engine("no speech model loaded".into()))?;
    let options = SttOptions {
        language: settings.language,
        vad: true,
        ..SttOptions::default()
    };
    let transcript =
        transcribe_without_loops(engine, audio, &options, &mut |_, _| Control::Continue)?;
    Ok(plain_text(&transcript))
}

/// Stops listening, turns the speech into text and puts it on the clipboard.
fn finish(app: &AppHandle, recorder: Recorder) {
    send(app, "working", None, None);
    let paths = Paths::resolve(None);
    let result = recorder.stop().and_then(|audio| {
        if audio.duration_ms() < MIN_MS {
            return Err(Error::Input("that was too short to hear".into()));
        }
        transcribe(&paths, &audio)
    });
    match result {
        Ok(text) if text.is_empty() => send(app, "failed", None, Some("no speech heard".into())),
        Ok(text) => match app.clipboard().write_text(text.clone()) {
            Ok(()) => send(app, "done", Some(text), None),
            Err(error) => send(app, "failed", Some(text), Some(error.to_string())),
        },
        Err(error) => send(app, "failed", None, Some(error.to_string())),
    }
}

/// Starts listening, or stops and transcribes if it is already listening.
pub fn toggle(app: &AppHandle) {
    let running = lock(&SESSION).take();
    match running {
        Some(recorder) => {
            let app = app.clone();
            std::thread::spawn(move || {
                finish(&app, recorder);
                std::thread::sleep(std::time::Duration::from_secs(3));
                hide_pill(&app);
            });
        }
        None => match Recorder::start(None) {
            Ok(recorder) => {
                *lock(&SESSION) = Some(recorder);
                show_pill(app);
                send(app, "listening", None, None);
            }
            Err(error) => {
                show_pill(app);
                send(app, "failed", None, Some(error.to_string()));
            }
        },
    }
}

/// The key combination that starts and stops dictation from anywhere.
const SHORTCUT: &str = "CommandOrControl+Shift+Space";

/// Registers the global shortcut. This can fail (another program has the keys, or the desktop
/// does not allow global shortcuts, as on Wayland); the tray menu still works then.
pub fn register(app: &AppHandle) {
    let registered = app
        .global_shortcut()
        .on_shortcut(SHORTCUT, |app, _, event| {
            if event.state == ShortcutState::Pressed {
                toggle(app);
            }
        });
    if let Err(error) = registered {
        eprintln!("dictation shortcut {SHORTCUT} not available: {error}");
    }
}

#[tauri::command]
pub fn toggle_dictation(app: AppHandle) {
    std::thread::spawn(move || toggle(&app));
}

#[cfg(test)]
mod tests {
    use vtt_core::Segment;

    use super::*;

    #[test]
    fn the_words_of_all_lines_are_joined_without_blanks() {
        let line = |text: &str| Segment {
            start_ms: 0,
            end_ms: 1,
            speaker: None,
            text: text.into(),
        };
        let transcript = Transcript {
            segments: vec![line(" Hello "), line("  "), line("there.")],
            ..Transcript::default()
        };
        assert_eq!(plain_text(&transcript), "Hello there.");
    }
}
