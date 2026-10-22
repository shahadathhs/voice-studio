//! Argument definitions. Every command in the plan is declared here; commands that need an
//! engine are wired to `NotImplemented` in `commands.rs` until their phase lands.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "vtt",
    version,
    about = "Offline speech-to-text and text-to-speech"
)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Debug)]
pub struct Global {
    /// Machine-readable output (JSON / NDJSON events)
    #[arg(long, global = true)]
    pub json: bool,
    /// Suppress progress and informational output
    #[arg(short, long, global = true)]
    pub quiet: bool,
    /// More detail (repeatable)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,
    /// Data directory (models, library)
    #[arg(long, global = true, env = "VTT_DATA_DIR", value_name = "PATH")]
    pub data_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Transcribe audio or video files
    Transcribe(Transcribe),
    /// Convert text to speech
    Speak(Speak),
    /// Record from the microphone
    Record(Record),
    /// Convert a saved transcript (JSON) to another format
    Export(Export),
    /// Manage speech models and voices
    #[command(subcommand)]
    Models(Models),
    /// Browse the shared library
    #[command(subcommand)]
    Library(Library),
    /// List audio devices
    Devices,
    /// Show configuration locations
    #[command(subcommand)]
    Config(Config),
    /// Check that everything needed is available
    Doctor,
    /// Print a shell completion script
    Completions { shell: clap_complete::Shell },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum DeviceChoice {
    Auto,
    Cpu,
    Gpu,
}

/// Output-shaping options shared by `transcribe` and `export`.
#[derive(Args, Debug)]
pub struct OutputArgs {
    /// Output format: txt, srt, vtt, json, md, docx
    #[arg(short, long, default_value = "txt")]
    pub format: String,
    /// Output file, or directory for several inputs; '-' for stdout
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Include timestamps in txt/md
    #[arg(long)]
    pub timestamps: bool,
    /// Omit speaker labels
    #[arg(long)]
    pub no_speaker_names: bool,
    /// Subtitle line width
    #[arg(long, default_value_t = 42, value_name = "N")]
    pub max_line: usize,
    /// Subtitle lines per cue
    #[arg(long, default_value_t = 2, value_name = "N")]
    pub lines: usize,
}

impl OutputArgs {
    pub fn export_options(&self) -> vtt_core::export::ExportOptions {
        vtt_core::export::ExportOptions {
            speaker_names: !self.no_speaker_names,
            timestamps: self.timestamps,
            max_chars_per_line: self.max_line,
            lines_per_cue: self.lines,
        }
    }
}

/// Speaker identification options.
#[derive(Args, Debug)]
pub struct SpeakerArgs {
    /// Identify speakers
    #[arg(long)]
    pub diarize: bool,
    /// Exact number of speakers (alias: --max-speakers)
    #[arg(long, alias = "max-speakers", value_name = "N", requires = "diarize")]
    pub speakers: Option<usize>,
    /// How different two voices must be to count as two speakers, 0 to 2 (default 0.45); used when
    /// --speakers is not set
    #[arg(long, value_name = "X", requires = "diarize")]
    pub diarize_threshold: Option<f32>,
}

/// How speech is recognised.
#[derive(Args, Debug)]
pub struct RecognitionArgs {
    /// Translate to English
    #[arg(long)]
    pub translate: bool,
    /// Transcribe the whole file instead of only the stretches that contain speech
    #[arg(long)]
    pub no_vad: bool,
}

#[derive(Args, Debug)]
pub struct Transcribe {
    /// Audio/video files; use '-' to read audio from stdin
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
    /// STT model id
    #[arg(short, long, env = "VTT_MODEL")]
    pub model: Option<String>,
    /// Source language code (default: auto-detect)
    #[arg(short, long)]
    pub language: Option<String>,
    #[command(flatten)]
    pub recognition: RecognitionArgs,
    #[command(flatten)]
    pub speakers: SpeakerArgs,
    #[command(flatten)]
    pub out: OutputArgs,
    #[arg(long, value_enum, default_value_t = DeviceChoice::Auto, env = "VTT_DEVICE")]
    pub device: DeviceChoice,
    /// Do not add the result to the shared library
    #[arg(long)]
    pub no_library: bool,
    /// Accept prompts (e.g. download a missing model)
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct Speak {
    /// Text to speak (or use --file, or pipe it in; '-' reads stdin)
    pub text: Option<String>,
    /// Read the text from a file
    #[arg(short, long, value_name = "PATH")]
    pub file: Option<PathBuf>,
    /// Voice name, e.g. `af_heart` or just `heart` (see --list-voices)
    #[arg(long)]
    pub voice: Option<String>,
    /// Speaking speed, 0.5 to 2.0
    #[arg(long, default_value_t = 1.0, value_name = "0.5-2.0")]
    pub speed: f32,
    /// Output format; only wav so far
    #[arg(long, default_value = "wav")]
    pub format: String,
    /// Output file, '-' for stdout; default speech.wav
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Play the audio instead of saving it (not available yet)
    #[arg(long)]
    pub play: bool,
    /// List the installed voices and exit
    #[arg(long)]
    pub list_voices: bool,
    /// Accept prompts (e.g. download a missing model)
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct Record {
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Input device name
    #[arg(long)]
    pub device: Option<String>,
    /// Stop after this long, e.g. 30s
    #[arg(long)]
    pub duration: Option<String>,
    /// Transcribe when finished
    #[arg(long)]
    pub transcribe: bool,
}

#[derive(Args, Debug)]
pub struct Export {
    /// Transcript JSON file ('-' for stdin)
    pub input: PathBuf,
    #[command(flatten)]
    pub out: OutputArgs,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum KindArg {
    Stt,
    Tts,
    Diarization,
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
}
