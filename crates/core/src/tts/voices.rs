//! Names for voices. Kokoro names a voice `<accent><gender>_<name>`: `af_heart` is the American
//! female voice Heart, `bm_george` the British male voice George.

use serde::Serialize;

use super::language::Language;
use super::presets::Preset;

/// A voice as a front end shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VoiceInfo {
    /// The name used in files and on the command line, e.g. `af_heart`.
    pub id: String,
    /// The voice's own name, e.g. `Heart`.
    pub name: String,
    /// e.g. `American English, female`.
    pub description: String,
    /// The language code the voice speaks (`es`, `en`), for grouping; empty if unknown.
    pub language: String,
    /// A voice the user made by blending shipped voices (see [`super::presets`]).
    pub custom: bool,
    /// A voice cloned from a recording (see [`super::clone`]); always shown as such.
    pub cloned: bool,
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
        let (_, g) = (codes.next()?, codes.next()?);
        codes.next().is_none().then_some(())?;
        Some((Language::of_voice(prefix)?, gender(g)?, name))
    });
    match parsed {
        Some((language, gender, name)) => VoiceInfo {
            id: id.to_string(),
            name: capitalised(name),
            description: format!("{}, {gender}", language.name()),
            language: language.code().to_string(),
            custom: false,
            cloned: false,
        },
        None => VoiceInfo {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            language: String::new(),
            custom: false,
            cloned: false,
        },
    }
}

/// Describes a custom voice: its own name, and which shipped voices it blends.
pub fn describe_preset(preset: &Preset) -> VoiceInfo {
    let language = preset.language();
    let mix: Vec<String> = preset
        .parts
        .iter()
        .map(|p| format!("{} {:.0}%", describe(&p.voice).name, p.weight * 100.0))
        .collect();
    VoiceInfo {
        id: preset.id.clone(),
        name: preset.name.clone(),
        description: format!(
            "{}, custom blend: {}",
            language.map_or("", Language::name),
            mix.join(", ")
        ),
        language: language.map_or_else(String::new, |l| l.code().to_string()),
        custom: true,
        cloned: false,
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
        let dora = describe("ef_dora");
        assert_eq!(
            (dora.description.as_str(), dora.language.as_str()),
            ("Spanish, female", "es")
        );
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
