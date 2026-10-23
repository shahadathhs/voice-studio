//! Preferences shared by the CLI and the desktop app, kept as TOML in the config file. A flag or
//! a choice made for one run always beats what is stored here.

use serde::{Deserialize, Serialize};

use crate::engines::Device;
use crate::paths::Paths;
use crate::{Error, Result};

/// What the user has chosen as their defaults. Every field is optional in the file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Transcription model to use when none is named: a catalog id such as `whisper-small`.
    pub model: Option<String>,
    /// Spoken language as a code such as `en`; `None` detects it for each file.
    pub language: Option<String>,
    /// Which hardware runs transcription.
    pub device: Device,
    /// Whether to label who is speaking unless told otherwise.
    pub identify_speakers: bool,
}

/// The names `Settings::set` and `Settings::get` understand, as the CLI spells them.
pub const KEYS: [&str; 4] = ["model", "language", "device", "identify-speakers"];

fn unknown_key(key: &str) -> Error {
    Error::Input(format!(
        "unknown setting: {key} (known: {})",
        KEYS.join(", ")
    ))
}

/// An empty value or `auto` clears an optional setting.
fn optional(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && value != "auto").then(|| value.to_string())
}

impl Settings {
    /// Reads the settings file. A missing file is the defaults; a file that cannot be read as
    /// settings is an input error that names the file, so nothing is silently ignored.
    pub fn load(paths: &Paths) -> Result<Settings> {
        let text = match std::fs::read_to_string(&paths.config) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Settings::default()),
            Err(e) => return Err(e.into()),
        };
        toml::from_str(&text)
            .map_err(|e| Error::Input(format!("{}: {}", paths.config.display(), e.message())))
    }

    /// Writes the settings file, creating its folder.
    pub fn save(&self, paths: &Paths) -> Result<()> {
        if let Some(folder) = paths.config.parent() {
            std::fs::create_dir_all(folder)?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|e| Error::Engine(format!("cannot write settings: {e}")))?;
        std::fs::write(&paths.config, text)?;
        Ok(())
    }

    /// The value of one setting as text, `auto` when it is not set.
    pub fn get(&self, key: &str) -> Result<String> {
        let auto = || "auto".to_string();
        match key {
            "model" => Ok(self.model.clone().unwrap_or_else(auto)),
            "language" => Ok(self.language.clone().unwrap_or_else(auto)),
            "device" => Ok(format!("{:?}", self.device).to_lowercase()),
            "identify-speakers" => Ok(self.identify_speakers.to_string()),
            _ => Err(unknown_key(key)),
        }
    }

    /// Changes one setting from text, as typed after `vtt config set`.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "model" => self.model = optional(value),
            "language" => self.language = optional(value),
            "device" => {
                self.device = toml::Value::String(value.trim().to_lowercase())
                    .try_into()
                    .map_err(|_| Error::Input("device must be auto, cpu or gpu".into()))?;
            }
            "identify-speakers" => {
                self.identify_speakers = value
                    .trim()
                    .parse()
                    .map_err(|_| Error::Input("identify-speakers must be true or false".into()))?;
            }
            _ => return Err(unknown_key(key)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::resolve(Some(dir.path().to_path_buf()));
        (dir, paths)
    }

    #[test]
    fn a_missing_file_is_the_defaults() {
        let (_dir, paths) = paths();
        assert_eq!(Settings::load(&paths).unwrap(), Settings::default());
    }

    #[test]
    fn settings_survive_a_save_and_load() {
        let (_dir, paths) = paths();
        let mut settings = Settings::default();
        settings.set("model", "whisper-small").unwrap();
        settings.set("language", "es").unwrap();
        settings.set("device", "CPU").unwrap();
        settings.set("identify-speakers", "true").unwrap();
        settings.save(&paths).unwrap();
        assert_eq!(Settings::load(&paths).unwrap(), settings);
        assert_eq!(settings.get("device").unwrap(), "cpu");
    }

    #[test]
    fn auto_or_empty_clears_an_optional_setting() {
        let mut settings = Settings::default();
        settings.set("model", "whisper-base").unwrap();
        settings.set("model", "auto").unwrap();
        assert_eq!(settings.model, None);
        assert_eq!(settings.get("model").unwrap(), "auto");
    }

    #[test]
    fn bad_values_and_unknown_keys_are_input_errors() {
        let mut settings = Settings::default();
        for (key, value) in [
            ("device", "tpu"),
            ("identify-speakers", "maybe"),
            ("colour", "red"),
        ] {
            assert!(matches!(settings.set(key, value), Err(Error::Input(_))));
        }
        assert!(matches!(settings.get("colour"), Err(Error::Input(_))));
    }

    #[test]
    fn a_damaged_file_is_an_input_error_naming_the_file() {
        let (_dir, paths) = paths();
        std::fs::write(&paths.config, "device = 7").unwrap();
        let message = Settings::load(&paths).unwrap_err().to_string();
        assert!(message.contains("config.toml"), "{message}");
    }

    #[test]
    fn a_data_override_keeps_its_settings_in_that_folder() {
        let (dir, paths) = paths();
        assert_eq!(paths.config, dir.path().join("config.toml"));
    }
}
