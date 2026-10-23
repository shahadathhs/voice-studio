# Features

What Voice Studio does, screen by screen. Mockups are in [../design](../design/README.md).
Command-line behavior is specified in [cli.md](cli.md); how it is built is in [../architecture.md](../architecture.md).

## Principles

1. **Offline and private.** No accounts, no telemetry, no uploads. The only network use is downloading models and checking for updates, and both are visible and optional.
2. **One core, three front ends.** GUI, CLI and (optional) local API call the same Rust crate. Behaviour cannot drift.
3. **No runtime to install.** No Python, no PyTorch, no ffmpeg. Models download on first use.
4. **Quiet UI.** System fonts, one accent colour, real content, no decoration. It should look like a tool, not a landing page.
5. **Honest about limits.** Show real progress, real speed, real disk use, and say when something is approximate (e.g. live captions are a rough draft).

## Features by screen

### Transcribe — drop zone ([01](../design/mockups/01-transcribe-empty.png))

- Drop or choose audio and video files (WAV, MP3, M4A, FLAC, OGG, AAC, MP4, MOV, MKV). Audio is extracted from video in-process.
- Per-batch options: model, language (auto-detect or fixed), identify speakers, translate to English.
- Recent items list with duration, speaker count, language.
- Files are read in place; a copy is only made if the user enables "keep a copy" in Storage.

### Transcribe — queue ([02](../design/mockups/02-transcribe-queue.png))

- Sequential queue by default (setting: parallel jobs). Pause, cancel, reorder, remove.
- Real progress: audio processed vs total, measured speed (× realtime), time left.
- Live preview of the latest decoded line while a job runs.
- Jobs continue in the tray when the window closes; completion notification opens the result.
- Crash-safe: partial results are checkpointed so a job resumes instead of restarting.

### Transcript viewer/editor ([03](../design/mockups/03-transcript.png), dark: [12](../design/mockups/12-transcript-dark.png))

- Audio player with waveform, speed control (0.75×–2×), speaker lanes under the waveform.
- Click a line to seek; the current line follows playback and can be locked or unlocked from scrolling.
- Inline text editing; edits are stored as a layer on top of the original model output (original always recoverable).
- Speaker panel: rename, recolour, merge two speakers, see talk-time share. Renames apply to every line and every export.
- Reassign a single line to another speaker (to fix diarization mistakes).
- Search with highlight, find next/previous.
- Details panel: source, model, language and confidence, word count, processing time.
- Copy (plain text or with speakers) and Export.

### Export dialog ([11](../design/mockups/11-export-dialog.png))

- Formats: TXT, SRT, VTT, JSON (word-level timestamps), DOCX, Markdown.
- Options: speaker names, timestamps in text, max characters per line, lines per cue (subtitle formats).
- Live preview of the first cue before writing the file.
- Batch export of several library items at once (Library → Export all).

### Record ([04](../design/mockups/04-record.png))

- Input device picker, live level meter, timer, pause/resume, discard.
- Optional live captions: a rough draft from a small streaming model while recording; the full-accuracy pass runs after Stop.
- Recording is written to disk continuously, so a crash loses at most a second or two.
- Stop & transcribe sends the recording straight into the queue.

### Speak — text to speech ([05](../design/mockups/05-speak.png))

- Text area (paste or open a .txt/.md file); character, word and estimated-duration counters.
- Voice list with preview, speed slider, output format (WAV, MP3, FLAC), optional paragraph pause.
- Sentence-level highlighting while playing; generation streams sentence by sentence, so playback starts before synthesis finishes.
- Save to file, or keep in Library as a generated item.
- Long documents are chunked and can be cancelled.

### Models ([06](../design/mockups/06-models.png))

- Tabs: Speech to text, Text to speech, Speaker identification.
- Per model: size, languages, speed on this machine (measured on first use), license, status.
- Download with progress, pause, resume, cancel, SHA-256 verification; remove to free space.
- Disk usage summary and data-folder location.
- A signed catalog file in the repo lists models, URLs, hashes and licenses; only permissively licensed models are listed by default.

### Library ([07](../design/mockups/07-library.png))

- One list of everything produced: transcripts and generated speech.
- Full-text search across transcript text (SQLite FTS5) with match snippets.
- Filter by kind, sort by date/length/name, bulk delete and export.

### Settings ([08](../design/mockups/08-settings.png))

- General: appearance (System / Light / Dark, also switchable from the sidebar), interface language, start at login, update checks.
- Audio: default input and output devices, test tone and mic check.
- Shortcuts: dictate into any app, read selection aloud, show window.
- Performance: compute device (auto, GPU, CPU), CPU threads, one job at a time.
- Storage: data folder, keep a copy of source audio, history retention.
- About: version, licenses of the app and every bundled model, link to source.

### First run ([09](../design/mockups/09-onboarding.png))

- Three steps: welcome, choose models, shortcuts and permissions (microphone; accessibility on macOS).
- Starter model is selectable; the app is usable as soon as that one finishes.
- "Skip for now" always available.

### Tray, dictation, read-aloud ([10](../design/mockups/10-tray-dictation.png))

- Tray/menu-bar menu: start dictation, read selection, transcribe a file, queue status, open, quit.
- **Dictation into any app:** global hotkey shows a small floating pill with a live level and partial text; on finish, the text is inserted into the focused app.
- **Read selection aloud:** global hotkey copies the selection and speaks it.
- Both depend on OS permissions and have platform limits (see Risks).

### Command line

One binary, same core, no GUI needed:

```
vtt transcribe talk.m4a --model small --diarize --format srt -o talk.srt
vtt speak "Hello" --voice heart --speed 1.1 -o hello.wav
vtt speak -f article.md --voice emma -o article.mp3
vtt models list | pull <id> | remove <id>
vtt library list | search <query> | export <id>
vtt serve --port 8765        # optional local HTTP API, off by default
```

Machine-readable output with `--json`; progress on stderr; exit codes documented.

## Not in scope (for now)

Cloud sync, accounts, team features, voice cloning, mobile apps, plugin system, real-time translation of live audio.
