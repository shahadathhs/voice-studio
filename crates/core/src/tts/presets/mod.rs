//! Voice presets: a saved blend of shipped voices of one language, e.g. 60% `af_heart` and 40%
//! `af_bella`. A preset is a small JSON file in `<data>/voices/<id>.json`; the blend is made from
//! the shipped style tables when the voice is used.
//!
//! **Ids.** `<language letter><gender letter>_my-<slug of the name>`, e.g. `af_my-warm`. The first
//! two letters come from the first part, so [`Language::of_voice`] and [`super::describe`] keep
//! working; `my-` marks a custom voice and can never clash with a shipped voice name. The slug is
//! the name in lower case with every run of characters other than ASCII letters and digits turned
//! into one `-`.
//!
//! **No real people.** A preset only mixes the synthetic voices that ship with Kokoro. It cannot
//! be made from a recording, so it cannot copy anyone's voice, and its description says it is a
//! custom blend.

mod blend;
mod store;

use serde::{Deserialize, Serialize};

use super::language::Language;
use crate::{Error, Result};

pub(super) use blend::{STYLE_SIZE, blend, read_table};
pub use store::{delete, list, load, save, voice_list};

pub const MIN_PARTS: usize = 2;
pub const MAX_PARTS: usize = 4;
pub const MAX_NAME_CHARS: usize = 40;
const MAX_SLUG_CHARS: usize = 30;
const ID_MARK: &str = "my-";

/// One shipped voice in a blend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Part {
    pub voice: String,
    pub weight: f32,
}

/// A custom voice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    /// Weights sum to 1.
    pub parts: Vec<Part>,
}

pub(crate) fn slug(name: &str) -> String {
    let mut slug = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.truncate(MAX_SLUG_CHARS);
    slug.trim_end_matches('-').to_string()
}

/// Whether `id` could be a preset id (and so is safe as a file name).
pub fn is_preset_id(id: &str) -> bool {
    id.split_once('_').is_some_and(|(prefix, rest)| {
        prefix.len() == 2
            && Language::of_voice(prefix).is_some()
            && rest.starts_with(ID_MARK)
            && rest.len() > ID_MARK.len()
            && rest
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}

impl Preset {
    /// Checks the parts and makes the preset: 2 to 4 distinct voices of one language, weights above
    /// zero (they are scaled to sum to 1), a name of 1 to 40 characters.
    pub fn new(name: &str, parts: &[(String, f32)]) -> Result<Preset> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
            return Err(Error::Input(format!(
                "a voice name needs 1 to {MAX_NAME_CHARS} characters"
            )));
        }
        let slug = slug(name);
        if slug.is_empty() {
            return Err(Error::Input(
                "a voice name needs at least one letter or digit (A-Z, 0-9)".into(),
            ));
        }
        if !(MIN_PARTS..=MAX_PARTS).contains(&parts.len()) {
            return Err(Error::Input(format!(
                "a blend needs {MIN_PARTS} to {MAX_PARTS} voices"
            )));
        }
        let prefix = voice_prefix(&parts[0].0)?;
        for (index, (voice, weight)) in parts.iter().enumerate() {
            if !weight.is_finite() || *weight <= 0.0 {
                return Err(Error::Input(format!(
                    "the weight of {voice} must be above 0"
                )));
            }
            if voice_prefix(voice)?.chars().next() != prefix.chars().next() {
                return Err(Error::Input(
                    "a blend can only mix voices of the same language".into(),
                ));
            }
            if parts[..index].iter().any(|(other, _)| other == voice) {
                return Err(Error::Input(format!("{voice} is in the blend twice")));
            }
        }
        let total: f32 = parts.iter().map(|(_, weight)| weight).sum();
        Ok(Preset {
            id: format!("{prefix}_{ID_MARK}{slug}"),
            name: name.to_string(),
            parts: parts
                .iter()
                .map(|(voice, weight)| Part {
                    voice: voice.clone(),
                    weight: weight / total,
                })
                .collect(),
        })
    }

    pub fn language(&self) -> Option<Language> {
        Language::of_voice(&self.id)
    }

    /// Blends the voices' tables, which `read` returns by voice id.
    pub(super) fn blend(&self, read: impl Fn(&str) -> Result<Vec<f32>>) -> Result<Vec<f32>> {
        let tables = self
            .parts
            .iter()
            .map(|p| Ok((read(&p.voice)?, p.weight)))
            .collect::<Result<Vec<_>>>()?;
        blend(&tables)
    }

    /// Changes when the id or the mix does, so a cached blend is never stale.
    pub(super) fn cache_key(&self) -> String {
        let mix: Vec<String> = self
            .parts
            .iter()
            .map(|p| format!("{}:{}", p.voice, p.weight))
            .collect();
        format!("{}|{}", self.id, mix.join(","))
    }
}

