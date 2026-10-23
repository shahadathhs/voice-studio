//! Getting a word ready to be read: which of its final letters are silent.

use super::lexicon;

/// A word with its silent ending taken off. A silent final `e` stays in `letters` (it still
/// tells `c`, `g` and `s` how to sound) but lies outside `len`.
pub struct Word {
    pub letters: Vec<char>,
    /// How many letters are read; a silent `e` after them only gives hints.
    pub len: usize,
    /// A silent consonant was cut off, so the last vowel is in a closed syllable (français).
    pub closed: bool,
}

pub fn is_vowel(c: char) -> bool {
    "aeiouyàâäéèêëîïôöùûüœæ".contains(c)
}

impl Word {
    /// The letter at `i` if it is read.
    pub fn at(&self, i: usize) -> Option<char> {
        (i < self.len).then(|| self.letters[i])
    }

    /// The letter at `i`, including a silent final `e`.
    pub fn raw(&self, i: usize) -> Option<char> {
        self.letters.get(i).copied()
    }

    pub fn vowel_at(&self, i: usize) -> bool {
        self.at(i).is_some_and(is_vowel)
    }

    pub fn consonant_at(&self, i: usize) -> bool {
        self.at(i).is_some_and(|c| !is_vowel(c))
    }

    /// Whether the letters from `i` on start with `text`.
    pub fn starts(&self, i: usize, text: &str) -> bool {
        text.chars()
            .enumerate()
            .all(|(k, c)| self.at(i + k) == Some(c))
    }

    pub fn before(&self, i: usize) -> Option<char> {
        i.checked_sub(1).and_then(|p| self.at(p))
    }
}

/// How many vowel sounds the letters make; the `u` of `qu` and `gu` is not one.
pub fn syllables(letters: &[char]) -> usize {
    let mut count = 0;
    let mut in_vowel = false;
    for (i, &c) in letters.iter().enumerate() {
        let silent_u = c == 'u' && i > 0 && matches!(letters[i - 1], 'q' | 'g');
        let vowel = is_vowel(c) && !silent_u;
        if vowel && !in_vowel {
            count += 1;
        }
        in_vowel = vowel;
    }
    count
}

fn ends(letters: &[char], tail: &str) -> bool {
    let tail: Vec<char> = tail.chars().collect();
    letters.ends_with(&tail)
}

fn cut(letters: &mut Vec<char>, count: usize) {
    letters.truncate(letters.len().saturating_sub(count));
}

/// Verbs in the third person plural (ils parlent) end in a silent -ent.
fn verb_ending(letters: &mut Vec<char>, text: &str) {
    let stem = &letters[..letters.len().saturating_sub(3)];
    let nasal = (ends(letters, "ment") && !ends(letters, "aiment"))
        || ends(letters, "tient")
        || ends(letters, "vient");
    if ends(letters, "ent") && syllables(stem) >= 1 && !nasal && !lexicon::noun_ent(text) {
        cut(letters, 2);
    }
}

/// -er, -ez and -et after a first syllable sound like é or è.
fn open_endings(letters: &mut Vec<char>, text: &str) {
    let first = syllables(&letters[..letters.len().saturating_sub(2)]) >= 1;
    let er = ends(letters, "er") && first && !lexicon::er_said(text);
    if er || ends(letters, "ez") {
        cut(letters, 2);
        letters.push('é');
    } else if ends(letters, "et") && first {
        cut(letters, 2);
        letters.push('è');
    }
}

fn silent_last(letters: &[char]) -> bool {
    let [.., before, last] = letters else {
        return false;
    };
    match last {
        't' | 'd' | 'p' | 'z' | 's' | 'x' => true,
        'g' | 'c' => *before == 'n',
        _ => false,
    }
}

/// The letters of a word without the ones that are not said.
pub fn prepare(text: &str) -> Word {
    let mut letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    let mut closed = false;
    verb_ending(&mut letters, text);
    if matches!(letters.last(), Some('s' | 'x')) && letters.len() > 2 {
        letters.pop();
        closed = true;
    }
    open_endings(&mut letters, text);
    while letters.len() > 2 && silent_last(&letters) {
        letters.pop();
        closed = true;
    }
    let len = if letters.last() == Some(&'e') && syllables(&letters[..letters.len() - 1]) >= 1 {
        letters.len() - 1
    } else {
        letters.len()
    };
    Word {
        letters,
        len,
        closed,
    }
}
