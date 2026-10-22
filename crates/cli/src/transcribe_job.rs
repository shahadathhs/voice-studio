//! Per-file work for `vtt transcribe`: where the result goes, progress reporting, and writing.

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use vtt_core::diarize::{SpeakerDiarizer, SpeakerModel};
use vtt_core::engines::{Control, DiarizeOptions, SttOptions, WhisperEngine};
use vtt_core::export::{self, ExportOptions, Format};
use vtt_core::jobs::{Event, Throttle};
use vtt_core::transcribe::{Speakers, transcribe_file};
use vtt_core::{Error, Result, Transcript};

use crate::output::Output;

const PROGRESS_EVERY: Duration = Duration::from_millis(100);

pub enum Destination {
    Stdout,
    File(PathBuf),
    Dir(PathBuf),
}

impl Destination {
    pub fn resolve(files: &[PathBuf], output: Option<&Path>) -> Result<Destination> {
        match output {
            None => Self::stdout_or_error(files),
            Some(p) if p == Path::new("-") => Self::stdout_or_error(files),
            Some(p) if files.len() > 1 || p.is_dir() => Ok(Destination::Dir(p.to_path_buf())),
            Some(p) => Ok(Destination::File(p.to_path_buf())),
        }
    }

    fn stdout_or_error(files: &[PathBuf]) -> Result<Destination> {
        if files.len() > 1 {
            return Err(Error::Input(
                "several input files need -o <directory>".into(),
            ));
        }
        Ok(Destination::Stdout)
    }
}

pub struct Job<'a> {
    pub file: &'a Path,
    pub format: Format,
    pub export_options: &'a ExportOptions,
    pub destination: &'a Destination,
}

/// Everything that turns audio into a transcript: the loaded models and their settings.
pub struct Recognizer<'a> {
    pub engine: &'a mut WhisperEngine,
    pub model: &'a str,
    pub options: &'a SttOptions,
    /// The speaker labeller and its settings, when `--diarize` was given.
    pub speakers: Option<(SpeakerDiarizer<SpeakerModel>, DiarizeOptions)>,
}

impl Job<'_> {
    pub fn run(&self, recognizer: &mut Recognizer<'_>, out: Output) -> Result<Transcript> {
        let model = recognizer.model;
        let name = self
            .file
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let mut reporter = Reporter::new(out, &name, model);
        let speakers = recognizer
            .speakers
            .as_mut()
            .map(|(diarizer, options)| Speakers { diarizer, options });
        let result = transcribe_file(
            recognizer.engine,
            self.file,
            recognizer.options,
            speakers,
            &mut |done, total| {
                reporter.update(done, total);
                Control::Continue
            },
        );
        reporter.finish_line();
        let transcript = result?;
        let written = self.write(&transcript, out)?;
        emit(
            out,
            &Event::Done {
                output: written,
                elapsed_ms: reporter.elapsed_ms(),
            },
        );
        Ok(transcript)
    }

    /// Writes the transcript where the user asked and returns the path, if it went to a file.
    fn write(&self, transcript: &Transcript, out: Output) -> Result<Option<String>> {
        let rendered = export::render(transcript, self.format, self.export_options)?;
        let path = match self.destination {
            Destination::Stdout => {
                if out.json {
                    emit(
                        out,
                        &Event::Transcript {
                            transcript: transcript.clone(),
                        },
                    );
                } else {
                    std::io::stdout().write_all(rendered.as_bytes())?;
                }
                return Ok(None);
            }
            Destination::File(path) => path.clone(),
            Destination::Dir(dir) => {
                std::fs::create_dir_all(dir)?;
                let stem = self
                    .file
                    .file_stem()
                    .map_or_else(|| "transcript".into(), |s| s.to_string_lossy().into_owned());
                dir.join(format!("{stem}.{}", self.format.extension()))
            }
        };
        std::fs::write(&path, rendered)?;
        Ok(Some(path.display().to_string()))
    }
}

/// Shows progress for one file: a percentage line on a terminal, or NDJSON events with `--json`.
struct Reporter<'a> {
    out: Output,
    name: &'a str,
    model: &'a str,
    started: Instant,
    throttle: Throttle,
    announced: bool,
    last_done: Option<u64>,
    show_bar: bool,
}

impl<'a> Reporter<'a> {
    fn new(out: Output, name: &'a str, model: &'a str) -> Reporter<'a> {
        Reporter {
            out,
            name,
            model,
            started: Instant::now(),
            throttle: Throttle::new(PROGRESS_EVERY),
            announced: false,
            last_done: None,
            show_bar: out.show_messages() && std::io::stderr().is_terminal(),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn update(&mut self, done: u64, total: u64) {
        if !self.announced {
            self.announced = true;
            let (file, model) = (self.name.to_string(), self.model.to_string());
            emit(
                self.out,
                &Event::Start {
                    file,
                    model,
                    duration_ms: total,
                },
            );
        }
        // Skip repeats of the same position (the engine reports 0 while it warms up).
        if self.last_done == Some(done) || !(self.throttle.ready() || done == total) {
            return;
        }
        self.last_done = Some(done);
        if self.out.json {
            let speed = done as f32 / (self.started.elapsed().as_millis().max(1) as f32);
            emit(
                self.out,
                &Event::Progress {
                    processed_ms: done,
                    total_ms: total,
                    speed,
                },
            );
        } else if self.show_bar {
            let percent = done.saturating_mul(100).checked_div(total).unwrap_or(100);
            eprint!("\r  {}  {percent:>3}%   ", self.name);
        }
    }

    fn finish_line(&self) {
        if self.show_bar {
            eprintln!();
        }
    }

    fn elapsed_ms(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

fn emit(out: Output, event: &Event) {
    if out.json
        && let Ok(line) = serde_json::to_string(event)
    {
        println!("{line}");
    }
}