/// The two letters (language, gender) a shipped voice id starts with.
fn voice_prefix(voice: &str) -> Result<&str> {
    voice
        .split_once('_')
        .map(|(prefix, _)| prefix)
        .filter(|p| p.len() == 2 && Language::of_voice(p).is_some())
        .ok_or_else(|| Error::Input(format!("{voice} is not a voice id like af_heart")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(list: &[(&str, f32)]) -> Vec<(String, f32)> {
        list.iter().map(|(v, w)| ((*v).to_string(), *w)).collect()
    }

    #[test]
    fn a_preset_gets_the_language_and_gender_of_its_first_voice() {
        let p = Preset::new(
            " Warm Voice! ",
            &parts(&[("af_heart", 3.0), ("af_bella", 1.0)]),
        )
        .unwrap();
        assert_eq!(p.id, "af_my-warm-voice");
        assert_eq!(p.name, "Warm Voice!");
        assert_eq!(p.language(), Some(Language::AmericanEnglish));
        assert!((p.parts[0].weight - 0.75).abs() < 1e-6);
        assert!((p.parts.iter().map(|x| x.weight).sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(is_preset_id(&p.id));
        let mixed = Preset::new("Mix", &parts(&[("bm_george", 1.0), ("bf_emma", 1.0)])).unwrap();
        assert_eq!(mixed.id, "bm_my-mix");
    }

    #[test]
    fn bad_blends_are_refused() {
        let two = parts(&[("af_heart", 1.0), ("af_bella", 1.0)]);
        assert!(Preset::new("", &two).is_err());
        assert!(Preset::new(&"x".repeat(41), &two).is_err());
        assert!(Preset::new("!!!", &two).is_err());
        assert!(Preset::new("A", &parts(&[("af_heart", 1.0)])).is_err());
        let five = parts(&[
            ("af_a", 1.0),
            ("af_b", 1.0),
            ("af_c", 1.0),
            ("af_d", 1.0),
            ("af_e", 1.0),
        ]);
        assert!(Preset::new("A", &five).is_err());
        assert!(Preset::new("A", &parts(&[("af_heart", 1.0), ("ef_dora", 1.0)])).is_err());
        assert!(Preset::new("A", &parts(&[("af_heart", 1.0), ("af_bella", 0.0)])).is_err());
        assert!(Preset::new("A", &parts(&[("af_heart", 1.0), ("af_bella", f32::NAN)])).is_err());
        assert!(Preset::new("A", &parts(&[("af_heart", 1.0), ("af_heart", 1.0)])).is_err());
        assert!(Preset::new("A", &parts(&[("heart", 1.0), ("bella", 1.0)])).is_err());
    }

    #[test]
    fn only_well_formed_ids_are_preset_ids() {
        assert!(is_preset_id("ef_my-mia"));
        for bad in [
            "af_heart",
            "af_my-",
            "af_my-../x",
            "af_my-A",
            "qq_my-x",
            "../af_my-x",
        ] {
            assert!(!is_preset_id(bad), "{bad}");
        }
    }
}
