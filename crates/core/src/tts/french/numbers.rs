//! French numbers as words. Plural spellings (quatre-vingts, deux cents) are not heard, so they
//! are only kept where they help a reader of the tests.

const UNITS: [&str; 20] = [
    "zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze",
    "douze", "treize", "quatorze", "quinze", "seize", "dix sept", "dix huit", "dix neuf",
];
const TENS: [&str; 7] = [
    "",
    "",
    "vingt",
    "trente",
    "quarante",
    "cinquante",
    "soixante",
];

fn at(n: u64) -> usize {
    usize::try_from(n).unwrap_or(0)
}

/// 0 to 99, with the 70s and 90s counted on from sixty and eighty.
fn below_hundred(n: u64) -> String {
    if n < 20 {
        return UNITS[at(n)].to_string();
    }
    let (tens, unit) = (n / 10, n % 10);
    match (tens, unit) {
        (8, 0) => "quatre vingts".into(),
        (8 | 9, _) => format!("quatre vingt {}", below_hundred(n - 80)),
        (7, 1) => "soixante et onze".into(),
        (7, _) => format!("soixante {}", below_hundred(n - 60)),
        (_, 0) => TENS[at(tens)].into(),
        (_, 1) => format!("{} et un", TENS[at(tens)]),
        _ => format!("{} {}", TENS[at(tens)], UNITS[at(unit)]),
    }
}

fn below_thousand(n: u64) -> String {
    let (hundreds, rest) = (n / 100, n % 100);
    let head = match (hundreds, rest) {
        (0, _) => return below_hundred(rest),
        (1, _) => "cent".to_string(),
        (_, 0) => format!("{} cents", UNITS[at(hundreds)]),
        _ => format!("{} cent", UNITS[at(hundreds)]),
    };
    if rest == 0 {
        head
    } else {
        format!("{head} {}", below_hundred(rest))
    }
}

/// The words of the million, milliard... part of a number, and what is left below a million.
fn big_scales(n: u64) -> (Vec<String>, u64) {
    let scales = [
        (1_000_000_000_000, "billion"),
        (1_000_000_000, "milliard"),
        (1_000_000, "million"),
    ];
    let mut rest = n;
    let mut words = Vec::new();
    for (size, name) in scales {
        let count = rest / size;
        if count > 0 {
            let plural = if count > 1 { "s" } else { "" };
            words.push(format!("{} {name}{plural}", cardinal(count)));
            rest %= size;
        }
    }
    (words, rest)
}

fn thousands(n: u64) -> Option<String> {
    match n / 1000 {
        0 => None,
        1 => Some("mille".into()),
        count => Some(format!("{} mille", below_thousand(count))),
    }
}

/// The words for a whole number, e.g. 1250 → "mille deux cent cinquante".
pub fn cardinal(n: u64) -> String {
    let (mut words, rest) = big_scales(n);
    words.extend(thousands(rest));
    if rest % 1000 > 0 || words.is_empty() {
        words.push(below_thousand(rest % 1000));
    }
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_become_french_words() {
        let cases: [(u64, &str); 16] = [
            (0, "zéro"),
            (21, "vingt et un"),
            (22, "vingt deux"),
            (71, "soixante et onze"),
            (77, "soixante dix sept"),
            (80, "quatre vingts"),
            (81, "quatre vingt un"),
            (91, "quatre vingt onze"),
            (99, "quatre vingt dix neuf"),
            (100, "cent"),
            (200, "deux cents"),
            (1000, "mille"),
            (1250, "mille deux cent cinquante"),
            (2024, "deux mille vingt quatre"),
            (1_000_000, "un million"),
            (3_500_000, "trois millions cinq cents mille"),
        ];
        for (n, words) in cases {
            assert_eq!(cardinal(n), words);
        }
    }
}
