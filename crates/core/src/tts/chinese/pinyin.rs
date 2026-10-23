//! One pinyin syllable (`zhōng`) as sounds Kokoro knows: the initial and final in IPA, then an
//! arrow for the tone.

/// A syllable split into its letters and tone (1 to 4, 5 for neutral).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllable {
    /// Plain letters, `v` for ü: `lv`, `zhong`.
    pub letters: String,
    pub tone: u8,
}

fn unmark(c: char) -> (char, u8) {
    match c {
        'ā' => ('a', 1),
        'á' => ('a', 2),
        'ǎ' => ('a', 3),
        'à' => ('a', 4),
        'ē' => ('e', 1),
        'é' => ('e', 2),
        'ě' => ('e', 3),
        'è' => ('e', 4),
        'ī' => ('i', 1),
        'í' => ('i', 2),
        'ǐ' => ('i', 3),
        'ì' => ('i', 4),
        'ō' => ('o', 1),
        'ó' => ('o', 2),
        'ǒ' => ('o', 3),
        'ò' => ('o', 4),
        'ū' => ('u', 1),
        'ú' => ('u', 2),
        'ǔ' => ('u', 3),
        'ù' => ('u', 4),
        'ǖ' => ('v', 1),
        'ǘ' => ('v', 2),
        'ǚ' => ('v', 3),
        'ǜ' => ('v', 4),
        'ü' => ('v', 5),
        'ń' => ('n', 2),
        'ň' => ('n', 3),
        'ǹ' => ('n', 4),
        'ḿ' => ('m', 2),
        'ê' => ('e', 5),
        other => (other, 5),
    }
}

impl Syllable {
    /// Reads `zhōng`, `lǜ` or `de` (no mark: neutral tone).
    pub fn parse(text: &str) -> Syllable {
        let mut tone = 5;
        let letters = text
            .chars()
            .map(|c| {
                let (plain, t) = unmark(c);
                if t != 5 {
                    tone = t;
                }
                plain
            })
            .filter(char::is_ascii_alphabetic)
            .collect::<String>();
        Syllable { letters, tone }
    }
}

const INITIALS: [(&str, &str); 21] = [
    ("zh", "ꭧ"),
    ("ch", "ꭧʰ"),
    ("sh", "ʂ"),
    ("b", "p"),
    ("p", "pʰ"),
    ("m", "m"),
    ("f", "f"),
    ("d", "t"),
    ("t", "tʰ"),
    ("n", "n"),
    ("l", "l"),
    ("g", "k"),
    ("k", "kʰ"),
    ("h", "x"),
    ("j", "ʨ"),
    ("q", "ʨʰ"),
    ("x", "ɕ"),
    ("r", "ɻ"),
    ("z", "ʦ"),
    ("c", "ʦʰ"),
    ("s", "s"),
];

const FINALS: [(&str, &str); 36] = [
    ("iang", "jaŋ"),
    ("iong", "jʊŋ"),
    ("uang", "waŋ"),
    ("ueng", "wəŋ"),
    ("iao", "jau"),
    ("ian", "jɛn"),
    ("uai", "wai"),
    ("uan", "wan"),
    ("van", "ɥɛn"),
    ("ang", "aŋ"),
    ("eng", "əŋ"),
    ("ing", "iŋ"),
    ("ong", "ʊŋ"),
    ("ia", "ja"),
    ("ie", "je"),
    ("iu", "jou"),
    ("ua", "wa"),
    ("uo", "wo"),
    ("ui", "wei"),
    ("ve", "ɥe"),
    ("ai", "ai"),
    ("ei", "ei"),
    ("ao", "au"),
    ("ou", "ou"),
    ("an", "an"),
    ("en", "ən"),
    ("in", "in"),
    ("un", "wən"),
    ("vn", "yn"),
    ("er", "ɚ"),
    ("a", "a"),
    ("o", "wo"),
    ("e", "ɤ"),
    ("i", "i"),
    ("u", "u"),
    ("v", "y"),
];

