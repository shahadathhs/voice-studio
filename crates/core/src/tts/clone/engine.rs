//! The Chatterbox engine: loads the four models and turns a recording into a voice and text into
//! speech in that voice.

use std::path::Path;

use ort::session::Session;
use ort::value::Tensor;
use tokenizers::Tokenizer;

use super::array::{Array, engine_error};
use super::generate::START_SPEECH;
use super::language::prepare;
use super::voice::{ClonedVoice, Consent, VoiceMeta};
use super::{
    DECODER_FILE, EMBED_FILE, ENCODER_FILE, LANGUAGE_MODEL_FILE, MODEL_ID, SAMPLE_RATE,
    TOKENIZER_FILE, now_seconds,
};
use crate::audio::{resample_to, voiced_span};
use crate::engines::{AudioBuffer, Control};
use crate::tts::levels::{quieten, silence};
use crate::tts::sentences::{Piece, pieces};
use crate::{Error, Result};

/// A reference recording has to hold between this many and [`MAX_REFERENCE_SECONDS`] of speech.
pub const MIN_REFERENCE_SECONDS: usize = 3;
pub const MAX_REFERENCE_SECONDS: usize = 30;
const SENTENCE_PAUSE_MS: usize = 250;
const PARAGRAPH_PAUSE_MS: usize = 650;
/// Speech tokens come at 25 a second; a sentence gets room for this many per character, plus a base.
const TOKENS_PER_CHARACTER: usize = 3;
const BASE_TOKENS: usize = 60;
const MAX_TOKENS: usize = 500;

/// The loaded Chatterbox models.
pub struct CloneEngine {
    pub(super) speech_encoder: Session,
    pub(super) embed_tokens: Session,
    pub(super) language_model: Session,
    pub(super) conditional_decoder: Session,
    tokenizer: Tokenizer,
}

fn session(dir: &Path, file: &str) -> Result<Session> {
    crate::onnx::load_session_cpu(&dir.join(file)).map_err(engine_error)
}

impl CloneEngine {
    /// Loads the models from `dir`, where the catalog installs `chatterbox-multilingual`.
    pub fn load(dir: &Path) -> Result<CloneEngine> {
        let files = [
            ENCODER_FILE,
            EMBED_FILE,
            LANGUAGE_MODEL_FILE,
            DECODER_FILE,
            TOKENIZER_FILE,
        ];
        if files.iter().any(|file| !dir.join(file).is_file()) {
            return Err(Error::ModelMissing(MODEL_ID.to_string()));
        }
        Ok(CloneEngine {
            speech_encoder: session(dir, ENCODER_FILE)?,
            embed_tokens: session(dir, EMBED_FILE)?,
            language_model: session(dir, LANGUAGE_MODEL_FILE)?,
            conditional_decoder: session(dir, DECODER_FILE)?,
            tokenizer: Tokenizer::from_file(dir.join(TOKENIZER_FILE)).map_err(engine_error)?,
        })
    }

    /// Makes a voice called `name` from `audio` (mono, any sample rate) of one speaker.
    ///
    /// `consent` is required: it is the person's confirmation that they own the voice or have the
    /// speaker's permission, and it is stored with the voice.
    pub fn voice_from_audio(
        &mut self,
        audio: &AudioBuffer,
        name: &str,
        consent: Consent,
    ) -> Result<ClonedVoice> {
        let id = super::store::id_for(name)?;
        let reference = reference_samples(audio)?;
        let length = i64::try_from(reference.len()).map_err(engine_error)?;
        let outputs = self
            .speech_encoder
            .run(ort::inputs![
                "audio_values" => Tensor::from_array((vec![1, length], reference)).map_err(engine_error)?,
            ])
            .map_err(engine_error)?;
        if outputs.len() < 4 {
            return Err(Error::Engine(
                "chatterbox: the speech encoder gave too few outputs".into(),
            ));
        }
        Ok(ClonedVoice {
            meta: VoiceMeta {
                id,
                name: name.trim().to_string(),
                cloned: true,
                created_at: super::voice::iso_time(now_seconds()),
                consent,
            },
            cond_emb: Array::from_value(&outputs[0])?,
            prompt_token: Array::from_value(&outputs[1])?,
            x_vector: Array::from_value(&outputs[2])?,
            prompt_feat: Array::from_value(&outputs[3])?,
        })
    }

