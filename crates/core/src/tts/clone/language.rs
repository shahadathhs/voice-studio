//! The languages Chatterbox is used for here, and how text is prepared for each.
//!
//! The model reads a language tag in front of the text (`[es]Hola`). English, Spanish, French,
//! Italian, Portuguese and Hindi need nothing else. Chinese (Cangjie codes), Japanese (kana),
//! Korean (jamo) and Hebrew (vowel marks) need extra front ends that are not built, so they are
//! refused rather than spoken badly.

use crate::{Error, Result};

/// Language codes accepted by [`prepare`].
pub const LANGUAGES: [&str; 6] = ["en", "es", "fr", "it", "pt", "hi"];

/// `text` as the model reads it: tidied, ending in a sentence mark, behind the tag for `language`.
pub fn prepare(text: &str, language: &str) -> Result<String> {
    let code = language.trim().to_lowercase();
    if !LANGUAGES.contains(&code.as_str()) {
        return Err(Error::Input(format!(
            "cloned voices speak {}; {language} is not supported yet",
            LANGUAGES.join(", ")
        )));
    }
    let mut tidy = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if tidy.is_empty() {
        return Err(Error::Input("there is no text to speak".into()));
    }
    if !tidy.ends_with(['.', '!', '?', '…', '。', '।', ',', ';', ':']) {
        tidy.push('.');
    }
    Ok(format!("[{code}]{tidy}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_gets_a_language_tag_and_a_final_mark() {
        assert_eq!(prepare("  Hola   mundo ", "ES").unwrap(), "[es]Hola mundo.");
        assert_eq!(prepare("Hi there!", "en").unwrap(), "[en]Hi there!");
        assert_eq!(prepare("नमस्ते", "hi").unwrap(), "[hi]नमस्ते.");
    }

    #[test]
    fn unsupported_languages_and_empty_text_are_input_errors() {
        assert!(matches!(prepare("x", "zh"), Err(Error::Input(m)) if m.contains("not supported")));
        assert!(prepare("x", "").is_err());
        assert!(matches!(prepare("   ", "en"), Err(Error::Input(_))));
    }
}
