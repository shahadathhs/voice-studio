use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Stt,
    Tts,
    Diarization,
    Vad,
}

/// One downloadable file of a model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelFile {
    pub name: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub kind: ModelKind,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub languages: Vec<String>,
    pub license: String,
    pub files: Vec<ModelFile>,
}

impl ModelEntry {
    pub fn size_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).sum()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Catalog {
    pub models: Vec<ModelEntry>,
}

impl Catalog {
    /// Parses and validates a catalog. Ids and file names become path components, so they must
    /// not be able to escape the models directory.
    pub fn parse(json: &str) -> Result<Catalog> {
        let catalog: Catalog = serde_json::from_str(json)?;
        for model in &catalog.models {
            ensure_safe_name(&model.id)?;
            if model.files.is_empty() {
                return Err(Error::Input(format!("model {} has no files", model.id)));
            }
            for file in &model.files {
                ensure_safe_name(&file.name)?;
            }
        }
        Ok(catalog)
    }

    /// The catalog bundled into the binary.
    pub fn bundled() -> Result<Catalog> {
        Catalog::parse(include_str!("../../../../catalog/models.json"))
    }

    pub fn find(&self, id: &str) -> Option<&ModelEntry> {
        self.models.iter().find(|m| m.id == id)
    }

    pub fn of_kind(&self, kind: ModelKind) -> impl Iterator<Item = &ModelEntry> {
        self.models.iter().filter(move |m| m.kind == kind)
    }
}

fn ensure_safe_name(name: &str) -> Result<()> {
    let ok = !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if ok {
        Ok(())
    } else {
        Err(Error::Input(format!("unsafe name in catalog: {name:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = r#"{"id":"m","name":"M","kind":"stt","license":"MIT",
        "files":[{"name":"m.bin","url":"http://x/m.bin","sha256":"00","size_bytes":3}]}"#;

    #[test]
    fn bundled_catalog_parses_and_is_not_empty() {
        let catalog = Catalog::bundled().unwrap();
        assert!(!catalog.models.is_empty());
        assert!(catalog.of_kind(ModelKind::Stt).count() > 0);
    }

    #[test]
    fn every_bundled_file_has_a_pinned_url_and_a_sha256() {
        for model in Catalog::bundled().unwrap().models {
            for file in model.files {
                assert_eq!(file.sha256.len(), 64, "{}", file.name);
                assert!(file.url.starts_with("https://"), "{}", file.name);
                assert!(
                    !file.url.contains("/resolve/main/"),
                    "{} must pin a commit",
                    file.name
                );
            }
        }
    }

    #[test]
    fn finds_by_id_and_sums_file_sizes() {
        let catalog = Catalog::parse(&format!(r#"{{"models":[{ENTRY}]}}"#)).unwrap();
        assert_eq!(catalog.find("m").unwrap().size_bytes(), 3);
        assert!(catalog.find("nope").is_none());
    }

    #[test]
    fn rejects_names_that_could_escape_the_models_directory() {
        let bad = ENTRY.replace("m.bin\",\"url", "../evil\",\"url");
        assert!(Catalog::parse(&format!(r#"{{"models":[{bad}]}}"#)).is_err());
        let bad_id = ENTRY.replace(r#""id":"m""#, r#""id":"a/b""#);
        assert!(Catalog::parse(&format!(r#"{{"models":[{bad_id}]}}"#)).is_err());
    }

    #[test]
    fn rejects_a_model_without_files() {
        let json = r#"{"models":[{"id":"m","name":"M","kind":"stt","license":"MIT","files":[]}]}"#;
        assert!(Catalog::parse(json).is_err());
    }
}
