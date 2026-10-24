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
- Language and voice pickers (35 voices in English, Spanish, French, Italian, Brazilian Portuguese, Hindi and Mandarin Chinese), speed slider, output format (WAV or FLAC; MP3 is not planned).
- "Translate, then speak": the text is translated with the language model and spoken in that language's voice.
- Sentence-level highlighting while playing; generation streams sentence by sentence, so playback starts before synthesis finishes.
- Save to file. (Keeping generated speech in the Library is not built yet.)
- Long documents are chunked and can be cancelled.

### Voices

- **Blend:** mix 2 to 4 built-in voices of one language into a named preset, with a preview. Presets work anywhere a voice does.
- **Clone:** make a voice from a 3 to 30 second sample. The person must confirm they own the voice or have permission; cloned voices are marked as such, can be deleted, and saved audio carries a "synthetic voice" note. There is no watermark.
- Cloning needs the Chatterbox model (about 1.6 GB) and supports English, Spanish, French, Italian, Portuguese and Hindi.

### Summaries, chapters and translation

- On a transcript: a summary with key points, chapters (titles that seek the player), action items, and a translation into one of 20 languages. All written by a small local language model (Qwen 2.5, 1.5B or 7B) and labelled as such.
- Results are kept with the transcript; a translation is stored as a new library item. Bengali output is flagged as unreliable.

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

- Tray/menu-bar menu: open the window, start or stop dictation, quit. (Transcribe a file and queue status in the menu are not built.)
- **Dictation:** a global shortcut starts and stops recording; a small floating window shows the state, and on finish the text is copied to the clipboard, ready to paste. It is not typed into the focused app, because that needs accessibility permissions on every system.
- **Read selection aloud:** not built.
- Dictation depends on OS permissions and has platform limits (see Risks).

### Command line

One binary, same core, no GUI needed:

```
vtt transcribe talk.m4a --model small --diarize --format srt -o talk.srt
vtt speak "Hello" --voice heart --speed 1.1 -o hello.wav
vtt speak -f article.txt --translate-to es -o articulo.flac
vtt record -o note.wav --live --transcribe
vtt voice create Warm --blend af_heart:0.6,af_bella:0.4
vtt summarize 12 --chapters --actions
vtt models list | pull <id> | remove <id>
vtt library list | search <query> | export <id>
```

Machine-readable output with `--json`; progress on stderr; exit codes documented.

## Not in scope (for now)

Cloud sync, accounts, team features, a local HTTP API (`vtt serve`), mobile apps, plugin system, real-time translation of live audio, MP3 output.
