//! Where the app keeps its data. The CLI and the desktop app share these locations,
//! so a transcript made by one appears in the other.

use std::path::PathBuf;

use directories::ProjectDirs;

/// Placeholder identifiers until the product name is final.
const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "voice-studio";
const APPLICATION: &str = "Voice Studio";

#[derive(Debug, Clone)]
pub struct Paths {
    pub data: PathBuf,
    pub config: PathBuf,
}

impl Paths {
    /// Resolve locations. `data_override` (flag) wins over `VTT_DATA_DIR`, which wins over the OS default.
    pub fn resolve(data_override: Option<PathBuf>) -> Paths {
        let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION);
        let overridden =
            data_override.or_else(|| std::env::var_os("VTT_DATA_DIR").map(PathBuf::from));
        // A relocated data directory is a self-contained setup (tests, portable use), so its
        // settings live with it instead of in the user's real config.
        let config = match (&overridden, &dirs) {
            (None, Some(d)) => d.config_dir().join("config.toml"),
            (Some(data), _) => data.join("config.toml"),
            (None, None) => PathBuf::from(".voice-studio").join("config.toml"),
        };
        let data = overridden
            .or_else(|| dirs.as_ref().map(|d| d.data_dir().to_path_buf()))
            .unwrap_or_else(|| PathBuf::from(".voice-studio"));
        Paths { data, config }
    }

    pub fn models_dir(&self) -> PathBuf {
        self.data.join("models")
    }

    pub fn library_db(&self) -> PathBuf {
        self.data.join("library.sqlite")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_override_wins() {
        let p = Paths::resolve(Some(PathBuf::from("/tmp/vtt-test")));
        assert_eq!(p.data, PathBuf::from("/tmp/vtt-test"));
        assert_eq!(p.models_dir(), PathBuf::from("/tmp/vtt-test/models"));
    }
}
