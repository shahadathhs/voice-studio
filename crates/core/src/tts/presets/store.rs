//! Presets on disk: one JSON file per preset in the voices folder.

use std::path::Path;

use super::{Preset, is_preset_id};
use crate::tts::kokoro::{KokoroEngine, VOICE_EXTENSION};
use crate::tts::voices::{VoiceInfo, describe, describe_preset};
use crate::{Error, Result};

fn file(dir: &Path, id: &str) -> Result<std::path::PathBuf> {
    if !is_preset_id(id) {
        return Err(Error::Input(format!("{id} is not a custom voice id")));
    }
    Ok(dir.join(format!("{id}.json")))
}

/// Saves `preset` in `dir` after checking that its voices are installed in `shipped` and can be
/// blended. A preset with the same id is not replaced.
pub fn save(dir: &Path, shipped: &Path, preset: &Preset) -> Result<()> {
    let path = file(dir, &preset.id)?;
    for part in &preset.parts {
        if !shipped
            .join(format!("{}.{VOICE_EXTENSION}", part.voice))
            .is_file()
        {
            return Err(Error::Input(format!(
                "unknown voice: {} (installed: {})",
                part.voice,
                KokoroEngine::voices_in(shipped).join(", ")
            )));
        }
    }
    preset.blend(|voice| super::read_table(&shipped.join(format!("{voice}.{VOICE_EXTENSION}"))))?;
    if path.exists() {
        return Err(Error::Input(format!(
            "a voice called {} exists; delete it first",
            preset.id
        )));
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(path, serde_json::to_vec_pretty(preset)?)?;
    Ok(())
}

/// The preset with this id.
pub fn load(dir: &Path, id: &str) -> Result<Preset> {
    let bytes = std::fs::read(file(dir, id)?)
        .map_err(|_| Error::Input(format!("unknown custom voice: {id}")))?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// Removes a preset.
pub fn delete(dir: &Path, id: &str) -> Result<()> {
    std::fs::remove_file(file(dir, id)?)
        .map_err(|_| Error::Input(format!("unknown custom voice: {id}")))
}

/// All presets in `dir`, by id; files that are not valid presets are skipped.
pub fn list(dir: &Path) -> Vec<Preset> {
    let mut presets: Vec<Preset> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let id = path
                .file_name()?
                .to_str()?
                .strip_suffix(".json")?
                .to_string();
            load(dir, &id).ok().filter(|p| p.id == id)
        })
        .collect();
    presets.sort_by(|a, b| a.id.cmp(&b.id));
    presets
}

/// Shipped voices followed by custom ones, as a front end lists them. A preset whose voices are
/// no longer installed is left out.
pub fn voice_list(shipped: &Path, presets: &Path) -> Vec<VoiceInfo> {
    let installed = KokoroEngine::voices_in(shipped);
    let mut voices: Vec<VoiceInfo> = installed.iter().map(|id| describe(id)).collect();
    voices.extend(
        list(presets)
            .iter()
            .filter(|p| p.parts.iter().all(|part| installed.contains(&part.voice)))
            .map(describe_preset),
    );
    voices
}

#[cfg(test)]
mod tests {
    use super::super::STYLE_SIZE;
    use super::*;

    fn fake_voice(dir: &Path, name: &str, value: f32, rows: usize) {
        let bytes: Vec<u8> = (0..STYLE_SIZE * rows)
            .flat_map(|_| value.to_le_bytes())
            .collect();
        std::fs::write(dir.join(format!("{name}.bin")), bytes).unwrap();
    }

    fn setup() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let shipped = root.path().join("models");
        std::fs::create_dir_all(&shipped).unwrap();
        fake_voice(&shipped, "af_heart", 1.0, 3);
        fake_voice(&shipped, "af_bella", 3.0, 3);
        fake_voice(&shipped, "af_short", 3.0, 1);
        let presets = root.path().join("voices");
        (root, shipped, presets)
    }

    fn pair(a: &str, b: &str) -> Vec<(String, f32)> {
        vec![(a.to_string(), 0.6), (b.to_string(), 0.4)]
    }

    #[test]
    fn a_saved_preset_is_listed_loaded_and_deleted() {
        let (_root, shipped, presets) = setup();
        let preset = Preset::new("Warm", &pair("af_heart", "af_bella")).unwrap();
        save(&presets, &shipped, &preset).unwrap();
        assert_eq!(load(&presets, "af_my-warm").unwrap(), preset);
        let voices = voice_list(&shipped, &presets);
        let ids: Vec<&str> = voices.iter().map(|v| v.id.as_str()).collect();
        assert_eq!(ids, ["af_bella", "af_heart", "af_short", "af_my-warm"]);
        let custom = voices.last().unwrap();
        assert!(custom.custom && !voices[0].custom);
        assert_eq!(
            (custom.name.as_str(), custom.language.as_str()),
            ("Warm", "en")
        );
        assert!(
            custom.description.contains("American English"),
            "{}",
            custom.description
        );
        delete(&presets, "af_my-warm").unwrap();
        assert_eq!(list(&presets).len(), 0);
        assert!(delete(&presets, "af_my-warm").is_err());
    }

    #[test]
    fn saving_checks_the_voices_and_does_not_overwrite() {
        let (_root, shipped, presets) = setup();
        let missing = Preset::new("A", &pair("af_heart", "af_nobody")).unwrap();
        assert!(save(&presets, &shipped, &missing).is_err());
        let uneven = Preset::new("B", &pair("af_heart", "af_short")).unwrap();
        assert!(save(&presets, &shipped, &uneven).is_err());
        let ok = Preset::new("C", &pair("af_heart", "af_bella")).unwrap();
        save(&presets, &shipped, &ok).unwrap();
        assert!(save(&presets, &shipped, &ok).is_err());
    }

    #[test]
    fn ids_that_could_leave_the_folder_are_refused() {
        let (_root, _shipped, presets) = setup();
        assert!(load(&presets, "../../etc/passwd").is_err());
        assert!(delete(&presets, "af_heart").is_err());
    }
}
