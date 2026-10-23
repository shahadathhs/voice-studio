//! Cloned voices for the Speak screen: make one from a recording (with the person's consent),
//! speak with it, delete it. The large Chatterbox model is kept loaded between requests.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use vtt_core::audio::load_at_rate;
use vtt_core::engines::{AudioBuffer, Control};
use vtt_core::models::{Catalog, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::tts::VoiceInfo;
use vtt_core::tts::clone::{
    self, CloneEngine, Consent, MODEL_ID, SAMPLE_RATE, VoiceMeta, clones_dir,
};
use vtt_core::{Error, Result};

use crate::commands::CommandError;

static ENGINE: Mutex<Option<CloneEngine>> = Mutex::new(None);

/// The cloned voices, marked as such, for the voice list.
pub fn listed(paths: &Paths) -> Vec<VoiceInfo> {
    clone::list(&clones_dir(&paths.voices_dir()))
        .iter()
        .map(VoiceMeta::info)
        .collect()
}

/// Runs `job` with the cloning model, loading it first if this is the first use.
fn with_engine<T>(paths: &Paths, job: impl FnOnce(&mut CloneEngine) -> Result<T>) -> Result<T> {
    let mut guard = ENGINE.lock().unwrap_or_else(PoisonError::into_inner);
    if guard.is_none() {
        let catalog = Catalog::bundled()?;
        let entry = catalog
            .find(MODEL_ID)
            .ok_or_else(|| Error::ModelMissing(MODEL_ID.into()))?;
        let store = ModelStore::new(paths);
        if !store.is_installed(entry) {
            return Err(Error::ModelMissing(MODEL_ID.into()));
        }
        *guard = Some(CloneEngine::load(&store.model_dir(entry))?);
    }
    let engine = guard
        .as_mut()
        .ok_or_else(|| Error::Engine("no cloning model loaded".into()))?;
    job(engine)
}

/// Speaks `text` in the cloned voice `id`; `language` is a code such as `es`.
pub fn speak(
    paths: &Paths,
    id: &str,
    text: &str,
    language: &str,
    on_progress: &mut dyn FnMut(usize, usize) -> Control,
) -> Result<AudioBuffer> {
    let voice = clone::load(&clones_dir(&paths.voices_dir()), id)?;
    with_engine(paths, |engine| {
        engine.speak(&voice, text, language, on_progress)
    })
}

/// Makes and saves a voice from the recording at `file`. `consent` is the ticked checkbox.
fn make(paths: &Paths, name: &str, file: &Path, consent: bool) -> Result<VoiceInfo> {
    let consent = Consent::confirm(consent)?;
    let id = clone::id_for(name)?;
    let dir = clones_dir(&paths.voices_dir());
    if dir.join(format!("{id}.bin")).exists() {
        return Err(Error::Input(format!(
            "a voice called {id} exists; delete it first"
        )));
    }
    let audio = load_at_rate(file, SAMPLE_RATE)?;
    let voice = with_engine(paths, |engine| {
        engine.voice_from_audio(&audio, name, consent)
    })?;
    clone::save(&dir, &voice)?;
    Ok(voice.meta.info())
}

/// Clones a voice from a recording. Takes a few seconds, the first time much longer while the model
/// loads, so it runs off the main thread.
#[tauri::command]
pub async fn clone_voice(
    name: String,
    path: String,
    consent: bool,
) -> std::result::Result<VoiceInfo, CommandError> {
    let made = tauri::async_runtime::spawn_blocking(move || {
        make(&Paths::resolve(None), &name, Path::new(&path), consent)
    })
    .await
    .map_err(|e| Error::Engine(e.to_string()))?;
    Ok(made?)
}

/// Deletes a cloned voice and its file.
#[tauri::command]
pub fn delete_clone(id: &str) -> std::result::Result<(), CommandError> {
    Ok(clone::delete(
        &clones_dir(&Paths::resolve(None).voices_dir()),
        id,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(dir: &tempfile::TempDir) -> Paths {
        Paths {
            data: dir.path().to_path_buf(),
            config: dir.path().join("config.toml"),
        }
    }

    #[test]
    fn a_voice_cannot_be_made_without_consent() {
        let dir = tempfile::tempdir().unwrap();
        let result = make(&paths(&dir), "Anna", Path::new("a.wav"), false);
        assert!(matches!(result, Err(Error::Input(m)) if m.contains("consent")));
    }

    #[test]
    fn without_the_model_cloning_and_speaking_say_it_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths(&dir);
        let file = dir.path().join("a.wav");
        std::fs::write(&file, b"x").unwrap();
        // Decoding comes first, so a file that is not audio is the error here.
        assert!(matches!(
            make(&paths, "Anna", &file, true),
            Err(Error::Input(_))
        ));
        let result = speak(&paths, "clone_anna", "hola", "es", &mut |_, _| {
            Control::Continue
        });
        assert!(matches!(result, Err(Error::Input(m)) if m.contains("unknown cloned voice")));
        assert_eq!(listed(&paths).len(), 0);
    }

    #[test]
    fn deleting_an_unknown_clone_is_an_error() {
        assert!(clone::delete(&clones_dir(Path::new("/no/such")), "clone_x").is_err());
    }
}
