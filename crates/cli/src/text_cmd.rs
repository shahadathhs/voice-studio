//! `vtt summarize` and `vtt translate`: a language model run on a transcript.

use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use clap::Args;
use vtt_core::engines::Control;
use vtt_core::export::{self, Format};
use vtt_core::library::{Library, NewNote, NoteKind};
use vtt_core::models::{Catalog, ModelKind, ModelStore, TEXT_MODEL_ID};
use vtt_core::paths::Paths;
use vtt_core::text::{
    QwenEngine, action_items, chapters, chapters_from_json, chapters_to_json, summarize, translate,
};
use vtt_core::{Error, Result, Transcript};

use crate::cli::OutputArgs;
use crate::models_cmd;
use crate::output::Output;

/// Where the transcript comes from, and which model works on it.
#[derive(Args, Debug)]
pub struct Input {
    /// A library id (see `vtt library list`), a transcript JSON file, or '-' for standard input
    pub source: String,
    /// Language model (see `vtt models list --kind text`)
    #[arg(short, long, default_value = TEXT_MODEL_ID)]
    pub model: String,
    /// Download the model if it is missing, without asking
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct Summarize {
    #[command(flatten)]
    pub source: Input,
    /// Language of the summary, e.g. es or zh (default: the language of the transcript)
    #[arg(long, value_name = "LANG")]
    pub to: Option<String>,
    /// Output file; '-' or absent for stdout
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Give titles to the parts of the recording instead, with the time each starts
    #[arg(long, conflicts_with = "actions")]
    pub chapters: bool,
    /// List the tasks and commitments instead
    #[arg(long)]
    pub actions: bool,
    /// Do not keep the result with the library item
    #[arg(long)]
    pub no_save: bool,
}

#[derive(Args, Debug)]
pub struct Translate {
    #[command(flatten)]
    pub source: Input,
    /// Language to translate into, e.g. es, zh, bn
    #[arg(long, value_name = "LANG")]
    pub to: String,
    #[command(flatten)]
    pub out: OutputArgs,
}

pub fn run_summarize(args: &Summarize, paths: &Paths, out: Output) -> Result<()> {
    let transcript = load(&args.source.source, paths)?;
    let mut engine = engine(&args.source, paths, out)?;
    let to = args.to.as_deref();
    let go = &mut progress(out);
    let (kind, text) = if args.chapters {
        let found = chapters(&mut engine, &transcript, go)?;
        (NoteKind::Chapters, chapters_to_json(&found)?)
    } else if args.actions {
        (
            NoteKind::Actions,
            action_items(&mut engine, &transcript, to, go)?,
        )
    } else {
        (
            NoteKind::Summary,
            summarize(&mut engine, &transcript, to, go)?,
        )
    };
    finish_progress(out);
    if !args.no_save
        && let Ok(id) = args.source.source.parse::<i64>()
    {
        let note = NewNote {
            kind,
            language: to,
            model: &args.source.model,
            text: &text,
        };
        Library::open(paths)?.add_note(id, &note)?;
    }
    let shown = match kind {
        NoteKind::Chapters => chapter_lines(&text)?,
        _ => text,
    };
    if out.json {
        let json = serde_json::json!({"kind": kind, "text": shown, "model": args.source.model});
        return write(&json.to_string(), args.output.as_deref());
    }
    write(&format!("{shown}\n"), args.output.as_deref())
}

/// `0:00  Title` for each chapter in a stored chapter list.
fn chapter_lines(json: &str) -> Result<String> {
    let found = chapters_from_json(json)?;
    let lines: Vec<String> = found
        .iter()
        .map(|c| {
            let seconds = c.start_ms / 1000;
            format!("{}:{:02}  {}", seconds / 60, seconds % 60, c.title)
        })
        .collect();
    Ok(lines.join("\n"))
}

pub fn run_translate(args: &Translate, paths: &Paths, out: Output) -> Result<()> {
    let format: Format = args.out.format.parse()?;
    let transcript = load(&args.source.source, paths)?;
    let mut engine = engine(&args.source, paths, out)?;
    let translated = translate(&mut engine, &transcript, &args.to, &mut progress(out))?;
    finish_progress(out);
    let rendered = export::render_bytes(&translated, format, &args.out.export_options())?;
    match args.out.output.as_deref() {
        Some(path) if path != Path::new("-") => std::fs::write(path, rendered)?,
        _ => std::io::stdout().write_all(&rendered)?,
    }
    Ok(())
}

fn write(text: &str, output: Option<&Path>) -> Result<()> {
    match output {
        Some(path) if path != Path::new("-") => std::fs::write(path, text)?,
        _ => std::io::stdout().write_all(text.as_bytes())?,
    }
    Ok(())
}

/// A library item by id, else a JSON file, else standard input.
fn load(source: &str, paths: &Paths) -> Result<Transcript> {
    if let Ok(id) = source.parse::<i64>() {
        return Ok(Library::open(paths)?.get(id)?.1);
    }
    let text = if source == "-" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        text
    } else {
        std::fs::read_to_string(source)
            .map_err(|e| Error::Input(format!("cannot read {source}: {e}")))?
    };
    Ok(serde_json::from_str(&text)?)
}

fn engine(source: &Input, paths: &Paths, out: Output) -> Result<QwenEngine> {
    let catalog = Catalog::bundled()?;
    let entry = catalog
        .find(&source.model)
        .filter(|entry| entry.kind == ModelKind::Text)
        .ok_or_else(|| Error::Input(format!("not a language model: {}", source.model)))?;
    let store = ModelStore::new(paths);
    models_cmd::ensure_installed(entry, &store, source.yes, out)?;
    QwenEngine::load(&store.model_dir(entry), &entry.id)
}

fn progress(out: Output) -> impl FnMut(f32) -> Control {
    let show = out.show_messages() && std::io::stderr().is_terminal();
    move |fraction| {
        if show {
            eprint!("\r  working {:>3.0}%   ", fraction * 100.0);
        }
        Control::Continue
    }
}

fn finish_progress(out: Output) {
    if out.show_messages() && std::io::stderr().is_terminal() {
        eprintln!();
    }
}
