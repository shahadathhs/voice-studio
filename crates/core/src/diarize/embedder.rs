//! The speaker embedding model (`WeSpeaker` `ResNet34`, `ONNX`), run in pure Rust with `tract`.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use tract_onnx::prelude::*;

use super::SpeakerEmbedder;
use super::fbank::{Fbank, MEL_BINS};
use crate::{Error, Result};

/// The model is compiled once per input length; this bounds how many compiled copies are kept.
const MAX_COMPILED: usize = 64;

type Plan = TypedRunnableModel<TypedModel>;

fn engine_error(e: impl Display) -> Error {
    Error::Engine(format!("speaker model: {e}"))
}

/// A loaded speaker embedding model.
pub struct SpeakerModel {
    model: InferenceModel,
    plans: HashMap<usize, Plan>,
    fbank: Fbank,
}

impl SpeakerModel {
    /// Loads the ONNX file. `model_id` only labels the error when the file is missing.
    pub fn load(path: &Path, model_id: &str) -> Result<SpeakerModel> {
        if !path.exists() {
            return Err(Error::ModelMissing(model_id.to_string()));
        }
        let model = tract_onnx::onnx()
            .model_for_path(path)
            .map_err(engine_error)?;
        Ok(SpeakerModel {
            model,
            plans: HashMap::new(),
            fbank: Fbank::new(),
        })
    }

    fn plan(&mut self, frames: usize) -> Result<&Plan> {
        if !self.plans.contains_key(&frames) {
            if self.plans.len() >= MAX_COMPILED {
                self.plans.clear();
            }
            let plan = self
                .model
                .clone()
                .with_input_fact(0, f32::fact([1, frames, MEL_BINS]).into())
                .and_then(InferenceModel::into_optimized)
                .and_then(TypedModel::into_runnable)
                .map_err(engine_error)?;
            self.plans.insert(frames, plan);
        }
        self.plans
            .get(&frames)
            .ok_or_else(|| Error::Engine("speaker model: no plan".into()))
    }
}

impl SpeakerEmbedder for SpeakerModel {
    fn embed(&mut self, samples: &[f32]) -> Result<Vec<f32>> {
        let features = self.fbank.features(samples);
        let frames = features.len() / MEL_BINS;
        if frames == 0 {
            return Err(Error::Engine("speaker model: audio too short".into()));
        }
        let input = Tensor::from_shape(&[1, frames, MEL_BINS], &features).map_err(engine_error)?;
        let output = self
            .plan(frames)?
            .run(tvec!(input.into()))
            .map_err(engine_error)?;
        let embedding = output[0].to_array_view::<f32>().map_err(engine_error)?;
        Ok(embedding.iter().copied().collect())
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
