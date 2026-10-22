# Voice Studio

Offline speech-to-text and text-to-speech for the desktop and the command line.
Cross-platform (Windows, macOS, Linux), open source, and private: audio and text never leave your computer.

> **Status: early development.** The core library, a first slice of the CLI and the desktop app shell
> exist. Transcription and speech synthesis are being built in phases. Nothing is released yet, and
> "Voice Studio" is a working name.

## What it will do

- **Speech to text:** transcribe audio and video, record from the microphone, identify speakers,
  translate to English, export to TXT, SRT, VTT, JSON, Markdown and DOCX.
- **Text to speech:** natural offline voices with speed control, saved as WAV, MP3 or FLAC.
- **Desktop app:** native Rust engine with a clean interface, light/dark/system themes, a searchable
  library and a model manager.
- **Command line:** the same engine as the app, scriptable, with machine-readable output.

Everything is documented in [docs/](docs/README.md): [features](docs/product/features.md),
[CLI design](docs/product/cli.md), [roadmap](docs/product/roadmap.md),
[architecture](docs/architecture.md) and [UI mockups](docs/design/README.md).

## What works today

```
vtt export transcript.json -f srt       # convert a transcript to txt, srt, vtt, md or json
vtt models list                         # the model catalog and what is installed
vtt models pull whisper-base            # download a model (resumable, checksum-verified)
vtt models remove whisper-base          # delete it again
vtt transcribe talk.m4a                 # transcribe audio or video (prints the text)
vtt transcribe talk.m4a -f srt -o talk.srt   # subtitles; also txt, vtt, md, json
vtt config path                         # where data and config live
vtt doctor                              # basic environment check
vtt completions zsh                     # shell completions
```

`transcribe` runs Whisper locally (GPU on macOS) on WAV, MP3, M4A/AAC, FLAC, OGG and MKV/MP4/MOV audio.
Speaker identification, `speak`, `record`, `library` and `devices` are defined but not implemented yet; they exit
with code 70 and a clear message. The desktop app opens with navigation, theming (system, light, dark),
a **working Models page** (install, cancel, remove, with live download progress) and Settings. The
transcription screens are present but disabled until the engine lands.

## Build from source

Requirements: [Rust](https://rustup.rs) (stable), a C++ compiler and [CMake](https://cmake.org) (whisper.cpp is built
from source), Node.js 22+, [pnpm](https://pnpm.io) and `make`. On Linux, also the
[Tauri system packages](https://tauri.app/start/prerequisites/).

```bash
make setup                        # Rust + CMake if missing, JS dependencies, git hooks, first UI build
make dev                          # desktop app with hot reload
make cli ARGS="export t.json -f srt"   # the CLI
make check                        # everything CI runs
```

GPU: macOS uses Metal automatically. On Windows and Linux the default build runs on the CPU; build with
`cargo build --release -p vtt-cli --features vulkan` (needs the Vulkan SDK) or `--features cuda` (needs the CUDA
toolkit) for a GPU build. `vtt-desktop` forwards the same features.

`make` lists every target. The package manager is [pnpm](https://pnpm.io). On Windows use Git Bash or
WSL, or run the underlying commands shown in the [Makefile](Makefile). More in [CONTRIBUTING](CONTRIBUTING.md).

## Repository layout

```
crates/core      vtt-core: types, errors, exporters, engine traits, model catalog, paths
crates/cli       vtt-cli: the `vtt` binary and its black-box tests
desktop/         Tauri 2 app: Svelte UI in src/, Rust shell in src-tauri/
catalog/         model catalog (compiled into the binary)
docs/            product docs, architecture, design mockups, code guidelines
```

The earlier Python/FastAPI prototype is preserved at the git tag `python-final`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/guidelines.md](docs/guidelines.md).
Please read the [Code of Conduct](CODE_OF_CONDUCT.md) and the [Security Policy](SECURITY.md).

## License

[MIT](LICENSE)
