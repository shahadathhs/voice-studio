# Roadmap

Status: early development. Phase 0 is complete.

## Phases

Each engine phase ships its CLI command and its desktop screen together, so the CLI and the app never drift.

0. **Foundations (done):** Rust workspace; shared types, exporters and errors; `vtt` CLI skeleton
   (`export`, `models list`, `config`, `doctor`, `completions`); Tauri 2 + Svelte shell with navigation and
   light/dark/system theming; lint, format and test tooling; CI on Linux, macOS and Windows.
1. **Models (mostly done):** catalog of seven Whisper models with pinned URLs, sizes and SHA-256;
   downloader with resume, cancel and checksum verification; `vtt models list|info|pull|remove|verify`;
   working Models screen. Still to do: first-run onboarding, TTS and diarization catalog entries.
2. **Transcribe:** audio decoding, Whisper STT, VAD, job queue, SQLite store, `vtt transcribe`;
   Transcribe screen with queue, transcript viewer, Library and Export dialog.
3. **Speaker identification and translation:** diarization pipeline and `--diarize`, `--translate`;
   speaker panel in the viewer. Evaluated on a small labelled set.
4. **Text to speech:** Kokoro and Piper, streaming sentence synthesis, `vtt speak`; Speak screen.
5. **Recording and live captions:** `vtt record`; Record screen.
6. **Tray, dictation, read-aloud:** global shortcuts and the floating dictation pill.
7. **Packaging:** installers for all three platforms, signing, updater, first release.

## Risks and open questions

1. **Global dictation is the hardest feature.** Needs microphone and accessibility permissions on macOS; Windows is straightforward; **Linux/Wayland restricts global hotkeys and synthetic input**, so it may be X11-only or clipboard-paste-only there. Plan: ship file transcription, recording and TTS first; dictation as a later phase with documented platform limits.
2. **Diarization quality** depends on the ONNX embedding model and clustering; expect it to match, not beat, the current Python version. Needs a small evaluation set with known speakers.
3. **Live captions** need a streaming-capable model; accuracy will be below the post-recording pass. Label as draft.
4. **Model licenses.** Whisper is MIT. Verify the exact license of each Kokoro/Piper voice and each diarization model before listing it in the catalog. Do not ship non-commercial models.
5. **Signing and distribution.** macOS notarization (Apple Developer account) and Windows code signing cost money; unsigned builds trigger OS warnings. Decide before the first public release.
6. **Whisper hallucination and long files.** Need VAD segmentation, no-speech thresholds and chunk stitching; a 3-hour file must not hold everything in memory.
7. **GPU matrix.** Metal (mac), Vulkan (Win/Linux), CUDA (optional). Needs CI hardware or manual testing for each.
8. **Naming/branding.** "Voice Studio" is a placeholder; check trademark conflicts before release.
9. **Accessibility.** Keyboard navigation, screen reader labels, contrast; planned from the start, tested per release.
10. **Localization.** English first; strings externalised from day one.
