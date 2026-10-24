export interface Feature {
  slug: string;
  name: string;
  title: string;
  /** One line for cards and link lists. */
  summary: string;
  /** Meta description for search results. */
  description: string;
  intro: string;
  points: string[];
  how: string[];
  cli: string;
  limits: string[];
  shot: string;
  alt: string;
  width: number;
  height: number;
  related: string[];
}

const WIDE = { width: 2560, height: 1600 };
const TALL = { width: 2560, height: 2000 };

/** Keep in step with README.md and docs/product/features.md. */
export const FEATURES: Feature[] = [
  {
    slug: "transcribe",
    name: "Transcribe",
    title: "Drop in recordings. Get transcripts.",
    summary: "Audio and video files in a saved queue, on seven Whisper models.",
    description:
      "Transcribe audio and video offline with Whisper. A saved queue, seven model sizes, and TXT, SRT, VTT, Markdown, JSON and Word export.",
    intro:
      "Audio and video files go into a queue that works through them one at a time, skipping silence. Pause, reorder or cancel. The queue is saved, so a quit or a crash loses nothing.",
    points: [
      "WAV, MP3, M4A, FLAC, OGG, AAC, MP4, MOV and MKV",
      "Seven Whisper models, from tiny to large",
      "Real progress: audio processed against the total",
      "TXT, SRT, VTT, Markdown, JSON and Word export",
    ],
    how: [
      "Drop files on the Transcribe screen, or choose them.",
      "Pick a model and a language, or let it detect the language.",
      "Watch the queue. Open a finished transcript to read, edit and export it.",
    ],
    cli: `vtt models pull whisper-small
vtt transcribe talk.m4a
vtt transcribe talk.m4a -f srt -o talk.srt`,
    limits: [
      "One file at a time; parallel jobs are not built.",
      "On Windows and Linux Whisper uses the CPU unless you build with the Vulkan or CUDA option.",
      "Silence is skipped with a simple energy detector, not a neural one.",
    ],
    shot: "transcribe",
    alt: "The transcription queue with a running job and recent transcripts",
    related: ["speakers", "library"],
    ...WIDE,
  },
  {
    slug: "speakers",
    name: "Speakers",
    title: "Know who said what.",
    summary: "Label every line, see talk time, rename or merge speakers.",
    description:
      "Offline speaker identification: label each line of a transcript, see who talked how much, and rename or merge speakers.",
    intro:
      "Turn on Identify speakers and every line gets a name. See who talked how much, rename or merge speakers, and fix a single line by moving it to someone else.",
    points: [
      "Click a line or the waveform to hear that moment",
      "Talk-time share for each speaker",
      "Edits sit on top of the original, which stays recoverable",
      "Renames apply to every line and every export",
    ],
    how: [
      "Tick Identify speakers before you transcribe.",
      "Open the transcript. Rename a speaker in the panel on the right.",
      "Give two speakers the same name to merge them.",
    ],
    cli: `vtt transcribe meeting.mp4 --diarize
vtt transcribe meeting.mp4 --diarize --speakers 3`,
    limits: [
      "Checked on synthetic voices and one noisy real recording, not a large set of real meetings.",
      "Overlapping speech still gets one label per line.",
      "If the count is wrong, give it with --speakers.",
    ],
    shot: "transcript",
    alt: "A transcript with three speakers, timestamps, a waveform and a speaker panel",
    related: ["transcribe", "library"],
    ...WIDE,
  },
  {
    slug: "record",
    name: "Record",
    title: "Record with live captions.",
    summary: "Microphone recording, live draft captions and clipboard dictation.",
    description:
      "Record from any microphone with live draft captions, then run the full-accuracy pass. Dictate to the clipboard with a global shortcut.",
    intro:
      "Record from any microphone and watch a rough draft of the words appear as you speak. The full-accuracy pass runs when you stop. A global shortcut also dictates into the clipboard, ready to paste anywhere.",
    points: [
      "Live captions, clearly labelled as a draft",
      "A level meter, a timer and a device picker",
      "Dictation from the tray or a shortcut",
      "Recordings stay on this computer until you discard them",
    ],
    how: [
      "Choose a microphone and press record.",
      "Turn on Live captions to see words as you speak.",
      "Stop, and send the recording straight to the queue.",
    ],
    cli: `vtt devices
vtt record -o note.wav --live --transcribe`,
    limits: [
      "Live captions are rougher than the final pass.",
      "Dictation copies text to the clipboard; it does not type into other apps.",
      "Global shortcuts on Linux with Wayland have not been checked.",
    ],
    shot: "record",
    alt: "The Record screen with a microphone picker, a level meter and a live captions option",
    related: ["transcribe", "speak"],
    ...WIDE,
  },
  {
    slug: "speak",
    name: "Speak",
    title: "Read text aloud in seven languages.",
    summary: "35 voices; translate first to speak in another language.",
    description:
      "Offline text to speech with 35 voices in English, Spanish, French, Italian, Brazilian Portuguese, Hindi and Mandarin Chinese.",
    intro:
      "Paste text or open a file and pick one of 35 voices: English (American and British), Spanish, French, Italian, Brazilian Portuguese, Hindi and Mandarin Chinese. Translate first to speak in another language.",
    points: [
      "Save as WAV or FLAC",
      "Speed control; generation goes sentence by sentence",
      "Numbers, years and abbreviations are read out as words",
      "Runs on the CPU, well under a second per sentence",
    ],
    how: [
      "Paste text or open a text file.",
      "Pick a language and a voice, and set the speed.",
      "Generate and play, then save the audio.",
    ],
    cli: `vtt speak --list-voices
vtt speak "Hello from my own computer" -o hello.wav
vtt speak "Hello" --translate-to es -o hola.wav`,
    limits: [
      "WAV and FLAC only; MP3 is not planned.",
      "Unusual names can sound off: words missing from the dictionary are guessed from spelling.",
      "Each language was checked by machine listening, not yet by a native speaker.",
      "Japanese and Bengali are not available.",
    ],
    shot: "speak",
    alt: "Text to speech with language, voice and speed controls",
    related: ["voices", "library"],
    ...WIDE,
  },
  {
    slug: "voices",
    name: "Voices",
    title: "Blend a voice. Or clone one, with permission.",
    summary: "Voice blends, and cloning from a short sample you have rights to.",
    description:
      "Blend built-in voices into a preset, or clone a voice from a 3 to 30 second sample with the speaker's permission. Everything runs locally.",
    intro:
      "Mix two to four built-in voices into a named preset. Or clone a voice from a 3 to 30 second sample: you must confirm you own the voice or have the speaker's permission, and saved audio carries a synthetic voice note.",
    points: [
      "Cloned voices are marked as such and can be deleted",
      "No third-party or celebrity voices are bundled",
      "Presets work anywhere a voice does, in the app and the CLI",
      "Cloning supports English, Spanish, French, Italian, Portuguese and Hindi",
    ],
    how: [
      "Open Voices and choose Blend or Clone.",
      "For a blend, pick voices and weights, name it and preview it.",
      "For a clone, choose a short sample and confirm permission.",
    ],
    cli: `vtt voice create Warm --blend af_heart:0.6,af_bella:0.4
vtt models pull chatterbox-multilingual
vtt voice clone Anna --from anna.wav --i-have-permission`,
    limits: [
      "There is no watermark on cloned audio, only a note in the file.",
      "Cloning needs a model of about 1.6 GB and is slower than the built-in voices.",
      "Cloned speech has not been judged by native speakers in each language.",
    ],
    shot: "voices",
    alt: "The Voices screen with tabs for my voices, blend and clone",
    related: ["speak", "models"],
    ...TALL,
  },
  {
    slug: "library",
    name: "Library",
    title: "Search, summarise, translate.",
    summary: "Full-text search, plus summaries, chapters and translation by a local model.",
    description:
      "Search every transcript, then summarise, split into chapters, list action items and translate into 20 languages with a language model that runs on your computer.",
    intro:
      "Everything you transcribe lands in one searchable library. A small language model on your computer writes summaries, chapters, action items and translations into 20 languages, labelled as machine-written.",
    points: [
      "Full-text search across every transcript",
      "Chapters jump the player to the right moment",
      "A translation is kept as a new library item",
      "Two model sizes: 1.5B (about 1.1 GB) and 7B (about 4.7 GB)",
    ],
    how: [
      "Open a transcript and choose Summary and translate.",
      "Pick a language and a model, then summarise or translate.",
      "Results are kept with the transcript. Copy or delete them any time.",
    ],
    cli: `vtt library search "roadmap"
vtt summarize 12 --chapters --actions
vtt translate 12 --to zh -f srt -o talk.zh.srt`,
    limits: [
      "A small model can be wrong or invent details; output is labelled and never replaces the transcript.",
      "Bengali output is unreliable and is flagged in the app.",
      "The 7B model needs about 6 GB of memory.",
    ],
    shot: "library",
    alt: "The library of transcripts with search",
    related: ["transcribe", "speakers"],
    ...WIDE,
  },
  {
    slug: "models",
    name: "Models",
    title: "You choose the models.",
    summary: "Sizes and licences up front; resumable, checksum-verified downloads.",
    description:
      "A model manager that shows what each model is for, its size and its licence. Downloads resume and are verified against a published SHA-256.",
    intro:
      "A model manager shows what each model is for, how big it is, and its licence. Downloads resume, can be cancelled, and are checked against a published SHA-256 checksum.",
    points: [
      "Only permissive licences are listed",
      "Remove a model to free the disk space",
      "The only network use is downloads you start",
      "Seven speech models, speaker labels, voices and language models",
    ],
    how: [
      "Open Models and browse by purpose.",
      "Download what you need; pause or cancel any time.",
      "Remove a model when you no longer use it.",
    ],
    cli: `vtt models list
vtt models pull whisper-small
vtt models verify`,
    limits: [
      "Models are not bundled; the first use of a feature needs a download.",
      "Some training-data licences still need review before a public release.",
    ],
    shot: "models",
    alt: "Models grouped by purpose, with sizes and licences",
    related: ["transcribe", "voices"],
    ...TALL,
  },
];

export function featureBySlug(slug: string): Feature | undefined {
  return FEATURES.find((feature) => feature.slug === slug);
}
