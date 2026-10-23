# `vtt` — command-line plan

The CLI is a first-class front end, not a debug tool. It links the same Rust `core` crate as the desktop app, so a transcript made on the command line is identical to one made in the GUI and shows up in the same Library (shared data directory).

Binary name: `vtt` (the Python package already exposes `vtt` and `voice-to-text`; both names are kept).

## Design rules

1. **stdout is data, stderr is everything else.** Results go to stdout (or `-o`); progress, logs and prompts go to stderr, so pipes work.
2. **`--json` makes it scriptable.** Results as one JSON document; long jobs emit newline-delimited JSON events (`progress`, `segment`, `done`, `error`).
3. **No surprises on non-interactive use.** Prompts only when stdin and stderr are TTYs; otherwise fail with a clear message and exit code. `--yes` accepts prompts.
4. **Same defaults as the app.** Model, device, data directory and language come from the shared config; flags override it for one run.
5. **Offline.** The only network commands are `models pull` (and the prompt that offers it).

## Commands

```
vtt transcribe <FILE>... [options]     audio/video → text
vtt speak [TEXT] [options]             text → audio
vtt record [options]                   microphone → file (optionally transcribe)
vtt export <TRANSCRIPT.json> [options] convert a transcript to txt/srt/vtt/md/json
vtt models <list|info|pull|remove|verify>
vtt library <list|search|show|export|delete>
vtt devices                            list audio input/output devices
vtt config <path|list|get|set>
vtt doctor                             diagnose GPU, devices, models, permissions
vtt completions <bash|zsh|fish|powershell>
vtt serve                              optional local HTTP API (later phase)
```

### `vtt transcribe`

**Implemented:** files in WAV, MP3, M4A/AAC, FLAC, OGG and the audio of MP4/MOV/MKV; `-m`, `-l`, `--translate`,
`--device`, `-f txt|srt|vtt|json|md`, `-o` (file or directory), `--timestamps`, `--no-speaker-names`, `--max-line`,
`--lines`, `-y`, `--no-vad`, `--json`, and saving every result to the shared library (`--no-library` skips it). **Not yet:**
`-f docx` and reading audio from stdin (`-`). These exit with code 70. Speaker identification is described in
[diarization.md](diarization.md).

```
vtt transcribe talk.m4a                               # prints text to stdout
vtt transcribe talk.m4a -m small --diarize -f srt -o talk.srt
vtt transcribe *.mp3 -f txt -o transcripts/           # many files → directory
vtt transcribe call.wav --translate --json
cat clip.wav | vtt transcribe - -f txt                # '-' reads audio from stdin
```

| Option                          | Meaning                                               |
| ------------------------------- | ----------------------------------------------------- |
| `-m, --model <id>`              | STT model (default from config, e.g. `whisper-base`)  |
| `-l, --language <code>`         | Source language; default auto-detect                  |
| `--translate`                   | Translate to English (not with the turbo models)      |
| `--diarize`                     | Identify speakers                                     |
| `--speakers <N>`                | Exact speaker count (replaces `--max-speakers`)       |
| `--diarize-threshold <X>`       | Voice distance, 0 to 2 (default 0.5), no `--speakers` |
| `-f, --format <fmt>`            | `txt` (default), `srt`, `vtt`, `json`, `md`, `docx`   |
| `-o, --output <path>`           | File, or directory for several inputs; `-` = stdout   |
| `--timestamps`                  | Include timestamps in `txt`/`md`                      |
| `--no-speaker-names`            | Omit speaker labels                                   |
| `--max-line <N>`, `--lines <N>` | Subtitle line width / lines per cue                   |
| `--device <auto\|cpu\|gpu>`     | Compute device                                        |
| `--no-vad`                      | Transcribe the whole file, not only the speech in it  |
| `--no-library`                  | Do not add the result to the shared Library           |
| `-y, --yes`                     | Auto-accept the "download missing model?" prompt      |

By default only the stretches that contain speech are transcribed, in chunks of at most 28 s (an energy-based
detector; it needs no extra model). Silence is skipped, so timestamps follow the speech and Whisper does not
invent text for quiet parts. `--no-vad` restores one pass over the whole file.

`--speakers` and `--diarize-threshold` need `--diarize` (exit 2 otherwise). `--translate` with a model that cannot
translate (the Whisper large-v3 turbo models) exits with code 3.

Behaviour: if the model is missing and the session is interactive, offer to download it; otherwise exit with code 4 and print the exact `vtt models pull` command.

### `vtt speak`

```
vtt speak "Hello there" -o hello.wav
vtt speak -f article.txt --voice adam --speed 1.1 -o article.wav
echo "Build finished" | vtt speak -o done.wav
vtt speak --list-voices
```

Speaks English text with Kokoro-82M (model id `kokoro-82m`, about 100 MB, six American voices; `vtt models pull
kokoro-82m`, or accept the offer to download it). Everything runs on the CPU; a sentence takes well under a second
once the model is loaded.

Options: `-f/--file <path>` (read the text from a file), `--voice <name>` (default `af_heart`; the part after the
prefix works too, `adam` for `am_adam`), `--speed <0.5-2.0>`, `-o/--output <path>` (a file, a folder, or `-` for
standard output; default `speech.wav`), `--list-voices`, `-y/--yes` and `--json`. Text comes from the argument, from
`--file`, or from standard input (`-` forces it).

Text is spoken one sentence at a time, with a short pause between sentences and a longer one between paragraphs
(blank lines). Numbers, years, ordinals, percentages, dollar amounts and common abbreviations are read out as words.
Words missing from the pronunciation dictionary are guessed from their spelling, so unusual names may sound off.

