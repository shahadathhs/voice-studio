use super::{ExportOptions, clock, non_empty, speaker_label};
use crate::{Segment, Transcript};

pub fn render(t: &Transcript, o: &ExportOptions) -> String {
    non_empty(t)
        .map(|(seg, text)| paragraph(seg, text, o))
        .collect()
}

fn paragraph(seg: &Segment, text: &str, o: &ExportOptions) -> String {
    let mut head = Vec::new();
    if let Some(who) = speaker_label(seg, o) {
        head.push(format!("**{who}**"));
    }
    if o.timestamps {
        head.push(format!("`{}`", clock(seg.start_ms)));
    }
    if head.is_empty() {
        format!("{text}\n\n")
    } else {
        format!("{}  \n{text}\n\n", head.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Segment;

    #[test]
    fn paragraph_per_segment() {
        let t = Transcript {
            segments: vec![Segment {
                start_ms: 0,
                end_ms: 1,
                speaker: Some("Maria".into()),
                text: "Hello.".into(),
            }],
            ..Default::default()
        };
        assert_eq!(
            render(&t, &ExportOptions::default()),
            "**Maria**  \nHello.\n\n"
        );
    }
}
