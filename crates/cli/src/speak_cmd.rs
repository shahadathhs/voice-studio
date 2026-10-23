//! `vtt speak`: text to speech.

use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use vtt_core::audio::AudioFormat;
use vtt_core::engines::{Control, TtsOptions};
use vtt_core::jobs::Event;
use vtt_core::models::{Catalog, ModelStore, SPEECH_MODEL_ID, TEXT_MODEL_ID};
use vtt_core::paths::Paths;
use vtt_core::text::translate_text;
use vtt_core::tts::{KokoroEngine, Language, MAX_SPEED, MIN_SPEED, describe};
use vtt_core::{Error, Result};

use crate::cli::Speak;
use crate::models_cmd;
use crate::output::Output;
use crate::text_cmd;

const DEFAULT_VOICE: &str = "af_heart";
const DEFAULT_OUTPUT_NAME: &str = "speech";

pub fn run(args: &Speak, paths: &Paths, out: Output) -> Result<()> {
    if args.play {
        return Err(Error::NotImplemented(
            "playing speech (--play); save it with -o",
        ));
    }
    // Checked first so a bad format fails before any model is loaded.
    output_format(args)?;
    if !(MIN_SPEED..=MAX_SPEED).contains(&args.speed) {
        return Err(Error::Input(format!(
            "speed must be between {MIN_SPEED} and {MAX_SPEED}"
        )));
    }
    let (text, language) = if args.list_voices {
        (String::new(), None)
    } else {
        translated(read_text(args)?, args, paths, out)?
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
    let wanted = args
        .voice
        .as_deref()
        .or(language.map(Language::default_voice));
    let voice = resolve_voice(&voices, wanted)?;
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

/// The text translated into `--translate-to`, if asked, with the language it is now in.
fn translated(
    text: String,
    args: &Speak,
    paths: &Paths,
    out: Output,
) -> Result<(String, Option<Language>)> {
    let Some(code) = args.translate_to.as_deref() else {
        return Ok((text, None));
    };
    let language = Language::of_code(code)
        .filter(|l| l.is_supported())
        .ok_or_else(|| Error::Input(format!("cannot speak {code}; see vtt speak --list-voices")))?;
    let mut engine = text_cmd::load_engine(TEXT_MODEL_ID, args.yes, paths, out)?;
    let mut go = text_cmd::progress(out);
    let text = translate_text(&mut engine, &text, language.code(), &mut go)?;
    Ok((text, Some(language)))
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
    let format = output_format(args)?;
    let written = write(&format.encode(&audio?)?, format, args.output.as_deref())?;
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

/// The format to write: `--format`, else what the output file's extension says, else WAV.
fn output_format(args: &Speak) -> Result<AudioFormat> {
    match (&args.format, &args.output) {
        (Some(name), _) => name.parse(),
        (None, Some(path)) => Ok(AudioFormat::from_path(path).unwrap_or_default()),
        (None, None) => Ok(AudioFormat::default()),
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

fn default_name(format: AudioFormat) -> String {
    format!("{DEFAULT_OUTPUT_NAME}.{}", format.extension())
}

/// Writes the audio where the user asked and returns the path, or `None` for standard output.
fn write(audio: &[u8], format: AudioFormat, output: Option<&Path>) -> Result<Option<String>> {
    let path: PathBuf = match output {
        Some(p) if p == Path::new("-") => {
            std::io::stdout().write_all(audio)?;
            return Ok(None);
        }
        Some(p) if p.is_dir() => p.join(default_name(format)),
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(default_name(format)),
    };
    std::fs::write(&path, audio)?;
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
    fn output_goes_to_a_file_a_folder_or_the_default_name() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.wav");
        assert_eq!(
            write(b"x", AudioFormat::Wav, Some(&file)).unwrap(),
            Some(file.display().to_string())
        );
        let in_folder = write(b"y", AudioFormat::Wav, Some(dir.path()))
            .unwrap()
            .unwrap();
        assert!(in_folder.ends_with("speech.wav"), "{in_folder}");
        let flac = write(b"z", AudioFormat::Flac, Some(dir.path()))
            .unwrap()
            .unwrap();
        assert!(flac.ends_with("speech.flac"), "{flac}");
        assert_eq!(std::fs::read(&file).unwrap(), b"x");
    }
}
