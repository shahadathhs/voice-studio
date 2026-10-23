//! The user's saved defaults, shared with the CLI through the same config file.

use vtt_core::paths::Paths;
use vtt_core::settings::Settings;

use crate::commands::CommandError;

fn load(paths: &Paths) -> Result<Settings, CommandError> {
    Ok(Settings::load(paths)?)
}

/// Stores the settings and hands back what was stored.
fn save(paths: &Paths, settings: Settings) -> Result<Settings, CommandError> {
    settings.save(paths)?;
    Ok(settings)
}

#[tauri::command]
pub fn get_settings() -> Result<Settings, CommandError> {
    load(&Paths::resolve(None))
}

/// Stores the settings. A new hardware choice applies the next time the app starts.
#[tauri::command]
pub fn save_settings(settings: Settings) -> Result<Settings, CommandError> {
    save(&Paths::resolve(None), settings)
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
    fn settings_saved_by_the_app_are_read_back() {
        let (_dir, paths) = paths();
        let settings = Settings {
            model: Some("whisper-small".into()),
            language: Some("es".into()),
            identify_speakers: true,
            ..Settings::default()
        };
        let stored = save(&paths, settings.clone()).unwrap();
        assert_eq!(stored, settings);
        assert_eq!(load(&paths).unwrap(), settings);
    }

    #[test]
    fn a_damaged_file_is_reported_to_the_ui_not_replaced() {
        let (_dir, paths) = paths();
        std::fs::write(&paths.config, "device = 7").unwrap();
        assert!(load(&paths).unwrap_err().0.contains("config.toml"));
    }
}
