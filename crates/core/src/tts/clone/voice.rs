//! A cloned voice, the consent that made it lawful, and the file it is kept in.
//!
//! **File format** (`<id>.bin`): the bytes `VSCV`, a little-endian `u32` with the length of a JSON
//! header, the header, then the four tensors of [`ClonedVoice`] as little-endian numbers in the order
//! `cond_emb` (f32), `prompt_token` (i64), `x_vector` (f32), `prompt_feat` (f32). The header holds
//! the [`VoiceMeta`] and each tensor's shape.

use serde::{Deserialize, Serialize};

use super::array::Array;
use crate::tts::VoiceInfo;
use crate::{Error, Result};

/// Starts every voice file.
const MAGIC: &[u8; 4] = b"VSCV";

/// What a person confirms before a voice can be made from a recording. It is stored with the voice.
pub const CONSENT_STATEMENT: &str = "I own this voice or have the speaker's permission";

/// A confirmation that the person owns the voice in a recording or has the speaker's permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consent {
    /// The sentence the person confirmed.
    pub statement: String,
    /// When they confirmed it, as UTC `YYYY-MM-DDTHH:MM:SSZ`.
    pub given_at: String,
}

impl Consent {
    /// The consent record for a box that was ticked or a flag that was given. Without it there is
    /// no voice: `confirmed == false` is an input error.
    pub fn confirm(confirmed: bool) -> Result<Consent> {
        if !confirmed {
            return Err(Error::Input(format!(
                "cloning a voice needs consent: confirm that \"{CONSENT_STATEMENT}\""
            )));
        }
        Ok(Consent {
            statement: CONSENT_STATEMENT.to_string(),
            given_at: iso_time(super::now_seconds()),
        })
    }
}

/// What is known about a cloned voice without loading its numbers; this is what lists show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceMeta {
    /// `clone_<slug of the name>`; what `vtt speak --voice` takes.
    pub id: String,
    pub name: String,
    /// Always true: a cloned voice is never presented as a built-in one.
    pub cloned: bool,
    /// When the voice was made, as UTC `YYYY-MM-DDTHH:MM:SSZ`.
    pub created_at: String,
    pub consent: Consent,
}

impl VoiceMeta {
    /// The voice as front ends list it, marked as cloned.
    pub fn info(&self) -> VoiceInfo {
        VoiceInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: format!("cloned voice, made {}", self.created_at),
            language: String::new(),
            custom: false,
            cloned: true,
        }
    }
}

/// A voice made from a recording: what the speech encoder learned about it.
#[derive(Debug, Clone, PartialEq)]
pub struct ClonedVoice {
    pub meta: VoiceMeta,
    pub(super) cond_emb: Array<f32>,
    pub(super) prompt_token: Array<i64>,
    pub(super) x_vector: Array<f32>,
    pub(super) prompt_feat: Array<f32>,
}

#[derive(Serialize, Deserialize)]
struct Header {
    meta: VoiceMeta,
    shapes: [Vec<i64>; 4],
}

fn corrupt() -> Error {
    Error::Input("this cloned voice file is damaged; delete the voice and make it again".into())
}

fn put<T: Copy, const N: usize>(out: &mut Vec<u8>, data: &[T], bytes: fn(T) -> [u8; N]) {
    for value in data {
        out.extend(bytes(*value));
    }
}

fn take<T, const N: usize>(
    input: &mut &[u8],
    count: usize,
    number: fn([u8; N]) -> T,
) -> Result<Vec<T>> {
    let length = count.checked_mul(N).ok_or_else(corrupt)?;
    if input.len() < length {
        return Err(corrupt());
    }
    let (now, rest) = input.split_at(length);
    *input = rest;
    let (chunks, _) = now.as_chunks::<N>();
    Ok(chunks.iter().map(|chunk| number(*chunk)).collect())
}

fn count(shape: &[i64]) -> Result<usize> {
    shape.iter().try_fold(1usize, |total, dim| {
        usize::try_from(*dim)
            .ok()
            .and_then(|dim| total.checked_mul(dim))
            .ok_or_else(corrupt)
    })
}

