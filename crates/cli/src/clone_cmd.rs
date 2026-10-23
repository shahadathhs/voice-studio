//! `vtt voice clone`, and `vtt speak` with a cloned voice.

use std::path::PathBuf;
use std::time::Instant;

use clap::Args;
use vtt_core::audio::load_at_rate;
use vtt_core::engines::Control;
use vtt_core::jobs::Event;
use vtt_core::models::{Catalog, ModelStore};
use vtt_core::paths::Paths;
use vtt_core::tts::VoiceInfo;
use vtt_core::tts::clone::{
    self, CloneEngine, Consent, DISCLOSURE, MODEL_ID, SAMPLE_RATE, clones_dir,
};
use vtt_core::{Error, Result};

use crate::cli::Speak;
use crate::output::Output;
use crate::speak_cmd;

#[derive(Args, Debug)]
pub struct CloneArgs {
    /// A name for the voice (1 to 40 characters)
    pub name: String,
    /// A recording of the speaker: 3 to 30 seconds of clear speech by one person
    #[arg(long, value_name = "AUDIO")]
    pub from: PathBuf,
    /// Required. States: "I own this voice or have the speaker's permission"
    #[arg(long, required = true)]
    pub i_have_permission: bool,
}

/// The cloned voices as `vtt voice list` shows them.
pub fn listed(paths: &Paths) -> Vec<VoiceInfo> {
    clone::list(&clones_dir(&paths.voices_dir()))
        .iter()
        .map(clone::VoiceMeta::info)
        .collect()
}

/// Loads the Chatterbox model, or says how to install it.
fn load_engine(paths: &Paths) -> Result<CloneEngine> {
    let catalog = Catalog::bundled()?;
    let entry = catalog
        .find(MODEL_ID)
        .ok_or_else(|| Error::ModelMissing(MODEL_ID.into()))?;
    let store = ModelStore::new(paths);
    if !store.is_installed(entry) {
        return Err(Error::ModelMissing(MODEL_ID.into()));
    }
    CloneEngine::load(&store.model_dir(entry))
}

/// Makes a cloned voice from `--from` and saves it.
pub fn clone_voice(args: &CloneArgs, paths: &Paths, out: Output) -> Result<()> {
    let consent = Consent::confirm(args.i_have_permission)?;
    // A name and a file that cannot work fail before the large model is loaded.
    let id = clone::id_for(&args.name)?;
    if !args.from.is_file() {
        return Err(Error::Input(format!(
            "cannot read audio from {}: no such file",
            args.from.display()
        )));
    }
    let dir = clones_dir(&paths.voices_dir());
    if dir.join(format!("{id}.bin")).exists() {
        return Err(Error::Input(format!(
            "a voice called {id} exists; delete it first"
        )));
    }
    let mut engine = load_engine(paths)?;
    let audio = load_at_rate(&args.from, SAMPLE_RATE)?;
    let voice = engine.voice_from_audio(&audio, &args.name, consent)?;
    clone::save(&dir, &voice)?;
    if out.json {
        println!("{}", serde_json::to_string(&voice.meta)?);
    } else {
        println!("{}", voice.meta.id);
        if out.show_messages() {
            eprintln!(
                "cloned the voice {}; use it with: vtt speak --voice {} --language <code>",
                voice.meta.name, voice.meta.id
            );
        }
    }
    Ok(())
}

/// Speaks `text` in the cloned voice `id`. Files get a note saying the voice is synthetic.
pub fn speak(id: &str, text: &str, args: &Speak, paths: &Paths, out: Output) -> Result<()> {
    let voice = clone::load(&clones_dir(&paths.voices_dir()), id)?;
    let language = args
        .language
        .as_deref()
        .or(args.translate_to.as_deref())
        .unwrap_or("en");
    let mut engine = load_engine(paths)?;
    let started = Instant::now();
    let show_bar = out.show_messages() && std::io::IsTerminal::is_terminal(&std::io::stderr());
    let audio = engine.speak(&voice, text, language, &mut |done, total| {
        if show_bar {
            eprint!("\r  speaking {done}/{total}   ");
        }
        Control::Continue
    });
    if show_bar {
        eprintln!();
    }
    let format = speak_cmd::output_format(args)?;
    let bytes = format.encode_with_note(&audio?, Some(DISCLOSURE))?;
    let written = speak_cmd::write(&bytes, format, args.output.as_deref())?;
    if out.json {
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let event = Event::Done {
            output: written,
            elapsed_ms,
        };
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}
