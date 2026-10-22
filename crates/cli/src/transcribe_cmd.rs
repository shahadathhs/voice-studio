//! `vtt transcribe`: audio or video files to text.

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use vtt_core::engines::{Device, SttOptions, WhisperEngine};
use vtt_core::export::Format;
use vtt_core::library::{Library, NewItem, title_for};
use vtt_core::models::{Catalog, ModelEntry, ModelKind, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::{Error, Result, Transcript};

use crate::cli::{DeviceChoice, Transcribe};
use crate::models_cmd;
use crate::output::Output;
use crate::transcribe_job::{Destination, Job};

pub fn run(args: &Transcribe, paths: &Paths, out: Output) -> Result<()> {
    if args.speakers.diarize {
        return Err(Error::NotImplemented("speaker identification (--diarize)"));
    }
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
    let entry = choose_model(&catalog, &store, args.model.as_deref())?;
    ensure_installed(entry, &store, args.yes, out)?;

    let model_path = store.file_path(entry, &entry.files[0]);
    let device = match args.device {
        DeviceChoice::Auto => Device::Auto,
        DeviceChoice::Cpu => Device::Cpu,
        DeviceChoice::Gpu => Device::Gpu,
    };
    let mut engine = WhisperEngine::load(&model_path, &entry.id, device)?;
    let options = SttOptions {
        language: args.language.clone(),
        translate: args.translate,
    };
    let export_options = args.out.export_options();
    for file in &args.files {
        let job = Job {
            file,
            format,
            export_options: &export_options,
            destination: &destination,
        };
        let transcript = job.run(&mut engine, &entry.id, &options, out)?;
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

/// `--model` accepts a catalog id (`whisper-base`) or the bare size the Python CLI used (`base`).
fn choose_model<'a>(
    catalog: &'a Catalog,
    store: &ModelStore,
    wanted: Option<&str>,
) -> Result<&'a ModelEntry> {
    let entry = match wanted {
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
    if entry.kind == ModelKind::Stt {
        Ok(entry)
    } else {
        Err(Error::Input(format!(
            "{} is not a speech-to-text model",
            entry.id
        )))
    }
}

/// Makes sure the model is on disk: downloads it with `--yes`, offers to when interactive, and
/// otherwise fails with exit code 4 so scripts get a clear signal.
fn ensure_installed(entry: &ModelEntry, store: &ModelStore, yes: bool, out: Output) -> Result<()> {
    if store.is_installed(entry) {
        return Ok(());
    }
    let interactive =
        !out.json && std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    if yes || (interactive && confirm_download(entry)) {
        return models_cmd::pull(entry, store, out);
    }
    Err(Error::ModelMissing(entry.id.clone()))
}

fn confirm_download(entry: &ModelEntry) -> bool {
    eprint!("{} is not installed. Download it now? [Y/n] ", entry.name);
    let _ = std::io::stderr().flush();
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer).is_ok()
        && matches!(
            answer.trim().to_ascii_lowercase().as_str(),
            "" | "y" | "yes"
        )
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