impl ClonedVoice {
    /// The voice as the bytes of a voice file.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let header = serde_json::to_vec(&Header {
            meta: self.meta.clone(),
            shapes: [
                self.cond_emb.shape.clone(),
                self.prompt_token.shape.clone(),
                self.x_vector.shape.clone(),
                self.prompt_feat.shape.clone(),
            ],
        })?;
        let mut out = Vec::new();
        out.extend(MAGIC);
        out.extend(
            u32::try_from(header.len())
                .map_err(|_| corrupt())?
                .to_le_bytes(),
        );
        out.extend(header);
        put(&mut out, &self.cond_emb.data, f32::to_le_bytes);
        put(&mut out, &self.prompt_token.data, i64::to_le_bytes);
        put(&mut out, &self.x_vector.data, f32::to_le_bytes);
        put(&mut out, &self.prompt_feat.data, f32::to_le_bytes);
        Ok(out)
    }

    /// The voice stored in `bytes`, as written by [`ClonedVoice::to_bytes`].
    pub fn from_bytes(bytes: &[u8]) -> Result<ClonedVoice> {
        let rest = bytes.strip_prefix(MAGIC).ok_or_else(corrupt)?;
        let (length, rest) = rest.split_first_chunk::<4>().ok_or_else(corrupt)?;
        let length = u32::from_le_bytes(*length) as usize;
        if rest.len() < length {
            return Err(corrupt());
        }
        let (header, mut data) = rest.split_at(length);
        let header: Header = serde_json::from_slice(header).map_err(|_| corrupt())?;
        let [s0, s1, s2, s3] = header.shapes;
        let voice = ClonedVoice {
            cond_emb: Array {
                data: take(&mut data, count(&s0)?, f32::from_le_bytes)?,
                shape: s0,
            },
            prompt_token: Array {
                data: take(&mut data, count(&s1)?, i64::from_le_bytes)?,
                shape: s1,
            },
            x_vector: Array {
                data: take(&mut data, count(&s2)?, f32::from_le_bytes)?,
                shape: s2,
            },
            prompt_feat: Array {
                data: take(&mut data, count(&s3)?, f32::from_le_bytes)?,
                shape: s3,
            },
            meta: header.meta,
        };
        if !data.is_empty() || !voice.meta.cloned {
            return Err(corrupt());
        }
        Ok(voice)
    }
}

/// `seconds` since 1970 as UTC `YYYY-MM-DDTHH:MM:SSZ`.
pub(super) fn iso_time(seconds: u64) -> String {
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    // Civil date from a day count (Howard Hinnant's algorithm); days since 1970-01-01.
    let z = days + 719_468;
    let era = z / 146_097;
    let day_of_era = z % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

#[cfg(test)]
pub(super) fn sample_voice(id: &str) -> ClonedVoice {
    ClonedVoice {
        meta: VoiceMeta {
            id: id.to_string(),
            name: "Sample".into(),
            cloned: true,
            created_at: iso_time(0),
            consent: Consent::confirm(true).unwrap(),
        },
        cond_emb: Array {
            shape: vec![1, 2, 3],
            data: vec![0.5, -1.0, 2.0, 3.0, 4.0, 5.5],
        },
        prompt_token: Array {
            shape: vec![1, 3],
            data: vec![7, -8, 9],
        },
        x_vector: Array {
            shape: vec![1, 2],
            data: vec![0.1, 0.2],
        },
        prompt_feat: Array {
            shape: vec![1, 1, 2],
            data: vec![9.0, 8.0],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_is_required_and_records_the_sentence_and_the_time() {
        assert!(matches!(Consent::confirm(false), Err(Error::Input(m)) if m.contains("consent")));
        let consent = Consent::confirm(true).unwrap();
        assert_eq!(consent.statement, CONSENT_STATEMENT);
        assert!(consent.given_at.starts_with("20") && consent.given_at.ends_with('Z'));
    }

    #[test]
    fn times_are_written_as_utc_dates() {
        assert_eq!(iso_time(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_time(951_782_400 + 3661), "2000-02-29T01:01:01Z");
        assert_eq!(iso_time(1_792_000_000), "2026-10-14T17:46:40Z");
    }

    #[test]
    fn a_voice_survives_being_written_and_read() {
        let voice = sample_voice("clone_sample");
        let back = ClonedVoice::from_bytes(&voice.to_bytes().unwrap()).unwrap();
        assert_eq!(back, voice);
        assert!(back.meta.cloned);
    }

    #[test]
    fn damaged_files_are_input_errors_not_panics() {
        let bytes = sample_voice("clone_sample").to_bytes().unwrap();
        for cut in [0, 3, 6, 40, bytes.len() - 1] {
            assert!(matches!(
                ClonedVoice::from_bytes(&bytes[..cut]),
                Err(Error::Input(_))
            ));
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(ClonedVoice::from_bytes(&extra).is_err());
        assert!(ClonedVoice::from_bytes(b"XXXXjunkjunkjunk").is_err());
    }
}
