# Roadmap

Status: early development. Phases 0, 2, 3 and 4 are complete; phase 1 is mostly done.

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
5. **Recording and live captions:** `vtt record`; Record screen.
6. **Tray, dictation, read-aloud:** global shortcuts and the floating dictation pill.
7. **Packaging:** installers for all three platforms, signing, updater, first release.

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
8. **Naming/branding.** "Voice Studio" is a placeholder; check trademark conflicts before release.
9. **Accessibility.** Keyboard navigation, screen reader labels, contrast; planned from the start, tested per release.
10. **Localization.** English first; strings externalised from day one.
