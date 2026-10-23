//! Reading numbers written with digits for languages other than English: which of `.` and `,`
//! is the decimal mark, and how a number is put into words once it is understood.

/// Numbers with more digits than this are read digit by digit.
const MAX_DIGITS: usize = 15;

/// A number split into its whole part and the digits after the decimal mark.
#[derive(Debug, PartialEq, Eq)]
pub struct Parsed {
    pub whole: String,
    pub fraction: Option<String>,
}

fn only_digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

/// A separator followed only by groups of three digits is a thousands mark.
fn groups_of_three(text: &str, separator: char) -> bool {
    let mut groups = text.split(separator).skip(1).peekable();
    groups.peek().is_some() && groups.all(|g| g.len() == 3)
}

/// Understands "1.250,5", "1,250.5", "3,5", "1.250" and "1250". `decimal` is the mark the
/// language prefers (`,` for most of Europe, `.` for Hindi and Chinese).
pub fn parse(text: &str, decimal: char) -> Parsed {
    let other = if decimal == ',' { '.' } else { ',' };
    let split_at = match (text.rfind(decimal), text.rfind(other)) {
        (Some(d), Some(o)) => Some(d.max(o)),
        (Some(d), None)
            if !groups_of_three(text, decimal) || text.matches(decimal).count() == 1 =>
        {
            Some(d)
        }
        (None, Some(o)) if !groups_of_three(text, other) => Some(o),
        _ => None,
    };
    match split_at {
        Some(at) => Parsed {
            whole: only_digits(&text[..at]),
            fraction: Some(only_digits(&text[at + 1..])),
        },
        None => Parsed {
            whole: only_digits(text),
            fraction: None,
        },
    }
}

/// The words of a language's numbers.
pub struct NumberWords {
    pub cardinal: fn(u64) -> String,
    /// The words for 0 to 9.
    pub digits: [&'static str; 10],
    /// The word for the decimal mark ("coma", "virgule").
    pub point: &'static str,
}

impl NumberWords {
    fn digit_by_digit(&self, digits: &str) -> String {
        digits
            .chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| self.digits[d as usize])
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The words for a number written with digits.
    pub fn spoken(&self, text: &str, decimal: char) -> String {
        let parsed = parse(text, decimal);
        let whole = if parsed.whole.is_empty() || parsed.whole.len() > MAX_DIGITS {
            self.digit_by_digit(&parsed.whole)
        } else {
            (self.cardinal)(parsed.whole.parse().unwrap_or(0))
        };
        match parsed.fraction.filter(|f| !f.is_empty()) {
            Some(fraction) => format!("{whole} {} {}", self.point, self.digit_by_digit(&fraction)),
            None => whole,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str, decimal: char) -> (String, Option<String>) {
        let p = parse(text, decimal);
        (p.whole, p.fraction)
    }

    #[test]
    fn the_decimal_mark_is_told_from_the_thousands_mark() {
        assert_eq!(parsed("3,5", ','), ("3".into(), Some("5".into())));
        assert_eq!(parsed("1.250", ','), ("1250".into(), None));
        assert_eq!(parsed("1.250,5", ','), ("1250".into(), Some("5".into())));
        assert_eq!(parsed("1,250.5", ','), ("1250".into(), Some("5".into())));
        assert_eq!(parsed("3.14", ','), ("3".into(), Some("14".into())));
        assert_eq!(parsed("1,250", '.'), ("1250".into(), None));
        assert_eq!(parsed("2024", ','), ("2024".into(), None));
    }
}
