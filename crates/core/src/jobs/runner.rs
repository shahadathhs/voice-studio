//! What actually transcribes a job. The queue only knows the [`JobRunner`] trait, so tests can
//! use a fake and the app uses [`WhisperRunner`].

use super::JobSpec;
use crate::engines::{Control, Device, SttOptions, WhisperEngine};
use crate::models::{Catalog, ModelStore};
use crate::paths::Paths;
use crate::transcribe::transcribe_file;
use crate::{Error, Result, Transcript};

/// Turns one job into a transcript. `on_progress(processed_ms, total_ms)` can cancel.
pub trait JobRunner: Send + 'static {
    fn run(
        &mut self,
        spec: &JobSpec,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript>;
}

/// Transcribes with whisper.cpp, keeping the loaded model between jobs that use the same one.
pub struct WhisperRunner {
    store: ModelStore,
    device: Device,
    loaded: Option<(String, WhisperEngine)>,
}

impl WhisperRunner {
    pub fn new(paths: &Paths, device: Device) -> WhisperRunner {
        WhisperRunner {
            store: ModelStore::new(paths),
            device,
            loaded: None,
        }
    }

    fn engine_for(&mut self, model_id: &str) -> Result<&mut WhisperEngine> {
        if self.loaded.as_ref().is_none_or(|(id, _)| id != model_id) {
            let catalog = Catalog::bundled()?;
            let entry = catalog
                .find(model_id)
                .ok_or_else(|| Error::Input(format!("unknown model: {model_id}")))?;
            if !self.store.is_installed(entry) {
                return Err(Error::ModelMissing(model_id.to_string()));
            }
            let path = self.store.file_path(entry, &entry.files[0]);
            let engine = WhisperEngine::load(&path, model_id, self.device)?;
            self.loaded = Some((model_id.to_string(), engine));
        }
        self.loaded
            .as_mut()
            .map(|(_, engine)| engine)
            .ok_or_else(|| Error::Engine("no model loaded".into()))
    }
}

impl JobRunner for WhisperRunner {
    fn run(
        &mut self,
        spec: &JobSpec,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript> {
        let options = SttOptions {
            language: spec.language.clone(),
            translate: spec.translate,
            vad: true,
        };
        let engine = self.engine_for(&spec.model_id)?;
        transcribe_file(engine, &spec.file, &options, on_progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn a_model_that_is_not_installed_fails_the_job_with_exit_code_4() {
        let dir = tempfile::tempdir().unwrap();
        let mut runner = WhisperRunner::new(&Paths::resolve(Some(dir.path().into())), Device::Cpu);
        let spec = JobSpec {
            file: PathBuf::from("a.wav"),
            model_id: "whisper-base".into(),
            language: None,
            translate: false,
        };
        let error = runner
            .run(&spec, &mut |_, _| Control::Continue)
            .unwrap_err();
        assert!(matches!(error, Error::ModelMissing(_)));
    }

    #[test]
    fn an_unknown_model_is_an_input_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut runner = WhisperRunner::new(&Paths::resolve(Some(dir.path().into())), Device::Cpu);
        let spec = JobSpec {
            file: PathBuf::from("a.wav"),
            model_id: "nope".into(),
            language: None,
            translate: false,
        };
        let error = runner
            .run(&spec, &mut |_, _| Control::Continue)
            .unwrap_err();
        assert!(matches!(error, Error::Input(_)));
    }
}
