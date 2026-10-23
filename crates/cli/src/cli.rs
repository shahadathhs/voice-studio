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
    /// Summarise a transcript with a language model
    Summarize(crate::text_cmd::Summarize),
    /// Translate a transcript with a language model
    Translate(crate::text_cmd::Translate),
    /// Record from the microphone
    Record(Record),
    /// Convert a saved transcript (JSON) to another format
    Export(Export),
    /// Manage speech models and voices
    #[command(subcommand)]
    Models(Models),
    /// Make and manage custom voices
    #[command(subcommand)]
    Voice(crate::voice_cmd::VoiceCommand),
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
    /// Identify speakers (also on when `vtt config set identify-speakers true`)
    #[arg(long)]
    pub diarize: bool,
    /// Exact number of speakers (alias: --max-speakers)
    #[arg(long, alias = "max-speakers", value_name = "N", requires = "diarize")]
    pub speakers: Option<usize>,
    /// How different two voices must be to count as two speakers, 0 to 2 (default: chosen
    /// automatically); used when --speakers is not set
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
    /// STT model id (default: from `vtt config`)
    #[arg(short, long, env = "VTT_MODEL")]
    pub model: Option<String>,
    /// Source language code (default: from `vtt config`, else auto-detect)
    #[arg(short, long)]
    pub language: Option<String>,
    #[command(flatten)]
    pub recognition: RecognitionArgs,
    #[command(flatten)]
    pub speakers: SpeakerArgs,
    #[command(flatten)]
    pub out: OutputArgs,
    /// Hardware to run on (default: from `vtt config`, else auto)
    #[arg(long, value_enum, env = "VTT_DEVICE")]
    pub device: Option<DeviceChoice>,
    /// Do not add the result to the shared library
    #[arg(long)]
    pub no_library: bool,
    /// Accept prompts (e.g. download a missing model)
    #[arg(short = 'y', long)]
    pub yes: bool,
}

impl Transcribe {
    /// The arguments `vtt transcribe <file>` would have with no other flags, plus these choices.
    pub fn plain(
        file: PathBuf,
        model: Option<String>,
        language: Option<String>,
        yes: bool,
    ) -> Self {
        Transcribe {
            files: vec![file],
            model,
            language,
            recognition: RecognitionArgs {
                translate: false,
                no_vad: false,
            },
            speakers: SpeakerArgs {
                diarize: false,
                speakers: None,
                diarize_threshold: None,
            },
            out: OutputArgs {
                format: "txt".into(),
                output: None,
                timestamps: false,
                no_speaker_names: false,
                max_line: 42,
                lines: 2,
            },
            device: None,
            no_library: false,
            yes,
        }
    }
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
    /// Language of the text, for a cloned voice (en, es, fr, it, pt, hi; default: --translate-to, else en)
    #[arg(long, value_name = "LANG")]
    pub language: Option<String>,
    /// Output format: wav or flac (default: from the output file's extension, else wav)
    #[arg(long)]
    pub format: Option<String>,
    /// Output file, '-' for stdout; default speech.wav (or speech.flac)
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Translate the text into this language first (e.g. es, zh) and speak it with a voice of
    /// that language; needs a language model (`vtt models list --kind text`)
    #[arg(long, value_name = "LANG")]
    pub translate_to: Option<String>,
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
    /// Where to save the recording (.wav or .flac); default recording.wav
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Input device name (see --list-devices)
    #[arg(long)]
    pub device: Option<String>,
    /// Stop after this long, e.g. 30s, 2m, 1m30s
    #[arg(long, value_parser = crate::record_cmd::parse_duration, value_name = "TIME")]
    pub duration: Option<std::time::Duration>,
    /// Transcribe the recording when finished (like `vtt transcribe`)
    #[arg(long)]
    pub transcribe: bool,
    /// Show draft captions on stderr while recording; the text goes to stdout at the end
    #[arg(long)]
    pub live: bool,
    /// List the input devices and exit
    #[arg(long)]
    pub list_devices: bool,
    #[command(flatten)]
    pub recognition: RecordRecognition,
}

/// Which speech model and language `vtt record` uses for `--live` and `--transcribe`.
#[derive(Args, Debug)]
pub struct RecordRecognition {
    /// STT model (default: from `vtt config`)
    #[arg(short, long, env = "VTT_MODEL")]
    pub model: Option<String>,
    /// Spoken language code (default: from `vtt config`)
    #[arg(short, long)]
    pub language: Option<String>,
    /// Accept prompts (e.g. download a missing model)
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct Export {
    /// Transcript JSON file ('-' for stdin)
    pub input: PathBuf,
    #[command(flatten)]
    pub out: OutputArgs,
}

pub use crate::manage::{Config, KindArg, Library, Models};
