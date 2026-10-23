//! Speaker segmentation: which of up to three local speakers talk at every moment of a 10 s window
//! (pyannote segmentation-3.0, `ONNX`, run with `ONNX Runtime`). Overlapping speech is part of the
//! output: two local speakers can be active in the same frame.

use std::fmt::Display;
use std::path::Path;

use ort::session::Session;
use ort::value::Tensor;

use crate::{Error, Result};

/// Length of the audio the model looks at once.
pub const WINDOW_SAMPLES: usize = 160_000;
/// Samples between two output frames (about 17 ms).
pub const FRAME_SAMPLES: usize = 270;
/// Samples one output frame listens to; its centre is half of this after the frame's start.
pub const FIELD_SAMPLES: usize = 991;
/// Local speakers the model tells apart inside one window.
pub const LOCAL_SPEAKERS: usize = 3;
/// Powerset classes: nobody, each speaker alone, each pair of speakers.
const CLASSES: usize = 7;
/// For each class, which local speakers are active.
const POWERSET: [[bool; LOCAL_SPEAKERS]; CLASSES] = [
    [false, false, false],
    [true, false, false],
    [false, true, false],
    [false, false, true],
    [true, true, false],
    [true, false, true],
    [false, true, true],
];

/// Which local speakers are active in one output frame.
pub type Frame = [bool; LOCAL_SPEAKERS];

/// Tells who is active when, inside one window of 16 kHz mono audio.
pub trait Segmenter: Send {
    /// One entry per output frame of a window of [`WINDOW_SAMPLES`] samples (zero-padded by the
    /// caller when the recording ends sooner).
    fn activity(&mut self, window: &[f32]) -> Result<Vec<Frame>>;
}

fn engine_error(e: impl Display) -> Error {
    Error::Engine(format!("segmentation model: {e}"))
}

/// The loaded segmentation model.
pub struct SegmentationModel {
    session: Session,
}

impl SegmentationModel {
    /// Loads the ONNX file. `model_id` only labels the error when the file is missing.
    pub fn load(path: &Path, model_id: &str) -> Result<SegmentationModel> {
        if !path.exists() {
            return Err(Error::ModelMissing(model_id.to_string()));
        }
        let session = crate::onnx::load_session(path).map_err(engine_error)?;
        Ok(SegmentationModel { session })
    }
}

/// The active speakers of the most likely class of every frame; `scores` holds `CLASSES` values
/// per frame.
fn decode(scores: &[f32]) -> Vec<Frame> {
    let (frames, _) = scores.as_chunks::<CLASSES>();
    frames
        .iter()
        .map(|frame| {
            let best = frame
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map_or(0, |(class, _)| class);
            POWERSET[best]
        })
        .collect()
}

impl Segmenter for SegmentationModel {
    fn activity(&mut self, window: &[f32]) -> Result<Vec<Frame>> {
        let input = Tensor::from_array(([1usize, 1, window.len()], window.to_vec()))
            .map_err(engine_error)?;
        let outputs = self
            .session
            .run(ort::inputs![input])
            .map_err(engine_error)?;
        let (shape, scores) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(engine_error)?;
        if shape.last().and_then(|n| usize::try_from(*n).ok()) != Some(CLASSES) {
            return Err(Error::Engine(format!(
                "segmentation model: expected {CLASSES} classes per frame, got shape {shape:?}"
            )));
        }
        Ok(decode(scores))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_frame_takes_the_speakers_of_its_most_likely_class() {
        let mut scores = vec![0.0; CLASSES * 3];
        scores[0] = 5.0; // nobody
        scores[CLASSES + 2] = 5.0; // speaker 1 alone
        scores[2 * CLASSES + 6] = 5.0; // speakers 1 and 2
        assert_eq!(
            decode(&scores),
            [
                [false, false, false],
                [false, true, false],
                [false, true, true]
            ]
        );
    }

    #[test]
    fn a_missing_model_file_is_reported_as_not_installed() {
        let result = SegmentationModel::load(Path::new("/no/such/model.onnx"), "speaker-models");
        assert!(matches!(result, Err(Error::ModelMissing(id)) if id == "speaker-models"));
    }

    #[test]
    fn a_file_that_is_not_a_model_is_an_engine_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.onnx");
        std::fs::write(&path, b"not onnx").unwrap();
        assert!(matches!(
            SegmentationModel::load(&path, "fake"),
            Err(Error::Engine(_))
        ));
    }
}
