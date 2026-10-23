//! Whisper sometimes gets stuck and writes the same words over and over ("we have a lot of
//! information about this" fifty times) while the audio holds something else. This finds such
//! loops and folds them into one occurrence. Real speech does repeat itself, so only runs far
//! longer than people speak are touched.

use crate::Segment;
use crate::engines::{AudioBuffer, Control, SttEngine, SttOptions};
use crate::{Result, Transcript};

/// This many identical lines in a row is a loop, not emphasis.
const LINE_RUN: usize = 3;
/// A group of words repeated this many times in a line is a loop...
const PHRASE_REPEATS: usize = 4;
/// ...except a single word, which people do repeat ("no, no, no, no"), so it takes more.
const WORD_REPEATS: usize = 6;
/// Longest group of words looked for.
const MAX_PHRASE_WORDS: usize = 8;
/// Randomness for the second try at audio that made the decoder loop.
const RETRY_TEMPERATURE: f32 = 0.5;

/// A word without case or punctuation, so "Okay." and "okay" count as the same.
fn word_key(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn line_key(text: &str) -> String {
    text.split_whitespace()
        .map(word_key)
        .collect::<Vec<_>>()
        .join(" ")
}

/// How many times the `len` words starting at `at` repeat back to back.
fn repeats(keys: &[String], at: usize, len: usize) -> usize {
    let phrase = &keys[at..at + len];
    keys[at..]
        .chunks_exact(len)
        .take_while(|chunk| *chunk == phrase)
        .count()
}

/// The words of `text` with every looped phrase kept once. Returns `None` when there is no loop.
fn fold_phrases(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let keys: Vec<String> = words.iter().map(|w| word_key(w)).collect();
    let mut kept: Vec<&str> = Vec::with_capacity(words.len());
    let mut folded = false;
    let mut at = 0;
    while at < words.len() {
        let found = (1..=MAX_PHRASE_WORDS.min(words.len() - at)).find_map(|len| {
            let needed = if len == 1 {
                WORD_REPEATS
            } else {
                PHRASE_REPEATS
            };
            let count = repeats(&keys, at, len);
            (count >= needed).then_some((len, count))
        });
        if let Some((len, count)) = found {
            kept.extend(&words[at..at + len]);
            at += len * count;
            folded = true;
        } else {
            kept.push(words[at]);
            at += 1;
        }
    }
    folded.then(|| kept.join(" "))
}

/// Whether the lines contain a loop: the same line several times in a row, or a phrase repeated
/// inside a line.
pub fn has_loop(lines: &[Segment]) -> bool {
    let mut run = 0;
    let mut previous = String::new();
    for line in lines {
        if fold_phrases(&line.text).is_some() {
            return true;
        }
        let key = line_key(&line.text);
        run = if key == previous { run + 1 } else { 1 };
        if run >= LINE_RUN {
            return true;
        }
        previous = key;
    }
    false
}

/// The lines with every loop folded into one occurrence: a run of identical lines becomes the
/// first of them, stretched over the whole run, and a looped phrase inside a line is kept once.
/// A run shorter than a loop is left as it is.
pub fn fold(lines: Vec<Segment>) -> Vec<Segment> {
    let lines: Vec<Segment> = lines
        .into_iter()
        .map(|mut line| {
            if let Some(text) = fold_phrases(&line.text) {
                line.text = text;
            }
            line
        })
        .collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut at = 0;
    while at < lines.len() {
        let key = line_key(&lines[at].text);
        let run = lines[at..]
            .iter()
            .take_while(|line| line_key(&line.text) == key)
            .count();
        if run >= LINE_RUN {
            let mut first = lines[at].clone();
            first.end_ms = lines[at + run - 1].end_ms;
            out.push(first);
        } else {
            out.extend_from_slice(&lines[at..at + run]);
        }
        at += run;
    }
    out
}

/// Transcribes `audio`. If the decoder got stuck in a loop, tries once more with some randomness
/// and keeps that when it is clean; either way the result has any remaining loop folded.
/// `on_progress` follows the first try; the retry only reports whether to cancel.
pub fn transcribe_without_loops(
    engine: &mut dyn SttEngine,
    audio: &AudioBuffer,
    options: &SttOptions,
    on_progress: &mut dyn FnMut(u64, u64) -> Control,
) -> Result<Transcript> {
    let mut transcript = engine.transcribe(audio, options, on_progress)?;
    if has_loop(&transcript.segments) {
        let retry_options = SttOptions {
            temperature: RETRY_TEMPERATURE,
            ..options.clone()
        };
        let retry = engine.transcribe(audio, &retry_options, &mut |_, _| Control::Continue)?;
        if !has_loop(&retry.segments) {
            transcript = retry;
        }
    }
    transcript.segments = fold(transcript.segments);
    Ok(transcript)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(start_ms: u64, text: &str) -> Segment {
        Segment {
            start_ms,
            end_ms: start_ms + 1000,
            speaker: None,
            text: text.into(),
        }
    }

    fn texts(lines: &[Segment]) -> Vec<&str> {
        lines.iter().map(|l| l.text.as_str()).collect()
    }

    /// Loops at temperature 0 and speaks normally when there is randomness, like a stuck decoder.
    struct Stuck;

    impl SttEngine for Stuck {
        fn transcribe(
            &mut self,
            _audio: &AudioBuffer,
            options: &SttOptions,
            _on_progress: &mut dyn FnMut(u64, u64) -> Control,
        ) -> Result<Transcript> {
            let texts: Vec<&str> = if options.temperature > 0.0 {
                vec!["Something different.", "And then it moves on."]
            } else {
                vec!["Same words."; 5]
            };
            let segments = texts
                .iter()
                .enumerate()
                .map(|(n, text)| line(n as u64 * 1000, text))
                .collect();
            Ok(Transcript {
                segments,
                ..Transcript::default()
            })
        }
    }

    fn silence() -> AudioBuffer {
        AudioBuffer {
            samples: vec![0.0; 16_000],
            sample_rate: 16_000,
        }
    }

    #[test]
    fn a_loop_is_retried_with_randomness_and_the_clean_try_is_kept() {
        let result = transcribe_without_loops(
            &mut Stuck,
            &silence(),
            &SttOptions::default(),
            &mut |_, _| Control::Continue,
        )
        .unwrap();
        assert_eq!(
            texts(&result.segments),
            ["Something different.", "And then it moves on."]
        );
    }

    /// Loops whatever the temperature.
    struct AlwaysStuck;
    impl SttEngine for AlwaysStuck {
        fn transcribe(
            &mut self,
            _: &AudioBuffer,
            _: &SttOptions,
            _: &mut dyn FnMut(u64, u64) -> Control,
        ) -> Result<Transcript> {
            Ok(Transcript {
                segments: (0..5).map(|n| line(n * 1000, "Same words.")).collect(),
                ..Transcript::default()
            })
        }
    }

    #[test]
    fn when_the_retry_loops_too_the_loop_is_folded() {
        let options = SttOptions {
            temperature: 0.9,
            ..SttOptions::default()
        };
        let result =
            transcribe_without_loops(&mut AlwaysStuck, &silence(), &options, &mut |_, _| {
                Control::Continue
            })
            .unwrap();
        assert_eq!(texts(&result.segments), ["Same words."]);
    }

    #[test]
    fn ordinary_speech_has_no_loop() {
        let lines = [
            line(0, "Okay, we've had a problem here."),
            line(1000, "This is Houston, say again please."),
            line(2000, "We've had a problem here."),
        ];
        assert!(!has_loop(&lines));
        assert_eq!(texts(&fold(lines.to_vec())).len(), 3);
    }

    #[test]
    fn two_repeated_lines_are_emphasis_but_three_are_a_loop() {
        let two = [line(0, "Yes."), line(1000, "yes"), line(2000, "Right.")];
        assert!(!has_loop(&two));
        let three = [line(0, "Yes."), line(1000, "yes"), line(2000, "Yes!")];
        assert!(has_loop(&three));
    }

    #[test]
    fn a_run_of_identical_lines_folds_into_the_first_stretched_over_the_run() {
        let lines: Vec<Segment> = (0..5)
            .map(|n| line(n * 1000, "We have a lot of information about this."))
            .chain([line(5000, "Next topic.")])
            .collect();
        let folded = fold(lines);
        assert_eq!(
            texts(&folded),
            ["We have a lot of information about this.", "Next topic."]
        );
        assert_eq!((folded[0].start_ms, folded[0].end_ms), (0, 5000));
    }

    #[test]
    fn a_phrase_repeated_inside_a_line_is_kept_once() {
        let text = "So we have a lot of information about this we have a lot of information about \
                    this we have a lot of information about this we have a lot of information \
                    about this and then we left.";
        let folded = fold(vec![line(0, text)]);
        assert_eq!(
            folded[0].text,
            "So we have a lot of information about this and then we left."
        );
        assert!(has_loop(&[line(0, text)]));
    }

    #[test]
    fn a_word_needs_more_repeats_than_a_phrase() {
        assert!(!has_loop(&[line(0, "No, no, no, no, I said no.")]));
        assert!(has_loop(&[line(0, "ha ha ha ha ha ha ha")]));
        assert_eq!(fold(vec![line(0, "ha ha ha ha ha ha ha")])[0].text, "ha");
    }
}
