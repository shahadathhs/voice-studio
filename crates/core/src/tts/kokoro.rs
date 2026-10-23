//! Text to speech with Kokoro-82M, run with ONNX Runtime.
//!
//! Text is cut into sentences, each sentence is turned into phonemes (see [`super::phonemes`]) and
//! then into numbers, and the model speaks it in the chosen voice. A voice is a table of 256-number
//! style vectors, one per possible sentence length.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use ort::session::Session;
use ort::value::Tensor;

use super::language::Language;
use super::levels::{quieten, silence};
use super::presets::{self, Preset, STYLE_SIZE, read_table};
use super::pronounce::{GOLD_FILE, Pronouncer, SILVER_FILE};
use super::sentences::{self, Piece};
use super::vocab::Vocab;
use crate::engines::{AudioBuffer, Control, TtsEngine, TtsOptions};
use crate::{Error, Result};

/// File names inside the model's folder; `catalog/models.json` lists the same names.
pub const MODEL_FILE: &str = "model_quantized.onnx";
pub const TOKENIZER_FILE: &str = "tokenizer.json";
/// Extension of a voice file, named after the voice (`af_heart.bin`).
pub(super) const VOICE_EXTENSION: &str = "bin";

/// Kokoro produces audio at this rate.
pub const SAMPLE_RATE: u32 = 24_000;
/// Slowest and fastest speaking speed.
pub const MIN_SPEED: f32 = 0.5;
pub const MAX_SPEED: f32 = 2.0;
/// The model reads up to 510 symbols; shorter pieces sound better, so longer ones are halved.
const MAX_TOKENS: usize = 400;
const SENTENCE_PAUSE_MS: usize = 250;
const PARAGRAPH_PAUSE_MS: usize = 650;
const SPLIT_PAUSE_MS: usize = 120;

fn engine_error(e: impl Display) -> Error {
    Error::Engine(format!("kokoro: {e}"))
}

/// A loaded Kokoro model with its pronunciation dictionary and voices.
pub struct KokoroEngine {
    session: Session,
    vocab: Vocab,
    pronouncer: Pronouncer,
    dir: PathBuf,
    voices: HashMap<String, Vec<f32>>,
    /// Where custom voices are kept; none means only shipped voices.
    presets_dir: Option<PathBuf>,
    /// A blend being tried out that is not saved yet.
    draft: Option<Preset>,
}

impl KokoroEngine {
    /// Loads the model from `dir`, where the catalog installs it. `model_id` only labels the
    /// error when files are missing.
    pub fn load(dir: &Path, model_id: &str) -> Result<KokoroEngine> {
        for file in [MODEL_FILE, TOKENIZER_FILE, GOLD_FILE, SILVER_FILE] {
            if !dir.join(file).is_file() {
                return Err(Error::ModelMissing(model_id.to_string()));
            }
        }
        let session = crate::onnx::load_session(&dir.join(MODEL_FILE)).map_err(engine_error)?;
        Ok(KokoroEngine {
            session,
            vocab: Vocab::from_tokenizer_json(&std::fs::read_to_string(dir.join(TOKENIZER_FILE))?)?,
            pronouncer: Pronouncer::load(dir)?,
            dir: dir.to_path_buf(),
            voices: HashMap::new(),
            presets_dir: None,
            draft: None,
        })
    }

    /// Lets the engine speak the custom voices saved in `dir`.
    #[must_use]
    pub fn with_presets(mut self, dir: &Path) -> KokoroEngine {
        self.presets_dir = Some(dir.to_path_buf());
        self
    }

