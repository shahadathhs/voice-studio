//! Italian numbers as words. Accents are kept (tré, ventitré) because they place the stress.

const UNITS: [&str; 20] = [
    "zero",
    "uno",
    "due",
    "tre",
    "quattro",
    "cinque",
    "sei",
    "sette",
    "otto",
    "nove",
    "dieci",
    "undici",
    "dodici",
    "tredici",
    "quattordici",
    "quindici",
    "sedici",
    "diciassette",
    "diciotto",
    "diciannove",
];
const TENS: [&str; 10] = [
    "",
    "",
    "venti",
    "trenta",
    "quaranta",
    "cinquanta",
    "sessanta",
    "settanta",
    "ottanta",
    "novanta",
];

fn at(n: u64) -> usize {
    usize::try_from(n).unwrap_or(0)
}

/// 21 is "ventuno" and 28 "ventotto": the tens lose their last vowel in front of uno and otto.
fn below_hundred(n: u64) -> String {
    if n < 20 {
        return UNITS[at(n)].to_string();
    }
    let tens = TENS[at(n / 10)];
    match n % 10 {
        0 => tens.to_string(),
        3 => format!("{tens}tré"),
        unit @ (1 | 8) => format!("{}{}", &tens[..tens.len() - 1], UNITS[at(unit)]),
        unit => format!("{tens}{}", UNITS[at(unit)]),
    }
}

fn below_thousand(n: u64) -> String {
    let hundreds = match n / 100 {
        0 => return below_hundred(n),
        1 => "cento".to_string(),
        h => format!("{}cento", UNITS[at(h)]),
    };
    match n % 100 {
        0 => hundreds,
        rest => format!("{hundreds} {}", below_hundred(rest)),
    }
}

/// "un milione" for one, "tre milioni" for more.
fn scaled(count: u64, one: &str, many: &str) -> String {
    if count == 1 {
        format!("un {one}")
    } else {
        format!("{} {many}", cardinal(count))
    }
}

/// The words for a whole number, e.g. 1250 → "mille duecento cinquanta".
pub fn cardinal(n: u64) -> String {
    let scales = [
        (1_000_000_000_000, "bilione", "bilioni"),
        (1_000_000_000, "miliardo", "miliardi"),
        (1_000_000, "milione", "milioni"),
    ];
    let mut rest = n;
    let mut words: Vec<String> = Vec::new();
    for (size, one, many) in scales {
        if rest >= size {
            words.push(scaled(rest / size, one, many));
            rest %= size;
        }
    }
    if rest >= 1000 {
        let thousands = rest / 1000;
        words.push(if thousands == 1 {
            "mille".into()
        } else {
            format!("{} mila", below_thousand(thousands))
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
    fn numbers_become_italian_words() {
        assert_eq!(cardinal(0), "zero");
        assert_eq!(cardinal(16), "sedici");
        assert_eq!(cardinal(21), "ventuno");
        assert_eq!(cardinal(23), "ventitré");
        assert_eq!(cardinal(28), "ventotto");
        assert_eq!(cardinal(45), "quarantacinque");
        assert_eq!(cardinal(100), "cento");
        assert_eq!(cardinal(101), "cento uno");
        assert_eq!(cardinal(1000), "mille");
        assert_eq!(cardinal(1250), "mille duecento cinquanta");
        assert_eq!(cardinal(2_024), "due mila ventiquattro");
        assert_eq!(cardinal(21_000), "ventuno mila");
        assert_eq!(cardinal(1_000_000), "un milione");
        assert_eq!(cardinal(3_500_000), "tre milioni cinquecento mila");
    }
}
