//! `vtt transcribe`: audio or video files to text.

use std::path::{Path, PathBuf};

use vtt_core::diarize::SpeakerLabeller;
use vtt_core::engines::{Device, DiarizeOptions, SttOptions, WhisperEngine};
use vtt_core::export::Format;
use vtt_core::library::{Library, NewItem, title_for};
use vtt_core::models::{Catalog, ModelEntry, ModelKind, ModelStore, SPEAKER_MODEL_ID};
use vtt_core::paths::Paths;
use vtt_core::settings::Settings;
use vtt_core::{Error, Result, Transcript};

use crate::cli::{DeviceChoice, Transcribe};
use crate::models_cmd;
use crate::output::Output;
use crate::transcribe_job::{Destination, Job, Recognizer};

pub fn run(args: &Transcribe, paths: &Paths, out: Output) -> Result<()> {
    let settings = Settings::load(paths)?;
    let format: Format = args.out.format.parse()?;
    let destination = Destination::resolve(&args.files, args.out.output.as_deref())?;
    check_inputs(&args.files)?;

    let mut library = if args.no_library {
        None
    } else {
        Some(Library::open(paths)?)
    };
    let catalog = Catalog::bundled()?;
    let store = ModelStore::new(paths);
    let entry = choose_model(&catalog, &store, args, &settings)?;
    models_cmd::ensure_installed(entry, &store, args.yes, out)?;
    let speakers = speaker_labeller(args, &settings, &catalog, &store, out)?;
    let mut engine = load_engine(args, &settings, entry, &store)?;
    let options = SttOptions {
        language: args.language.clone().or_else(|| settings.language.clone()),
        translate: args.recognition.translate,
        vad: !args.recognition.no_vad,
    };
    let export_options = args.out.export_options();
    let mut recognizer = Recognizer {
        engine: &mut engine,
        model: &entry.id,
        options: &options,
        speakers,
    };
    for file in &args.files {
        let job = Job {
            file,
            format,
            export_options: &export_options,
            destination: &destination,
        };
        let transcript = job.run(&mut recognizer, out)?;
        if let Some(library) = library.as_mut() {
            keep(library, file, &transcript)?;
        }
    }
    Ok(())
}

/// Stores a finished transcript in the shared library, titled after the file.
fn keep(library: &mut Library, file: &Path, transcript: &Transcript) -> Result<()> {
    let title = title_for(file);
    let source = std::path::absolute(file)?.display().to_string();
    let new = NewItem {
        title: &title,
        source_path: Some(&source),
    };
    library.add(transcript, &new)?;
    Ok(())
}

fn load_engine(
    args: &Transcribe,
    settings: &Settings,
    entry: &ModelEntry,
    store: &ModelStore,
) -> Result<WhisperEngine> {
    let device = match args.device {
        Some(DeviceChoice::Auto) => Device::Auto,
        Some(DeviceChoice::Cpu) => Device::Cpu,
        Some(DeviceChoice::Gpu) => Device::Gpu,
        None => settings.device,
    };
    WhisperEngine::load(&store.file_path(entry, &entry.files[0]), &entry.id, device)
}

/// Loads the speaker model (offering to download it) when `--diarize` was given.
fn speaker_labeller(
    args: &Transcribe,
    settings: &Settings,
    catalog: &Catalog,
    store: &ModelStore,
    out: Output,
) -> Result<Option<(SpeakerLabeller, DiarizeOptions)>> {
    if !args.speakers.diarize && !settings.identify_speakers {
        return Ok(None);
    }
    let entry = catalog
        .find(SPEAKER_MODEL_ID)
        .ok_or_else(|| Error::ModelMissing(SPEAKER_MODEL_ID.into()))?;
    models_cmd::ensure_installed(entry, store, args.yes, out)?;
    let options = DiarizeOptions {
        speakers: args.speakers.speakers,
        threshold: args.speakers.diarize_threshold,
    };
    Ok(Some((vtt_core::diarize::load(store, entry)?, options)))
}

fn check_inputs(files: &[PathBuf]) -> Result<()> {
    for file in files {
        if file == Path::new("-") {
            return Err(Error::NotImplemented("reading audio from stdin"));
        }
        if !file.is_file() {
            return Err(Error::Input(format!(
                "cannot open {}: no such file",
                file.display()
            )));
        }
    }
    Ok(())
}

/// `--model` (or the `model` setting) accepts a catalog id (`whisper-base`) or the bare size the Python CLI used (`base`).
/// Fails when the model is not a speech model, or cannot translate and `--translate` was given.
fn choose_model<'a>(
    catalog: &'a Catalog,
    store: &ModelStore,
    args: &Transcribe,
    settings: &Settings,
) -> Result<&'a ModelEntry> {
    let entry = match args.model.as_deref().or(settings.model.as_deref()) {
        Some(name) => catalog
            .find(name)
            .or_else(|| catalog.find(&format!("whisper-{name}")))
            .ok_or_else(|| Error::Input(format!("unknown model: {name}")))?,
        None => match store.default_stt(&catalog.models) {
            Some(entry) => entry,
            None => catalog
                .find("whisper-base")
                .ok_or_else(|| Error::ModelMissing("whisper-base".into()))?,
        },
    };
    if entry.kind != ModelKind::Stt {
        return Err(Error::Input(format!(
            "{} is not a speech-to-text model",
            entry.id
        )));
    }
    if args.recognition.translate && !entry.can_translate {
        return Err(Error::Input(format!(
            "{} cannot translate; choose another model with -m",
            entry.name
        )));
    }
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use vtt_core::library::Library;
    use vtt_core::paths::Paths;

    use super::*;

    #[test]
    fn a_finished_transcript_is_stored_under_the_file_name() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = Library::open(&Paths::resolve(Some(dir.path().to_path_buf()))).unwrap();
        let transcript = Transcript {
            duration_ms: 1500,
            ..Transcript::default()
        };

        keep(&mut library, Path::new("talks/standup.m4a"), &transcript).unwrap();

        let items = library.list().unwrap();
        assert_eq!(items[0].title, "standup");
        assert_eq!(items[0].duration_ms, 1500);
        assert!(
            items[0]
                .source_path
                .as_deref()
                .unwrap()
                .ends_with("standup.m4a")
        );
    }
}
