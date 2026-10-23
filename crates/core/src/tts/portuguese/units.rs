//! Cutting a Portuguese word into vowels and consonant units (`lh`, `nh`, `ch`, `rr`, `ss`, `qu`…).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Vowel,
    Consonant,
}

/// The accent a vowel is written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Plain,
    Acute,
    Circumflex,
    Tilde,
}

/// A vowel, or a consonant sound as written.
#[derive(Debug, Clone)]
pub struct Unit {
    pub kind: UnitKind,
    /// The letters of a consonant unit, lower case.
    pub text: String,
    /// The vowel without its accent (`' '` for consonants).
    pub base: char,
    pub mark: Mark,
    /// The vowel is nasal: written `ã`, or followed by an `m`/`n` that is not said.
    pub nasal: bool,
    /// The `m` or `n` that was folded into this vowel.
    pub consumed: Option<char>,
}

pub fn is_vowel(unit: &Unit) -> bool {
    unit.kind == UnitKind::Vowel
}

fn vowel_of(c: char) -> Option<(char, Mark, bool)> {
    Some(match c {
        'a' | 'e' | 'i' | 'o' | 'u' | 'ü' => (if c == 'ü' { 'u' } else { c }, Mark::Plain, false),
        'y' => ('i', Mark::Plain, false),
        'á' | 'à' => ('a', Mark::Acute, false),
        'é' => ('e', Mark::Acute, false),
        'í' => ('i', Mark::Acute, false),
        'ó' => ('o', Mark::Acute, false),
        'ú' => ('u', Mark::Acute, false),
        'â' => ('a', Mark::Circumflex, false),
        'ê' => ('e', Mark::Circumflex, false),
        'ô' => ('o', Mark::Circumflex, false),
        'ã' => ('a', Mark::Tilde, true),
        'õ' => ('o', Mark::Tilde, true),
        'ũ' => ('u', Mark::Tilde, true),
        _ => return None,
    })
}

fn consonant(text: &str) -> Unit {
    Unit {
        kind: UnitKind::Consonant,
        text: text.to_string(),
        base: ' ',
        mark: Mark::Plain,
        nasal: false,
        consumed: None,
    }
}

fn vowel(c: char) -> Option<Unit> {
    let (base, mark, nasal) = vowel_of(c)?;
    Some(Unit {
        kind: UnitKind::Vowel,
        text: base.to_string(),
        base,
        mark,
        nasal,
        consumed: None,
    })
}

fn is_front(c: char) -> bool {
    "eiéêíy".contains(c)
}

/// `qu` and `gu`: the u is silent before e or i, and said (`qw`, `gw`) before a or o.
fn gu(c: char, after: Option<char>) -> Option<&'static str> {
    let q = c == 'q';
    match after {
        Some(a) if is_front(a) => Some(if q { "qu" } else { "gu" }),
        Some(a) if vowel_of(a).is_some() => Some(if q { "qw" } else { "gw" }),
        _ => None,
    }
}

fn digraph(c: char, next: char, after: Option<char>) -> Option<&'static str> {
    match (c, next) {
        ('c', 'h') => Some("ch"),
        ('l', 'h') => Some("lh"),
        ('n', 'h') => Some("nh"),
        ('r', 'r') => Some("rr"),
        ('s', 's') => Some("ss"),
        ('q' | 'g', 'u') => gu(c, after),
        ('g', 'ü') => Some("gw"),
        ('q', 'ü') => Some("qw"),
        _ => None,
    }
}

fn step(chars: &[char], i: usize, units: &mut Vec<Unit>) -> usize {
    let c = chars[i];
    let next = chars.get(i + 1).copied();
    if let Some(n) = next
        && let Some(text) = digraph(c, n, chars.get(i + 2).copied())
    {
        units.push(consonant(text));
        return 2;
    }
    if let Some(unit) = vowel(c) {
        units.push(unit);
        return 1;
    }
    // An m or n after a vowel and before a consonant (or the end) only nasalises the vowel.
    let silent = next.is_none_or(|n| vowel_of(n).is_none());
    if let (true, 'm' | 'n', Some(last)) = (silent, c, units.last_mut())
        && is_vowel(last)
    {
        last.nasal = true;
        last.consumed = Some(c);
        return 1;
    }
    units.push(consonant(&c.to_string()));
    1
}

pub fn units_of(word: &str) -> Vec<Unit> {
    let chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    let mut units = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        i += step(&chars, i, &mut units);
    }
    units
}

/// The word with every vowel written without its accent.
pub fn unaccented(word: &str) -> String {
    word.chars()
        .map(|c| vowel_of(c).map_or(c, |(base, _, _)| base))
        .collect()
}
