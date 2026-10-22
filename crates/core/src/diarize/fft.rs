//! A small radix-2 FFT, enough for the fixed 512-point transforms of the filterbank.

use std::f32::consts::TAU;

/// A count or index as `f32`, exact up to 65535 and saturating above. Every size in the filterbank
/// is far below that.
pub fn as_f32(n: usize) -> f32 {
    u16::try_from(n).map_or(f32::from(u16::MAX), f32::from)
}

/// Power spectrum calculator for real input of one fixed power-of-two size.
pub struct PowerSpectrum {
    size: usize,
    cos: Vec<f32>,
    sin: Vec<f32>,
}

impl PowerSpectrum {
    /// `size` must be a power of two.
    pub fn new(size: usize) -> PowerSpectrum {
        let angle = |k: usize| TAU * as_f32(k) / as_f32(size);
        PowerSpectrum {
            size,
            cos: (0..size / 2).map(|k| angle(k).cos()).collect(),
            sin: (0..size / 2).map(|k| -angle(k).sin()).collect(),
        }
    }

    /// `|X[k]|^2` for `k` in `0..=size/2`. `input` is zero-padded or cut to the transform size.
    pub fn power(&self, input: &[f32]) -> Vec<f32> {
        let mut re = vec![0.0; self.size];
        let mut im = vec![0.0; self.size];
        for (slot, value) in re.iter_mut().zip(input) {
            *slot = *value;
        }
        self.transform(&mut re, &mut im);
        (0..=self.size / 2)
            .map(|k| re[k] * re[k] + im[k] * im[k])
            .collect()
    }

    fn reorder(&self, re: &mut [f32], im: &mut [f32]) {
        let bits = self.size.trailing_zeros();
        for i in 0..self.size {
            let j = i.reverse_bits() >> (usize::BITS - bits);
            if j > i {
                re.swap(i, j);
                im.swap(i, j);
            }
        }
    }

    fn transform(&self, re: &mut [f32], im: &mut [f32]) {
        self.reorder(re, im);
        let mut half = 1;
        while half < self.size {
            let step = self.size / (half * 2);
            for start in (0..self.size).step_by(half * 2) {
                for k in 0..half {
                    let (c, s) = (self.cos[k * step], self.sin[k * step]);
                    let (a, b) = (start + k, start + k + half);
                    let (tr, ti) = (re[b] * c - im[b] * s, re[b] * s + im[b] * c);
                    re[b] = re[a] - tr;
                    im[b] = im[a] - ti;
                    re[a] += tr;
                    im[a] += ti;
                }
            }
            half *= 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_a_direct_dft() {
        let input: Vec<f32> = (0..16).map(|n| as_f32(n * 5 % 7) - 3.0).collect();
        let fast = PowerSpectrum::new(16).power(&input);
        for (k, got) in fast.iter().enumerate() {
            let (mut re, mut im) = (0.0_f32, 0.0_f32);
            for (n, x) in input.iter().enumerate() {
                let angle = TAU * as_f32(k * n) / 16.0;
                re += x * angle.cos();
                im -= x * angle.sin();
            }
            let want = re * re + im * im;
            assert!(
                (got - want).abs() < 1e-2 * want.max(1.0),
                "bin {k}: {got} vs {want}"
            );
        }
    }

    #[test]
    fn a_pure_tone_lands_in_its_bin() {
        let input: Vec<f32> = (0..64)
            .map(|n| (TAU * 8.0 * as_f32(n) / 64.0).sin())
            .collect();
        let power = PowerSpectrum::new(64).power(&input);
        let loudest = power
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(k, _)| k);
        assert_eq!(loudest, Some(8));
    }
}
