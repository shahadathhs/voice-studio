# Roadmap

Status: early development. Phases 0 to 4 are complete. The plan for phases 5 to 11 is below; the order and the
gates are decided, the details of each phase are settled when it starts.

## Phases

Each engine phase ships its CLI command and its desktop screen together, so the CLI and the app never drift.

0. **Foundations (done):** Rust workspace; shared types, exporters and errors; `vtt` CLI skeleton
   (`export`, `models list`, `config`, `doctor`, `completions`); Tauri 2 + Svelte shell with navigation and
   light/dark/system theming; lint, format and test tooling; CI on Linux, macOS and Windows.
1. **Models (mostly done):** catalog of seven Whisper models with pinned URLs, sizes and SHA-256;
   downloader with resume, cancel and checksum verification; `vtt models list|info|pull|remove|verify`;
   working Models screen. Still to do: first-run onboarding, TTS and diarization catalog entries.
2. **Transcribe (done):** audio and video decoding with streaming resampling, whisper.cpp through
   `whisper-rs` (Metal GPU on macOS; Vulkan and CUDA as opt-in build features), cancel and progress,
   energy-based voice-activity detection that transcribes only the speech in chunks, `vtt transcribe` with
   txt, srt, vtt, md and json output; a sequential job queue (cancel, reorder, pause) and the SQLite
   library with full-text search (`vtt library`); the Transcribe, Library, transcript viewer and Export
   screens in the app. Transcript lines can be corrected and moved to another speaker in the viewer and with `vtt library edit`. The app's queue is saved to `queue.json` in the data folder, so unfinished files resume after a quit or a crash (a file that was started twice without finishing is shown as failed rather than retried forever). The viewer draws the recording as a waveform you can click or arrow through to jump. Not yet:
   parallel jobs, a neural VAD (Silero), and Vulkan/CUDA builds in CI.
3. **Speaker identification and translation (done):** `vtt transcribe --diarize` with pyannote segmentation and a
   WeSpeaker ResNet34 voice model on ONNX Runtime: overlap-aware segmentation, voiceprints, average-linkage
   clustering with an automatic speaker count, and voting onto the transcript lines; `--translate` refuses the turbo
   models, which cannot translate. The app has an "Identify speakers" option, a speaker panel with talk-time shares,
   and rename or merge of speakers. Evaluated on synthetic voices and one noisy real recording (see
   [diarization.md](diarization.md)). Not yet: a set of real labelled recordings,
   recolouring speakers.
4. **Text to speech (done):** Kokoro-82M through ONNX Runtime, English text spoken sentence by
   sentence with pauses, pronunciation from the misaki dictionaries in pure Rust, six American voices,
   `vtt speak`. Piper is not included (needs GPL espeak-ng and per-voice licence review; see
   [speech.md](speech.md)). The Speak screen has a text box (or a text file), a voice list, speed, progress per sentence, playback
   and Save audio as WAV or FLAC (`--format flac` in the CLI). Still to do: playing audio from the CLI, mp3 output (needs an
   LGPL encoder), keeping speech in the library, British voices, a waveform in the player.
