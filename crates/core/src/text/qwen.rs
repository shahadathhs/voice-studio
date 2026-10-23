//! Qwen 2.5 Instruct from a GGUF file, run with candle. (`llama.cpp` cannot share a binary with
//! `whisper.cpp`: both bundle ggml.)

use std::path::Path;

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::quantized_qwen2::ModelWeights;
use candle_transformers::utils::apply_repeat_penalty;
use tokenizers::Tokenizer;

use crate::engines::{Control, TextEngine};
use crate::{Error, Result};

pub const TOKENIZER_FILE: &str = "tokenizer.json";

/// The prompt and the answer together must fit here; the model itself handles far more, but the
/// memory for a longer text is not worth it on a laptop.
const CONTEXT_TOKENS: usize = 8192;
const REPEAT_PENALTY: f32 = 1.1;
const REPEAT_WINDOW: usize = 64;
const END_MARKS: [&str; 2] = ["<|im_end|>", "<|endoftext|>"];

pub struct QwenEngine {
    model: ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
    end_ids: Vec<u32>,
}

fn engine_error(error: impl std::fmt::Display) -> Error {
    Error::Engine(format!("language model: {error}"))
}

/// The GPU on a Mac, otherwise (or if it cannot be used) the processor.
fn pick_device() -> Device {
    #[cfg(target_os = "macos")]
    if let Ok(device) = Device::new_metal(0) {
        return device;
    }
    Device::Cpu
}

/// The weights file of an installed model: the one `.gguf` file in its folder.
fn gguf_in(dir: &Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "gguf"))
}

impl QwenEngine {
    pub fn load(dir: &Path, model_id: &str) -> Result<QwenEngine> {
        let weights = gguf_in(dir).filter(|_| dir.join(TOKENIZER_FILE).is_file());
        let Some(weights) = weights else {
            return Err(Error::ModelMissing(model_id.to_string()));
        };
        let tokenizer = Tokenizer::from_file(dir.join(TOKENIZER_FILE)).map_err(engine_error)?;
        let mut file = std::fs::File::open(weights)?;
        let content = gguf_file::Content::read(&mut file).map_err(engine_error)?;
        let device = pick_device();
        let model = ModelWeights::from_gguf(content, &mut file, &device).map_err(engine_error)?;
        let end_ids = END_MARKS
            .iter()
            .filter_map(|mark| tokenizer.token_to_id(mark))
            .collect();
        Ok(QwenEngine {
            model,
            tokenizer,
            device,
            end_ids,
        })
    }

    fn tokens_of(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self.tokenizer.encode(text, false).map_err(engine_error)?;
        Ok(encoding.get_ids().to_vec())
    }

    /// The logits for the token after `tokens`, which start at position `at`.
    fn next_logits(&mut self, tokens: &[u32], at: usize) -> Result<Tensor> {
        let input = Tensor::new(tokens, &self.device)
            .and_then(|t| t.unsqueeze(0))
            .map_err(engine_error)?;
        let logits = self.model.forward(&input, at).map_err(engine_error)?;
        logits.squeeze(0).map_err(engine_error)
    }
}

/// Qwen's chat format.
fn chat_prompt(instruction: &str, input: &str) -> String {
    format!(
        "<|im_start|>system\n{instruction}<|im_end|>\n<|im_start|>user\n{input}<|im_end|>\n<|im_start|>assistant\n"
    )
}

impl TextEngine for QwenEngine {
    fn generate(
        &mut self,
        instruction: &str,
        input: &str,
        max_tokens: usize,
        on_progress: &mut dyn FnMut(usize) -> Control,
    ) -> Result<String> {
        let prompt = self.tokens_of(&chat_prompt(instruction, input))?;
        if prompt.len() + max_tokens > CONTEXT_TOKENS {
            return Err(Error::Input(
                "this text is too long for the language model; try a shorter part".into(),
            ));
        }
        self.model.clear_kv_cache();
        // Always the likeliest word: a summary or translation should not vary between runs.
        let mut sampler = LogitsProcessor::from_sampling(0, Sampling::ArgMax);
        let mut written: Vec<u32> = Vec::new();
        let mut logits = self.next_logits(&prompt, 0)?;
        for step in 0..max_tokens {
            let recent = &written[written.len().saturating_sub(REPEAT_WINDOW)..];
            let penalised =
                apply_repeat_penalty(&logits, REPEAT_PENALTY, recent).map_err(engine_error)?;
            let next = sampler.sample(&penalised).map_err(engine_error)?;
            if self.end_ids.contains(&next) {
                break;
            }
            written.push(next);
            if on_progress(written.len()) == Control::Cancel {
                return Err(Error::Cancelled);
            }
            logits = self.next_logits(&[next], prompt.len() + step)?;
        }
        let text = self
            .tokenizer
            .decode(&written, true)
            .map_err(engine_error)?;
        Ok(text.trim().to_string())
    }
}
