//! The language model half of Chatterbox: text and a voice in, speech tokens out.
//!
//! This follows the model card's script: the text and the voice's conditioning are embedded, a
//! Llama-style model writes one speech token at a time with its attention cache carried from step to
//! step, repetition is discouraged with a penalty of 1.2, and the most likely token is taken (the
//! script does not sample with a temperature).

use std::borrow::Cow;
use std::collections::HashSet;

use ort::session::SessionInputValue;
use ort::value::{DynValue, Tensor};

use super::array::{Array, engine_error};
use super::engine::CloneEngine;
use crate::{Error, Result};

/// The speech tokens the decoder understands are `0..START_SPEECH`.
pub(super) const START_SPEECH: i64 = 6561;
const STOP_SPEECH: i64 = 6562;
const LAYERS: usize = 30;
const HEADS: i64 = 16;
const HEAD_DIM: i64 = 64;
const REPETITION_PENALTY: f32 = 1.2;
/// How strongly the voice's emotion is pushed; 0.5 is the model card's default.
const EXAGGERATION: f32 = 0.5;

type Cache = Vec<(String, DynValue)>;

fn empty_cache() -> Result<Cache> {
    let mut cache = Vec::with_capacity(LAYERS * 2);
    for layer in 0..LAYERS {
        for part in ["key", "value"] {
            let tensor = Tensor::from_array((vec![1, HEADS, 0, HEAD_DIM], Vec::<f32>::new()))
                .map_err(engine_error)?;
            cache.push((format!("past_key_values.{layer}.{part}"), tensor.into_dyn()));
        }
    }
    Ok(cache)
}

/// Lowers the scores of tokens that were already spoken, as the reference implementation does.
fn penalise(scores: &mut [f32], spoken: &HashSet<i64>) {
    for token in spoken {
        if let Some(score) = usize::try_from(*token)
            .ok()
            .and_then(|at| scores.get_mut(at))
        {
            *score = if *score < 0.0 {
                *score * REPETITION_PENALTY
            } else {
                *score / REPETITION_PENALTY
            };
        }
    }
}

fn best(scores: &[f32]) -> i64 {
    scores
        .iter()
        .enumerate()
        .fold((0, f32::NEG_INFINITY), |top, (at, score)| {
            if *score > top.1 { (at, *score) } else { top }
        })
        .0
        .try_into()
        .unwrap_or(0)
}

impl CloneEngine {
    /// Embeds `ids` (text, or one speech token) at `positions`.
    fn embed(&mut self, ids: Vec<i64>, positions: Vec<i64>) -> Result<Array<f32>> {
        let length = i64::try_from(ids.len()).map_err(engine_error)?;
        let outputs = self
            .embed_tokens
            .run(ort::inputs![
                "input_ids" => Tensor::from_array((vec![1, length], ids)).map_err(engine_error)?,
                "position_ids" => Tensor::from_array((vec![1, length], positions)).map_err(engine_error)?,
                "exaggeration" => Tensor::from_array((vec![1_i64], vec![EXAGGERATION])).map_err(engine_error)?,
            ])
            .map_err(engine_error)?;
        Array::from_value(&outputs[0])
    }

