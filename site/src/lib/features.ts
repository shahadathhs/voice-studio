export interface Feature {
  title: string;
  body: string;
}

export const FEATURES: Feature[] = [
  {
    title: "Transcribe audio and video",
    body: "Drop in WAV, MP3, M4A, FLAC, OGG, MP4, MOV or MKV files. A queue works through them one at a time, skipping silence, with progress you can pause, reorder or cancel.",
  },
  {
    title: "Know who said what",
    body: "Turn on Identify speakers to label each line, see who talked how much, and rename or merge speakers afterwards.",
  },
  {
    title: "Search everything",
    body: "Every transcript goes into a library with full-text search. Click a timestamp to hear that moment.",
  },
  {
    title: "Export anywhere",
    body: "TXT, SRT and VTT subtitles, Markdown, JSON and Word documents, with control over line width, timestamps and speaker names.",
  },
  {
    title: "Read text aloud",
    body: "Paste text or open a file, pick one of six natural English voices, and play or save it.",
  },
  {
    title: "Choose your models",
    body: "A model manager shows what each model is for, how big it is, and its licence. Downloads are resumable and checksum-verified.",
  },
];
