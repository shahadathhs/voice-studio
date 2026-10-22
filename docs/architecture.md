# Architecture

One Rust core, three front ends. See [product/features.md](product/features.md) for what the app does and
[guidelines.md](guidelines.md) for the code rules.

## Overview

```
┌────────────┐   ┌────────────┐   ┌────────────┐
│ Tauri GUI  │   │ vtt (CLI)  │   │ vtt serve  │
│ (web UI)   │   │            │   │ (optional) │
└─────┬──────┘   └─────┬──────┘   └─────┬──────┘
      │ Tauri commands + events (in-process)
      └─────────┬──────┴────────────────┘
        ┌───────▼────────────────────────────────┐
        │ core (Rust crate)                      │
        │  engines: stt · tts · diarize · vad    │
        │  audio: decode (symphonia), I/O (cpal) │
        │  jobs: queue, progress, cancel, resume │
        │  models: catalog, download, verify     │
        │  store: SQLite (+FTS5), settings       │
        │  export: txt srt vtt json docx md      │
        └────────────────────────────────────────┘
```

No sidecar process and no localhost port in the desktop app: the UI calls Rust directly. `vtt serve` is a separate, opt-in front end.

**Repository layout**

```
Cargo.toml          Rust workspace (core, cli, desktop/src-tauri)
crates/core         vtt-core: types, errors, exporters, engine traits, model catalog, paths, job events
crates/cli          vtt-cli: the `vtt` binary (clap) and its black-box tests
desktop/            Tauri 2 app
  src/                Svelte 5 + TypeScript UI (components, pages, lib, styles)
  src-tauri/          Rust shell: commands that call vtt-core in-process
catalog/            models.json, compiled into the binary
docs/               product, architecture, design (mockups), guidelines
```

**How the desktop app talks to the core.** The UI calls Tauri commands (`invoke`), which are thin
functions in `desktop/src-tauri/src/commands.rs` that call `vtt-core` directly in the same process.
Types crossing the boundary are plain serde structs mirrored in `desktop/src/lib/types.ts`. When the
UI is opened in a plain browser (`pnpm dev`), `desktop/src/lib/api.ts` returns harmless stand-in
data so the interface can be developed without the Rust side.

**Front-end structure.** `components/` are small, reusable and presentational; `pages/` compose them;
`lib/` holds state stores (theme, navigation, app info) and typed wrappers; `styles/tokens.css` holds
the colour tokens shared with the mockups. Logic that is not about presentation belongs in Rust.

**Technology choices**

| Concern        | Choice                                                                                                                         | Why                                                                                     |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| Shell          | Tauri 2                                                                                                                        | Native Rust backend, small binaries, signed installers, updater, tray, global shortcuts |
| UI             | Svelte 5 + TypeScript + Vite                                                                                                   | Small runtime, plain CSS tokens as in the mockups, no UI kit                            |
| Speech to text | whisper.cpp or sherpa-onnx (Whisper models)                                                                                    | Prebuilt, fast, Metal/Vulkan/CUDA capable                                               |
| Text to speech | Kokoro-82M, Piper voices via ONNX Runtime                                                                                      | Small, CPU-capable, permissive licenses (verify per voice)                              |
| Diarization    | Speaker embeddings + agglomerative clustering + temporal smoothing, ported from the earlier prototype (git tag `python-final`) | Proven heuristics; models via ONNX                                                      |
| VAD            | Silero VAD (ONNX)                                                                                                              | Skips silence and prevents Whisper hallucinating on it                                  |
| Audio          | `symphonia` (decode), `cpal` (devices), `rubato` (resample)                                                                    | No ffmpeg dependency                                                                    |
| Storage        | SQLite via `rusqlite`, FTS5 for search                                                                                         | One file, searchable, easy backup                                                       |
| Updates        | Tauri updater against GitHub Releases                                                                                          | Signed, optional                                                                        |

## Data model (sketch)

- `item(id, kind[transcript|speech], title, created_at, duration_ms, source_path, model_id, language, settings_json)`
- `segment(item_id, idx, start_ms, end_ms, speaker_id, text_original, text_edited)` and `word(segment_id, start_ms, end_ms, text, confidence)`
- `speaker(item_id, id, display_name, color)`
- `model(id, kind, version, path, sha256, installed_at)`
- `job(id, item_id, state, progress, checkpoint_json, error)`
- FTS5 virtual table over `segment.text_*`.
