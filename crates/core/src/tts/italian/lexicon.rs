//! Words whose stress or sounds cannot be read from the spelling, respelled with an accent that
//! marks the stressed vowel (the accent only places the stress; è and ò also open the vowel).

/// Common words and verb forms stressed on the third syllable from the end, and a few hiatuses.
const RESPELLED: &[(&str, &str)] = &[
    ("parlano", "pàrlano"),
    ("mangiano", "màngiano"),
    ("arrivano", "arrìvano"),
    ("abitano", "àbitano"),
    ("lavorano", "lavòrano"),
    ("guardano", "guàrdano"),
    ("cercano", "cèrcano"),
    ("trovano", "tròvano"),
    ("prendono", "prèndono"),
    ("vendono", "vèndono"),
    ("scrivono", "scrìvono"),
    ("dicono", "dìcono"),
    ("telefono", "telèfono"),
    ("numero", "nùmero"),
    ("musica", "mùsica"),
    ("macchina", "màcchina"),
    ("camera", "càmera"),
    ("fabbrica", "fàbbrica"),
    ("tavolo", "tàvolo"),
    ("ultimo", "ùltimo"),
    ("primo", "prìmo"),
    ("medico", "mèdico"),
    ("pubblico", "pùbblico"),
    ("classico", "clàssico"),
    ("simpatico", "simpàtico"),
    ("paura", "paùra"),
    ("farmacia", "farmacìa"),
    ("pizzeria", "pizzerìa"),
    ("simpatia", "simpatìa"),
    ("energia", "energìa"),
    ("poesia", "poesìa"),
    ("zio", "zìo"),
    ("mio", "mìo"),
    ("più", "piu"),
    ("subito", "sùbito"),
    ("povero", "pòvero"),
    ("giovane", "giòvane"),
    ("cinema", "cìnema"),
];

/// The accented respelling of a lower-case word, or the word itself.
pub fn respell(word: &str) -> &str {
    RESPELLED
        .iter()
        .find(|(plain, _)| *plain == word)
        .map_or(word, |(_, respelt)| respelt)
}
