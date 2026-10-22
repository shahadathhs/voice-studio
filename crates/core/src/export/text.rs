use super::{ExportOptions, clock, non_empty, speaker_label};
use crate::{Segment, Transcript};

pub fn render(t: &Transcript, o: &ExportOptions) -> String {
    non_empty(t).map(|(seg, text)| line(seg, text, o)).collect()
}

fn line(seg: &Segment, text: &str, o: &ExportOptions) -> String {
    let stamp = if o.timestamps {
        format!("[{}] ", clock(seg.start_ms))
    } else {
        String::new()
    };
    let who = speaker_label(seg, o)
        .map(|s| format!("{s}: "))
        .unwrap_or_default();
    format!("{stamp}{who}{text}\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Segment;

    fn sample() -> Transcript {
        Transcript {
            segments: vec![
                Segment {
                    start_ms: 0,
                    end_ms: 2000,
                    speaker: Some("Maria".into()),
                    text: " Hello. ".into(),
                },
                Segment {
                    start_ms: 2000,
                    end_ms: 3000,
                    speaker: None,
                    text: String::new(),
                },
                Segment {
                    start_ms: 65_000,
                    end_ms: 70_000,
                    speaker: Some("Daniel".into()),
                    text: "Hi.".into(),
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn speakers_and_timestamps() {
        let o = ExportOptions {
            timestamps: true,
            ..Default::default()
        };
        assert_eq!(
            render(&sample(), &o),
            "[00:00:00] Maria: Hello.\n[00:01:05] Daniel: Hi.\n"
        );
    }

    #[test]
    fn speaker_names_can_be_hidden() {
        let o = ExportOptions {
            speaker_names: false,
            ..Default::default()
        };
        assert_eq!(render(&sample(), &o), "Hello.\nHi.\n");
    }
}
