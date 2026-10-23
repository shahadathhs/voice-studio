//! `vtt voice`: custom voices made by blending shipped ones.

use clap::Subcommand;
use vtt_core::models::{Catalog, ModelStore, SPEECH_MODEL_ID};
use vtt_core::paths::Paths;
use vtt_core::tts::{Preset, VoiceInfo, delete_preset, save_preset, voice_list};
use vtt_core::{Error, Result};

use crate::output::Output;

#[derive(Subcommand, Debug)]
pub enum VoiceCommand {
    /// List shipped and custom voices
    List,
    /// Save a blend of shipped voices of one language as a new voice
    Create {
        /// The voice's name (1 to 40 characters)
        name: String,
        /// 2 to 4 voices with weights, e.g. `af_heart:0.6,af_bella:0.4`
        #[arg(long, value_name = "VOICE:WEIGHT,...")]
        blend: String,
    },
    /// Clone a voice from a recording of someone who has given permission
    Clone(crate::clone_cmd::CloneArgs),
    /// Delete a custom or cloned voice
    Delete {
        /// The voice id shown by `vtt voice list`, e.g. af_my-warm or `clone_anna`
        id: String,
    },
}

pub fn run(command: VoiceCommand, paths: &Paths, out: Output) -> Result<()> {
    let catalog = Catalog::bundled()?;
    let entry = catalog
        .find(SPEECH_MODEL_ID)
        .ok_or_else(|| Error::ModelMissing(SPEECH_MODEL_ID.into()))?;
    let shipped = ModelStore::new(paths).model_dir(entry);
    let presets = paths.voices_dir();
    match command {
        VoiceCommand::List => {
            let mut voices = voice_list(&shipped, &presets);
            voices.extend(crate::clone_cmd::listed(paths));
            if out.json {
                println!("{}", serde_json::to_string(&voices)?);
            }
            if !out.json {
                print_voices(&voices);
            }
            Ok(())
        }
        VoiceCommand::Create { name, blend } => {
            if !shipped.is_dir() {
                return Err(Error::ModelMissing(SPEECH_MODEL_ID.into()));
            }
            let preset = Preset::new(&name, &parse_blend(&blend)?)?;
            save_preset(&presets, &shipped, &preset)?;
            if out.json {
                println!("{}", serde_json::to_string(&preset)?);
            } else {
                println!("{}", preset.id);
                say(
                    out,
                    &format!(
                        "created the voice {}; use it with: vtt speak --voice {}",
                        preset.name, preset.id
                    ),
                );
            }
            Ok(())
        }
        VoiceCommand::Clone(args) => crate::clone_cmd::clone_voice(&args, paths, out),
        VoiceCommand::Delete { id } if vtt_core::tts::clone::is_clone_id(&id) => {
            vtt_core::tts::clone::delete(&vtt_core::tts::clone::clones_dir(&presets), &id)?;
            say(out, &format!("deleted {id}"));
            Ok(())
        }
        VoiceCommand::Delete { id } => {
            delete_preset(&presets, &id)?;
            say(out, &format!("deleted {id}"));
            Ok(())
        }
    }
}

/// One line per voice; custom voices are marked.
pub fn print_voices(voices: &[VoiceInfo]) {
    for voice in voices {
        let mark = match (voice.cloned, voice.custom) {
            (true, _) => " [cloned]",
            (false, true) => " [custom]",
            (false, false) => "",
        };
        println!(
            "{:<20} {} ({}){mark}",
            voice.id, voice.name, voice.description
        );
    }
}

fn say(out: Output, message: &str) {
    if out.show_messages() {
        eprintln!("{message}");
    }
}

/// `af_heart:0.6,af_bella:0.4` as voice and weight pairs.
fn parse_blend(text: &str) -> Result<Vec<(String, f32)>> {
    text.split(',')
        .map(|item| {
            let bad = || {
                Error::Input(format!(
                    "cannot read '{item}'; write voice:weight, e.g. af_heart:0.6"
                ))
            };
            let (voice, weight) = item.trim().split_once(':').ok_or_else(bad)?;
            let weight: f32 = weight.trim().parse().map_err(|_| bad())?;
            Ok((voice.trim().to_string(), weight))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blend_is_voice_colon_weight_pairs() {
        let parts = parse_blend("af_heart:0.6, af_bella:0.4").unwrap();
        assert_eq!(
            parts,
            [("af_heart".to_string(), 0.6), ("af_bella".to_string(), 0.4)]
        );
        assert!(parse_blend("af_heart").is_err());
        assert!(parse_blend("af_heart:loud").is_err());
    }
}
