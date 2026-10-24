<div align="center">

# Voice Studio

**Turn speech into text, and text into speech, without sending anything anywhere.**

An offline desktop app and command-line tool for Windows, macOS and Linux.
No account, no cloud, no per-minute fees.

[![CI](https://github.com/shahadathhs/voice-to-text/actions/workflows/ci.yml/badge.svg)](https://github.com/shahadathhs/voice-to-text/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/shahadathhs/voice-to-text?include_prereleases&sort=semver)](https://github.com/shahadathhs/voice-to-text/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[Download](https://github.com/shahadathhs/voice-to-text/releases/latest) ·
[Documentation](docs/README.md) · [CLI reference](docs/product/cli.md) ·
[Roadmap](docs/product/roadmap.md) · [Report a bug](https://github.com/shahadathhs/voice-to-text/issues/new/choose)

</div>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/transcript-dark.png">
  <img alt="A transcript with three speakers, timestamps and a speaker panel" src="docs/screenshots/transcript-light.png">
</picture>

_Screenshots show the real interface with made-up sample data._

> [!NOTE]
> **Early development.** Transcription, speaker labels and text to speech work. Recording, live captions and
> dictation are still to come (see the [roadmap](docs/product/roadmap.md)). Installers are not code-signed yet.
> "Voice Studio" is a working name.

## Table of contents

- [About](#about)
- [Features](#features)
- [Private by design](#private-by-design)
- [Screens](#screens)
- [Installation](#installation)
- [Usage](#usage)
- [How it works](#how-it-works)
- [Roadmap](#roadmap)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [Security](#security)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## About

Voice Studio transcribes recordings, labels who said what, and reads text aloud, all on your own computer.
Models download once; after that it works offline. The desktop app and the `vtt` command-line tool share
one engine, so what you script is what you click.

## Features

- **Transcribe audio and video.** Drop in WAV, MP3, M4A, FLAC, OGG, MP4, MOV or MKV files. A queue works
  through them one at a time, skipping silence, with progress you can pause, reorder or cancel.
- **Know who said what.** Turn on _Identify speakers_ to label each line, see who talked how much, and
  rename or merge speakers afterwards.
- **Search everything you have transcribed.** Every transcript goes into a library with full-text search.
  Click a timestamp to hear that moment.
- **Export anywhere.** TXT, SRT and VTT subtitles, Markdown, JSON and Word documents, with control over line width,
  timestamps and speaker names.
- **Read text aloud.** Paste text or open a file, pick one of six natural English voices, and play or save it.
- **Choose your models.** A model manager shows what each model is for, how big it is, and its licence.
- **Light, dark or system theme.**

## Private by design

Audio and text never leave your computer. The only network use is downloading models, which you start
yourself, and each download is checked against a published checksum. The app has no telemetry.

## Screens

<table>
  <tr>
    <td width="50%">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/transcribe-dark.png">
        <img alt="The transcription queue and recent transcripts" src="docs/screenshots/transcribe-light.png">
      </picture>
      <br><sub><b>Transcribe:</b> a queue for many files.</sub>
    </td>
    <td width="50%">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/models-dark.png">
        <img alt="Models grouped by purpose, with sizes and licences" src="docs/screenshots/models-light.png">
      </picture>
      <br><sub><b>Models:</b> grouped by what they are for.</sub>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/speak-dark.png">
        <img alt="Text to speech with a voice list and speed control" src="docs/screenshots/speak-light.png">
      </picture>
      <br><sub><b>Speak:</b> text to speech with six voices.</sub>
    </td>
    <td width="50%">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/transcript-light.png">
        <img alt="A transcript in the light theme" src="docs/screenshots/transcript-light.png">
      </picture>
      <br><sub><b>Transcript:</b> speakers, timestamps, playback and export.</sub>
    </td>
  </tr>
</table>

## Installation

### Download

Get the installer for your system from the [latest release](https://github.com/shahadathhs/voice-to-text/releases/latest).
Every file comes with a `.sha256` checksum, and each release also ships the `vtt` command-line binary.

Builds are not signed yet, so the first launch shows a warning: on macOS right-click the app and choose
**Open**; on Windows choose **More info**, then **Run anyway**.

### Build from source

```bash
git clone https://github.com/shahadathhs/voice-to-text.git
cd voice-to-text
make setup     # Rust and CMake if missing, JS dependencies, git hooks, first UI build
make dev       # run the desktop app with hot reload
make check     # everything CI runs
```

You need [Rust](https://rustup.rs), a C++ compiler and [CMake](https://cmake.org) (whisper.cpp is built
from source), Node.js 22+, [pnpm](https://pnpm.io) and `make`. On Linux, also the
[Tauri system packages](https://tauri.app/start/prerequisites/) and the ALSA headers (`libasound2-dev`) for
microphone recording. On Windows use Git Bash or WSL. `make` lists every target.

GPU on Windows and Linux: `cargo build --release -p vtt-cli --features vulkan` (needs the Vulkan SDK) or
`--features cuda` (needs the CUDA toolkit). The desktop crate forwards the same features.

## Usage

### Command line

The same engine as the app, for scripts and servers.

```bash
vtt models pull whisper-small                  # download a model (resumable, checksum-verified)
vtt transcribe talk.m4a                        # print the text
vtt transcribe talk.m4a -f srt -o talk.srt     # subtitles; also txt, vtt, md, json
vtt transcribe meeting.mp4 --diarize           # label the speakers
vtt speak "Hello from my own computer" -o hello.wav
vtt library search "roadmap"                   # search everything you have transcribed
vtt --json transcribe talk.m4a                 # machine-readable events for scripts
```

Run `vtt --help` or see the [CLI reference](docs/product/cli.md) for every command, option and exit code.
Whisper uses the GPU on macOS (Metal) automatically; on Windows and Linux it uses the CPU unless you build
with the optional Vulkan or CUDA feature (see [Installation](#installation)).

## How it works

A Rust core (`vtt-core`) does the work: whisper.cpp for transcription, pyannote and WeSpeaker models for speaker
labels, Kokoro for speech, SQLite for the library. The desktop app is a [Tauri 2](https://tauri.app)
shell with a Svelte interface calling the core in-process, so there is no server and no sidecar. The CLI
is a thin front end on the same core. See [architecture](docs/architecture.md).

## Roadmap

Recording, live captions, dictation, signed installers and an updater are planned, along with a Bengali voice
and voice cloning (both still research gates). See the [roadmap](docs/product/roadmap.md) for status.

## Documentation

[Features](docs/product/features.md) · [CLI](docs/product/cli.md) · [Roadmap](docs/product/roadmap.md) ·
[Architecture](docs/architecture.md) · [Design](docs/design/README.md) · [Website](docs/site.md) ·
[Releasing](docs/releasing.md) · [Index](docs/README.md)

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) and the [code guidelines](docs/guidelines.md)
first; work happens on the `dev` branch and commits follow
[Conventional Commits](https://www.conventionalcommits.org). Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Report vulnerabilities privately; see the [Security Policy](SECURITY.md).

## Acknowledgements

Voice Studio stands on [whisper.cpp](https://github.com/ggml-org/whisper.cpp) (transcription),
[pyannote](https://github.com/pyannote/pyannote-audio) and [WeSpeaker](https://github.com/wenet-e2e/wespeaker)
(speaker labels), [Kokoro](https://github.com/hexgrad/kokoro) (speech), [Tauri](https://tauri.app) and
[Svelte](https://svelte.dev). Each model's licence is listed in the app and in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## License

[MIT](LICENSE)
