use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};

use super::WHISPER_SAMPLE_RATE;
use crate::{Error, Result};

const CHUNK_FRAMES: usize = 4096;

fn resample_error(e: impl std::fmt::Display) -> Error {
    Error::Engine(format!("resampling failed: {e}"))
}

/// Converts mono audio to 16 kHz as it is decoded, so a long file never has to be held in memory
/// at its original rate. Push samples as they arrive, then call [`StreamResampler::finish`].
pub struct StreamResampler {
    fft: Fft<f32>,
    pending: Vec<f32>,
    output: Vec<f32>,
    delay_left: usize,
    input_frames: u64,
    ratio: f64,
}

impl StreamResampler {
    pub fn new(input_rate: u32) -> Result<StreamResampler> {
        let fft = Fft::<f32>::new(
            input_rate as usize,
            WHISPER_SAMPLE_RATE as usize,
            CHUNK_FRAMES,
            1,
            FixedSync::Input,
        )
        .map_err(resample_error)?;
        let delay_left = fft.output_delay();
        Ok(StreamResampler {
            fft,
            pending: Vec::new(),
            output: Vec::new(),
            delay_left,
            input_frames: 0,
            ratio: f64::from(WHISPER_SAMPLE_RATE) / f64::from(input_rate),
        })
    }

    pub fn push(&mut self, mono: &[f32]) -> Result<()> {
        self.input_frames += mono.len() as u64;
        self.pending.extend_from_slice(mono);
        loop {
            let needed = self.fft.input_frames_next();
            if self.pending.len() < needed {
                return Ok(());
            }
            let chunk: Vec<f32> = self.pending.drain(..needed).collect();
            self.run(&chunk, None)?;
        }
    }

    /// Flushes the filter delay and returns exactly the resampled length of everything pushed.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn finish(mut self) -> Result<Vec<f32>> {
        let expected = (self.input_frames as f64 * self.ratio).ceil() as usize;
        if !self.pending.is_empty() {
            let real = self.pending.len();
            let mut chunk = std::mem::take(&mut self.pending);
            chunk.resize(self.fft.input_frames_next(), 0.0);
            self.run(&chunk, Some(real))?;
        }
        while self.output.len() < expected {
            let silence = vec![0.0; self.fft.input_frames_next()];
            self.run(&silence, None)?;
        }
        self.output.truncate(expected);
        Ok(self.output)
    }

    fn run(&mut self, chunk: &[f32], real_frames: Option<usize>) -> Result<()> {
        let input = InterleavedSlice::new(chunk, 1, chunk.len()).map_err(resample_error)?;
        let indexing = real_frames.map(|n| Indexing {
            partial_len: Some(n),
            ..Indexing::default()
        });
        let out = self
            .fft
            .process(&input, indexing.as_ref())
            .map_err(resample_error)?;
        let data = out.take_data();
        let skip = self.delay_left.min(data.len());
        self.delay_left -= skip;
        self.output.extend_from_slice(&data[skip..]);
        Ok(())
    }
}

#[cfg(test)]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
mod tests {
    use super::*;

    fn sine(rate: u32, seconds: f32, hz: f32) -> Vec<f32> {
        let n = (rate as f32 * seconds) as usize;
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * hz * i as f32 / rate as f32).sin())
            .collect()
    }

    fn resample(rate: u32, data: &[f32], piece: usize) -> Vec<f32> {
        let mut r = StreamResampler::new(rate).unwrap();
        for part in data.chunks(piece) {
            r.push(part).unwrap();
        }
        r.finish().unwrap()
    }

    #[test]
    fn output_length_matches_the_rate_ratio() {
        for rate in [44_100, 48_000, 22_050, 8_000] {
            let data = sine(rate, 2.0, 440.0);
            let out = resample(rate, &data, 1000);
            let expected = (data.len() as f64 * 16_000.0 / f64::from(rate)).ceil() as usize;
            assert_eq!(out.len(), expected, "rate {rate}");
        }
    }

    #[test]
    fn a_tone_keeps_its_frequency_and_level() {
        let out = resample(48_000, &sine(48_000, 1.0, 440.0), 777);
        // Skip the edges, then count rising zero crossings: one per cycle, so about 440 per second.
        let mid = &out[1000..out.len() - 1000];
        let crossings = mid.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
        let seconds = mid.len() as f32 / 16_000.0;
        let measured = crossings as f32 / seconds;
        assert!((measured - 440.0).abs() < 5.0, "measured {measured} Hz");
        let peak = mid.iter().fold(0.0f32, |p, s| p.max(s.abs()));
        assert!((0.9..=1.1).contains(&peak), "peak {peak}");
    }

    #[test]
    fn chunking_of_the_input_does_not_change_the_result() {
        let data = sine(44_100, 1.5, 300.0);
        assert_eq!(resample(44_100, &data, 100), resample(44_100, &data, 5000));
    }
}
