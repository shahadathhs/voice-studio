use std::io::{Read, Write};
use std::path::Path;

use vtt_core::export::{self, Format};
use vtt_core::paths::Paths;
use vtt_core::{Error, Result, Transcript};

use crate::cli::{Cli, Command, Config, Export};
use crate::library_cmd;
use crate::models_cmd;
use crate::output::Output;
use crate::transcribe_cmd;

pub fn run(cli: Cli) -> Result<()> {
    let json = cli.global.json;
    let out = Output {
        json,
        quiet: cli.global.quiet,
    };
    let paths = Paths::resolve(cli.global.data_dir.clone());
    match cli.command {
        Command::Export(a) => export_cmd(&a),
        Command::Models(m) => models_cmd::run(m, &paths, out),
        Command::Config(Config::Path) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"config": paths.config, "data": paths.data})
                );
            } else {
                println!("config: {}", paths.config.display());
                println!("data:   {}", paths.data.display());
            }
            Ok(())
        }
        Command::Doctor => doctor(&paths, json),
        Command::Completions { shell } => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "vtt", &mut std::io::stdout());
            Ok(())
        }
        Command::Transcribe(args) => transcribe_cmd::run(&args, &paths, out),
        Command::Speak(_) => Err(Error::NotImplemented("speak")),
        Command::Record(_) => Err(Error::NotImplemented("record")),
        Command::Library(command) => library_cmd::run(command, &paths, out),
        Command::Devices => Err(Error::NotImplemented("devices")),
    }
}

fn read_input(path: &Path) -> Result<String> {
    if path == Path::new("-") {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        return Ok(s);
    }
    std::fs::read_to_string(path)
        .map_err(|e| Error::Input(format!("cannot read {}: {e}", path.display())))
}

fn export_cmd(a: &Export) -> Result<()> {
    let transcript: Transcript = serde_json::from_str(&read_input(&a.input)?)?;
    let format: Format = a.out.format.parse()?;
    let rendered = export::render(&transcript, format, &a.out.export_options())?;
    match a.out.output.as_deref() {
        Some(p) if p != Path::new("-") => std::fs::write(p, rendered)?,
        _ => std::io::stdout().write_all(rendered.as_bytes())?,
    }
    Ok(())
}

fn doctor(paths: &Paths, json: bool) -> Result<()> {
    let data_ok = std::fs::create_dir_all(&paths.data).is_ok();
    if json {
        println!(
            "{}",
            serde_json::json!({"data_dir": paths.data, "data_dir_writable": data_ok})
        );
    } else {
        println!(
            "data directory: {} ({})",
            paths.data.display(),
            if data_ok { "ok" } else { "not writable" }
        );
        println!("audio devices, GPU and models: not checked yet");
    }
    if data_ok {
        Ok(())
    } else {
        Err(Error::Io(std::io::Error::other(
            "data directory not writable",
        )))
    }
}
