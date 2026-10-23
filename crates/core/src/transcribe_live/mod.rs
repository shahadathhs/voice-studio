//! Live captions: while a recording runs, transcribe its last seconds again and again and split
//! the text into a settled part and a draft tail.
//!
//! Every pass the engine hears a sliding window (the last ~15 s). Words that two passes in a row
//! agree on are final; whatever the latest pass adds after them is a draft that may still change.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::audio::record::{Monitor, Recorder};
use crate::engines::{AudioBuffer, Control, SttEngine, SttOptions};
use crate::repetition::transcribe_without_loops;
use crate::{Error, Result};

/// How often the window is transcribed again.
pub const PASS_INTERVAL: Duration = Duration::from_secs(3);
/// How much recent audio each pass hears.
pub const WINDOW_SECONDS: f32 = 15.0;
/// Passes wait for at least this much audio.
const MIN_AUDIO: Duration = Duration::from_secs(1);
/// How often the stop flag is checked while waiting for the next pass.
const POLL: Duration = Duration::from_millis(100);
/// A window whose loudest sample is below this is silence; the engine is not asked about it
/// because speech models invent words for silence.
const SILENCE_PEAK: f32 = 0.005;
/// Fewest words that count as the same stretch when lining a window up with settled text.
const MIN_OVERLAP: usize = 2;

/// The captions at one moment.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Captions {
    /// Text that will not change any more.
    #[serde(rename = "final")]
    pub final_text: String,
    /// The tail of the latest pass; later passes may revise it.
    pub draft: String,
}

impl Captions {
    /// Settled text followed by the draft, as one string.
    pub fn full_text(&self) -> String {
        match (self.final_text.is_empty(), self.draft.is_empty()) {
            (_, true) => self.final_text.clone(),
            (true, false) => self.draft.clone(),
            (false, false) => format!("{} {}", self.final_text, self.draft),
        }
    }
}

/// Turns a series of overlapping window transcriptions into [`Captions`]. Pure: no audio, no
/// model, so the stabilising rule is tested directly.
#[derive(Debug, Default)]
pub struct Stabilizer {
    settled: Vec<String>,
    previous: Vec<String>,
    captions: Captions,
}

fn comparable(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn same_words(a: &[String], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| comparable(x) == comparable(y))
}

/// The part of `window` that is not already in `settled`: the window may start somewhere inside
/// the settled text, so its words that continue it are skipped.
fn after_settled<'a>(settled: &[String], window: &'a [String]) -> &'a [String] {
    let longest = settled.len().min(window.len());
    for overlap in (MIN_OVERLAP..=longest).rev() {
        if same_words(&settled[settled.len() - overlap..], &window[..overlap]) {
            return &window[overlap..];
        }
    }
    window
}

fn shared_prefix(a: &[String], b: &[String]) -> usize {
    a.iter()
        .zip(b)
        .take_while(|(x, y)| comparable(x) == comparable(y))
        .count()
}

impl Stabilizer {
    /// Takes the text of the newest pass and returns the captions after it.
    pub fn update(&mut self, window_text: &str) -> &Captions {
        let window: Vec<String> = window_text.split_whitespace().map(str::to_string).collect();
        let new = after_settled(&self.settled, &window);
        let old = after_settled(&self.settled, &self.previous);
        let stable = shared_prefix(old, new);
        let (agreed, draft) = new.split_at(stable);
        let draft = draft.join(" ");
        self.settled.extend(agreed.iter().cloned());
        self.captions = Captions {
            final_text: self.settled.join(" "),
            draft,
        };
        self.previous = window;
        &self.captions
    }

    /// The captions after the latest update.
    pub fn captions(&self) -> &Captions {
        &self.captions
    }
}

/// Where live captions get their audio; [`Recorder`] and [`Monitor`] are sources.
pub trait AudioSource {
    /// How much audio there is so far.
    fn elapsed(&self) -> Duration;
    /// The last `last_seconds`, mono at 16 kHz.
    fn snapshot(&self, last_seconds: f32) -> Result<AudioBuffer>;
}

impl AudioSource for Monitor {
    fn elapsed(&self) -> Duration {
        Monitor::elapsed(self)
    }
    fn snapshot(&self, last_seconds: f32) -> Result<AudioBuffer> {
        Monitor::snapshot(self, last_seconds)
    }
}

impl AudioSource for Recorder {
    fn elapsed(&self) -> Duration {
        Recorder::elapsed(self)
    }
    fn snapshot(&self, last_seconds: f32) -> Result<AudioBuffer> {
        Recorder::snapshot(self, last_seconds)
    }
}

fn is_silent(audio: &AudioBuffer) -> bool {
    audio.samples.iter().all(|s| s.abs() < SILENCE_PEAK)
}

/// Live captions for one recording.
pub struct LiveCaptions<'a> {
    /// The recording to listen to.
    pub source: &'a dyn AudioSource,
    /// Language and the like; `vad` is not used (silent windows are skipped instead).
    pub options: &'a SttOptions,
    /// Set this to end [`LiveCaptions::run`]; it also interrupts a pass in progress.
    pub stop: &'a AtomicBool,
    /// Time between passes. Use [`PASS_INTERVAL`] unless testing.
    pub interval: Duration,
    /// Seconds of audio per pass. Use [`WINDOW_SECONDS`] unless testing.
    pub window_seconds: f32,
}

impl LiveCaptions<'_> {
    /// Runs passes until `stop` is set, calling `on_update` with the captions after each one, and
    /// returns the last captions. Run it on its own thread; it blocks.
    pub fn run(
        &self,
        engine: &mut dyn SttEngine,
        on_update: &mut dyn FnMut(&Captions),
    ) -> Result<Captions> {
        let mut stabilizer = Stabilizer::default();
        let mut next_pass = Instant::now() + self.interval;
        while !self.stop.load(Ordering::Relaxed) {
            if Instant::now() < next_pass || self.source.elapsed() < MIN_AUDIO {
                sleep(POLL);
                continue;
            }
            let started = Instant::now();
            match self.pass(engine) {
                Ok(text) => on_update(stabilizer.update(&text)),
                Err(Error::Cancelled) => break,
                Err(e) => return Err(e),
            }
            next_pass = (started + self.interval).max(Instant::now() + POLL);
        }
        Ok(stabilizer.captions().clone())
    }

    fn pass(&self, engine: &mut dyn SttEngine) -> Result<String> {
        let audio = self.source.snapshot(self.window_seconds)?;
        if is_silent(&audio) {
            return Ok(String::new());
        }
        let stop = self.stop;
        let transcript = transcribe_without_loops(engine, &audio, self.options, &mut |_, _| {
            if stop.load(Ordering::Relaxed) {
                Control::Cancel
            } else {
                Control::Continue
            }
        })?;
        let lines: Vec<&str> = transcript.segments.iter().map(|s| s.text.trim()).collect();
        Ok(lines.join(" "))
    }
}

#[cfg(test)]
mod tests;
