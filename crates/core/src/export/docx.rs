//! Word documents: one paragraph per line of the transcript, the speaker in bold.

use super::{ExportOptions, clock, non_empty, speaker_label, zip};
use crate::Transcript;

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;

const ROOT_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

const DOCUMENT_HEAD: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#;

fn escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn run(text: &str, bold: bool) -> String {
    let properties = if bold { "<w:rPr><w:b/></w:rPr>" } else { "" };
    format!(
        "<w:r>{properties}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
        escape(text)
    )
}

fn paragraph(label: Option<&str>, stamp: Option<String>, text: &str) -> String {
    let mut runs = String::new();
    if let Some(stamp) = stamp {
        runs.push_str(&run(&format!("[{stamp}] "), false));
    }
    if let Some(label) = label {
        runs.push_str(&run(&format!("{label}: "), true));
    }
    runs.push_str(&run(text, false));
    format!("<w:p>{runs}</w:p>")
}

/// The transcript as the bytes of a `.docx` file.
pub fn build(transcript: &Transcript, options: &ExportOptions) -> Vec<u8> {
    let mut document = String::from(DOCUMENT_HEAD);
    for (segment, text) in non_empty(transcript) {
        let stamp = options.timestamps.then(|| clock(segment.start_ms));
        document.push_str(&paragraph(speaker_label(segment, options), stamp, text));
    }
    document.push_str("</w:body></w:document>");
    zip::store(&[
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", ROOT_RELATIONSHIPS.as_bytes()),
        ("word/document.xml", document.as_bytes()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Segment;

    fn transcript() -> Transcript {
        Transcript {
            segments: vec![Segment {
                start_ms: 65_000,
                end_ms: 70_000,
                speaker: Some("Ana & Co".into()),
                text: "Is 2 < 3?".into(),
            }],
            ..Transcript::default()
        }
    }

    fn contents(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }

    #[test]
    fn the_file_holds_the_three_parts_of_a_word_document() {
        let text = contents(&build(&transcript(), &ExportOptions::default()));
        for part in ["[Content_Types].xml", "_rels/.rels", "word/document.xml"] {
            assert!(text.contains(part), "{part}");
        }
    }

    #[test]
    fn speaker_text_and_timestamp_are_in_the_document_and_escaped() {
        let options = ExportOptions {
            timestamps: true,
            ..ExportOptions::default()
        };
        let text = contents(&build(&transcript(), &options));
        assert!(text.contains("[00:01:05] "));
        assert!(text.contains("Ana &amp; Co: "));
        assert!(text.contains("Is 2 &lt; 3?"));
    }

    #[test]
    fn speaker_names_can_be_left_out() {
        let options = ExportOptions {
            speaker_names: false,
            ..ExportOptions::default()
        };
        assert!(!contents(&build(&transcript(), &options)).contains("Ana"));
    }
}