**Not yet:** `--format mp3|flac` and `--play` (exit code 70), voices for other languages, and keeping the result in the
library. Only WAV (16-bit mono, 24 kHz) is written. Pronunciation is described in [speech.md](speech.md).

### `vtt record`

`vtt record -o note.wav [--device <name>] [--duration 30s] [--transcribe]`. Ctrl-C stops and finalises the file (it is written continuously, so nothing is lost).

### `vtt export`

Converts a saved transcript (the JSON format) to another format without re-running a model. This is the first command that works end-to-end because it needs no engine.

### `vtt models`

`list [--kind stt|tts|diarization] [--installed] [--json]`, `info <id>`, `pull <id>` (progress bar, resumable, SHA-256 verified), `remove <id>`, `verify`.
Voices are models of kind `tts`; `vtt models list --kind tts` is the voice list.

### `vtt library`

`list`, `search <query> [--limit N]` (full-text, every word must match, the last as a prefix; the match is shown
between `[` and `]`), `show <id> [-f fmt]`, `export <id>... -f fmt -o dir` (one `<id>-<title>.<ext>` file each) and
`delete <id>...`. With `--json`, `list` and `search` print one JSON array. Ids that are not numbers or not in the
library exit with code 3. The library is one SQLite file, `library.sqlite`, in the data directory.

### `vtt config`, `vtt doctor`

`config path` prints the config file and data directory. `config list` shows every setting, `config get KEY` one
of them, and `config set KEY VALUE` changes it. The settings are shared with the desktop app's Settings screen:

| Key                 | Values                     | Used by                                                  |
| ------------------- | -------------------------- | -------------------------------------------------------- |
| `model`             | a model id, or `auto`      | `transcribe` when `-m` and `VTT_MODEL` are not given     |
| `language`          | a language code, or `auto` | `transcribe` when `-l` is not given                      |
| `device`            | `auto`, `cpu`, `gpu`       | `transcribe` when `--device` and `VTT_DEVICE` are absent |
| `identify-speakers` | `true`, `false`            | `transcribe` as if `--diarize` was given                 |

A flag or environment variable always wins for one run. An unknown key, a bad value or a damaged file is exit
code 3 and names the file. With `--data-dir` (or `VTT_DATA_DIR`) the config file is `config.toml` inside that
folder, so a relocated setup does not touch the user's real settings. `doctor` checks: data directory writable, audio devices, GPU backend, installed models, and (macOS) microphone permission. Exit code 0 only if everything required is OK.

## Global options

`--json`, `-q/--quiet`, `-v/--verbose` (repeatable), `--no-color` (also honours `NO_COLOR`), `--data-dir <path>`, `--config <file>`, `-h/--help`, `-V/--version`.

Environment: `VTT_DATA_DIR`, `VTT_MODEL`, `VTT_DEVICE`, `VTT_CONFIG`, `NO_COLOR`.
Config file: TOML in the platform config directory; `vtt config set model whisper-small` edits it.

## Exit codes

| Code | Meaning                                                             |
| ---- | ------------------------------------------------------------------- |
| 0    | Success                                                             |
| 1    | Unexpected error, or the library database cannot be read/written    |
| 2    | Usage error (bad flags)                                             |
| 3    | Input error (file missing/unreadable/unsupported, bad JSON)         |
| 4    | Required model is not installed                                     |
| 6    | Engine failure (decode, inference, device)                          |
| 7    | Download failed (no connection, incomplete data, checksum mismatch) |
| 70   | Feature not implemented yet                                         |
| 130  | Cancelled (Ctrl-C)                                                  |

## `--json` event stream (long jobs)

```
{"event":"start","file":"talk.m4a","model":"whisper-base","duration_ms":2892000}
{"event":"progress","processed_ms":600000,"total_ms":2892000,"speed":4.1}
{"event":"segment","start_ms":1020,"end_ms":4810,"speaker":"SPEAKER_00","text":"…"}
{"event":"download","id":"whisper-base","downloaded_bytes":60000000,"total_bytes":147951465}
{"event":"done","output":"talk.srt","elapsed_ms":701000}
{"event":"error","code":4,"message":"model not installed: whisper-base"}
```

The final transcript document uses the same schema the app stores and `vtt export` reads:

```
{"language":"en","duration_ms":2892000,"model":"whisper-base",
 "segments":[{"start_ms":1020,"end_ms":4810,"speaker":"SPEAKER_00","text":"…"}]}
```

## Migration from the Python CLI

| Python (`transcribe.py` / `cli.py`)   | `vtt`                                                   |
| ------------------------------------- | ------------------------------------------------------- |
| `--model base`                        | `-m whisper-base` (bare `base` still accepted)          |
| `--translate`                         | `--translate`                                           |
| `--diarize`                           | `--diarize`                                             |
| `--max-speakers N`                    | `--speakers N` (old flag kept as an alias)              |
| `--diarize-threshold X`               | `--diarize-threshold X`                                 |
| output always to `media/transcripts/` | stdout by default; `-o` to choose; Library keeps a copy |

## Testing

- Unit tests in `core` for every exporter and the argument parsing.
- CLI integration tests (`assert_cmd`) for exit codes, stdout/stderr separation and `--json` output.
- Golden-file tests for SRT/VTT cue splitting.
- A short fixture clip per supported container once the decoder lands.

## Build order for the CLI

1. `export`, `models list`, `config`, `doctor`, `completions` (no model needed). **Scaffolded first.**
2. `models pull/verify/remove`. **Done.**
3. `transcribe` (file → text), then `--diarize`, `--translate`.
4. `speak`.
5. `record`, `library`, `devices`.
6. `serve`.
