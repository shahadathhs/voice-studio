//! Portuguese spelling to phonemes: the units of a word, its stress, then each sound.

use super::consonants::{Spelling, x_sound};
use super::syllables::{head, nuclei, stressed};
use super::units::{Mark, Unit, is_vowel, unaccented, units_of};
use super::vowels::{Stress, sound};

const STRESS: char = 'ˈ';
/// Small words said without stress, so their vowels are reduced (o → u, e → i, a → ɐ).
const REDUCED: [&str; 19] = [
    "a", "as", "o", "os", "e", "de", "do", "da", "dos", "das", "no", "na", "nos", "nas", "que",
    "se", "me", "te", "lhe",
];
/// Words whose spelling does not give their sound.
const SPECIAL: [(&str, &str); 6] = [
    ("é", "ɛ"),
    ("ao", "aw"),
    ("aos", "aws"),
    ("por", "poɾ"),
    ("nove", "nˈɔvi"),
    ("sete", "sˈɛʧi"),
];

/// A stressed vowel in front of `m` or `n` and another vowel is nasal too (cama, ano, uma).
fn nasalize_stressed(units: &mut [Unit], at: usize) {
    let nasal_consonant = units
        .get(at + 1)
        .is_some_and(|n| matches!(n.text.as_str(), "m" | "n"));
    let vowel_after = units.get(at + 2).is_some_and(is_vowel);
    let open = units[at].mark == Mark::Acute;
    if nasal_consonant && vowel_after && !open {
        units[at].nasal = true;
    }
}

/// The phonemes of a Portuguese word, with a stress mark in front of the stressed syllable.
pub fn word(word: &str) -> String {
    let lower = word.to_lowercase().replace("muit", "mũit");
    if let Some((_, sounds)) = SPECIAL.iter().find(|(w, _)| *w == lower) {
        return (*sounds).to_string();
    }
    let mut units = units_of(&lower);
    let nuclei = nuclei(&units);
    if nuclei.is_empty() {
        return String::new();
    }
    let heads: Vec<usize> = nuclei.iter().map(|n| head(&units, n)).collect();
    let at = heads[stressed(&units, &nuclei)];
    let multi = nuclei.len() > 1;
    nasalize_stressed(&mut units, at);
    let stress = Stress {
        heads,
        at,
        reduced: !multi && REDUCED.contains(&lower.as_str()),
    };
    let vowels: Vec<String> = (0..units.len())
        .map(|i| {
            if is_vowel(&units[i]) {
                sound(&units, i, &stress)
            } else {
                String::new()
            }
        })
        .collect();
    let spelling = Spelling {
        units: &units,
        vowels: &vowels,
        x: x_sound(&unaccented(&lower)),
    };
    let mut out = String::new();
    for (i, unit) in units.iter().enumerate() {
        if multi && i == at {
            out.push(STRESS);
        }
        out.push_str(&if is_vowel(unit) {
            vowels[i].clone()
        } else {
            spelling.sound(i)
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::word as spell;

    /// Spells nasal vowels as a vowel plus a combining tilde, as the engine does.
    fn word(text: &str) -> String {
        spell(text)
    }

    fn d(text: &str) -> String {
        text.replace('ũ', "u\u{303}")
            .replace('ẽ', "e\u{303}")
            .replace('ĩ', "i\u{303}")
    }

    #[test]
    fn stress_follows_the_accent_or_the_ending() {
        assert_eq!(word("casa"), d("kˈazɐ"));
        assert_eq!(word("falar"), d("falˈaɾ"));
        assert_eq!(word("você"), d("vosˈe"));
        assert_eq!(word("médico"), d("mˈɛʤiku"));
        assert_eq!(word("animais"), d("animˈajs"));
        assert_eq!(word("história"), d("istˈɔɾjɐ"));
    }

    #[test]
    fn nasal_vowels_and_diphthongs() {
        assert_eq!(word("não"), d("nɐ̃w̃"));
        assert_eq!(word("canção"), d("kɐ̃sˈɐ̃w̃"));
        assert_eq!(word("mãe"), d("mɐ̃j̃"));
        assert_eq!(word("bem"), d("bẽj̃"));
        assert_eq!(word("campo"), d("kˈɐ̃pu"));
        assert_eq!(word("muito"), d("mˈũj̃tu"));
    }

    #[test]
    fn letters_have_their_brazilian_sounds() {
        assert_eq!(word("filho"), d("fˈilju"));
        assert_eq!(word("dinheiro"), d("ʤiɲˈejɾu"));
        assert_eq!(word("chuva"), d("ʃˈuvɐ"));
        assert_eq!(word("carro"), d("kˈaxu"));
        assert_eq!(word("rua"), d("xˈuɐ"));
        assert_eq!(word("caro"), d("kˈaɾu"));
        assert_eq!(word("tarde"), d("tˈaxʤi"));
        assert_eq!(word("noite"), d("nˈojʧi"));
        assert_eq!(word("cidade"), d("sidˈaʤi"));
        assert_eq!(word("mal"), d("maw"));
        assert_eq!(word("queijo"), d("kˈejʒu"));
        assert_eq!(word("guerra"), d("ɡˈɛxɐ"));
        assert_eq!(word("quando"), d("kwˈɐ̃du"));
    }

    #[test]
    fn s_and_x_follow_their_neighbours() {
        assert_eq!(word("casa"), d("kˈazɐ"));
        assert_eq!(word("sol"), d("sɔw"));
        assert_eq!(word("passo"), d("pˈasu"));
        assert_eq!(word("peixe"), d("pˈejʃi"));
        assert_eq!(word("exame"), d("ezˈɐ̃mi"));
        assert_eq!(word("táxi"), d("tˈaksi"));
        assert_eq!(word("nascer"), d("nasˈɛɾ"));
    }
}