    /// Speaks one prepared sentence in `voice`.
    fn say(&mut self, voice: &ClonedVoice, text: &str, language: &str) -> Result<Vec<f32>> {
        let prepared = prepare(text, language)?;
        let encoding = self
            .tokenizer
            .encode(prepared, true)
            .map_err(engine_error)?;
        let ids: Vec<i64> = encoding.get_ids().iter().map(|id| i64::from(*id)).collect();
        let limit = (BASE_TOKENS + TOKENS_PER_CHARACTER * text.chars().count()).min(MAX_TOKENS);
        let mut tokens = voice.prompt_token.data.clone();
        let generated = self.speech_tokens(&voice.cond_emb, &ids, limit)?;
        tokens.extend(generated.into_iter().filter(|token| *token < START_SPEECH));
        let count = i64::try_from(tokens.len()).map_err(engine_error)?;
        let outputs = self
            .conditional_decoder
            .run(ort::inputs![
                "speech_tokens" => Tensor::from_array((vec![1, count], tokens)).map_err(engine_error)?,
                "speaker_embeddings" => voice.x_vector.tensor()?,
                "speaker_features" => voice.prompt_feat.tensor()?,
            ])
            .map_err(engine_error)?;
        Ok(Array::<f32>::from_value(&outputs[0])?.data)
    }

    /// Speaks `text` in `voice`, one sentence at a time. `language` is a code such as `es`.
    /// `on_progress(done, total)` is called after each sentence and can cancel.
    ///
    /// The audio is synthetic speech in a cloned voice; whoever saves or shares it should keep the
    /// [`super::DISCLOSURE`] note with it.
    pub fn speak(
        &mut self,
        voice: &ClonedVoice,
        text: &str,
        language: &str,
        on_progress: &mut dyn FnMut(usize, usize) -> Control,
    ) -> Result<AudioBuffer> {
        let parts = pieces(text);
        if parts.is_empty() {
            return Err(Error::Input("there is no text to speak".into()));
        }
        let total = parts.len();
        let mut samples = Vec::new();
        for (
            n,
            Piece {
                text,
                paragraph_end,
            },
        ) in parts.iter().enumerate()
        {
            let mut audio = self.say(voice, text, language)?;
            quieten(&mut audio);
            samples.extend(audio);
            if n + 1 < total {
                let pause = if *paragraph_end {
                    PARAGRAPH_PAUSE_MS
                } else {
                    SENTENCE_PAUSE_MS
                };
                samples.extend(silence(pause));
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

/// The speech in `audio` at the model's sample rate, or an input error when there is too little,
/// too much or none.
fn reference_samples(audio: &AudioBuffer) -> Result<Vec<f32>> {
    let audio = resample_to(audio, SAMPLE_RATE)?;
    let span = voiced_span(&audio.samples, SAMPLE_RATE)
        .ok_or_else(|| Error::Input("the recording has no speech in it (it is silent)".into()))?;
    let per_second = SAMPLE_RATE as usize;
    let tenths = span.len() * 10 / per_second;
    if span.len() < MIN_REFERENCE_SECONDS * per_second {
        return Err(Error::Input(format!(
            "the recording has {}.{} s of speech; a voice needs {MIN_REFERENCE_SECONDS} to {MAX_REFERENCE_SECONDS} s",
            tenths / 10,
            tenths % 10
        )));
    }
    if span.len() > MAX_REFERENCE_SECONDS * per_second {
        return Err(Error::Input(format!(
            "the recording has {} s of speech; use at most {MAX_REFERENCE_SECONDS} s",
            tenths / 10
        )));
    }
    Ok(audio.samples[span].to_vec())
}
