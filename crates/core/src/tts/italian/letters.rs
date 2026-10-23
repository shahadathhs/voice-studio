//! Italian spelling cut into vowels and consonant sounds. Letters that only mark a sound (the i
//! of "ciao", the h of "che") produce nothing.

/// How a long consonant (a double letter) is written: the sound twice, which Whisper heard
/// better than a length mark (ː).
fn lengthen(sound: &str) -> String {
    format!("{sound}{sound}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vowel {
    pub base: char,
    /// Written with an accent, which marks the stressed syllable.
    pub stressed: bool,
    /// è and ò.
    pub open: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unit {
    Vowel(Vowel),
    /// The phonemes of a consonant sound.
    Consonant(String),
}

fn vowel_of(c: char) -> Option<Vowel> {
    let (base, stressed, open) = match c {
        'a' | 'e' | 'i' | 'o' | 'u' => (c, false, false),
        'y' | 'j' => ('i', false, false),
        'à' => ('a', true, false),
        'è' => ('e', true, true),
        'é' => ('e', true, false),
        'ì' => ('i', true, false),
        'ò' => ('o', true, true),
        'ó' => ('o', true, false),
        'ù' => ('u', true, false),
        _ => return None,
    };
    Some(Vowel {
        base,
        stressed,
        open,
    })
}

fn is_vowel(c: Option<&char>) -> bool {
    c.is_some_and(|c| vowel_of(*c).is_some())
}

/// e or i, which make c and g soft.
fn is_front(c: Option<&char>) -> bool {
    matches!(c, Some('e' | 'i' | 'è' | 'é' | 'ì'))
}

/// 1 if `rest[at]` is an i that only softens the letter before it (ciao, giorno, scienza).
fn silent_i(rest: &[char], at: usize) -> usize {
    let next = rest.get(at + 1);
    usize::from(rest.get(at) == Some(&'i') && is_vowel(next) && next != Some(&'i'))
}

/// A consonant sound read from the letters.
struct Sound {
    text: &'static str,
    /// Letters after the first that it uses up.
    used: usize,
    /// Always long between vowels (gn, gli, sc), not only when written double.
    long: bool,
}

fn sound(text: &'static str, used: usize) -> Sound {
    Sound {
        text,
        used,
        long: false,
    }
}

fn long(text: &'static str, used: usize) -> Sound {
    Sound {
        text,
        used,
        long: true,
    }
}

/// Where a letter stands: the letters after it and what comes before.
pub struct Place<'a> {
    pub rest: &'a [char],
    pub after_vowel: bool,
    pub at_start: bool,
    pub double: bool,
}

fn soft_c(rest: &[char]) -> Sound {
    match rest.first() {
        Some('h') => sound("k", 1),
        Some('q') => long("k", 1),
        _ if is_front(rest.first()) => sound("ʧ", silent_i(rest, 0)),
        _ => sound("k", 0),
    }
}

fn soft_g(rest: &[char]) -> Sound {
    match rest.first() {
        Some('h') => sound("ɡ", 1),
        Some('n') => long("ɲ", 1),
        Some('l') if rest.get(1) == Some(&'i') => {
            sound("ʎ", if is_vowel(rest.get(2)) { 2 } else { 1 })
        }
        _ if is_front(rest.first()) => sound("ʤ", silent_i(rest, 0)),
        _ => sound("ɡ", 0),
    }
}

/// s is said z between vowels and before b d g l m n r v.
fn letter_s(place: &Place) -> Sound {
    let rest = place.rest;
    if rest.first() == Some(&'c') && is_front(rest.get(1)) {
        return long("ʃ", 1 + silent_i(rest, 1));
    }
    let voiced = !place.double
        && (place.after_vowel && is_vowel(rest.first())
            || matches!(
                rest.first(),
                Some('b' | 'd' | 'g' | 'l' | 'm' | 'n' | 'r' | 'v')
            ));
    sound(if voiced { "z" } else { "s" }, 0)
}

fn letter_n(rest: &[char]) -> Sound {
    let hard = |c: char| rest.first() == Some(&c) && !is_front(rest.get(1));
    if hard('c') || hard('g') || rest.first() == Some(&'q') {
        sound("ŋ", 0)
    } else {
        sound("n", 0)
    }
}

/// The sound of a consonant letter, or `None` for a letter that is not Italian.
fn consonant(c: char, place: &Place) -> Option<Sound> {
    Some(match c {
        'c' => soft_c(place.rest),
        'g' => soft_g(place.rest),
        's' => letter_s(place),
        'n' => letter_n(place.rest),
        'z' if place.at_start && matches!(place.rest.first(), Some('e' | 'o')) => sound("ʣ", 0),
        'z' => sound("ʦ", 0),
        'r' => sound("r", 0),
        'q' | 'k' => sound("k", 0),
        'x' => sound("ks", 0),
        'h' => sound("", 0),
        'b' => sound("b", 0),
        'd' => sound("d", 0),
        'f' => sound("f", 0),
        'l' => sound("l", 0),
        'm' => sound("m", 0),
        'p' => sound("p", 0),
        't' => sound("t", 0),
        'v' | 'w' => sound("v", 0),
        _ => return None,
    })
}

/// The units of a lower-case Italian word.
pub fn units(word: &str) -> Vec<Unit> {
    let chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    let mut out: Vec<Unit> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(vowel) = vowel_of(c) {
            out.push(Unit::Vowel(vowel));
            i += 1;
            continue;
        }
        let double = c != 'h' && chars.get(i + 1) == Some(&c);
        let at = i + usize::from(double);
        let place = Place {
            rest: &chars[at + 1..],
            after_vowel: matches!(out.last(), Some(Unit::Vowel(_))),
            at_start: out.is_empty(),
            double,
        };
        if let Some(found) = consonant(c, &place) {
            let between = place.after_vowel && is_vowel(chars.get(at + 1 + found.used));
            let text = if double || (found.long && between) {
                lengthen(found.text)
            } else {
                found.text.to_string()
            };
            if !text.is_empty() {
                out.push(Unit::Consonant(text));
            }
            i = at + 1 + found.used;
        } else {
            i = at + 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sounds(word: &str) -> Vec<String> {
        units(word)
            .into_iter()
            .map(|u| match u {
                Unit::Vowel(v) => v.base.to_string(),
                Unit::Consonant(c) => c,
            })
            .collect()
    }

    #[test]
    fn c_and_g_soften_before_e_and_i() {
        assert_eq!(sounds("cena"), ["ʧ", "e", "n", "a"]);
        assert_eq!(sounds("chiesa"), ["k", "i", "e", "z", "a"]);
        assert_eq!(sounds("ciao"), ["ʧ", "a", "o"]);
        assert_eq!(sounds("gelato"), ["ʤ", "e", "l", "a", "t", "o"]);
        assert_eq!(sounds("ghiaccio"), ["ɡ", "i", "a", "ʧʧ", "o"]);
    }

    #[test]
    fn digraphs_have_one_sound() {
        assert_eq!(sounds("gnocchi"), ["ɲ", "o", "kk", "i"]);
        assert_eq!(sounds("figlio"), ["f", "i", "ʎ", "o"]);
        assert_eq!(sounds("pesce"), ["p", "e", "ʃʃ", "e"]);
        assert_eq!(sounds("schiena"), ["s", "k", "i", "e", "n", "a"]);
    }

    #[test]
    fn s_is_voiced_between_vowels_and_z_is_an_affricate() {
        assert_eq!(sounds("casa"), ["k", "a", "z", "a"]);
        assert_eq!(sounds("sole"), ["s", "o", "l", "e"]);
        assert_eq!(sounds("pizza"), ["p", "i", "ʦʦ", "a"]);
        assert_eq!(sounds("zero"), ["ʣ", "e", "r", "o"]);
    }
}
