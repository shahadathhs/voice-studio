//! Numbers as English words, so "1,250 people in 2024" can be pronounced.

const ONES: [&str; 20] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];
const TENS: [&str; 10] = [
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];
const SCALES: [(u64, &str); 4] = [
    (1_000_000_000_000, "trillion"),
    (1_000_000_000, "billion"),
    (1_000_000, "million"),
    (1_000, "thousand"),
];
/// Numbers with more digits than this are read digit by digit.
const MAX_DIGITS: usize = 15;

/// A number below 100 as an index into the word tables.
fn index(n: u64) -> usize {
    usize::try_from(n).unwrap_or(0)
}

fn below_hundred(n: u64) -> Vec<String> {
    if n < 20 {
        return vec![ONES[index(n)].to_string()];
    }
    let tens = TENS[index(n / 10)];
    match n % 10 {
        0 => vec![tens.to_string()],
        unit => vec![format!("{tens} {}", ONES[index(unit)])],
    }
}

fn below_thousand(n: u64) -> Vec<String> {
    let mut words = Vec::new();
    if n >= 100 {
        words.push(ONES[index(n / 100)].to_string());
        words.push("hundred".to_string());
    }
    if !n.is_multiple_of(100) {
        words.extend(below_hundred(n % 100));
    }
    words
}

/// The words for a whole number, e.g. 1250 → "one thousand two hundred fifty".
pub fn cardinal(n: u64) -> String {
    if n == 0 {
        return "zero".into();
    }
    let mut rest = n;
    let mut words: Vec<String> = Vec::new();
    for (size, name) in SCALES {
        if rest >= size {
            words.extend(below_thousand(rest / size));
            words.push(name.to_string());
            rest %= size;
        }
    }
    if rest > 0 {
        words.extend(below_thousand(rest));
    }
    words.join(" ")
}

/// A digit string read one digit at a time, e.g. "007" → "zero zero seven".
pub fn digits(text: &str) -> String {
    text.chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| ONES[d as usize])
        .collect::<Vec<_>>()
        .join(" ")
}

/// "first", "second", … for a position, e.g. 21 → "twenty first".
pub fn ordinal(n: u64) -> String {
    let words = cardinal(n);
    let (head, last) = words
        .rsplit_once(' ')
        .map_or(("", words.as_str()), |(h, l)| (h, l));
    let ordinal_last = match last {
        "one" => "first".to_string(),
        "two" => "second".to_string(),
        "three" => "third".to_string(),
        "five" => "fifth".to_string(),
        "eight" => "eighth".to_string(),
        "nine" => "ninth".to_string(),
        "twelve" => "twelfth".to_string(),
        word if word.ends_with('y') => format!("{}ieth", &word[..word.len() - 1]),
        word => format!("{word}th"),
    };
    if head.is_empty() {
        ordinal_last
    } else {
        format!("{head} {ordinal_last}")
    }
}

/// A year as people say it: 1984 → "nineteen eighty four", 2005 → "two thousand five".
pub fn year(n: u64) -> String {
    let (high, low) = (n / 100, n % 100);
    match (high, low) {
        (_, 0) if n.is_multiple_of(1000) => cardinal(n),
        (_, 0) => format!("{} hundred", cardinal(high)),
        (20, 1..=9) => format!("two thousand {}", cardinal(low)),
        (_, 1..=9) => format!("{} oh {}", cardinal(high), cardinal(low)),
        _ => format!("{} {}", cardinal(high), cardinal(low)),
    }
}

/// Whether a number written without separators reads as a year.
pub fn looks_like_year(text: &str) -> bool {
    text.len() == 4
        && text
            .parse::<u64>()
            .is_ok_and(|n| (1100..=2099).contains(&n))
}

/// A number as written in text: optional thousands commas and a decimal part.
/// "1,250.5" → "one thousand two hundred fifty point five".
pub fn spoken(text: &str) -> String {
    let cleaned: String = text.chars().filter(|c| *c != ',').collect();
    let (whole, fraction) = match cleaned.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (cleaned.as_str(), None),
    };
    let mut words = if whole.is_empty() {
        String::new()
    } else if whole.len() > MAX_DIGITS {
        digits(whole)
    } else if fraction.is_none() && looks_like_year(text) {
        year(whole.parse().unwrap_or(0))
    } else {
        cardinal(whole.parse().unwrap_or(0))
    };
    if let Some(fraction) = fraction.filter(|f| !f.is_empty()) {
        words = format!("{words} point {}", digits(fraction))
            .trim()
            .to_string();
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_numbers_are_spelled_out() {
        assert_eq!(cardinal(0), "zero");
        assert_eq!(cardinal(13), "thirteen");
        assert_eq!(cardinal(42), "forty two");
        assert_eq!(cardinal(100), "one hundred");
        assert_eq!(cardinal(1250), "one thousand two hundred fifty");
        assert_eq!(cardinal(2_000_003), "two million three");
        assert_eq!(cardinal(7_000_000_000), "seven billion");
    }

    #[test]
    fn ordinals_change_the_last_word() {
        assert_eq!(ordinal(1), "first");
        assert_eq!(ordinal(2), "second");
        assert_eq!(ordinal(12), "twelfth");
        assert_eq!(ordinal(20), "twentieth");
        assert_eq!(ordinal(21), "twenty first");
        assert_eq!(ordinal(100), "one hundredth");
        assert_eq!(ordinal(4), "fourth");
    }

    #[test]
    fn years_are_read_in_pairs() {
        assert_eq!(year(1984), "nineteen eighty four");
        assert_eq!(year(1905), "nineteen oh five");
        assert_eq!(year(1900), "nineteen hundred");
        assert_eq!(year(2000), "two thousand");
        assert_eq!(year(2005), "two thousand five");
        assert_eq!(year(2024), "twenty twenty four");
        assert_eq!(year(2010), "twenty ten");
    }

    #[test]
    fn numbers_as_written_in_text() {
        assert_eq!(spoken("1,250"), "one thousand two hundred fifty");
        assert_eq!(spoken("3.14"), "three point one four");
        assert_eq!(spoken("2024"), "twenty twenty four");
        assert_eq!(spoken("2024.5"), "two thousand twenty four point five");
        assert_eq!(spoken("007"), "seven");
        assert_eq!(
            spoken("12345678901234567"),
            "one two three four five six seven eight nine zero one two three four five six seven"
        );
    }

    #[test]
    fn digit_strings_are_read_one_by_one() {
        assert_eq!(digits("007"), "zero zero seven");
    }
}
