//! Transcript exporters. Pure functions: transcript in, string out.
//! DOCX needs a zip writer and arrives later.

mod cues;
mod json;
mod markdown;
mod text;

use std::str::FromStr;

use crate::{Error, Result, Segment, Transcript};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Txt,
    Srt,
    Vtt,
    Json,
    Markdown,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Txt => "txt",
            Format::Srt => "srt",
            Format::Vtt => "vtt",
            Format::Json => "json",
            Format::Markdown => "md",
        }
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Format> {
        match s.to_ascii_lowercase().as_str() {
            "txt" | "text" => Ok(Format::Txt),
            "srt" => Ok(Format::Srt),
            "vtt" => Ok(Format::Vtt),
            "json" => Ok(Format::Json),
            "md" | "markdown" => Ok(Format::Markdown),
            "docx" => Err(Error::NotImplemented("docx export")),
            other => Err(Error::Input(format!("unknown format: {other}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub speaker_names: bool,
    pub timestamps: bool,
    pub max_chars_per_line: usize,
    pub lines_per_cue: usize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            speaker_names: true,
            timestamps: false,
            max_chars_per_line: 42,
            lines_per_cue: 2,
        }
    }
}

pub fn render(transcript: &Transcript, format: Format, options: &ExportOptions) -> Result<String> {
    Ok(match format {
        Format::Txt => text::render(transcript, options),
        Format::Srt => cues::render_srt(transcript, options),
        Format::Vtt => cues::render_vtt(transcript, options),
        Format::Json => json::render(transcript)?,
        Format::Markdown => markdown::render(transcript, options),
    })
}

/// `HH:MM:SS` for text exports.
pub(crate) fn clock(ms: u64) -> String {
    let s = ms / 1000;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

/// Segments that contain text, paired with that text trimmed.
pub(crate) fn non_empty(t: &Transcript) -> impl Iterator<Item = (&Segment, &str)> {
    t.segments.iter().filter_map(|seg| {
        let text = seg.text.trim();
        (!text.is_empty()).then_some((seg, text))
    })
}

/// The speaker label to show, or `None` when names are turned off or unknown.
pub(crate) fn speaker_label<'a>(seg: &'a Segment, o: &ExportOptions) -> Option<&'a str> {
    if o.speaker_names {
        seg.speaker.as_deref()
    } else {
        None
    }
}
