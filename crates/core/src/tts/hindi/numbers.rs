//! Hindi numbers as words. Every number below 100 has its own word, so all of them are listed.

const BELOW_HUNDRED: [&str; 100] = [
    "शून्य",
    "एक",
    "दो",
    "तीन",
    "चार",
    "पाँच",
    "छह",
    "सात",
    "आठ",
    "नौ",
    "दस",
    "ग्यारह",
    "बारह",
    "तेरह",
    "चौदह",
    "पंद्रह",
    "सोलह",
    "सत्रह",
    "अठारह",
    "उन्नीस",
    "बीस",
    "इक्कीस",
    "बाईस",
    "तेईस",
    "चौबीस",
    "पच्चीस",
    "छब्बीस",
    "सत्ताईस",
    "अट्ठाईस",
    "उनतीस",
    "तीस",
    "इकतीस",
    "बत्तीस",
    "तैंतीस",
    "चौंतीस",
    "पैंतीस",
    "छत्तीस",
    "सैंतीस",
    "अड़तीस",
    "उनतालीस",
    "चालीस",
    "इकतालीस",
    "बयालीस",
    "तैंतालीस",
    "चौवालीस",
    "पैंतालीस",
    "छियालीस",
    "सैंतालीस",
    "अड़तालीस",
    "उनचास",
    "पचास",
    "इक्यावन",
    "बावन",
    "तिरपन",
    "चौवन",
    "पचपन",
    "छप्पन",
    "सत्तावन",
    "अट्ठावन",
    "उनसठ",
    "साठ",
    "इकसठ",
    "बासठ",
    "तिरसठ",
    "चौंसठ",
    "पैंसठ",
    "छियासठ",
    "सड़सठ",
    "अड़सठ",
    "उनहत्तर",
    "सत्तर",
    "इकहत्तर",
    "बहत्तर",
    "तिहत्तर",
    "चौहत्तर",
    "पचहत्तर",
    "छिहत्तर",
    "सतहत्तर",
    "अठहत्तर",
    "उनासी",
    "अस्सी",
    "इक्यासी",
    "बयासी",
    "तिरासी",
    "चौरासी",
    "पचासी",
    "छियासी",
    "सत्तासी",
    "अट्ठासी",
    "नवासी",
    "नब्बे",
    "इक्यानवे",
    "बानवे",
    "तिरानवे",
    "चौरानवे",
    "पंचानवे",
    "छियानवे",
    "सत्तानवे",
    "अट्ठानवे",
    "निन्यानवे",
];

/// Indian grouping: each unit holds this many of the one below, from the crore down to the hundred.
const UNITS: [(u64, &str); 4] = [
    (10_000_000, "करोड़"),
    (100_000, "लाख"),
    (1_000, "हज़ार"),
    (100, "सौ"),
];

pub const DIGITS: [&str; 10] = [
    "शून्य",
    "एक",
    "दो",
    "तीन",
    "चार",
    "पाँच",
    "छह",
    "सात",
    "आठ",
    "नौ",
];

/// The words of a whole number.
pub fn cardinal(n: u64) -> String {
    let mut rest = n;
    let mut words: Vec<String> = Vec::new();
    for (size, name) in UNITS {
        let count = rest / size;
        rest %= size;
        if count > 0 {
            words.push(format!("{} {name}", cardinal(count)));
        }
    }
    if rest > 0 || words.is_empty() {
        words.push(BELOW_HUNDRED[usize::try_from(rest).unwrap_or(0)].to_string());
    }
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_use_indian_grouping() {
        assert_eq!(cardinal(0), "शून्य");
        assert_eq!(cardinal(21), "इक्कीस");
        assert_eq!(cardinal(105), "एक सौ पाँच");
        assert_eq!(cardinal(2500), "दो हज़ार पाँच सौ");
        assert_eq!(cardinal(150_000), "एक लाख पचास हज़ार");
        assert_eq!(cardinal(25_000_000), "दो करोड़ पचास लाख");
        assert_eq!(cardinal(1_000_000_000), "एक सौ करोड़");
    }
}
