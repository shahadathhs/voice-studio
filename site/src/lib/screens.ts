export interface ScreenInfo {
  id: string;
  name: string;
  title: string;
  body: string;
  points: string[];
  shot: string;
  alt: string;
  width: number;
  height: number;
}

const WIDE = { width: 2560, height: 1600 };
const TALL = { width: 2560, height: 2000 };

export const SCREENS: ScreenInfo[] = [
  {
    id: "transcribe",
    name: "Transcribe",
    title: "Drop in recordings. Get transcripts.",
    body: "Audio and video files go into a queue that works through them one at a time, skipping silence. Pause, reorder or cancel. The queue is saved, so a quit or a crash loses nothing.",
    points: [
      "WAV, MP3, M4A, FLAC, OGG, AAC, MP4, MOV, MKV",
      "Seven Whisper models, tiny to large",
      "TXT, SRT, VTT, Markdown, JSON and Word export",
    ],
    shot: "transcribe",
    alt: "The transcription queue with a running job and recent transcripts",
    ...WIDE,
  },
  {
    id: "speakers",
    name: "Speakers",
    title: "Know who said what.",
    body: "Turn on Identify speakers and every line gets a name. See who talked how much, rename or merge speakers, and fix a line by editing it or moving it to someone else.",
    points: [
      "Click a line or the waveform to hear that moment",
      "Talk-time share for each speaker",
      "Edits sit on top of the original, which stays recoverable",
    ],
    shot: "transcript",
    alt: "A transcript with three speakers, timestamps, a waveform and a speaker panel",
    ...WIDE,
  },
  {
    id: "record",
    name: "Record",
    title: "Record with live captions.",
    body: "Record from any microphone and watch a rough draft of the words appear as you speak. The full-accuracy pass runs when you stop. A global shortcut also dictates into the clipboard, ready to paste anywhere.",
    points: [
      "Live captions are labelled as a draft",
      "Dictation from the tray or a shortcut",
      "Recordings stay on this computer until you discard them",
    ],
    shot: "record",
    alt: "The Record screen with a microphone picker, a level meter and a live captions option",
    ...WIDE,
  },
  {
    id: "speak",
    name: "Speak",
    title: "Read text aloud in seven languages.",
    body: "Paste text or open a file and pick one of 35 voices: English (American and British), Spanish, French, Italian, Brazilian Portuguese, Hindi and Mandarin Chinese. Translate first to speak in another language.",
    points: [
      "Save as WAV or FLAC",
      "Speed control and sentence-by-sentence generation",
      "Runs on the CPU, well under a second per sentence",
    ],
    shot: "speak",
    alt: "Text to speech with language, voice and speed controls",
    ...WIDE,
  },
  {
    id: "voices",
    name: "Voices",
    title: "Blend a voice. Or clone one, with permission.",
    body: "Mix two to four built-in voices into a named preset. Or clone a voice from a 3 to 30 second sample: you must confirm you own the voice or have the speaker's permission, and saved audio carries a synthetic voice note.",
    points: [
      "Cloned voices are marked and can be deleted",
      "No third-party or celebrity voices are bundled",
      "Cloning supports English, Spanish, French, Italian, Portuguese and Hindi",
    ],
    shot: "voices",
    alt: "The Voices screen with tabs for my voices, blend and clone",
    ...TALL,
  },
  {
    id: "library",
    name: "Library",
    title: "Search, summarise, translate.",
    body: "Everything you transcribe lands in one searchable library. A small language model on your computer writes summaries, chapters, action items and translations into 20 languages, labelled as machine-written.",
    points: [
      "Full-text search across every transcript",
      "Chapters jump the player to the right moment",
      "Bengali output is flagged as unreliable",
    ],
    shot: "library",
    alt: "The library of transcripts with search",
    ...WIDE,
  },
  {
    id: "models",
    name: "Models",
    title: "You choose the models.",
    body: "A model manager shows what each model is for, how big it is, and its licence. Downloads resume, can be cancelled, and are checked against a published SHA-256 checksum.",
    points: [
      "Only permissive licences are listed",
      "Remove a model to free the disk space",
      "The only network use is downloads you start",
    ],
    shot: "models",
    alt: "Models grouped by purpose, with sizes and licences",
    ...TALL,
  },
];
