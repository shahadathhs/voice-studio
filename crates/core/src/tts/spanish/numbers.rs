//! Spanish numbers as words. Accents are kept (veintidós, millón) because they place the stress.

const UNITS: [&str; 30] = [
    "cero",
    "uno",
    "dos",
    "tres",
    "cuatro",
    "cinco",
    "seis",
    "siete",
    "ocho",
    "nueve",
    "diez",
    "once",
    "doce",
    "trece",
    "catorce",
    "quince",
    "dieciséis",
    "diecisiete",
    "dieciocho",
    "diecinueve",
    "veinte",
    "veintiuno",
    "veintidós",
    "veintitrés",
    "veinticuatro",
    "veinticinco",
    "veintiséis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];
const TENS: [&str; 10] = [
    "",
    "",
    "",
    "treinta",
    "cuarenta",
    "cincuenta",
    "sesenta",
    "setenta",
    "ochenta",
    "noventa",
];
const HUNDREDS: [&str; 10] = [
    "",
    "ciento",
    "doscientos",
    "trescientos",
    "cuatrocientos",
    "quinientos",
    "seiscientos",
    "setecientos",
    "ochocientos",
    "novecientos",
];

fn at(n: u64) -> usize {
    usize::try_from(n).unwrap_or(0)
}

fn below_hundred(n: u64) -> String {
    if n < 30 {
        return UNITS[at(n)].to_string();
    }
    match n % 10 {
        0 => TENS[at(n / 10)].to_string(),
        unit => format!("{} y {}", TENS[at(n / 10)], UNITS[at(unit)]),
    }
}

fn below_thousand(n: u64) -> String {
    match (n / 100, n % 100) {
        (1, 0) => "cien".into(),
        (0, rest) => below_hundred(rest),
        (hundreds, 0) => HUNDREDS[at(hundreds)].into(),
        (hundreds, rest) => format!("{} {}", HUNDREDS[at(hundreds)], below_hundred(rest)),
    }
}

/// "un" in front of a noun-like scale (un millón), "uno" alone.
fn before_scale(n: u64) -> String {
    match below_thousand(n).as_str() {
        "uno" => "un".into(),
        other => other
            .replace("veintiuno", "veintiún")
            .replace(" y uno", " y un"),
    }
}

/// The words for a whole number, e.g. 1250 → "mil doscientos cincuenta".
pub fn cardinal(n: u64) -> String {
    let scales = [
        (1_000_000_000_000, "billón", "billones"),
        (1_000_000, "millón", "millones"),
    ];
    let mut rest = n;
    let mut words: Vec<String> = Vec::new();
    for (size, one, many) in scales {
        if rest >= size {
            let count = rest / size;
            let prefix = if count == 1 {
                "un".to_string()
            } else {
                cardinal(count)
            };
            words.push(format!("{prefix} {}", if count == 1 { one } else { many }));
            rest %= size;
        }
    }
    if rest >= 1000 {
        let thousands = rest / 1000;
        words.push(if thousands == 1 {
            "mil".into()
        } else {
            format!("{} mil", before_scale(thousands))
        });
        rest %= 1000;
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
    fn numbers_become_spanish_words() {
        assert_eq!(cardinal(0), "cero");
        assert_eq!(cardinal(21), "veintiuno");
        assert_eq!(cardinal(31), "treinta y uno");
        assert_eq!(cardinal(100), "cien");
        assert_eq!(cardinal(101), "ciento uno");
        assert_eq!(cardinal(1000), "mil");
        assert_eq!(cardinal(1250), "mil doscientos cincuenta");
        assert_eq!(cardinal(21_000), "veintiún mil");
        assert_eq!(cardinal(2_024), "dos mil veinticuatro");
        assert_eq!(cardinal(1_000_000), "un millón");
        assert_eq!(cardinal(3_500_000), "tres millones quinientos mil");
    }
}
