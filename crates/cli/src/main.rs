//! `vtt` — command-line front end for Voice Studio. See docs/product/cli.md for the full plan.

mod cli;
mod commands;
mod config_cmd;
mod library_cmd;
mod manage;
mod models_cmd;
mod output;
mod speak_cmd;
mod transcribe_cmd;
mod transcribe_job;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let args = cli::Cli::parse();
    let json = args.global.json;
    match commands::run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if json {
                let event = vtt_core::jobs::Event::Error {
                    code: e.exit_code(),
                    message: e.to_string(),
                };
                // Errors go to stderr so stdout stays pure data.
                eprintln!("{}", serde_json::to_string(&event).unwrap_or_default());
            } else {
                eprintln!("error: {e}");
                if let vtt_core::Error::ModelMissing(id) = &e {
                    eprintln!("hint: run `vtt models pull {id}`");
                }
            }
            ExitCode::from(e.exit_code())
        }
    }
}
