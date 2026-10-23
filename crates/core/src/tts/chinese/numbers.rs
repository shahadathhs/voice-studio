//! Numbers written with digits, as Chinese characters to be read.

const DIGITS: [char; 10] = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
/// More digits than this are read one by one, as a phone number would be.
const MAX_DIGITS: usize = 10;

/// The character for a digit 0 to 9.
fn digit_at(n: u64) -> char {
    DIGITS[usize::try_from(n).unwrap_or(0).min(9)]
}

fn digit(c: char) -> char {
    c.to_digit(10).map_or(c, |d| DIGITS[d as usize])
}

/// One digit after another: 2024 → 二零二四.
pub fn by_digit(digits: &str) -> String {
    digits.chars().map(digit).collect()
}

fn below_ten_thousand(n: u64, out: &mut String) {
    let base = out.len();
    let mut zero_pending = false;
    let places = [(1000, '千'), (100, '百'), (10, '十')];
    let mut rest = n;
    for (size, name) in places {
        let count = rest / size;
        rest %= size;
        if count > 0 {
            if zero_pending {
                out.push('零');
                zero_pending = false;
            }
            // 10 to 19 at the start of a number are 十 …, not 一十 ….
            if !(size == 10 && count == 1 && out.is_empty()) {
                out.push(digit_at(count));
            }
            out.push(name);
        } else if out.len() > base && rest > 0 {
            zero_pending = true;
        }
    }
    if rest > 0 {
        if zero_pending {
            out.push('零');
        }
        out.push(digit_at(rest));
    }
}

/// A whole number as characters: 1250 → 一千二百五十.
pub fn whole(n: u64) -> String {
    if n == 0 {
        return "零".into();
    }
    let mut out = String::new();
    let mut zero_pending = false;
    for (size, name) in [(100_000_000, '亿'), (10_000, '万')] {
        let count = n / size % 10_000;
        if count > 0 {
            let mut part = String::new();
            below_ten_thousand(count, &mut part);
            if zero_pending {
                out.push('零');
            }
            out.push_str(&part);
            out.push(name);
            zero_pending = false;
        } else if !out.is_empty() {
            zero_pending = true;
        }
    }
    let tail = n % 10_000;
    if tail > 0 {
        if zero_pending || (!out.is_empty() && tail < 1000) {
            out.push('零');
        }
        below_ten_thousand(tail, &mut out);
    }
    out
}

/// A number written with digits (and perhaps a decimal point) as characters.
pub fn spoken(text: &str) -> String {
    let (whole_part, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digits: String = whole_part.chars().filter(char::is_ascii_digit).collect();
    let words = if digits.len() > MAX_DIGITS || (digits.len() > 1 && digits.starts_with('0')) {
        by_digit(&digits)
    } else {
        whole(digits.parse().unwrap_or(0))
    };
    if fraction.is_empty() {
        words
    } else {
        format!("{words}点{}", by_digit(fraction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_read_the_chinese_way() {
        assert_eq!(whole(0), "零");
        assert_eq!(whole(10), "十");
        assert_eq!(whole(11), "十一");
        assert_eq!(whole(25), "二十五");
        assert_eq!(whole(101), "一百零一");
        assert_eq!(whole(110), "一百一十");
        assert_eq!(whole(1001), "一千零一");
        assert_eq!(whole(1250), "一千二百五十");
        assert_eq!(whole(10_000), "一万");
        assert_eq!(whole(10_005), "一万零五");
        assert_eq!(whole(123_456), "十二万三千四百五十六");
        assert_eq!(whole(100_000_000), "一亿");
    }

    #[test]
    fn decimals_and_long_numbers() {
        assert_eq!(spoken("3.14"), "三点一四");
        assert_eq!(spoken("007"), "零零七");
        assert_eq!(spoken("13800138000"), "一三八零零一三八零零零");
        assert_eq!(by_digit("2024"), "二零二四");
    }
}