    /// One pass of the language model; returns the scores of the next token and the new cache.
    fn step(&mut self, embeds: &Array<f32>, mask: i64, cache: Cache) -> Result<(Vec<f32>, Cache)> {
        let mut inputs: Vec<(Cow<str>, SessionInputValue)> = vec![
            ("inputs_embeds".into(), embeds.tensor()?.into()),
            (
                "attention_mask".into(),
                Tensor::from_array((
                    vec![1, mask],
                    vec![1_i64; usize::try_from(mask).unwrap_or(0)],
                ))
                .map_err(engine_error)?
                .into(),
            ),
        ];
        inputs.extend(
            cache
                .into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
        let outputs = self.language_model.run(inputs).map_err(engine_error)?;
        let mut scores = Vec::new();
        let mut next = Vec::with_capacity(LAYERS * 2);
        for (at, (_, value)) in outputs.into_iter().enumerate() {
            if at == 0 {
                scores = last_scores(&value)?;
                continue;
            }
            let part = if at % 2 == 1 { "key" } else { "value" };
            next.push((format!("past_key_values.{}.{part}", (at - 1) / 2), value));
        }
        if next.len() != LAYERS * 2 {
            return Err(Error::Engine(
                "chatterbox: unexpected language model outputs".into(),
            ));
        }
        Ok((scores, next))
    }

    /// Speech tokens for `text_ids` in the voice of `conditioning`, at most `limit` of them.
    pub(super) fn speech_tokens(
        &mut self,
        conditioning: &Array<f32>,
        text_ids: &[i64],
        limit: usize,
    ) -> Result<Vec<i64>> {
        let positions = text_ids
            .iter()
            .enumerate()
            .map(|(at, id)| {
                if *id >= START_SPEECH {
                    0
                } else {
                    i64::try_from(at).unwrap_or(0) - 1
                }
            })
            .collect();
        let text = self.embed(text_ids.to_vec(), positions)?;
        let mut embeds = concat(conditioning, &text)?;
        let prompt_length = embeds.shape[1];
        let mut cache = empty_cache()?;
        let mut spoken = HashSet::from([START_SPEECH]);
        let mut tokens = Vec::new();
        for step in 0..limit {
            let step_number = i64::try_from(step).map_err(engine_error)?;
            let mask = prompt_length + step_number;
            let (mut scores, next_cache) = self.step(&embeds, mask, cache)?;
            cache = next_cache;
            penalise(&mut scores, &spoken);
            let token = best(&scores);
            if token == STOP_SPEECH {
                break;
            }
            spoken.insert(token);
            tokens.push(token);
            embeds = self.embed(vec![token], vec![step_number + 1])?;
        }
        Ok(tokens)
    }
}

/// The scores for the token after the last position of a `[1, positions, vocabulary]` tensor.
fn last_scores(logits: &DynValue) -> Result<Vec<f32>> {
    let (shape, data) = logits.try_extract_tensor::<f32>().map_err(engine_error)?;
    let vocabulary = shape
        .last()
        .and_then(|v| usize::try_from(*v).ok())
        .filter(|v| *v > 0 && data.len() >= *v)
        .ok_or_else(|| Error::Engine("chatterbox: unexpected logits shape".into()))?;
    Ok(data[data.len() - vocabulary..].to_vec())
}

/// `[1, a, size]` followed by `[1, b, size]` along the middle axis.
fn concat(first: &Array<f32>, second: &Array<f32>) -> Result<Array<f32>> {
    let same_width = first.shape.len() == 3 && first.shape.get(2) == second.shape.get(2);
    if !same_width {
        return Err(Error::Engine("chatterbox: embedding shapes differ".into()));
    }
    let mut data = first.data.clone();
    data.extend_from_slice(&second.data);
    Ok(Array {
        shape: vec![1, first.shape[1] + second.shape[1], first.shape[2]],
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_tokens_lose_score_whatever_its_sign() {
        let mut scores = vec![1.2, -1.0, 3.0];
        penalise(&mut scores, &HashSet::from([0, 1, 99]));
        assert!((scores[0] - 1.0).abs() < 1e-6);
        assert!((scores[1] + 1.2).abs() < 1e-6);
        assert!((scores[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn the_best_token_has_the_highest_score() {
        assert_eq!(best(&[0.1, 0.9, 0.3]), 1);
        assert_eq!(best(&[]), 0);
    }

    #[test]
    fn embeddings_are_joined_along_the_sequence() {
        let a = Array {
            shape: vec![1, 1, 2],
            data: vec![1.0, 2.0],
        };
        let b = Array {
            shape: vec![1, 2, 2],
            data: vec![3.0, 4.0, 5.0, 6.0],
        };
        let joined = concat(&a, &b).unwrap();
        assert_eq!(joined.shape, [1, 3, 2]);
        assert_eq!(joined.data, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let wrong = Array {
            shape: vec![1, 1, 3],
            data: vec![0.0; 3],
        };
        assert!(concat(&a, &wrong).is_err());
    }

    #[test]
    fn the_empty_cache_covers_every_layer_for_keys_and_values() {
        let cache = empty_cache().unwrap();
        assert_eq!(cache.len(), 60);
        assert_eq!(cache[0].0, "past_key_values.0.key");
        assert_eq!(cache[59].0, "past_key_values.29.value");
    }
}
