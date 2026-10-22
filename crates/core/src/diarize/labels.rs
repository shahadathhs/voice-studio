//! From speaker embeddings of audio chunks to one speaker per transcript line: which chunks to
//! embed, how chunk labels become line labels, and smoothing of one-off flips.

use crate::Segment;

/// Lines at least this long are embedded in sliding windows, shorter ones as a whole.
const WINDOWED_FROM_MS: u64 = 3_000;
const WINDOW_MS: u64 = 1_500;
const STRIDE_MS: u64 = 500;
/// Audio shorter than this carries too little voice to embed.
const MIN_CHUNK_MS: u64 = 500;
/// Lines shorter than this are not embedded themselves; they take the label of what is near them.
const MIN_LINE_MS: u64 = 1_000;
/// A short line that differs from both neighbours, which agree with each other, is flipped.
const SMOOTHING_MAX_MS: u64 = 2_000;

/// A stretch of audio to embed, and the line it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chunk {
    pub start_ms: u64,
    pub end_ms: u64,
    pub line: usize,
}

/// The chunks to embed for these lines.
pub fn plan_chunks(lines: &[Segment]) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    for (line, segment) in lines.iter().enumerate() {
        let (start, end) = (segment.start_ms, segment.end_ms);
        let length = end.saturating_sub(start);
        if length >= WINDOWED_FROM_MS {
            let mut t = start;
            while t + WINDOW_MS <= end {
                chunks.push(Chunk {
                    start_ms: t,
                    end_ms: t + WINDOW_MS,
                    line,
                });
                t += STRIDE_MS;
            }
            if t < end && end - t >= MIN_CHUNK_MS {
                let from = t.max(end - WINDOW_MS);
                chunks.push(Chunk {
                    start_ms: from,
                    end_ms: end,
                    line,
                });
            }
        } else if length >= MIN_LINE_MS {
            chunks.push(Chunk {
                start_ms: start,
                end_ms: end,
                line,
            });
        }
    }
    chunks
}

fn center(start_ms: u64, end_ms: u64) -> u64 {
    u64::midpoint(start_ms, end_ms)
}

/// One label per line: the most common label among the line's chunks, or for lines without
/// chunks the label of the chunk whose centre is closest in time. `chunks[i]` has label
/// `labels[i]`; both must be non-empty.
pub fn label_lines(lines: &[Segment], chunks: &[Chunk], labels: &[usize]) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .map(|(line, segment)| {
            vote(line, chunks, labels).unwrap_or_else(|| {
                let middle = center(segment.start_ms, segment.end_ms);
                chunks
                    .iter()
                    .zip(labels)
                    .min_by_key(|(c, _)| center(c.start_ms, c.end_ms).abs_diff(middle))
                    .map_or(0, |(_, label)| *label)
            })
        })
        .collect()
}

/// Most common label among the chunks of `line`; the one seen first wins a tie.
fn vote(line: usize, chunks: &[Chunk], labels: &[usize]) -> Option<usize> {
    let mut counts: Vec<(usize, usize)> = Vec::new();
    for (chunk, label) in chunks.iter().zip(labels) {
        if chunk.line != line {
            continue;
        }
        match counts.iter_mut().find(|(l, _)| l == label) {
            Some((_, n)) => *n += 1,
            None => counts.push((*label, 1)),
        }
    }
    counts
        .iter()
        .fold(None, |best: Option<(usize, usize)>, entry| match best {
            Some(b) if b.1 >= entry.1 => Some(b),
            _ => Some(*entry),
        })
        .map(|(label, _)| label)
}

/// Removes flicker: a short line between two lines of the same other speaker joins them.
pub fn smooth(lines: &[Segment], labels: &mut [usize]) {
    let mut changed = true;
    while changed {
        changed = false;
        for i in 1..labels.len().saturating_sub(1) {
            let length = lines[i].end_ms.saturating_sub(lines[i].start_ms);
            if length <= SMOOTHING_MAX_MS
                && labels[i - 1] == labels[i + 1]
                && labels[i] != labels[i - 1]
            {
                labels[i] = labels[i - 1];
                changed = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(start_ms: u64, end_ms: u64) -> Segment {
        Segment {
            start_ms,
            end_ms,
            speaker: None,
            text: "x".into(),
        }
    }

    #[test]
    fn short_lines_are_dropped_medium_ones_embedded_whole() {
        let chunks = plan_chunks(&[line(0, 800), line(1000, 2500)]);
        assert_eq!(
            chunks,
            [Chunk {
                start_ms: 1000,
                end_ms: 2500,
                line: 1
            }]
        );
    }

    #[test]
    fn long_lines_get_overlapping_windows_and_a_final_one_ending_at_the_line_end() {
        let chunks = plan_chunks(&[line(0, 4200)]);
        let spans: Vec<(u64, u64)> = chunks.iter().map(|c| (c.start_ms, c.end_ms)).collect();
        assert_eq!(
            spans,
            [
                (0, 1500),
                (500, 2000),
                (1000, 2500),
                (1500, 3000),
                (2000, 3500),
                (2500, 4000),
                (3000, 4200)
            ]
        );
    }

    #[test]
    fn a_line_takes_the_most_common_label_of_its_chunks() {
        let lines = [line(0, 3500)];
        let chunks = plan_chunks(&lines);
        assert_eq!(chunks.len(), 6);
        assert_eq!(label_lines(&lines, &chunks, &[1, 0, 0, 1, 1, 1]), [1]);
        // Two votes each: the label seen first wins.
        assert_eq!(label_lines(&lines, &chunks, &[2, 2, 0, 1, 0, 1]), [2]);
    }

    #[test]
    fn lines_without_chunks_take_the_label_of_the_nearest_chunk() {
        let lines = [line(0, 2000), line(2600, 2900), line(3000, 5000)];
        let chunks = plan_chunks(&lines);
        assert_eq!(chunks.len(), 2);
        assert_eq!(label_lines(&lines, &chunks, &[0, 1]), [0, 1, 1]);
    }

    #[test]
    fn a_short_line_between_two_lines_of_one_speaker_is_flipped() {
        let lines = [line(0, 5000), line(5000, 6000), line(6000, 11000)];
        let mut labels = [0, 1, 0];
        smooth(&lines, &mut labels);
        assert_eq!(labels, [0, 0, 0]);
    }

    #[test]
    fn long_lines_and_real_changes_of_speaker_stay() {
        let lines = [
            line(0, 5000),
            line(5000, 9000),
            line(9000, 12000),
            line(12000, 13000),
        ];
        let mut labels = [0, 1, 0, 1];
        smooth(&lines, &mut labels);
        assert_eq!(labels, [0, 1, 0, 1]);
    }
}
