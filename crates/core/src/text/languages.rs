//! Names of the languages a text can be translated into or summarised in, as the language model
//! is told them.

const NAMES: [(&str, &str); 20] = [
    ("en", "English"),
    ("es", "Spanish"),
    ("fr", "French"),
    ("de", "German"),
    ("it", "Italian"),
    ("pt", "Portuguese"),
    ("nl", "Dutch"),
    ("ru", "Russian"),
    ("pl", "Polish"),
    ("tr", "Turkish"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
    ("bn", "Bengali"),
    ("ur", "Urdu"),
    ("zh", "Chinese"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("vi", "Vietnamese"),
    ("id", "Indonesian"),
    ("th", "Thai"),
];

/// The codes that can be asked for, in the order shown to people.
pub fn codes() -> impl Iterator<Item = &'static str> {
    NAMES.iter().map(|(code, _)| *code)
}

/// The English name of a language code such as `bn` or `zh-CN`; `None` if it is not supported.
pub fn name(code: &str) -> Option<&'static str> {
    let main = code.split(['-', '_']).next().unwrap_or(code);
    NAMES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(main))
        .map(|(_, name)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regional_codes_find_their_language() {
        assert_eq!(name("zh-CN"), Some("Chinese"));
        assert_eq!(name("BN"), Some("Bengali"));
        assert_eq!(name("xx"), None);
    }

    #[test]
    fn every_code_has_a_name() {
        assert!(codes().all(|code| name(code).is_some()));
    }
}
