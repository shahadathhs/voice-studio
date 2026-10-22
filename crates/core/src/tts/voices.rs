//! Names for voices. Kokoro names a voice `<accent><gender>_<name>`: `af_heart` is the American
//! female voice Heart, `bm_george` the British male voice George.

use serde::Serialize;

/// A voice as a front end shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VoiceInfo {
    /// The name used in files and on the command line, e.g. `af_heart`.
    pub id: String,
    /// The voice's own name, e.g. `Heart`.
    pub name: String,
    /// e.g. `American English, female`.
    pub description: String,
}

fn accent(code: char) -> Option<&'static str> {
    match code {
        'a' => Some("American English"),
        'b' => Some("British English"),
        _ => None,
    }
}

fn gender(code: char) -> Option<&'static str> {
    match code {
        'f' => Some("female"),
        'm' => Some("male"),
        _ => None,
    }
}

fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// Describes a voice from its id. Ids that do not follow the pattern keep their id as the name.
pub fn describe(id: &str) -> VoiceInfo {
    let parsed = id.split_once('_').and_then(|(prefix, name)| {
        let mut codes = prefix.chars();
        let (a, g) = (codes.next()?, codes.next()?);
        codes.next().is_none().then_some(())?;
        Some((accent(a)?, gender(g)?, name))
    });
    match parsed {
        Some((accent, gender, name)) => VoiceInfo {
            id: id.to_string(),
            name: capitalised(name),
            description: format!("{accent}, {gender}"),
        },
        None => VoiceInfo {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_voice_id_tells_the_accent_gender_and_name() {
        let heart = describe("af_heart");
        assert_eq!(heart.name, "Heart");
        assert_eq!(heart.description, "American English, female");
        assert_eq!(describe("bm_george").description, "British English, male");
    }

    #[test]
    fn ids_that_do_not_follow_the_pattern_keep_their_id() {
        let odd = describe("custom");
        assert_eq!(
            (odd.name.as_str(), odd.description.as_str()),
            ("custom", "")
        );
        assert_eq!(describe("zz_nobody").name, "zz_nobody");
        assert_eq!(describe("afx_name").description, "");
    }
}
