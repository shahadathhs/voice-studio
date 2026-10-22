//! SRT and VTT. A segment is word-wrapped to `max_chars_per_line`, grouped into cues of
//! `lines_per_cue` lines, and the segment's time span is split between its cues in proportion
//! to their length, so cue boundaries always tile the original segment exactly.

use super::{ExportOptions, non_empty, speaker_label};
use crate::Transcript;

struct Cue {
    start_ms: u64,
    end_ms: u64,
    lines: Vec<String>,
}

pub fn render_srt(t: &Transcript, o: &ExportOptions) -> String {
    cues(t, o)
        .iter()
        .enumerate()
        .map(|(i, cue)| srt_cue(i + 1, cue))
        .collect()
}

pub fn render_vtt(t: &Transcript, o: &ExportOptions) -> String {
    let body: String = cues(t, o).iter().map(vtt_cue).collect();
    format!("WEBVTT\n\n{body}")
}

fn srt_cue(number: usize, cue: &Cue) -> String {
    let (start, end) = (stamp(cue.start_ms, ','), stamp(cue.end_ms, ','));
    format!("{number}\n{start} --> {end}\n{}\n\n", cue.lines.join("\n"))
}

fn vtt_cue(cue: &Cue) -> String {
    let (start, end) = (stamp(cue.start_ms, '.'), stamp(cue.end_ms, '.'));
    format!("{start} --> {end}\n{}\n\n", cue.lines.join("\n"))
}

fn stamp(ms: u64, sep: char) -> String {
    format!(
        "{:02}:{:02}:{:02}{sep}{:03}",
        ms / 3_600_000,
        (ms / 60_000) % 60,
        (ms / 1000) % 60,
        ms % 1000
    )
}

fn cues(t: &Transcript, o: &ExportOptions) -> Vec<Cue> {
    let per_cue = o.lines_per_cue.max(1);
    let mut out = Vec::new();
    for (seg, text) in non_empty(t) {
        let full = match speaker_label(seg, o) {
            Some(who) => format!("{who}: {text}"),
            None => text.to_string(),
        };
        let lines = wrap(&full, o.max_chars_per_line);
        let total: u64 = lines
            .iter()
            .map(|l| l.chars().count() as u64)
            .sum::<u64>()
            .max(1);
        let span = seg.end_ms.saturating_sub(seg.start_ms);
        let mut used = 0u64;
        for group in lines.chunks(per_cue) {
            let start = seg.start_ms + span * used / total;
            used += group.iter().map(|l| l.chars().count() as u64).sum::<u64>();
            let end = seg.start_ms + span * used / total;
            out.push(Cue {
                start_ms: start,
                end_ms: end,
                lines: group.to_vec(),
            });
        }
    }
    out
}

/// Greedy word wrap. A single word longer than `max` is kept whole on its own line.
fn wrap(text: &str, max: usize) -> Vec<String> {
    let max = max.max(1);
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            cur.push_str(word);
        } else if cur.chars().count() + 1 + word.chars().count() <= max {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur.push_str(word);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Segment;

    fn one(text: &str, start: u64, end: u64) -> Transcript {
        Transcript {
            segments: vec![Segment {
                start_ms: start,
                end_ms: end,
                speaker: Some("Maria Chen".into()),
                text: text.into(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn srt_wraps_to_two_lines_in_one_cue() {
        let t = one("So if we ship the importer behind a flag first", 0, 4000);
        assert_eq!(
            render_srt(&t, &ExportOptions::default()),
            "1\n00:00:00,000 --> 00:00:04,000\nMaria Chen: So if we ship the importer\nbehind a flag first\n\n"
        );
    }

    #[test]
    fn long_segment_splits_into_cues_that_tile_the_span() {
        let t = one(&"word ".repeat(40), 1000, 9000);
        let cues = cues(
            &t,
            &ExportOptions {
                speaker_names: false,
                ..Default::default()
            },
        );
        assert!(cues.len() > 1);
        assert_eq!(cues.first().unwrap().start_ms, 1000);
        assert_eq!(cues.last().unwrap().end_ms, 9000);
        for pair in cues.windows(2) {
            assert_eq!(pair[0].end_ms, pair[1].start_ms);
        }
    }

    #[test]
    fn vtt_has_header_and_dot_separator() {
        let t = one("Hi.", 1500, 2500);
        let o = ExportOptions {
            speaker_names: false,
            ..Default::default()
        };
        assert_eq!(
            render_vtt(&t, &o),
            "WEBVTT\n\n00:00:01.500 --> 00:00:02.500\nHi.\n\n"
        );
    }

    #[test]
    fn timestamps_roll_over_hours() {
        assert_eq!(stamp(3_723_004, ','), "01:02:03,004");
    }

    #[test]
    fn overlong_word_stays_whole() {
        assert_eq!(wrap("abcdefghij kl", 4), vec!["abcdefghij", "kl"]);
    }
}
