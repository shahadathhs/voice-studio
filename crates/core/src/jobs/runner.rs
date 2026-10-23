//! What actually transcribes a job. The queue only knows the [`JobRunner`] trait, so tests can
//! use a fake and the app uses [`WhisperRunner`].

use super::JobSpec;
use crate::diarize::SpeakerLabeller;
use crate::engines::{Control, Device, SttOptions, WhisperEngine};
use crate::models::{Catalog, ModelEntry, ModelStore, SPEAKER_MODEL_ID};
use crate::paths::Paths;
use crate::transcribe::{Speakers, transcribe_file};
use crate::{Error, Result, Transcript};

/// Turns one job into a transcript. `on_progress(processed_ms, total_ms)` can cancel.
pub trait JobRunner: Send + 'static {
    fn run(
        &mut self,
        spec: &JobSpec,
        on_progress: &mut dyn FnMut(u64, u64) -> Control,
    ) -> Result<Transcript>;
}

/// Transcribes with whisper.cpp and labels speakers with the speaker model, keeping both loaded
/// between jobs that use the same ones.
pub struct WhisperRunner {
    store: ModelStore,
    device: Device,
    loaded: Option<(String, WhisperEngine)>,
    speakers: Option<SpeakerLabeller>,
}

impl WhisperRunner {
    pub fn new(paths: &Paths, device: Device) -> WhisperRunner {
        WhisperRunner {
            store: ModelStore::new(paths),
            device,
            loaded: None,
            speakers: None,
        }
    }

    /// The catalog entry of an installed model.
    fn installed_entry(&self, catalog: &Catalog, model_id: &str) -> Result<ModelEntry> {
        let entry = catalog
            .find(model_id)
            .ok_or_else(|| Error::Input(format!("unknown model: {model_id}")))?;
        if self.store.is_installed(entry) {
            Ok(entry.clone())
        } else {
            Err(Error::ModelMissing(model_id.to_string()))
        }
    }

    fn load_engine(&mut self, spec: &JobSpec) -> Result<()> {
        let catalog = Catalog::bundled()?;
        let entry = self.installed_entry(&catalog, &spec.model_id)?;
        if spec.translate && !entry.can_translate {
            return Err(Error::Input(format!(
                "{} cannot translate; choose another model",
                entry.name
            )));
        }
        if self
            .loaded
            .as_ref()
            .is_none_or(|(id, _)| *id != spec.model_id)
        {
            let path = self.store.file_path(&entry, &entry.files[0]);
            let engine = WhisperEngine::load(&path, &spec.model_id, self.device)?;
            self.loaded = Some((spec.model_id.clone(), engine));
        }
        Ok(())
    }

    fn load_speakers(&mut self) -> Result<()> {
        if self.speakers.is_none() {
            let catalog = Catalog::bundled()?;
            let entry = self.installed_entry(&catalog, SPEAKER_MODEL_ID)?;
            self.speakers = Some(crate::diarize::load(&self.store, &entry)?);
        }
        Ok(())
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
            ..SttOptions::default()
        };
        self.load_engine(spec)?;
        if spec.speakers.is_some() {
            self.load_speakers()?;
        }
        let engine = self
            .loaded
            .as_mut()
            .map(|(_, engine)| engine)
            .ok_or_else(|| Error::Engine("no model loaded".into()))?;
        let speakers = match (self.speakers.as_mut(), spec.speakers.as_ref()) {
            (Some(diarizer), Some(options)) => Some(Speakers { diarizer, options }),
            _ => None,
        };
        transcribe_file(engine, &spec.file, &options, speakers, on_progress)
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
            speakers: None,
        };
        let error = runner
            .run(&spec, &mut |_, _| Control::Continue)
            .unwrap_err();
        assert!(matches!(error, Error::ModelMissing(_)));
    }

    /// Makes a model count as installed without downloading it: files of the right size, all zeros.
    fn fake_install(paths: &Paths, id: &str) {
        let catalog = Catalog::bundled().unwrap();
        let entry = catalog.find(id).unwrap();
        let store = ModelStore::new(paths);
        std::fs::create_dir_all(store.model_dir(entry)).unwrap();
        for file in &entry.files {
            let handle = std::fs::File::create(store.file_path(entry, file)).unwrap();
            handle.set_len(file.size_bytes).unwrap();
        }
    }

    #[test]
    fn translating_with_a_turbo_model_fails_the_job_before_loading_anything() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::resolve(Some(dir.path().into()));
        fake_install(&paths, "whisper-large-v3-turbo");
        let mut runner = WhisperRunner::new(&paths, Device::Cpu);
        let spec = JobSpec {
            file: PathBuf::from("a.wav"),
            model_id: "whisper-large-v3-turbo".into(),
            language: None,
            translate: true,
            speakers: None,
        };
        let error = runner
            .run(&spec, &mut |_, _| Control::Continue)
            .unwrap_err();
        assert!(
            matches!(&error, Error::Input(m) if m.contains("cannot translate")),
            "{error}"
        );
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
            speakers: None,
        };
        let error = runner
            .run(&spec, &mut |_, _| Control::Continue)
            .unwrap_err();
        assert!(matches!(error, Error::Input(_)));
    }
}
