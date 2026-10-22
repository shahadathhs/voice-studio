//! `vtt library`: browse, search and export what `vtt transcribe` and the app stored.

use std::io::Write;
use std::path::Path;

use vtt_core::Result;
use vtt_core::export::{self, Format};
use vtt_core::library::{Item, Library as Store};
use vtt_core::paths::Paths;

use crate::cli::{Library, OutputArgs};
use crate::output::Output;

pub fn run(command: Library, paths: &Paths, out: Output) -> Result<()> {
    let mut store = Store::open(paths)?;
    match command {
        Library::List => list(&store, out),
        Library::Search { query, limit } => search(&store, &query, limit, out),
        Library::Show { id, out: args } => show(&store, &id, &args),
        Library::Export { ids, out: args } => export_all(&store, &ids, &args),
        Library::Delete { ids } => delete(&mut store, &ids, out),
    }
}

fn parse_id(text: &str) -> Result<i64> {
    text.parse()
        .map_err(|_| vtt_core::Error::Input(format!("not a library id: {text}")))
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

fn clock(duration_ms: u64) -> String {
    let seconds = duration_ms / 1000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn describe(item: &Item) -> String {
    let language = item.language.as_deref().unwrap_or("--");
    format!(
        "{:>4}  {:>7}  {:<4}  {}",
        item.id,
        clock(item.duration_ms),
        language,
        item.title
    )
}

fn list(store: &Store, out: Output) -> Result<()> {
    let items = store.list()?;
    if out.json {
        return print_json(&items);
    }
    for item in &items {
        println!("{}", describe(item));
    }
    Ok(())
}

fn search(store: &Store, query: &str, limit: usize, out: Output) -> Result<()> {
    let hits = store.search(query, limit)?;
    if out.json {
        return print_json(&hits);
    }
    for hit in &hits {
        println!("{}\n        {}", describe(&hit.item), hit.snippet);
    }
    Ok(())
}

fn rendered(store: &Store, id: i64, args: &OutputArgs) -> Result<(String, Format)> {
    let format: Format = args.format.parse()?;
    let (_, transcript) = store.get(id)?;
    let text = export::render(&transcript, format, &args.export_options())?;
    Ok((text, format))
}

fn show(store: &Store, id: &str, args: &OutputArgs) -> Result<()> {
    let (text, _) = rendered(store, parse_id(id)?, args)?;
    std::io::stdout().write_all(text.as_bytes())?;
    Ok(())
}

/// One id may go to a file or stdout; several ids need a directory, one file per transcript.
fn export_all(store: &Store, ids: &[String], args: &OutputArgs) -> Result<()> {
    let target = args.output.as_deref().filter(|p| *p != Path::new("-"));
    let single = ids.len() == 1 && target.is_none_or(|p| !p.is_dir());
    if !single && target.is_none() {
        return Err(vtt_core::Error::Input(
            "several ids need -o <directory>".into(),
        ));
    }
    for id in ids {
        let id = parse_id(id)?;
        let (text, format) = rendered(store, id, args)?;
        match target {
            None => std::io::stdout().write_all(text.as_bytes())?,
            Some(path) if single => std::fs::write(path, text)?,
            Some(dir) => {
                std::fs::create_dir_all(dir)?;
                let (item, _) = store.get(id)?;
                let name = format!("{id}-{}.{}", slug(&item.title), format.extension());
                std::fs::write(dir.join(name), text)?;
            }
        }
    }
    Ok(())
}

/// A file-name-safe form of a title.
fn slug(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = cleaned.trim_matches('-');
    if trimmed.is_empty() {
        "transcript".into()
    } else {
        trimmed.to_string()
    }
}

fn delete(store: &mut Store, ids: &[String], out: Output) -> Result<()> {
    for id in ids {
        let id = parse_id(id)?;
        if !store.delete(id)? {
            return Err(vtt_core::Error::Input(format!(
                "no library item with id {id}"
            )));
        }
        if out.show_messages() {
            eprintln!("deleted {id}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_become_safe_file_names() {
        assert_eq!(slug("Standup: 12/03"), "Standup--12-03");
        assert_eq!(slug("///"), "transcript");
    }

    #[test]
    fn durations_are_shown_as_minutes_and_seconds() {
        assert_eq!(clock(754_000), "12:34");
    }
}
