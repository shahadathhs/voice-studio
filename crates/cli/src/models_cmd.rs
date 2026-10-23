//! `vtt models ...`: list, inspect, download, remove and verify speech models.

use std::io::{BufRead, IsTerminal, Write};
use std::time::Duration;

use vtt_core::engines::Control;
use vtt_core::jobs::{Event, Throttle};
use vtt_core::models::{Catalog, ModelEntry, ModelKind, ModelStatus, ModelStore, Progress};
use vtt_core::paths::Paths;
use vtt_core::{Error, Result};

use crate::cli::{KindArg, Models};
use crate::output::Output;

pub fn run(cmd: Models, paths: &Paths, out: Output) -> Result<()> {
    let catalog = Catalog::bundled()?;
    let store = ModelStore::new(paths);
    match cmd {
        Models::List { kind, installed } => list(&catalog, &store, kind, installed, out),
        Models::Info { id } => info(find(&catalog, &id)?, &store, out),
        Models::Pull { id } => pull(find(&catalog, &id)?, &store, out),
        Models::Remove { id } => remove(find(&catalog, &id)?, &store, out),
        Models::Verify { id } => verify(&catalog, &store, id.as_deref(), out),
    }
}

fn find<'a>(catalog: &'a Catalog, id: &str) -> Result<&'a ModelEntry> {
    catalog
        .find(id)
        .ok_or_else(|| Error::Input(format!("unknown model: {id}")))
}

fn kind_filter(kind: KindArg) -> ModelKind {
    match kind {
        KindArg::Stt => ModelKind::Stt,
        KindArg::Tts => ModelKind::Tts,
        KindArg::Diarization => ModelKind::Diarization,
        KindArg::Text => ModelKind::Text,
        KindArg::Vad => ModelKind::Vad,
    }
}

/// Decimal units, matching what file managers show and what the desktop app displays.
#[allow(clippy::cast_precision_loss)]
fn human_size(bytes: u64) -> String {
    let (mut value, mut unit) = (bytes as f64, 0);
    while value >= 1000.0 && unit < 3 {
        value /= 1000.0;
        unit += 1;
    }
    let label = ["B", "KB", "MB", "GB"][unit];
    if value >= 100.0 || unit == 0 {
        format!("{value:.0} {label}")
    } else {
        format!("{value:.1} {label}")
    }
}

fn list(
    catalog: &Catalog,
    store: &ModelStore,
    kind: Option<KindArg>,
    installed_only: bool,
    out: Output,
) -> Result<()> {
    let kind = kind.map(kind_filter);
    let statuses: Vec<ModelStatus> = catalog
        .models
        .iter()
        .filter(|m| kind.is_none_or(|k| m.kind == k))
        .map(|m| store.status(m))
        .filter(|s| !installed_only || s.installed)
        .collect();
    if out.json {
        println!("{}", serde_json::to_string_pretty(&statuses)?);
        return Ok(());
    }
    if statuses.is_empty() && !out.quiet {
        eprintln!("no models match");
    }
    for s in &statuses {
        let state = if s.installed { "installed" } else { "-" };
        println!(
            "{:<28} {:>9}  {:<10} {}",
            s.id,
            human_size(s.size_bytes),
            state,
            s.name
        );
    }
    Ok(())
}

fn info(entry: &ModelEntry, store: &ModelStore, out: Output) -> Result<()> {
    let status = store.status(entry);
    if out.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }
    println!("{} ({})", entry.name, entry.id);
    println!("  {}", entry.description);
    println!("  size:      {}", human_size(status.size_bytes));
    println!("  license:   {}", entry.license);
    println!("  languages: {}", entry.languages.join(", "));
    println!(
        "  installed: {}",
        if status.installed { "yes" } else { "no" }
    );
    println!("  location:  {}", store.model_dir(entry).display());
    Ok(())
}

/// Makes sure the model is on disk: downloads it with `--yes`, offers to when interactive, and
/// otherwise fails with exit code 4 so scripts get a clear signal.
pub fn ensure_installed(
    entry: &ModelEntry,
    store: &ModelStore,
    yes: bool,
    out: Output,
) -> Result<()> {
    if store.is_installed(entry) {
        return Ok(());
    }
    let interactive =
        !out.json && std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    if yes || (interactive && confirm_download(entry)) {
        return pull(entry, store, out);
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

pub fn pull(entry: &ModelEntry, store: &ModelStore, out: Output) -> Result<()> {
    if store.is_installed(entry) {
        if !out.quiet {
            eprintln!("{} is already installed", entry.id);
        }
        return Ok(());
    }
    let show_bar = !out.json && !out.quiet && std::io::stderr().is_terminal();
    let mut throttle = Throttle::new(Duration::from_millis(100));
    let result = store.install(entry, &mut |p| {
        if throttle.ready() || p.downloaded == p.total {
            print_progress(&entry.id, p, out, show_bar);
        }
        Control::Continue
    });
    if show_bar {
        eprintln!();
    }
    result?;
    if !out.quiet && !out.json {
        eprintln!(
            "installed {} ({})",
            entry.id,
            human_size(entry.size_bytes())
        );
    }
    Ok(())
}

fn print_progress(id: &str, p: Progress, out: Output, show_bar: bool) {
    if out.json {
        let event = Event::Download {
            id: id.to_string(),
            downloaded_bytes: p.downloaded,
            total_bytes: p.total,
        };
        if let Ok(line) = serde_json::to_string(&event) {
            println!("{line}");
        }
    } else if show_bar {
        let percent = p
            .downloaded
            .saturating_mul(100)
            .checked_div(p.total)
            .unwrap_or(100);
        eprint!(
            "\r  {id}  {percent:>3}%  {} / {}   ",
            human_size(p.downloaded),
            human_size(p.total)
        );
    }
}

fn remove(entry: &ModelEntry, store: &ModelStore, out: Output) -> Result<()> {
    store.remove(entry)?;
    if !out.quiet && !out.json {
        eprintln!("removed {}", entry.id);
    }
    Ok(())
}

fn verify(catalog: &Catalog, store: &ModelStore, id: Option<&str>, out: Output) -> Result<()> {
    let targets: Vec<&ModelEntry> = match id {
        Some(id) => vec![find(catalog, id)?],
        None => catalog
            .models
            .iter()
            .filter(|m| store.is_installed(m))
            .collect(),
    };
    let mut corrupt = Vec::new();
    let mut results = Vec::new();
    for entry in targets {
        let ok = store.verify(entry)?;
        if !ok {
            corrupt.push(entry.id.clone());
        }
        results.push(serde_json::json!({ "id": entry.id, "ok": ok }));
        if !out.json {
            println!(
                "{:<28} {}",
                entry.id,
                if ok { "ok" } else { "CORRUPT or missing" }
            );
        }
    }
    if out.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    }
    if corrupt.is_empty() {
        Ok(())
    } else {
        Err(Error::Engine(format!(
            "failed verification: {}. Run `vtt models remove <id>` and pull again.",
            corrupt.join(", ")
        )))
    }
}
