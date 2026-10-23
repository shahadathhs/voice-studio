//! Cloned voices on disk: one `<id>.bin` file each, in a folder of their own.

use std::path::{Path, PathBuf};

use super::voice::{ClonedVoice, VoiceMeta};
use crate::tts::presets::slug;
use crate::{Error, Result};

/// Every cloned voice's id starts with this, which no built-in voice does.
pub const ID_PREFIX: &str = "clone_";
const MAX_NAME_CHARS: usize = 40;
const EXTENSION: &str = "bin";

/// Where cloned voices are kept, inside the voices folder.
pub fn clones_dir(voices_dir: &Path) -> PathBuf {
    voices_dir.join("clones")
}

/// Whether `voice` names a cloned voice rather than a built-in or custom one.
pub fn is_clone_id(voice: &str) -> bool {
    voice.strip_prefix(ID_PREFIX).is_some_and(|slug| {
        !slug.is_empty() && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

/// The id for a voice called `name`, or an input error when the name has nothing to build one from.
pub fn id_for(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(Error::Input(format!(
            "a voice name has 1 to {MAX_NAME_CHARS} characters"
        )));
    }
    let slug = slug(name);
    if slug.is_empty() {
        return Err(Error::Input(
            "a voice name needs at least one letter or digit".into(),
        ));
    }
    Ok(format!("{ID_PREFIX}{slug}"))
}

fn path_of(dir: &Path, id: &str) -> Result<PathBuf> {
    if !is_clone_id(id) {
        return Err(Error::Input(format!("{id} is not a cloned voice id")));
    }
    Ok(dir.join(format!("{id}.{EXTENSION}")))
}

/// Saves `voice` in `dir`. A voice with the same id is not replaced: delete it first.
pub fn save(dir: &Path, voice: &ClonedVoice) -> Result<()> {
    let path = path_of(dir, &voice.meta.id)?;
    if path.exists() {
        return Err(Error::Input(format!(
            "a voice called {} exists; delete it first",
            voice.meta.id
        )));
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(path, voice.to_bytes()?)?;
    Ok(())
}

/// The cloned voice with this id.
pub fn load(dir: &Path, id: &str) -> Result<ClonedVoice> {
    let bytes = std::fs::read(path_of(dir, id)?)
        .map_err(|_| Error::Input(format!("unknown cloned voice: {id}")))?;
    ClonedVoice::from_bytes(&bytes)
}

/// Deletes a cloned voice and its file.
pub fn delete(dir: &Path, id: &str) -> Result<()> {
    std::fs::remove_file(path_of(dir, id)?)
        .map_err(|_| Error::Input(format!("unknown cloned voice: {id}")))
}

/// Every cloned voice in `dir` by id; files that are not valid voices are left out.
pub fn list(dir: &Path) -> Vec<VoiceMeta> {
    let mut found: Vec<VoiceMeta> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let id = path
                .file_name()?
                .to_str()?
                .strip_suffix(".bin")?
                .to_string();
            load(dir, &id)
                .ok()
                .map(|voice| voice.meta)
                .filter(|m| m.id == id)
        })
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

#[cfg(test)]
mod tests {
    use super::super::voice::sample_voice;
    use super::*;

    #[test]
    fn ids_come_from_names_and_only_clone_ids_count_as_clones() {
        assert_eq!(id_for("  My Voice! ").unwrap(), "clone_my-voice");
        assert!(id_for("   ").is_err() && id_for("!!!").is_err());
        assert!(id_for(&"a".repeat(41)).is_err());
        assert!(is_clone_id("clone_my-voice"));
        for other in [
            "af_heart",
            "af_my-warm",
            "clone_",
            "clone_../x",
            "clone_a b",
        ] {
            assert!(!is_clone_id(other), "{other}");
        }
    }

    #[test]
    fn a_voice_is_saved_listed_loaded_and_deleted_with_its_file() {
        let root = tempfile::tempdir().unwrap();
        let dir = clones_dir(root.path());
        assert_eq!(list(&dir).len(), 0);
        let voice = sample_voice("clone_sample");
        save(&dir, &voice).unwrap();
        assert!(dir.join("clone_sample.bin").is_file());
        assert!(matches!(save(&dir, &voice), Err(Error::Input(m)) if m.contains("exists")));
        let listed = list(&dir);
        assert_eq!(listed.len(), 1);
        assert!(listed[0].cloned);
        assert_eq!(load(&dir, "clone_sample").unwrap(), voice);
        delete(&dir, "clone_sample").unwrap();
        assert!(!dir.join("clone_sample.bin").exists());
        assert!(load(&dir, "clone_sample").is_err() && delete(&dir, "clone_sample").is_err());
    }

    #[test]
    fn other_files_and_foreign_ids_are_ignored_and_refused() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("clone_x.bin"), b"junk").unwrap();
        std::fs::write(root.path().join("notes.txt"), b"hi").unwrap();
        assert_eq!(list(root.path()).len(), 0);
        assert!(load(root.path(), "../etc").is_err());
        assert!(delete(root.path(), "af_heart").is_err());
    }
}
