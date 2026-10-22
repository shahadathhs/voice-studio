use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use super::download::sha256_file;
use super::{ModelEntry, ModelFile, ModelKind, Progress, download_file};
use crate::engines::Control;
use crate::paths::Paths;
use crate::{Error, Result};

/// What a front end shows for one model: catalog data plus whether it is installed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelStatus {
    pub id: String,
    pub name: String,
    pub kind: ModelKind,
    pub description: String,
    pub languages: Vec<String>,
    pub license: String,
    pub can_translate: bool,
    pub size_bytes: u64,
    pub installed: bool,
}

/// Installed models on disk: `<models dir>/<model id>/<file>`.
#[derive(Debug, Clone)]
pub struct ModelStore {
    root: PathBuf,
}

impl ModelStore {
    pub fn new(paths: &Paths) -> ModelStore {
        ModelStore {
            root: paths.models_dir(),
        }
    }

    #[cfg(test)]
    pub(super) fn at(root: PathBuf) -> ModelStore {
        ModelStore { root }
    }

    pub fn model_dir(&self, entry: &ModelEntry) -> PathBuf {
        self.root.join(&entry.id)
    }

    pub fn file_path(&self, entry: &ModelEntry, file: &ModelFile) -> PathBuf {
        self.model_dir(entry).join(&file.name)
    }

    /// Cheap check: every file exists with its expected size. A file is only ever moved into place
    /// after its checksum matched, so size is a sufficient marker; [`ModelStore::verify`] re-hashes.
    pub fn is_installed(&self, entry: &ModelEntry) -> bool {
        entry
            .files
            .iter()
            .all(|f| fs::metadata(self.file_path(entry, f)).is_ok_and(|m| m.len() == f.size_bytes))
    }

    /// The speech-to-text model to use when the user did not choose one: the first installed model
    /// in a sensible order (a good balance first, then smaller, then larger).
    pub fn default_stt<'a>(&self, models: &'a [ModelEntry]) -> Option<&'a ModelEntry> {
        const PREFERENCE: [&str; 6] = [
            "whisper-base",
            "whisper-small",
            "whisper-tiny",
            "whisper-medium",
            "whisper-large-v3-turbo-q5",
            "whisper-large-v3-turbo",
        ];
        PREFERENCE
            .iter()
            .filter_map(|id| models.iter().find(|m| m.id == *id))
            .find(|m| self.is_installed(m))
            .or_else(|| {
                models
                    .iter()
                    .find(|m| m.kind == ModelKind::Stt && self.is_installed(m))
            })
    }

    pub fn status(&self, entry: &ModelEntry) -> ModelStatus {
        ModelStatus {
            id: entry.id.clone(),
            name: entry.name.clone(),
            kind: entry.kind,
            description: entry.description.clone(),
            languages: entry.languages.clone(),
            license: entry.license.clone(),
            can_translate: entry.can_translate,
            size_bytes: entry.size_bytes(),
            installed: self.is_installed(entry),
        }
    }

    /// Downloads every missing file of `entry`. Progress is reported across all files.
    pub fn install(
        &self,
        entry: &ModelEntry,
        on_progress: &mut dyn FnMut(Progress) -> Control,
    ) -> Result<()> {
        let total = entry.size_bytes();
        let mut finished = 0u64;
        for file in &entry.files {
            let dest = self.file_path(entry, file);
            let present = fs::metadata(&dest).is_ok_and(|m| m.len() == file.size_bytes);
            if !present {
                let base = finished;
                download_file(file, &dest, &mut |p| {
                    on_progress(Progress {
                        downloaded: base + p.downloaded,
                        total,
                    })
                })?;
            }
            finished += file.size_bytes;
        }
        Ok(())
    }

    pub fn remove(&self, entry: &ModelEntry) -> Result<()> {
        let dir = self.model_dir(entry);
        if !dir.exists() {
            return Err(Error::ModelMissing(entry.id.clone()));
        }
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    /// Re-hashes every file of an installed model. `Ok(false)` means a file is missing or corrupt.
    pub fn verify(&self, entry: &ModelEntry) -> Result<bool> {
        for file in &entry.files {
            let path = self.file_path(entry, file);
            if !path.exists() || !sha256_file(&path)?.eq_ignore_ascii_case(&file.sha256) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(size: u64) -> ModelEntry {
        ModelEntry {
            id: "tiny".into(),
            name: "Tiny".into(),
            kind: ModelKind::Stt,
            description: String::new(),
            languages: vec![],
            license: "MIT".into(),
            can_translate: true,
            files: vec![ModelFile {
                name: "tiny.bin".into(),
                url: "http://127.0.0.1:9/never".into(),
                // sha256 of the bytes "abc"
                sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
                size_bytes: size,
            }],
        }
    }

    fn place_file(store: &ModelStore, entry: &ModelEntry, bytes: &[u8]) {
        let path = store.file_path(entry, &entry.files[0]);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn installed_means_all_files_present_with_the_expected_size() {
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::at(dir.path().to_path_buf());
        let entry = entry(3);
        assert!(!store.is_installed(&entry));
        place_file(&store, &entry, b"ab");
        assert!(!store.is_installed(&entry), "wrong size must not count");
        place_file(&store, &entry, b"abc");
        assert!(store.is_installed(&entry));
        assert!(store.status(&entry).installed);
    }

    #[test]
    fn default_stt_prefers_base_then_falls_back_to_any_installed_model() {
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::at(dir.path().to_path_buf());
        let mut small = entry(3);
        small.id = "whisper-small".into();
        let mut odd = entry(3);
        odd.id = "custom-stt".into();
        let models = vec![small.clone(), odd.clone()];
        assert!(store.default_stt(&models).is_none());
        place_file(&store, &odd, b"abc");
        assert_eq!(store.default_stt(&models).unwrap().id, "custom-stt");
        place_file(&store, &small, b"abc");
        assert_eq!(store.default_stt(&models).unwrap().id, "whisper-small");
    }

    #[test]
    fn install_skips_files_that_are_already_present() {
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::at(dir.path().to_path_buf());
        let entry = entry(3);
        place_file(&store, &entry, b"abc");
        // The URL is unreachable, so this only succeeds if nothing is downloaded.
        store.install(&entry, &mut |_| Control::Continue).unwrap();
    }

    #[test]
    fn verify_detects_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::at(dir.path().to_path_buf());
        let entry = entry(3);
        place_file(&store, &entry, b"abc");
        assert!(store.verify(&entry).unwrap());
        place_file(&store, &entry, b"abd");
        assert!(!store.verify(&entry).unwrap());
    }

    #[test]
    fn remove_deletes_the_model_and_reports_a_missing_one() {
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::at(dir.path().to_path_buf());
        let entry = entry(3);
        place_file(&store, &entry, b"abc");
        store.remove(&entry).unwrap();
        assert!(!store.is_installed(&entry));
        assert!(matches!(store.remove(&entry), Err(Error::ModelMissing(_))));
    }
}
