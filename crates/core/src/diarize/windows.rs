//! Cutting a recording into overlapping segmentation windows, and finding in each window the stretches
//! each local speaker talks, which are what gets embedded.

use super::segmenter::{
    FIELD_SAMPLES, FRAME_SAMPLES, Frame, LOCAL_SPEAKERS, Segmenter, WINDOW_SAMPLES,
};
use crate::engines::Control;
use crate::{Error, Result};

/// A local speaker needs at least this many frames (about 1 s) of speech to be embedded; less
/// gives an unreliable voiceprint.
const MIN_EMBED_FRAMES: usize = 60;
/// A pause of more than this many frames (about 0.25 s) ends a stretch of speech.
const MAX_GAP_FRAMES: usize = 15;

/// One segmentation window: where it starts in the recording and its frames.
pub struct Window {
    pub start: usize,
    pub frames: Vec<Frame>,
}

/// One local speaker of one window, with the frames to listen to and the frames to vote with.
#[derive(Debug, PartialEq, Eq)]
pub struct LocalTurn {
    pub window: usize,
    /// Frames where this speaker talks.
    pub active: Vec<usize>,
    /// Frames to embed: the active ones where nobody else talks, or all active ones when too few
    /// are free of overlap.
    pub listen: Vec<usize>,
}

/// Where each window starts: every `shift` samples, and one that ends exactly at the end of the
/// recording. A recording shorter than a window has a single zero-padded one.
pub fn window_starts(length: usize, shift: usize) -> Vec<usize> {
    if length <= WINDOW_SAMPLES {
        return vec![0];
    }
    let last = length - WINDOW_SAMPLES;
    let mut starts: Vec<usize> = (0..=last).step_by(shift.max(1)).collect();
    if starts.last() != Some(&last) {
        starts.push(last);
    }
    starts
}

/// Runs the segmenter over the whole recording. `on_window(done, total)` is called after each
/// window and can stop the work by returning [`Control::Cancel`].
pub fn analyse(
    samples: &[f32],
    shift: usize,
    segmenter: &mut dyn Segmenter,
    on_window: &mut dyn FnMut(usize, usize) -> Control,
) -> Result<Vec<Window>> {
    let starts = window_starts(samples.len(), shift);
    let total = starts.len();
    let mut windows = Vec::with_capacity(total);
    for (done, start) in starts.into_iter().enumerate() {
        let end = (start + WINDOW_SAMPLES).min(samples.len());
        let mut window = samples[start..end].to_vec();
        window.resize(WINDOW_SAMPLES, 0.0);
        windows.push(Window {
            start,
            frames: segmenter.activity(&window)?,
        });
        if on_window(done + 1, total) == Control::Cancel {
            return Err(Error::Cancelled);
        }
    }
    Ok(windows)
}

/// The first sample of the 270 samples a frame stands for, centred in its receptive field.
pub fn frame_start(window_start: usize, frame: usize) -> usize {
    window_start + frame * FRAME_SAMPLES + (FIELD_SAMPLES - FRAME_SAMPLES) / 2
}

/// Splits ascending `frames` into runs, wherever more than [`MAX_GAP_FRAMES`] frames are skipped or
/// one of the `cuts` (sample positions, ascending) lies between two frames.
fn runs(frames: &[usize], window_start: usize, cuts: &[usize]) -> Vec<Vec<usize>> {
    let region = |frame: usize| cuts.partition_point(|c| *c <= frame_middle(window_start, frame));
    let mut out: Vec<Vec<usize>> = Vec::new();
    for frame in frames {
        match out.last_mut() {
            Some(run)
                if frame - run[run.len() - 1] <= MAX_GAP_FRAMES
                    && region(*frame) == region(run[run.len() - 1]) =>
            {
                run.push(*frame);
            }
            _ => out.push(vec![*frame]),
        }
    }
    out
}

/// A turn from one run of a local speaker's frames, when it has enough speech to be embedded.
fn turn_of(window: &Window, index: usize, run: Vec<usize>) -> Option<LocalTurn> {
    let alone: Vec<usize> = run
        .iter()
        .copied()
        .filter(|f| window.frames[*f].iter().filter(|a| **a).count() == 1)
        .collect();
    let listen = if alone.len() >= MIN_EMBED_FRAMES {
        alone
    } else if run.len() >= MIN_EMBED_FRAMES {
        run.clone()
    } else {
        return None;
    };
    Some(LocalTurn {
        window: index,
        active: run,
        listen,
    })
}

/// The centre of the 270 samples a frame stands for.
fn frame_middle(window_start: usize, frame: usize) -> usize {
    frame_start(window_start, frame) + FRAME_SAMPLES / 2
}

/// Every stretch of one local speaker's speech with enough in it to be embedded, window by window.
/// A pause, or the start or end of a transcript line (`cuts`), ends a stretch, so one voiceprint is
/// not made from several people taking turns.
pub fn local_turns(windows: &[Window], cuts: &[usize]) -> Vec<LocalTurn> {
    let mut turns = Vec::new();
    for (index, window) in windows.iter().enumerate() {
        for speaker in 0..LOCAL_SPEAKERS {
            let active: Vec<usize> = (0..window.frames.len())
                .filter(|f| window.frames[*f][speaker])
                .collect();
            turns.extend(
                runs(&active, window.start, cuts)
                    .into_iter()
                    .filter_map(|run| turn_of(window, index, run)),
            );
        }
    }
    turns
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: usize = 16_000;

    #[test]
    fn windows_overlap_and_the_last_one_ends_with_the_recording() {
        assert_eq!(
            window_starts(32 * SECOND, 5 * SECOND),
            [0, 80_000, 160_000, 240_000, 320_000, 352_000]
        );
        assert_eq!(
            window_starts(25 * SECOND, 5 * SECOND),
            [0, 80_000, 160_000, 240_000]
        );
        assert_eq!(window_starts(10 * SECOND, 5 * SECOND), [0]);
    }

    #[test]
    fn a_short_recording_has_one_padded_window() {
        assert_eq!(window_starts(3 * SECOND, 5 * SECOND), [0]);
        assert_eq!(window_starts(0, 5 * SECOND), [0]);
    }

    fn window_with(frames: &[Frame]) -> Window {
        Window {
            start: 0,
            frames: frames.to_vec(),
        }
    }

    #[test]
    fn speech_free_of_overlap_is_what_gets_embedded() {
        let one = [true, false, false];
        let both = [true, true, false];
        let mut frames = vec![one; 70];
        frames.extend(vec![both; 30]);
        let turns = local_turns(&[window_with(&frames)], &[]);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].active.len(), 100);
        assert_eq!(turns[0].listen.len(), 70);
    }

    #[test]
    fn overlapped_speech_is_embedded_when_little_else_is_left() {
        let both = [true, true, false];
        let turns = local_turns(&[window_with(&vec![both; 80])], &[]);
        assert_eq!(turns.len(), 2);
        assert!(turns.iter().all(|t| t.listen.len() == 80));
    }

    #[test]
    fn a_speaker_with_under_a_second_of_speech_is_not_embedded() {
        let turns = local_turns(&[window_with(&[[true, false, false]; 40])], &[]);
        assert_eq!(turns.len(), 0);
    }
}
