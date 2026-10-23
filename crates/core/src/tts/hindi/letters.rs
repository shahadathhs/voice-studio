//! The Devanagari alphabet as sounds. Kokoro has no dental or breathy-voiced symbols, so dental
//! stops are written plainly and breathy voice as aspiration (`ɡʰ`).

/// Stands for the virama (U+094D) while the text goes through the shared tokenizer, which only
/// keeps letters inside a word.
pub const VIRAMA: char = 'ˣ';
/// Stands for the nukta (U+093C) for the same reason.
pub const NUKTA: char = 'ˑ';

/// A consonant letter and its sound.
pub fn consonant(c: char) -> Option<&'static str> {
    Some(match c {
        'क' => "k",
        'ख' => "kʰ",
        'ग' => "ɡ",
        'घ' => "ɡʰ",
        'ङ' => "ŋ",
        'च' => "ʧ",
        'छ' => "ʧʰ",
        'ज' => "ʤ",
        'झ' => "ʤʰ",
        'ञ' => "ɲ",
        'ट' => "ʈ",
        'ठ' => "ʈʰ",
        'ड' => "ɖ",
        'ढ' => "ɖʰ",
        'ण' => "ɳ",
        'त' => "t",
        'थ' => "tʰ",
        'द' => "d",
        'ध' => "dʰ",
        'न' => "n",
        'प' => "p",
        'फ' => "pʰ",
        'ब' => "b",
        'भ' => "bʰ",
        'म' => "m",
        'य' => "j",
        'र' => "ɾ",
        'ल' => "l",
        'व' => "ʋ",
        'श' | 'ष' => "ʃ",
        'स' => "s",
        'ह' => "h",
        _ => return None,
    })
}

/// The letter a consonant becomes with a nukta dot under it.
pub fn with_nukta(c: char) -> Option<&'static str> {
    Some(match c {
        'क' => "q",
        'ख' => "x",
        'ग' => "ɣ",
        'ज' => "z",
        'ड' => "ɽ",
        'ढ' => "ɽʰ",
        'फ' => "f",
        _ => return None,
    })
}

/// A vowel sign written after a consonant.
pub fn matra(c: char) -> Option<&'static str> {
    Some(match c {
        'ा' => "aː",
        'ि' => "ɪ",
        'ी' => "iː",
        'ु' => "ʊ",
        'ू' => "uː",
        'ृ' => "ɾɪ",
        'े' => "eː",
        'ै' => "ɛː",
        'ो' => "oː",
        'ौ' => "ɔː",
        'ॉ' => "ɔ",
        'ॅ' => "æ",
        _ => return None,
    })
}

/// A vowel letter that stands alone.
pub fn independent(c: char) -> Option<&'static str> {
    Some(match c {
        'अ' => "ə",
        'आ' => "aː",
        'इ' => "ɪ",
        'ई' => "iː",
        'उ' => "ʊ",
        'ऊ' => "uː",
        'ऋ' => "ɾɪ",
        'ए' => "eː",
        'ऐ' => "ɛː",
        'ओ' => "oː",
        'औ' => "ɔː",
        'ऑ' => "ɔ",
        'ऍ' => "æ",
        _ => return None,
    })
}

/// How a Latin letter is named in Hindi, for abbreviations and English words in Hindi text.
pub fn latin_name(c: char) -> Option<&'static str> {
    const NAMES: [&str; 26] = [
        "eː",
        "biː",
        "siː",
        "diː",
        "iː",
        "ɛf",
        "ʤiː",
        "eːʧ",
        "aɪ",
        "ʤeː",
        "keː",
        "ɛl",
        "ɛm",
        "ɛn",
        "oː",
        "piː",
        "kjuː",
        "aːɾ",
        "ɛs",
        "tiː",
        "juː",
        "viː",
        "dʌbəlju",
        "ɛks",
        "vaɪ",
        "zɛd",
    ];
    let c = c.to_ascii_lowercase();
    c.is_ascii_lowercase()
        .then(|| NAMES[usize::from(c as u8 - b'a')])
}
