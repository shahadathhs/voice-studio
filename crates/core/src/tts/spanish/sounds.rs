//! Spanish letters to phonemes.

use super::syllables::{head, is_vowel, nuclei, stressed};

const STRESS: char = 'ˈ';
/// Castilian `c` before e or i, and `z`. Latin American Spanish says `s` here.
const THETA: &str = "θ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Vowel,
    Consonant,
}

/// A vowel, or a consonant sound as written (`ch`, `ll`, `qu`…).
#[derive(Debug, Clone)]
pub struct Unit {
    pub kind: UnitKind,
    /// The letters, lower case and without accents.
    pub text: String,
    /// The vowel without its accent.
    pub base: char,
    pub accented: bool,
}

fn plain(c: char) -> (char, bool) {
    match c {
        'á' => ('a', true),
        'é' => ('e', true),
        'í' => ('i', true),
        'ó' => ('o', true),
        'ú' => ('u', true),
        'ü' => ('u', false),
        other => (other, false),
    }
}

fn consonant(text: &str) -> Unit {
    Unit {
        kind: UnitKind::Consonant,
        text: text.to_string(),
        base: ' ',
        accented: false,
    }
}

fn vowel(c: char) -> Unit {
    let (base, accented) = plain(c);
    Unit {
        kind: UnitKind::Vowel,
        text: base.to_string(),
        base,
        accented,
    }
}

fn is_front(c: Option<&char>) -> bool {
    c.is_some_and(|c| "eiéí".contains(*c))
}

/// Cuts a word into vowels and consonant units.
fn units_of(word: &str) -> Vec<Unit> {
    let chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    let mut units = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let (c, next) = (chars[i], chars.get(i + 1));
        let pair = |a: char, b: char| c == a && next == Some(&b);
        let step = if pair('c', 'h') || pair('l', 'l') || pair('r', 'r') {
            units.push(consonant(&format!("{c}{}", chars[i + 1])));
            2
        } else if (pair('q', 'u') || (c == 'g' && next == Some(&'u') && is_front(chars.get(i + 2))))
            && chars.len() > i + 2
        {
            units.push(consonant(&format!("{c}u")));
            2
        } else if pair('g', 'ü') {
            units.push(consonant("gü"));
            2
        } else if "aeiouáéíóúü".contains(c) {
            units.push(vowel(c));
            1
        } else if c == 'y'
            && (i + 1 == chars.len() || next.is_none_or(|n| !"aeiouáéíóú".contains(*n)))
        {
            units.push(vowel('i'));
            1
        } else {
            units.push(consonant(&c.to_string()));
            1
        };
        i += step;
    }
    units
}

/// Sounds that do not depend on the letters around them.
fn fixed(text: &str) -> Option<&'static str> {
    Some(match text {
        "ch" => "ʧ",
        "j" => "x",
        "ll" | "y" => "ʝ",
        "ñ" => "ɲ",
        "q" | "qu" | "k" => "k",
        "rr" => "r",
        "z" => THETA,
        "h" => "",
        _ => return None,
    })
}

fn pick<'a>(condition: bool, yes: &'a str, no: &'a str) -> &'a str {
    if condition { yes } else { no }
}

/// The sound of a consonant unit given the unit before it (`None` at the start of the word).
fn consonant_sound(unit: &Unit, previous: Option<&Unit>, next: Option<&Unit>) -> String {
    if let Some(sound) = fixed(&unit.text) {
        return sound.to_string();
    }
    let before = previous.map(|p| p.text.as_str());
    let after = next.map(|n| n.text.as_str());
    let after_nasal = matches!(before, Some("n" | "m"));
    let hard = before.is_none() || after_nasal;
    let before_front = next.is_some_and(|n| n.kind == UnitKind::Vowel && "eiy".contains(n.base));
    let sound = match unit.text.as_str() {
        "b" | "v" => pick(hard, "b", "β"),
        "c" => pick(before_front, THETA, "k"),
        "d" => pick(hard || before == Some("l"), "d", "ð"),
        "g" if before_front => "x",
        "g" | "gu" => pick(hard, "ɡ", "ɣ"),
        "gü" => pick(hard, "ɡw", "ɣw"),
        "r" => pick(
            before.is_none() || after_nasal || matches!(before, Some("s" | "l")),
            "r",
            "ɾ",
        ),
        "x" => pick(before.is_none(), "s", "ks"),
        "n" if matches!(after, Some("b" | "v" | "p")) => "m",
        "n" if matches!(after, Some("g" | "gu" | "gü" | "k" | "qu" | "j")) => "ŋ",
        other => other,
    };
    sound.to_string()
}

/// The sound of a vowel; a weak vowel beside the head of its syllable is a glide.
fn vowel_sound(unit: &Unit, glide: bool) -> String {
    match (unit.base, glide) {
        ('i', true) => "j".into(),
        ('u', true) => "w".into(),
        (base, _) => base.to_string(),
    }
}

/// The phonemes of a lower-case Spanish word, with a stress mark in front of the stressed syllable.
pub fn word(word: &str) -> String {
    let lower = word.to_lowercase();
    // The x of México and Oaxaca is said like a j.
    let spelled = if ["méxi", "mexi", "oaxaca"]
        .iter()
        .any(|w| lower.starts_with(w))
    {
        lower.replace('x', "j")
    } else {
        lower
    };
    let units = units_of(&spelled);
    if units.is_empty() {
        return String::new();
    }
    let nuclei = nuclei(&units);
    let heads: Vec<usize> = nuclei.iter().map(|n| head(&units, n)).collect();
    // A word of one syllable carries no mark.
    let stress_at = (nuclei.len() > 1).then(|| heads[stressed(&units, &nuclei)]);
    let mut out = String::new();
    for (index, unit) in units.iter().enumerate() {
        if stress_at == Some(index) {
            out.push(STRESS);
        }
        if is_vowel(unit) {
            let glide = !heads.contains(&index);
            out.push_str(&vowel_sound(unit, glide));
        } else {
            let previous = index.checked_sub(1).map(|i| &units[i]);
            out.push_str(&consonant_sound(unit, previous, units.get(index + 1)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::word;

    #[test]
    fn stress_follows_the_accent_or_the_ending() {
        assert_eq!(word("hola"), "ˈola");
        assert_eq!(word("casa"), "kˈasa");
        assert_eq!(word("papel"), "papˈel");
        assert_eq!(word("canción"), "kanθjˈon");
        assert_eq!(word("México"), "mˈexiko");
    }

    #[test]
    fn letters_have_their_spanish_sounds() {
        assert_eq!(word("gracias"), "ɡɾˈaθjas");
        assert_eq!(word("perro"), "pˈero");
        assert_eq!(word("pero"), "pˈeɾo");
        assert_eq!(word("queso"), "kˈeso");
        assert_eq!(word("pingüino"), "piŋɡwˈino");
        assert_eq!(word("aire"), "ˈajɾe");
        assert_eq!(word("ciudad"), "θjuðˈað");
        assert_eq!(word("muchacho"), "muʧˈaʧo");
    }
}
