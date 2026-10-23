//! Custom voices for the Speak screen: save, delete and try out a blend of shipped voices.

use serde::Deserialize;
use tauri::AppHandle;
use vtt_core::models::{Catalog, ModelStore, SPEECH_MODEL_ID};
use vtt_core::paths::Paths;
use vtt_core::tts::{Preset, VoiceInfo, delete_preset, describe_preset, save_preset};
use vtt_core::{Error, Result};

use crate::commands::CommandError;
use crate::speech;

/// One voice of a blend. Mirrored by `VoicePart` in `desktop/src/lib/types.ts`.
#[derive(Debug, Deserialize)]
pub struct VoicePart {
    voice: String,
    weight: f32,
}

fn preset(name: &str, parts: Vec<VoicePart>) -> Result<Preset> {
    let parts: Vec<(String, f32)> = parts.into_iter().map(|p| (p.voice, p.weight)).collect();
    Preset::new(name, &parts)
}

fn create(paths: &Paths, name: &str, parts: Vec<VoicePart>) -> Result<VoiceInfo> {
    let preset = preset(name, parts)?;
    let catalog = Catalog::bundled()?;
    let entry = catalog
        .find(SPEECH_MODEL_ID)
        .ok_or_else(|| Error::ModelMissing(SPEECH_MODEL_ID.into()))?;
    let shipped = ModelStore::new(paths).model_dir(entry);
    save_preset(&paths.voices_dir(), &shipped, &preset)?;
    Ok(describe_preset(&preset))
}

/// Saves a blend as a new voice and returns how the list shows it.
#[tauri::command]
pub fn create_voice(
    name: &str,
    parts: Vec<VoicePart>,
) -> std::result::Result<VoiceInfo, CommandError> {
    Ok(create(&Paths::resolve(None), name, parts)?)
}

#[tauri::command]
pub fn delete_voice(id: &str) -> std::result::Result<(), CommandError> {
    Ok(delete_preset(&Paths::resolve(None).voices_dir(), id)?)
}

/// Speaks a sample sentence in the blend without saving it; follow it with the speech events.
#[tauri::command]
pub fn preview_voice(
    app: AppHandle,
    name: &str,
    parts: Vec<VoicePart>,
) -> std::result::Result<(), CommandError> {
    let draft = preset(name, parts)?;
    let text = draft
        .language()
        .map(vtt_core::tts::Language::sample_sentence)
        .ok_or_else(|| Error::Input("unknown voice language".into()))?;
    let request = speech::Request {
        text: text.to_string(),
        voice: draft.id.clone(),
        speed: 1.0,
        translate: false,
        draft: Some(draft),
        language: "en".into(),
    };
    Ok(speech::start(app, Paths::resolve(None), request)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(voice: &str, weight: f32) -> VoicePart {
        VoicePart {
            voice: voice.into(),
            weight,
        }
    }

    #[test]
    fn a_blend_becomes_a_custom_voice_with_normalised_weights() {
        let made = preset("Warm", vec![part("af_heart", 3.0), part("af_bella", 1.0)]).unwrap();
        assert_eq!(made.id, "af_my-warm");
        assert!((made.parts[0].weight - 0.75).abs() < 1e-6);
        assert!(preset("Warm", vec![part("af_heart", 1.0)]).is_err());
    }

    #[test]
    fn creating_needs_the_installed_voices_and_listing_and_deleting_follow() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths {
            data: dir.path().to_path_buf(),
            config: dir.path().join("config.toml"),
        };
        let parts = || vec![part("af_heart", 1.0), part("af_bella", 1.0)];
        assert!(create(&paths, "Mix", parts()).is_err());
        let catalog = Catalog::bundled().unwrap();
        let shipped = ModelStore::new(&paths).model_dir(catalog.find(SPEECH_MODEL_ID).unwrap());
        std::fs::create_dir_all(&shipped).unwrap();
        for name in ["af_heart", "af_bella"] {
            std::fs::write(shipped.join(format!("{name}.bin")), vec![0_u8; 256 * 4]).unwrap();
        }
        let info = create(&paths, "Mix", parts()).unwrap();
        assert!(info.custom);
        assert_eq!(info.id, "af_my-mix");
        delete_preset(&paths.voices_dir(), &info.id).unwrap();
    }
}
