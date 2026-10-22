//! A rough guess at how a word the dictionary does not know is pronounced, from its spelling.
//! It is deliberately small: it makes unknown words sayable, not correct. Names of letters,
//! digraphs ("sh", "ee"), a silent final "e" that lengthens the vowel before it, and the soft
//! "c" and "g" are covered.

/// Phoneme symbols that are vowels, used to place the stress mark.
const VOWELS: &str = "AIOWYæɑɔəɛɪʊʌiuɜɚ";

/// Letter groups and their phonemes. Longer groups are tried first.
const GROUPS: [(&str, &str); 38] = [
    ("tion", "ʃᵻn"),
    ("sion", "ʒᵻn"),
    ("ture", "ʧɚ"),
    ("eigh", "A"),
    ("tch", "ʧ"),
    ("dge", "ʤ"),
    ("igh", "I"),
    ("ch", "ʧ"),
    ("sh", "ʃ"),
    ("th", "θ"),
    ("ph", "f"),
    ("gh", ""),
    ("ck", "k"),
    ("ng", "ŋ"),
    ("qu", "kw"),
    ("wh", "w"),
    ("wr", "ɹ"),
    ("kn", "n"),
    ("oo", "u"),
    ("ee", "i"),
    ("ea", "i"),
    ("ai", "A"),
    ("ay", "A"),
    ("oa", "O"),
    ("ou", "W"),
    ("ow", "W"),
    ("oi", "Y"),
    ("oy", "Y"),
    ("au", "ɔ"),
    ("aw", "ɔ"),
    ("ie", "i"),
    ("ei", "A"),
    ("ue", "u"),
    ("ew", "u"),
    ("ar", "ɑɹ"),
    ("or", "ɔɹ"),
    ("er", "ɚ"),
    ("ur", "ɚ"),
];

fn is_vowel_letter(c: char) -> bool {
    "aeiou".contains(c)
}

/// The short vowel sound of a letter.
fn short(c: char) -> &'static str {
    match c {
        'a' => "æ",
        'e' => "ɛ",
        'i' => "ɪ",
        'o' => "ɑ",
        _ => "ʌ",
    }
}

/// The long vowel sound of a letter, as in "make", "pete", "bike", "hope", "cute".
fn long(c: char) -> &'static str {
    match c {
        'a' => "A",
        'e' => "i",
        'i' => "I",
        'o' => "O",
        _ => "u",
    }
}

fn consonant(c: char, next: Option<char>) -> String {
    let soft = next.is_some_and(|n| "eiy".contains(n));
    match c {
        'c' if soft => "s".into(),
        'c' | 'k' | 'q' => "k".into(),
        'g' if soft => "ʤ".into(),
        'g' => "ɡ".into(),
        'j' => "ʤ".into(),
        'r' => "ɹ".into(),
        'x' => "ks".into(),
        other => other.to_string(),
    }
}

/// Whether the vowel at `i` is followed by one consonant and then a final "e" ("make").
fn is_magic_e(letters: &[char], i: usize) -> bool {
    letters.len() == i + 3 && !is_vowel_letter(letters[i + 1]) && letters[i + 2] == 'e'
}

/// Puts a primary stress mark before the first vowel sound.
fn stressed(phonemes: &str) -> String {
    match phonemes.char_indices().find(|(_, c)| VOWELS.contains(*c)) {
        Some((at, _)) => format!("{}ˈ{}", &phonemes[..at], &phonemes[at..]),
        None => phonemes.to_string(),
    }
}

/// Pronunciation of a lowercase word made of ASCII letters.
pub fn pronounce(word: &str) -> String {
    let letters: Vec<char> = word.chars().filter(char::is_ascii_alphabetic).collect();
    let mut out = String::new();
    let mut i = 0;
    while i < letters.len() {
        let rest: String = letters[i..].iter().collect();
        if let Some((group, phonemes)) = GROUPS.iter().find(|(g, _)| rest.starts_with(g)) {
            out.push_str(phonemes);
            i += group.len();
            continue;
        }
        let c = letters[i];
        let at_end = i + 1 == letters.len();
        if is_vowel_letter(c) {
            if c == 'e' && at_end && i > 0 {
                // A final "e" after a consonant is silent.
            } else if is_magic_e(&letters, i) {
                out.push_str(long(c));
            } else {
                out.push_str(short(c));
            }
        } else if c == 'y' {
            out.push_str(if i == 0 {
                "j"
            } else if at_end {
                "i"
            } else {
                "ɪ"
            });
        } else if i == 0 || letters[i - 1] != c {
            out.push_str(&consonant(c, letters.get(i + 1).copied()));
        }
        i += 1;
    }
    stressed(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_silent_e_lengthens_the_vowel_before_it() {
        assert_eq!(pronounce("make"), "mˈAk");
        assert_eq!(pronounce("bike"), "bˈIk");
        assert_eq!(pronounce("mat"), "mˈæt");
    }

    #[test]
    fn digraphs_and_soft_letters() {
        assert_eq!(pronounce("ship"), "ʃˈɪp");
        assert_eq!(pronounce("cell"), "sˈɛl");
        assert_eq!(pronounce("cat"), "kˈæt");
        assert_eq!(pronounce("gem"), "ʤˈɛm");
        assert_eq!(pronounce("tree"), "tɹˈi");
    }

    #[test]
    fn doubled_consonants_are_said_once_and_y_depends_on_its_place() {
        assert_eq!(pronounce("butter"), "bˈʌtɚ");
        assert_eq!(pronounce("yes"), "jˈɛs");
        assert_eq!(pronounce("happy"), "hˈæpi");
    }

    #[test]
    fn a_word_without_letters_has_no_sound() {
        assert_eq!(pronounce(""), "");
        assert_eq!(pronounce("123"), "");
    }
}
