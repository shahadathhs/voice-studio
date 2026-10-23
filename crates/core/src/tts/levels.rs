//! Pauses and loudness of the audio Kokoro makes.

use super::kokoro::SAMPLE_RATE;

pub(super) fn silence(ms: usize) -> Vec<f32> {
    vec![0.0; ms * SAMPLE_RATE as usize / 1000]
}

/// Scales audio down when its peak would clip; quiet audio is left alone.
pub(super) fn quieten(samples: &mut [f32]) {
    let peak = samples.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));
    if peak > 1.0 {
        for sample in samples.iter_mut() {
            *sample /= peak;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_audio_that_would_clip_is_scaled_down() {
        let mut loud = vec![2.0, -1.0, 0.5];
        quieten(&mut loud);
        assert_eq!(loud, [1.0, -0.5, 0.25]);
        let mut quiet = vec![0.2, -0.4];
        quieten(&mut quiet);
        assert_eq!(quiet, [0.2, -0.4]);
    }

    #[test]
    fn pauses_have_the_right_length() {
        assert_eq!(silence(250).len(), 6000);
    }
}
