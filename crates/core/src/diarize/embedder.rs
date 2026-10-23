//! The speaker embedding model (`WeSpeaker` `ResNet34`, `ONNX`), run with `ONNX Runtime`.

use std::fmt::Display;
use std::path::Path;

use ort::session::Session;
use ort::value::Tensor;

use super::SpeakerEmbedder;
use super::fbank::{Fbank, MEL_BINS};
use crate::{Error, Result};

fn engine_error(e: impl Display) -> Error {
    Error::Engine(format!("speaker model: {e}"))
}

/// A loaded speaker embedding model.
pub struct SpeakerModel {
    session: Session,
    fbank: Fbank,
}

impl SpeakerModel {
    /// Loads the ONNX file. `model_id` only labels the error when the file is missing.
    pub fn load(path: &Path, model_id: &str) -> Result<SpeakerModel> {
        if !path.exists() {
            return Err(Error::ModelMissing(model_id.to_string()));
        }
        let session = crate::onnx::load_session(path).map_err(engine_error)?;
        Ok(SpeakerModel {
            session,
            fbank: Fbank::new(),
        })
    }
}

impl SpeakerEmbedder for SpeakerModel {
    fn embed(&mut self, samples: &[f32]) -> Result<Vec<f32>> {
        let features = self.fbank.features(samples);
        let frames = features.len() / MEL_BINS;
        if frames == 0 {
            return Err(Error::Engine("speaker model: audio too short".into()));
        }
        let input =
            Tensor::from_array(([1usize, frames, MEL_BINS], features)).map_err(engine_error)?;
        let outputs = self
            .session
            .run(ort::inputs![input])
            .map_err(engine_error)?;
        let (_, embedding) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(engine_error)?;
        Ok(embedding.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_model_file_is_reported_as_not_installed() {
        let result = SpeakerModel::load(Path::new("/no/such/model.onnx"), "wespeaker-resnet34");
        assert!(matches!(result, Err(Error::ModelMissing(id)) if id == "wespeaker-resnet34"));
    }

    #[test]
    fn a_file_that_is_not_a_model_is_an_engine_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.onnx");
        std::fs::write(&path, b"not onnx").unwrap();
        assert!(matches!(
            SpeakerModel::load(&path, "fake"),
            Err(Error::Engine(_))
        ));
    }
}
