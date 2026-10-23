//! How tones change in speech: 不 and 一 before other tones, and a run of third tones.

use super::pinyin::Syllable;

const COUNTING: &str = "零〇一二三四五六七八九十百千万亿";

/// Changes the tones of `syllables`, which come from `run` one for one.
pub fn apply(syllables: &mut [Syllable], run: &[char]) {
    if syllables.len() != run.len() {
        third_tones(syllables);
        return;
    }
    for at in 0..syllables.len().saturating_sub(1) {
        let next_tone = syllables[at + 1].tone;
        let counting = COUNTING.contains(run[at + 1]);
        let current = &mut syllables[at];
        match (run[at], current.tone) {
            ('不', 4) if next_tone == 4 => current.tone = 2,
            ('一', 1) if !counting => current.tone = if next_tone == 4 { 2 } else { 4 },
            _ => {}
        }
    }
    third_tones(syllables);
}

/// Of several third tones in a row, all but the last are said as second tones.
fn third_tones(syllables: &mut [Syllable]) {
    let mut end = syllables.len();
    while end > 0 {
        if syllables[end - 1].tone != 3 {
            end -= 1;
            continue;
        }
        let mut start = end - 1;
        while start > 0 && syllables[start - 1].tone == 3 {
            start -= 1;
        }
        for syllable in &mut syllables[start..end - 1] {
            syllable.tone = 2;
        }
        end = start;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tones(text: &str, readings: &[&str]) -> Vec<u8> {
        let run: Vec<char> = text.chars().collect();
        let mut syllables: Vec<Syllable> = readings.iter().map(|r| Syllable::parse(r)).collect();
        apply(&mut syllables, &run);
        syllables.iter().map(|s| s.tone).collect()
    }

    #[test]
    fn bu_and_yi_change_before_other_tones() {
        assert_eq!(tones("不是", &["bù", "shì"]), [2, 4]);
        assert_eq!(tones("不好", &["bù", "hǎo"]), [4, 3]);
        assert_eq!(tones("一个", &["yī", "gè"]), [2, 4]);
        assert_eq!(tones("一天", &["yī", "tiān"]), [4, 1]);
        assert_eq!(tones("一二", &["yī", "èr"]), [1, 4]);
    }

    #[test]
    fn runs_of_third_tones_rise_except_the_last() {
        assert_eq!(tones("你好", &["nǐ", "hǎo"]), [2, 3]);
        assert_eq!(tones("我很好", &["wǒ", "hěn", "hǎo"]), [2, 2, 3]);
        assert_eq!(tones("好人", &["hǎo", "rén"]), [3, 2]);
    }
}
