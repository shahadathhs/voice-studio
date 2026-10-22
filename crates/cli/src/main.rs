//! `vtt` — command-line front end for Voice Studio. See docs/product/cli.md for the full plan.

mod cli;
mod commands;
mod models_cmd;

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
            }
            ExitCode::from(e.exit_code())
        }
    }
}
