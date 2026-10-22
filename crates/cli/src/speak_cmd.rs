//! `vtt speak`: text to speech.

use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use vtt_core::audio::encode_wav;
use vtt_core::engines::{Control, TtsOptions};
use vtt_core::jobs::Event;
use vtt_core::models::{Catalog, ModelStore, SPEECH_MODEL_ID};
use vtt_core::paths::Paths;
use vtt_core::tts::{KokoroEngine, MAX_SPEED, MIN_SPEED, describe};
use vtt_core::{Error, Result};

use crate::cli::Speak;
use crate::models_cmd;
use crate::output::Output;

const DEFAULT_VOICE: &str = "af_heart";
const DEFAULT_OUTPUT: &str = "speech.wav";

pub fn run(args: &Speak, paths: &Paths, out: Output) -> Result<()> {
    if args.play {
        return Err(Error::NotImplemented(
            "playing speech (--play); save it with -o",
        ));
    }
    check_format(&args.format)?;
    if !(MIN_SPEED..=MAX_SPEED).contains(&args.speed) {
        return Err(Error::Input(format!(
            "speed must be between {MIN_SPEED} and {MAX_SPEED}"
        )));
    }
    let text = if args.list_voices {
        String::new()
    } else {
        read_text(args)?
    };

    let catalog = Catalog::bundled()?;
    let store = ModelStore::new(paths);
    let entry = catalog
        .find(SPEECH_MODEL_ID)
        .ok_or_else(|| Error::ModelMissing(SPEECH_MODEL_ID.into()))?;
    models_cmd::ensure_installed(entry, &store, args.yes, out)?;
    let dir = store.model_dir(entry);
    let voices = KokoroEngine::voices_in(&dir);
    if args.list_voices {
        for id in &voices {
            let voice = describe(id);
            println!("{:<12} {} ({})", voice.id, voice.name, voice.description);
        }
        return Ok(());
    }
    let voice = resolve_voice(&voices, args.voice.as_deref())?;
    let mut engine = KokoroEngine::load(&dir, &entry.id)?;
    speak(
        &mut engine,
        &text,
        &TtsOptions {
            voice,
            speed: args.speed,
        },
        args,
        out,
    )
}

/// Speaks `text` and writes the result.
fn speak(
    engine: &mut KokoroEngine,
    text: &str,
    options: &TtsOptions,
    args: &Speak,
    out: Output,
) -> Result<()> {
    let started = Instant::now();
    let show_bar = out.show_messages() && std::io::stderr().is_terminal();
    let audio = engine.speak(text, options, &mut |done, total| {
        if show_bar {
            eprint!("\r  speaking {done}/{total}   ");
        }
        Control::Continue
    });
    if show_bar {
        eprintln!();
    }
    let written = write(&encode_wav(&audio?), args.output.as_deref())?;
    if out.json {
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let event = Event::Done {
            output: written,
            elapsed_ms,
        };
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}

/// Only WAV is written so far; other known formats say so, unknown ones are a usage mistake.
fn check_format(format: &str) -> Result<()> {
    match format.to_ascii_lowercase().as_str() {
        "wav" => Ok(()),
        "mp3" | "flac" => Err(Error::NotImplemented("speech output other than wav")),
        other => Err(Error::Input(format!("unknown format: {other}"))),
    }
}

/// The text from the argument, a file, or standard input.
fn read_text(args: &Speak) -> Result<String> {
    let text = match (&args.text, &args.file) {
        (Some(text), _) if text != "-" => text.clone(),
        (_, Some(path)) if path != Path::new("-") => std::fs::read_to_string(path)
            .map_err(|e| Error::Input(format!("cannot read {}: {e}", path.display())))?,
        (None, None) if std::io::stdin().is_terminal() => String::new(),
        _ => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            text
        }
    };
    if text.trim().is_empty() {
        return Err(Error::Input(
            "there is no text to speak: give some, use --file, or pipe it in".into(),
        ));
    }
    Ok(text)
}

/// A voice by its full name (`af_heart`) or by the part after the prefix (`heart`).
fn resolve_voice(installed: &[String], wanted: Option<&str>) -> Result<String> {
    let wanted = wanted.unwrap_or(DEFAULT_VOICE);
    let found = installed
        .iter()
        .find(|voice| voice.as_str() == wanted)
        .or_else(|| {
            installed
                .iter()
                .find(|voice| voice.ends_with(&format!("_{wanted}")))
        });
    found.cloned().ok_or_else(|| {
        Error::Input(format!(
            "unknown voice: {wanted} (installed: {})",
            installed.join(", ")
        ))
    })
}

/// Writes the WAV where the user asked and returns the path, or `None` for standard output.
fn write(wav: &[u8], output: Option<&Path>) -> Result<Option<String>> {
    let path: PathBuf = match output {
        Some(p) if p == Path::new("-") => {
            std::io::stdout().write_all(wav)?;
            return Ok(None);
        }
        Some(p) if p.is_dir() => p.join(DEFAULT_OUTPUT),
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(DEFAULT_OUTPUT),
    };
    std::fs::write(&path, wav)?;
    Ok(Some(path.display().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voices() -> Vec<String> {
        ["af_heart", "am_adam"].map(String::from).to_vec()
    }

    #[test]
    fn voices_are_found_by_full_name_or_by_the_name_after_the_prefix() {
        assert_eq!(
            resolve_voice(&voices(), Some("am_adam")).unwrap(),
            "am_adam"
        );
        assert_eq!(resolve_voice(&voices(), Some("adam")).unwrap(), "am_adam");
        assert_eq!(resolve_voice(&voices(), None).unwrap(), "af_heart");
    }

    #[test]
    fn an_unknown_voice_lists_the_installed_ones() {
        let error = resolve_voice(&voices(), Some("nobody"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("af_heart, am_adam"), "{error}");
    }

    #[test]
    fn formats_other_than_wav_are_not_available_yet() {
        assert!(check_format("WAV").is_ok());
        assert!(matches!(check_format("mp3"), Err(Error::NotImplemented(_))));
        assert!(matches!(check_format("ogg"), Err(Error::Input(_))));
    }

    #[test]
    fn output_goes_to_a_file_a_folder_or_the_default_name() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.wav");
        assert_eq!(
            write(b"x", Some(&file)).unwrap(),
            Some(file.display().to_string())
        );
        let in_folder = write(b"y", Some(dir.path())).unwrap().unwrap();
        assert!(in_folder.ends_with("speech.wav"), "{in_folder}");
        assert_eq!(std::fs::read(&file).unwrap(), b"x");
    }
}