5. **Prove it on all three systems (next):** push `dev` and get CI green on Linux, macOS and Windows, including the
   Windows and Linux builds nobody has run yet; build installers once as CI artifacts (not published) and run the
   app on a real Windows and a real Linux machine; write the third-party notices file (see
   [Legal and safety](#legal-and-safety)). Exit: the app starts, installs a model, transcribes and speaks on each
   system. Why first: every later phase adds weight to code that has only run on macOS.
6. **Language models: summaries and translation (built; Bengali quality open).** A `TextEngine` trait and a Qwen 2.5
   engine on `candle` (Metal on Macs, CPU elsewhere) reading GGUF files, with two optional models: 1.5B and 7B (both
   Apache-2.0). The 3B model is excluded: it uses the Qwen Research licence. Not `llama.cpp`: it cannot share a binary
   with `whisper-rs` (both bundle ggml; measured, it aborts). `vtt summarize`, `vtt translate --to <lang>`, and a
   Summary and translate dialog on the transcript screen (a translation is stored as a new library item). Long
   transcripts are split, summarised in parts and merged. Measured on a short meeting: Spanish and Chinese readable;
   **Bengali is poor with the 1.5B model and still flawed with 7B**, so it is warned about in the app. Chapters (titles with the time each part starts, which
   seek the player) and action items are written the same way, and summaries, chapters and action items are kept with
   the transcript (`vtt summarize --chapters --actions`, the app's dialog). Key points of a summary do not link to
   timestamps: a small model cannot cite lines reliably, so only chapters do. Still to do: a native speaker's judgement
   of Chinese and Bengali (exit criterion) and a decision on a stronger Bengali model.
7. **Speech in more languages (built for seven; Japanese and Bengali open).** Pronunciation is written in Rust per
   language (no espeak-ng: GPL), and Kokoro's own voices are used: British English, Spanish, Italian, Brazilian
   Portuguese, French, Hindi and Mandarin Chinese (pinyin word lists under MIT), 35 voices in all. "Translate, then
   speak" works with `vtt speak --translate-to` and a checkbox on the Speak screen; the voice list is grouped by
   language. Each language was checked by having Whisper listen (see [speech.md](speech.md)), **not by a native
   speaker, which remains the exit test for each language**. Still open: Japanese (needs a kanji dictionary
   decision), and **Bengali, a research gate, not a build task:** find a Bengali voice that is permissively licensed
   (candidates: Indic Parler-TTS, AI4Bharat Indic-TTS, Thiva-indic; licences conflict between sources, read the
   licence files) and exportable to ONNX; Meta MMS is non-commercial and excluded. If none qualifies, Bengali stays
   text-only (translation and summaries) and we say so.
8. **Custom voices and voice cloning (must have).** Two steps. First, voice presets: saved blends of the shipped
   voices, with a name and a preview. Second, cloning from a short sample: model chosen by a spike that compares
   OpenVoice v2 and Chatterbox (MIT as far as known: verify), checks ONNX export, speed and quality; a decision gate
   before any build. Rules that are part of the feature, not extras: the person must confirm consent for the voice
   in the sample; everything runs locally; cloned voices are marked as such and a disclosure is added to saved
   audio (and a watermark if a permissive one exists); no third-party or celebrity voice is ever bundled; a cloned
   voice can be deleted. `vtt voice create|list|delete`, a Voices screen.
9. **Recording and live captions:** `vtt record`; Record screen; live captions with a streaming model, labelled
   draft.
10. **Tray, dictation, read-aloud:** global shortcuts and the floating dictation pill, with the documented limits on
    Linux/Wayland.
11. **Packaging and first release:** installers for all three platforms, signing and notarization, updater, first
    public release (the release workflow and version script already exist).

Each of phases 6 to 8 ends with the same checks: CLI and app ship together, tests that need no model, a documented
evaluation, every new model entered in the licence register, and the interface text translated (English and
Spanish at least).

## Legal and safety

These are conditions for shipping, checked at the end of every phase.

- **Licence register.** Every model in `catalog/models.json` records its licence, where it was verified and when. Only
  permissive licences (MIT, Apache-2.0, BSD, CC BY with attribution). Non-commercial, research-only and "custom"
  licences are excluded, including NLLB, XTTS and Meta MMS. A model whose licence cannot be confirmed is not listed.
- **Training data.** Some models say "follows the licence of the dataset" (WeSpeaker, VoxCeleb); review the dataset
  terms before the first release.
- **Third-party notices.** A generated notices file with every dependency and model, shown in the app and shipped in
  installers, satisfying the attribution that Apache-2.0, MIT and CC BY require.
- **Consent and disclosure** for cloned voices as described in phase 8; no feature that impersonates a real person.
- **Privacy.** Audio and text stay on the device; the only network use is downloading models, which is visible.
- **Name.** "Voice Studio" is kept; run a trademark search before the first public release.

## Risks and open questions

1. **Global dictation is the hardest feature.** Needs microphone and accessibility permissions on macOS; Windows is straightforward; **Linux/Wayland restricts global hotkeys and synthetic input**, so it may be X11-only or clipboard-paste-only there. Plan: ship file transcription, recording and TTS first; dictation as a later phase with documented platform limits.
2. **Diarization quality** depends on the embedding model and clustering. The shipped model scores perfectly on a small set of synthetic voices, which says little about real meetings (overlap, noise, similar voices). Needs a set of real recordings with known speakers; until then `--speakers N` is the reliable setting.
3. **Live captions** need a streaming-capable model; accuracy will be below the post-recording pass. Label as draft.
4. **Model licenses.** Whisper is MIT. The speaker model is Apache-2.0 on its card, but its VoxCeleb training data is CC BY 4.0 and was not reviewed. Verify the exact license of each Kokoro/Piper voice and each diarization model before listing it in the catalog. Do not ship non-commercial models.
5. **Signing and distribution.** macOS notarization (Apple Developer account) and Windows code signing cost money; unsigned builds trigger OS warnings. Decide before the first public release.
6. **Whisper repetition loops and long files.** Audio that repeats the same paragraph with no pauses sends greedy
   decoding into a loop ("we have a lot of information about this" repeated). Mitigations: the energy-based VAD
   transcribes speech in chunks of at most 28 s, which bounds a loop to one chunk; a loop (three identical lines in
   a row, or a phrase repeated four times inside a line) now triggers one retry of that chunk with some
   randomness, and any loop left is folded into one occurrence (`repetition.rs`). Not measured on real looping
   audio yet. Audio is resampled in a streaming fashion, but the 16 kHz samples are still held in memory (a
   3-hour file is about 700 MB), so very long files need chunked inference.
7. **GPU matrix.** Metal (mac), Vulkan (Win/Linux), CUDA (optional). Needs CI hardware or manual testing for each.
8. **Naming/branding.** "Voice Studio" is the name; run a trademark search before the first public release.
9. **Accessibility.** Keyboard navigation, screen reader labels, contrast; planned from the start, tested per release.
10. **Localization.** Done for the interface (English and Spanish, see [localization.md](../localization.md));
    text that comes from Rust (error messages, voice descriptions, model names) is still English.
11. **The Python ecosystem gap.** Many speech and language models and their text-processing code are Python-first.
    Rule: models are exported once to ONNX (or GGUF for language models) on a developer machine and run in Rust;
    Python never ships. The cost is slower work per language and model, so languages are prioritised, not all
    promised. An optional external-engine interface is held back as an escape hatch, only if cloning needs it.
12. **Small language models can be wrong.** Summaries and translations can contain errors or invented details.
    They are labelled AI-generated, link back to the transcript, and are never written over the transcript itself.
13. **Native review.** Spanish interface text and any non-English speech or translation quality need a native
    speaker's review before a release claims support for the language.