/// Syllables that start with `y` or `w` are written differently from how they are built:
/// `yan` is `ian`, `wei` is `uei` (written `ui`), `yu` is `ü`.
fn expand_glide(letters: &str) -> String {
    match letters {
        "you" => return "iu".into(),
        "wu" => return "u".into(),
        "wei" => return "ui".into(),
        "wen" => return "un".into(),
        _ => {}
    }
    if let Some(rest) = letters.strip_prefix("yu") {
        format!("v{rest}")
    } else if let Some(rest) = letters.strip_prefix("yi") {
        format!("i{rest}")
    } else if let Some(rest) = letters.strip_prefix('y') {
        format!("i{rest}")
    } else if let Some(rest) = letters.strip_prefix('w') {
        format!("u{rest}")
    } else {
        letters.to_string()
    }
}

fn split_initial(letters: &str) -> (&'static str, String) {
    let expanded = expand_glide(letters);
    for (spelling, sound) in INITIALS {
        if let Some(rest) = expanded.strip_prefix(spelling) {
            // After j, q, x the letter u is really ü.
            let rest = if matches!(spelling, "j" | "q" | "x") && rest.starts_with('u') {
                format!("v{}", &rest[1..])
            } else {
                rest.to_string()
            };
            return (sound, rest);
        }
    }
    ("", expanded)
}

fn tone_mark(tone: u8) -> &'static str {
    match tone {
        1 => "→",
        2 => "↗",
        3 => "↓",
        4 => "↘",
        _ => "",
    }
}

/// The sound of the final; `i` after z, c, s, zh, ch, sh, r is a buzzing vowel, not `i`.
fn final_sound(initial: &str, rest: &str, tone: u8) -> String {
    if rest == "i" && matches!(initial, "ʦ" | "ʦʰ" | "s") {
        return "ɨ".into();
    }
    if rest == "i" && initial == "ɻ" {
        return String::new();
    }
    if rest == "i" && matches!(initial, "ꭧ" | "ꭧʰ" | "ʂ") {
        return "ɻ".into();
    }
    let sound = FINALS
        .iter()
        .find(|(spelling, _)| *spelling == rest)
        .map_or(rest, |(_, sound)| *sound);
    if tone == 5 && sound == "ɤ" {
        "ə".into()
    } else {
        sound.to_string()
    }
}

impl Syllable {
    /// The phonemes of the syllable, with its tone as an arrow after it.
    pub fn phonemes(&self) -> String {
        let (initial, rest) = split_initial(&self.letters);
        format!(
            "{initial}{}{}",
            final_sound(initial, &rest, self.tone),
            tone_mark(self.tone)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn say(text: &str) -> String {
        Syllable::parse(text).phonemes()
    }

    #[test]
    fn tones_are_read_from_the_marks() {
        assert_eq!(
            Syllable::parse("zhōng"),
            Syllable {
                letters: "zhong".into(),
                tone: 1
            }
        );
        assert_eq!(Syllable::parse("lǜ").letters, "lv");
        assert_eq!(Syllable::parse("de").tone, 5);
    }

    #[test]
    fn syllables_become_initial_final_and_tone() {
        assert_eq!(say("nǐ"), "ni↓");
        assert_eq!(say("hǎo"), "xau↓");
        assert_eq!(say("zhōng"), "ꭧʊŋ→");
        assert_eq!(say("guó"), "kwo↗");
        assert_eq!(say("shì"), "ʂɻ↘");
        assert_eq!(say("sì"), "sɨ↘");
        assert_eq!(say("xué"), "ɕɥe↗");
        assert_eq!(say("de"), "tə");
    }

    #[test]
    fn y_and_w_syllables_are_spelled_out() {
        assert_eq!(say("yī"), "i→");
        assert_eq!(say("wǒ"), "wo↓");
        assert_eq!(say("yǒu"), "jou↓");
        assert_eq!(say("wèi"), "wei↘");
        assert_eq!(say("yuán"), "ɥɛn↗");
        assert_eq!(say("jūn"), "ʨyn→");
    }
}
