//! Subcommands that manage what is stored: models, the library and the settings.

use clap::{Subcommand, ValueEnum};

use crate::cli::OutputArgs;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum KindArg {
    Stt,
    Tts,
    Diarization,
    Text,
    Vad,
}

#[derive(Subcommand, Debug)]
pub enum Models {
    /// List available models
    List {
        #[arg(long, value_enum)]
        kind: Option<KindArg>,
        /// Only installed models
        #[arg(long)]
        installed: bool,
    },
    Info {
        id: String,
    },
    /// Download a model
    Pull {
        id: String,
    },
    Remove {
        id: String,
    },
    /// Re-check installed files against their checksums (all models, or one)
    Verify {
        id: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum Library {
    /// List stored transcripts, newest first
    List,
    /// Find transcripts by the words they contain
    Search {
        query: String,
        /// Most results to show
        #[arg(long, default_value_t = 20, value_name = "N")]
        limit: usize,
    },
    /// Print one transcript
    Show {
        id: String,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Number the lines of one transcript, to pick the one to edit
    Lines { id: String },
    /// Correct one line's words or speaker (line numbers from `vtt library lines`)
    Edit {
        id: String,
        /// Line number, starting at 1
        line: usize,
        /// The corrected words
        #[arg(long, required_unless_present = "speaker")]
        text: Option<String>,
        /// Who says it; a new name creates a speaker
        #[arg(long)]
        speaker: Option<String>,
    },
    /// Write transcripts to files (-o is a directory for several ids)
    Export {
        #[arg(required = true)]
        ids: Vec<String>,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Remove transcripts from the library
    Delete {
        #[arg(required = true)]
        ids: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum Config {
    /// Print the config file and data directory
    Path,
    /// Show every setting and its value
    List,
    /// Show one setting: model, language, device or identify-speakers
    Get { key: String },
    /// Change a setting; `auto` clears model and language. Flags still win for one run
    Set { key: String, value: String },
}
