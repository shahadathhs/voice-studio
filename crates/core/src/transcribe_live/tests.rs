use std::sync::Mutex;

use super::*;
use crate::{Segment, Transcript};

fn update(stabilizer: &mut Stabilizer, text: &str) -> (String, String) {
    let captions = stabilizer.update(text);
    (captions.final_text.clone(), captions.draft.clone())
}

#[test]
fn the_first_pass_is_all_draft() {
    let mut s = Stabilizer::default();
    assert_eq!(
        update(&mut s, "hello wor"),
        (String::new(), "hello wor".into())
    );
}

#[test]
fn words_two_passes_agree_on_become_final() {
    let mut s = Stabilizer::default();
    update(&mut s, "hello world how");
    let (done, draft) = update(&mut s, "Hello, world how are you");
    assert_eq!(done, "Hello, world how");
    assert_eq!(draft, "are you");
}

#[test]
fn a_revised_word_stays_draft_until_it_settles() {
    let mut s = Stabilizer::default();
    update(&mut s, "we went to the beach");
    let (done, draft) = update(&mut s, "we went to the speech today");
    assert_eq!(
        (done.as_str(), draft.as_str()),
        ("we went to the", "speech today")
    );
    let (done, draft) = update(&mut s, "we went to the speech today and");
    assert_eq!(
        (done.as_str(), draft.as_str()),
        ("we went to the speech today", "and")
    );
}

#[test]
fn a_sliding_window_does_not_repeat_settled_text() {
    let mut s = Stabilizer::default();
    update(&mut s, "one two three four five");
    update(&mut s, "one two three four five six");
    // The window has moved on: it starts at "three" and the earlier words are gone.
    update(&mut s, "three four five six seven");
    let (done, draft) = update(&mut s, "three four five six seven eight");
    assert_eq!(done, "one two three four five six seven");
    assert_eq!(draft, "eight");
}

#[test]
fn silence_keeps_what_is_settled_and_clears_the_draft() {
    let mut s = Stabilizer::default();
    update(&mut s, "good morning everyone");
    update(&mut s, "good morning everyone");
    let (done, draft) = update(&mut s, "");
    assert_eq!(
        (done.as_str(), draft.as_str()),
        ("good morning everyone", "")
    );
    assert_eq!(s.captions().full_text(), "good morning everyone");
}

#[test]
fn full_text_joins_final_and_draft() {
    let captions = Captions {
        final_text: "a b".into(),
        draft: "c".into(),
    };
    assert_eq!(captions.full_text(), "a b c");
    assert_eq!(Captions::default().full_text(), "");
}

struct Steady;

impl AudioSource for Steady {
    fn elapsed(&self) -> Duration {
        Duration::from_secs(10)
    }
    fn snapshot(&self, _: f32) -> Result<AudioBuffer> {
        Ok(AudioBuffer {
            samples: vec![0.2; 16_000],
            sample_rate: 16_000,
        })
    }
}

/// Answers with the next scripted sentence each time it is asked.
struct Scripted(Mutex<Vec<&'static str>>);

impl SttEngine for Scripted {
    fn transcribe(
        &mut self,
        _: &AudioBuffer,
        _: &SttOptions,
        _: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript> {
        let mut script = self.0.lock().unwrap();
        let text = if script.is_empty() {
            ""
        } else {
            script.remove(0)
        };
        Ok(Transcript {
            segments: vec![Segment {
                start_ms: 0,
                end_ms: 1000,
                speaker: None,
                text: text.into(),
            }],
            ..Transcript::default()
        })
    }
}

#[test]
fn run_reports_each_pass_and_returns_the_last_captions() {
    let stop = AtomicBool::new(false);
    let options = SttOptions::default();
    let live = LiveCaptions {
        source: &Steady,
        options: &options,
        stop: &stop,
        interval: Duration::from_millis(1),
        window_seconds: 15.0,
    };
    let mut engine = Scripted(Mutex::new(vec!["hello there", "hello there my friend"]));
    let mut seen = Vec::new();
    let last = live
        .run(&mut engine, &mut |captions| {
            seen.push(captions.clone());
            if seen.len() == 2 {
                stop.store(true, Ordering::Relaxed);
            }
        })
        .unwrap();
    assert_eq!(seen[0].draft, "hello there");
    assert_eq!(
        (last.final_text.as_str(), last.draft.as_str()),
        ("hello there", "my friend")
    );
}

#[test]
fn a_silent_window_never_reaches_the_engine() {
    struct Quiet;
    impl AudioSource for Quiet {
        fn elapsed(&self) -> Duration {
            Duration::from_secs(5)
        }
        fn snapshot(&self, _: f32) -> Result<AudioBuffer> {
            Ok(AudioBuffer {
                samples: vec![0.0; 16_000],
                sample_rate: 16_000,
            })
        }
    }
    struct Forbidden;
    impl SttEngine for Forbidden {
        fn transcribe(
            &mut self,
            _: &AudioBuffer,
            _: &SttOptions,
            _: &mut dyn FnMut(u64, u64) -> Control,
        ) -> Result<Transcript> {
            Err(Error::Engine("must not be called".into()))
        }
    }
    let stop = AtomicBool::new(false);
    let options = SttOptions::default();
    let live = LiveCaptions {
        source: &Quiet,
        options: &options,
        stop: &stop,
        interval: Duration::from_millis(1),
        window_seconds: 15.0,
    };
    let mut passes = 0;
    live.run(&mut Forbidden, &mut |_| {
        passes += 1;
        stop.store(true, Ordering::Relaxed);
    })
    .unwrap();
    assert_eq!(passes, 1);
}