    /// The voices installed in `dir`, by name, in alphabetical order.
    pub fn voices_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                (path.extension()? == VOICE_EXTENSION)
                    .then(|| path.file_stem()?.to_str().map(str::to_string))?
            })
            .collect();
        names.sort();
        names
    }

    /// Sets (or clears) an unsaved blend that can be spoken under its own id.
    pub fn set_draft(&mut self, draft: Option<Preset>) {
        self.draft = draft;
    }

    /// The custom voice called `voice`, if there is one.
    fn preset(&self, voice: &str) -> Option<Preset> {
        if let Some(draft) = self.draft.as_ref().filter(|d| d.id == voice) {
            return Some(draft.clone());
        }
        let dir = self.presets_dir.as_deref()?;
        presets::load(dir, voice).ok()
    }

    fn voice_path(&self, voice: &str) -> PathBuf {
        self.dir.join(format!("{voice}.{VOICE_EXTENSION}"))
    }

    /// The style table of a shipped voice or a custom blend.
    fn table(&self, voice: &str, preset: Option<&Preset>) -> Result<Vec<f32>> {
        if let Some(preset) = preset {
            return preset.blend(|part| read_table(&self.voice_path(part)));
        }
        let path = self.voice_path(voice);
        if !path.is_file() {
            return Err(Error::Input(format!(
                "unknown voice: {voice} (installed: {})",
                KokoroEngine::voices_in(&self.dir).join(", ")
            )));
        }
        read_table(&path)
    }

    /// The style vector for a sentence of `tokens` symbols in `voice`.
    fn style(&mut self, voice: &str, tokens: usize) -> Result<Vec<f32>> {
        let preset = self.preset(voice);
        let key = preset
            .as_ref()
            .map_or_else(|| voice.to_string(), Preset::cache_key);
        if !self.voices.contains_key(&key) {
            let table = self.table(voice, preset.as_ref())?;
            self.voices.insert(key.clone(), table);
        }
        let table = &self.voices[&key];
        let rows = table.len() / STYLE_SIZE;
        let row = tokens.min(rows - 1);
        Ok(table[row * STYLE_SIZE..(row + 1) * STYLE_SIZE].to_vec())
    }

    fn infer(&mut self, tokens: &[i64], style: Vec<f32>, speed: f32) -> Result<Vec<f32>> {
        // The model expects a padding symbol (0) before and after the sentence.
        let mut ids = Vec::with_capacity(tokens.len() + 2);
        ids.push(0);
        ids.extend_from_slice(tokens);
        ids.push(0);
        let length = ids.len();
        let outputs = self
            .session
            .run(ort::inputs![
                "input_ids" => Tensor::from_array(([1usize, length], ids)).map_err(engine_error)?,
                "style" => Tensor::from_array(([1usize, STYLE_SIZE], style)).map_err(engine_error)?,
                "speed" => Tensor::from_array(([1usize], vec![speed])).map_err(engine_error)?,
            ])
            .map_err(engine_error)?;
        let (_, audio) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(engine_error)?;
        Ok(audio.to_vec())
    }

    /// Speaks one sentence, halving it first when it is too long for the model.
    fn say(&mut self, text: &str, options: &TtsOptions) -> Result<Vec<f32>> {
        let language = Language::of_voice(&options.voice)
            .ok_or_else(|| Error::Input(format!("unknown voice: {}", options.voice)))?;
        let tokens = self
            .vocab
            .encode(&self.pronouncer.phonemize(language, text)?);
        if tokens.is_empty() {
            return Ok(Vec::new());
        }
        if tokens.len() > MAX_TOKENS
            && let Some((head, tail)) = sentences::halve(text)
        {
            let mut audio = self.say(&head, options)?;
            audio.extend(silence(SPLIT_PAUSE_MS));
            audio.extend(self.say(&tail, options)?);
            return Ok(audio);
        }
        let style = self.style(&options.voice, tokens.len())?;
        self.infer(
            &tokens[..tokens.len().min(MAX_TOKENS + 100)],
            style,
            options.speed,
        )
    }

    /// Speaks `text`. `on_progress(done, total)` is called after each sentence and can cancel.
    pub fn speak(
        &mut self,
        text: &str,
        options: &TtsOptions,
        on_progress: &mut dyn FnMut(usize, usize) -> Control,
    ) -> Result<AudioBuffer> {
        if !(MIN_SPEED..=MAX_SPEED).contains(&options.speed) {
            return Err(Error::Input(format!(
                "speed must be between {MIN_SPEED} and {MAX_SPEED}"
            )));
        }
        let pieces = sentences::pieces(text);
        if pieces.is_empty() {
            return Err(Error::Input("there is no text to speak".into()));
        }
        let total = pieces.len();
        let mut samples = Vec::new();
        for (
            n,
            Piece {
                text,
                paragraph_end,
            },
        ) in pieces.iter().enumerate()
        {
            let mut audio = self.say(text, options)?;
            quieten(&mut audio);
            samples.extend(audio);
            if n + 1 < total {
                samples.extend(silence(if *paragraph_end {
                    PARAGRAPH_PAUSE_MS
                } else {
                    SENTENCE_PAUSE_MS
                }));
            }
            if on_progress(n + 1, total) == Control::Cancel {
                return Err(Error::Cancelled);
            }
        }
        Ok(AudioBuffer {
            samples,
            sample_rate: SAMPLE_RATE,
        })
    }
}

impl TtsEngine for KokoroEngine {
    fn synthesize(&mut self, text: &str, options: &TtsOptions) -> Result<AudioBuffer> {
        self.speak(text, options, &mut |_, _| Control::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_model_files_are_reported_as_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let result = KokoroEngine::load(dir.path(), "kokoro-82m");
        assert!(matches!(result, Err(Error::ModelMissing(id)) if id == "kokoro-82m"));
    }

    #[test]
    fn voices_are_the_bin_files_in_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["bf_emma.bin", "af_heart.bin", "tokenizer.json"] {
            std::fs::write(dir.path().join(name), b"").unwrap();
        }
        assert_eq!(KokoroEngine::voices_in(dir.path()), ["af_heart", "bf_emma"]);
        assert_eq!(KokoroEngine::voices_in(Path::new("/no/such/dir")).len(), 0);
    }

    #[test]
    fn the_catalog_installs_every_file_the_engine_reads() {
        let catalog = crate::models::Catalog::bundled().unwrap();
        let entry = catalog.find("kokoro-82m").unwrap();
        let names: Vec<&str> = entry.files.iter().map(|f| f.name.as_str()).collect();
        for needed in [MODEL_FILE, TOKENIZER_FILE, GOLD_FILE, SILVER_FILE] {
            assert!(names.contains(&needed), "{needed}");
        }
        assert!(
            names
                .iter()
                .filter(|n| Path::new(n).extension().is_some_and(|e| e == "bin"))
                .count()
                >= 2
        );
        assert_eq!(entry.kind, crate::models::ModelKind::Tts);
    }
}
