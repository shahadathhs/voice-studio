//! `vtt config`: where things are kept, and the defaults shared with the desktop app.

use vtt_core::Result;
use vtt_core::paths::Paths;
use vtt_core::settings::{KEYS, Settings};

use crate::cli::Config;

pub fn run(config: Config, paths: &Paths, json: bool) -> Result<()> {
    match config {
        Config::Path => {
            show_paths(paths, json);
            Ok(())
        }
        Config::List => list(&Settings::load(paths)?, json),
        Config::Get { key } => {
            println!("{}", Settings::load(paths)?.get(&key)?);
            Ok(())
        }
        Config::Set { key, value } => {
            let mut settings = Settings::load(paths)?;
            settings.set(&key, &value)?;
            settings.save(paths)
        }
    }
}

fn show_paths(paths: &Paths, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::json!({"config": paths.config, "data": paths.data})
        );
    } else {
        println!("config: {}", paths.config.display());
        println!("data:   {}", paths.data.display());
    }
}

fn list(settings: &Settings, json: bool) -> Result<()> {
    let mut object = serde_json::Map::new();
    for key in KEYS {
        let value = settings.get(key)?;
        if !json {
            println!("{key} = {value}");
        }
        object.insert(key.to_string(), value.into());
    }
    if json {
        println!("{}", serde_json::Value::Object(object));
    }
    Ok(())
}
