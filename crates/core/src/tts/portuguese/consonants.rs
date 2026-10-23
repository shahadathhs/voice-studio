//! The sound of each consonant unit, given the sounds around it.

use super::units::{Unit, UnitKind, is_vowel};

/// How the word's `x` is said when it is not between the usual letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XSound {
    Sh,
    Ks,
    S,
}

/// Words with these letters say `x` as `ks` (sexo, táxi, fixo) or as `s` (máximo, próximo).
pub fn x_sound(spelled: &str) -> XSound {
    const KS: [&str; 10] = [
        "sex", "fix", "tax", "tox", "lex", "flux", "oxig", "anex", "conex", "axio",
    ];
    const S: [&str; 3] = ["maxim", "proxim", "trouxe"];
    if S.iter().any(|p| spelled.contains(p)) {
        XSound::S
    } else if KS.iter().any(|p| spelled.contains(p)) {
        XSound::Ks
    } else {
        XSound::Sh
    }
}

/// Letters that make a stop in front of them take a helping `i` (adivogado, ritmo, optar).
const HELPED: [&str; 14] = [
    "b", "c", "d", "f", "g", "j", "k", "m", "n", "p", "s", "t", "v", "z",
];
const VOICED: [&str; 11] = ["b", "d", "g", "v", "l", "m", "n", "r", "rr", "z", "j"];

/// A word's units with the sounds already found for its vowels.
pub struct Spelling<'a> {
    pub units: &'a [Unit],
    pub vowels: &'a [String],
    pub x: XSound,
}

fn fixed(text: &str) -> Option<&'static str> {
    Some(match text {
        "ch" => "ʃ",
        "lh" => "lj",
        "nh" => "ɲ",
        "ss" | "ç" => "s",
        "qu" | "q" | "k" => "k",
        "qw" => "kw",
        "gw" => "ɡw",
        "gu" => "ɡ",
        "j" => "ʒ",
        "rr" => "x",
        "h" => "",
        "w" => "w",
        _ => return None,
    })
}

impl Spelling<'_> {
    fn at(&self, i: usize) -> Option<&Unit> {
        self.units.get(i)
    }

    fn before(&self, i: usize) -> Option<&Unit> {
        i.checked_sub(1).and_then(|p| self.units.get(p))
    }

    fn vowel_after(&self, i: usize) -> bool {
        self.at(i + 1).is_some_and(is_vowel)
    }

    fn front_after(&self, i: usize) -> bool {
        self.at(i + 1)
            .is_some_and(|n| is_vowel(n) && matches!(n.base, 'e' | 'i'))
    }

    /// The next vowel is said `i` or `j`: t and d turn into `ʧ` and `ʤ`.
    fn palatal_after(&self, i: usize) -> bool {
        self.vowels
            .get(i + 1)
            .is_some_and(|v| v.starts_with('i') || v.starts_with('j'))
    }

    /// A stop that ends a syllable gets a helping `i`, as Brazilians say it.
    fn helped(&self, i: usize) -> bool {
        match self.at(i + 1) {
            None => self.before(i).is_some_and(is_vowel),
            Some(next) => next.kind == UnitKind::Consonant && HELPED.contains(&next.text.as_str()),
        }
    }

    fn stop(&self, i: usize, plain: &str, palatal: &str) -> String {
        let helped = self.helped(i);
        let sound = if helped || self.palatal_after(i) {
            palatal
        } else {
            plain
        };
        format!("{sound}{}", if helped { "i" } else { "" })
    }

    fn r(&self, i: usize) -> &'static str {
        let initial = self
            .before(i)
            .is_none_or(|p| p.consumed.is_some() || matches!(p.text.as_str(), "s" | "l" | "n"));
        if initial {
            "x"
        } else if self.vowel_after(i) || self.at(i + 1).is_none() {
            "ɾ"
        } else {
            "x"
        }
    }

    fn s(&self, i: usize) -> &'static str {
        let after_vowel = self
            .before(i)
            .is_some_and(|p| is_vowel(p) && p.consumed.is_none());
        match self.at(i + 1) {
            Some(n) if after_vowel && is_vowel(n) => "z",
            Some(n) if after_vowel && VOICED.contains(&n.text.as_str()) => "z",
            _ => "s",
        }
    }

    fn c(&self, i: usize) -> String {
        if !self.front_after(i) {
            return self.stop(i, "k", "k");
        }
        let silent = self
            .before(i)
            .is_some_and(|p| matches!(p.text.as_str(), "s" | "x"));
        if silent { String::new() } else { "s".into() }
    }

    fn x(&self, i: usize) -> &'static str {
        let next = self.at(i + 1);
        if next.is_some_and(|n| n.text == "c") {
            return "s";
        }
        let after_e = i == 1 && self.units[0].base == 'e' && is_vowel(&self.units[0]);
        if after_e {
            return if self.vowel_after(i) { "z" } else { "s" };
        }
        match self.x {
            XSound::Ks => "ks",
            XSound::S => "s",
            XSound::Sh if i > 0 && next.is_some_and(|n| n.kind == UnitKind::Consonant) => "s",
            XSound::Sh => "ʃ",
        }
    }

    /// The sound of consonant unit `i`.
    pub fn sound(&self, i: usize) -> String {
        let text = self.units[i].text.as_str();
        if let Some(sound) = fixed(text) {
            return sound.to_string();
        }
        match text {
            "t" => self.stop(i, "t", "ʧ"),
            "d" => self.stop(i, "d", "ʤ"),
            "p" | "b" => self.stop(i, text, text),
            "g" if self.front_after(i) => "ʒ".into(),
            "g" => self.stop(i, "ɡ", "ɡ"),
            "c" => self.c(i),
            "r" => self.r(i).into(),
            "s" => self.s(i).into(),
            "x" => self.x(i).into(),
            "z" if self.at(i + 1).is_none() => "s".into(),
            "l" if self.vowel_after(i) => "l".into(),
            "l" => "w".into(),
            "f" | "v" | "m" | "n" | "z" => text.into(),
            _ => String::new(),
        }
    }
}
