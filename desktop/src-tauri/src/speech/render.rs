//! Turning a speech request into a WAV file: Kokoro for the built-in and custom voices, Chatterbox
//! for cloned ones.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use vtt_core::audio::encode_wav_with_note;
use vtt_core::engines::{AudioBuffer, Control, TtsOptions};
use vtt_core::models::ModelStore;
use vtt_core::paths::Paths;
use vtt_core::tts::KokoroEngine;
use vtt_core::tts::clone::{DISCLOSURE, is_clone_id};
use vtt_core::{Error, Result};

use super::{ENGINE, FILE_PREFIX, Request, SPEECH_DIR, lock, speech_entry};
use crate::clones;

/// Deletes earlier speech files so the folder never holds more than the latest one.
pub(super) fn keep_only(dir: &Path, keep: &Path) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let is_speech = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(FILE_PREFIX));
        if is_speech && path != keep {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Speaks the request with Kokoro.
fn kokoro_audio(
    paths: &Paths,
    request: &Request,
    on_progress: &mut dyn FnMut(usize, usize) -> Control,
) -> Result<AudioBuffer> {
    let store = ModelStore::new(paths);
    let entry = speech_entry(&store)?;
    let mut engine = lock(&ENGINE);
    if engine.is_none() {
        let loaded = KokoroEngine::load(&store.model_dir(&entry), &entry.id)?;
        *engine = Some(loaded.with_presets(&paths.voices_dir()));
    }
    let engine = engine
        .as_mut()
        .ok_or_else(|| Error::Engine("no voice model loaded".into()))?;
    engine.set_draft(request.draft.clone());
    let options = TtsOptions {
        voice: request.voice.clone(),
        speed: request.speed,
    };
    engine.speak(&request.text, &options, on_progress)
}

/// Speaks the request into a new WAV in the data directory and returns its path and length. Audio
/// in a cloned voice carries the note that it is synthetic.
pub(super) fn synthesize(
    paths: &Paths,
    request: &Request,
    on_progress: &mut dyn FnMut(usize, usize) -> Control,
) -> Result<(PathBuf, u64)> {
    let cloned = is_clone_id(&request.voice);
    let audio = if cloned {
        let language = &request.language;
        clones::speak(paths, &request.voice, &request.text, language, on_progress)?
    } else {
        kokoro_audio(paths, request, on_progress)?
    };
    let dir = paths.data.join(SPEECH_DIR);
    std::fs::create_dir_all(&dir)?;
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let path = dir.join(format!("{FILE_PREFIX}{millis}.wav"));
    std::fs::write(
        &path,
        encode_wav_with_note(&audio, cloned.then_some(DISCLOSURE)),
    )?;
    keep_only(&dir, &path);
    let duration_ms = u64::try_from(audio.samples.len()).unwrap_or(u64::MAX) * 1000
        / u64::from(audio.sample_rate);
    Ok((path, duration_ms))
}
