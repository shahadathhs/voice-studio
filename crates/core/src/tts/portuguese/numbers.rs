//! Brazilian Portuguese numbers as words. Accents are kept (três, milhão) because they place the stress.

const UNITS: [&str; 20] = [
    "zero",
    "um",
    "dois",
    "três",
    "quatro",
    "cinco",
    "seis",
    "sete",
    "oito",
    "nove",
    "dez",
    "onze",
    "doze",
    "treze",
    "catorze",
    "quinze",
    "dezesseis",
    "dezessete",
    "dezoito",
    "dezenove",
];
const TENS: [&str; 10] = [
    "",
    "",
    "vinte",
    "trinta",
    "quarenta",
    "cinquenta",
    "sessenta",
    "setenta",
    "oitenta",
    "noventa",
];
const HUNDREDS: [&str; 10] = [
    "",
    "cento",
    "duzentos",
    "trezentos",
    "quatrocentos",
    "quinhentos",
    "seiscentos",
    "setecentos",
    "oitocentos",
    "novecentos",
];

fn at(n: u64) -> usize {
    usize::try_from(n).unwrap_or(0)
}

fn below_hundred(n: u64) -> String {
    if n < 20 {
        return UNITS[at(n)].to_string();
    }
    match n % 10 {
        0 => TENS[at(n / 10)].to_string(),
        unit => format!("{} e {}", TENS[at(n / 10)], UNITS[at(unit)]),
    }
}

fn below_thousand(n: u64) -> String {
    match (n / 100, n % 100) {
        (1, 0) => "cem".into(),
        (0, rest) => below_hundred(rest),
        (hundreds, 0) => HUNDREDS[at(hundreds)].into(),
        (hundreds, rest) => format!("{} e {}", HUNDREDS[at(hundreds)], below_hundred(rest)),
    }
}

/// The remainder after a scale word takes "e" when it is under a hundred or whole hundreds.
fn join(words: &mut Vec<String>, rest: u64) {
    if rest > 0 && (rest < 100 || rest.is_multiple_of(100)) {
        words.push("e".into());
    }
}

/// The words for a whole number, e.g. 2020 → "dois mil e vinte".
pub fn cardinal(n: u64) -> String {
    let scales = [
        (1_000_000_000_000, "trilhão", "trilhões"),
        (1_000_000_000, "bilhão", "bilhões"),
        (1_000_000, "milhão", "milhões"),
    ];
    let mut rest = n;
    let mut words: Vec<String> = Vec::new();
    for (size, one, many) in scales {
        if rest >= size {
            let count = rest / size;
            words.push(cardinal(count));
            words.push((if count == 1 { one } else { many }).into());
            rest %= size;
            join(&mut words, rest);
        }
    }
    if rest >= 1000 {
        let thousands = rest / 1000;
        if thousands > 1 {
            words.push(below_thousand(thousands));
        }
        words.push("mil".into());
        rest %= 1000;
        join(&mut words, rest);
    }
    if rest > 0 || words.is_empty() {
        words.push(below_thousand(rest));
    }
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_become_portuguese_words() {
        assert_eq!(cardinal(0), "zero");
        assert_eq!(cardinal(16), "dezesseis");
        assert_eq!(cardinal(21), "vinte e um");
        assert_eq!(cardinal(100), "cem");
        assert_eq!(cardinal(121), "cento e vinte e um");
        assert_eq!(cardinal(1000), "mil");
        assert_eq!(cardinal(1250), "mil duzentos e cinquenta");
        assert_eq!(cardinal(1500), "mil e quinhentos");
        assert_eq!(cardinal(2020), "dois mil e vinte");
        assert_eq!(cardinal(1_000_000), "um milhão");
        assert_eq!(cardinal(3_500_000), "três milhões e quinhentos mil");
        assert_eq!(cardinal(2_000_001), "dois milhões e um");
    }
}
