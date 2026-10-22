# Roadmap

Status: early development. Phase 0 is complete; phases 1 and 2 are partly done.

## Phases

Each engine phase ships its CLI command and its desktop screen together, so the CLI and the app never drift.

0. **Foundations (done):** Rust workspace; shared types, exporters and errors; `vtt` CLI skeleton
   (`export`, `models list`, `config`, `doctor`, `completions`); Tauri 2 + Svelte shell with navigation and
   light/dark/system theming; lint, format and test tooling; CI on Linux, macOS and Windows.
1. **Models (mostly done):** catalog of seven Whisper models with pinned URLs, sizes and SHA-256;
   downloader with resume, cancel and checksum verification; `vtt models list|info|pull|remove|verify`;
   working Models screen. Still to do: first-run onboarding, TTS and diarization catalog entries.
2. **Transcribe (CLI done, app next):** audio and video decoding with streaming resampling,
   whisper.cpp through `whisper-rs` (Metal GPU on macOS), cancel and progress, `vtt transcribe` with
   txt, srt, vtt, md and json output. Still to do: job queue and SQLite library, the Transcribe,
   transcript viewer and Export screens in the app, Windows and Linux GPU backends, VAD.
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
6. **Whisper repetition loops and long files.** Measured on this machine: natural speech with pauses transcribes
   correctly (54 s in about 1 s on the GPU, identical text on CPU), but audio that repeats the same paragraph
   with no pauses sends greedy decoding into a loop ("we have a lot of information about this" repeated).
   Turning off previous-window prompting did not help. Candidates: voice-activity detection to cut silence and
   re-segment, beam search, and repetition detection with a retry at a higher temperature. Audio is resampled in
   a streaming fashion, but the 16 kHz samples are still held in memory (a 3-hour file is about 700 MB), so very
   long files need chunked inference.
7. **GPU matrix.** Metal (mac), Vulkan (Win/Linux), CUDA (optional). Needs CI hardware or manual testing for each.
8. **Naming/branding.** "Voice Studio" is a placeholder; check trademark conflicts before release.
9. **Accessibility.** Keyboard navigation, screen reader labels, contrast; planned from the start, tested per release.
10. **Localization.** English first; strings externalised from day one.
