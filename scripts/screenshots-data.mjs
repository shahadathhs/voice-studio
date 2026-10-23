// Made-up sample data for the screenshots. Nothing here is real audio, a real person or a real model.
const MINUTE = 60_000;
const model = ({ kind, languages = ["multilingual"], license = "MIT", ...rest }) => ({
  kind,
  languages,
  license,
  can_translate: kind === "stt",
  ...rest,
});

const item = (id, title, minutes, speakers, language = "en") => ({
  id,
  title,
  created_at: 1_790_000_000 - id * 86_400,
  duration_ms: minutes * MINUTE,
  source_path: `/Recordings/${title}.m4a`,
  model: "whisper-small",
  language,
  segment_count: minutes * 6,
  speaker_count: speakers,
});

const library = [
  item(1, "Quarterly planning meeting", 42, 3),
  item(2, "Interview with a ceramicist", 28, 2),
  item(3, "Lecture 4: Memory and attention", 55, 1),
  item(4, "Voice memo: trip ideas", 3, 1),
];

const lines = [
  [
    "Priya",
    "Let's start with the roadmap. The main question is whether we ship the editor this quarter or wait.",
  ],
  [
    "Marcus",
    "I'd wait. The export work isn't finished, and I'd rather launch one thing that works well.",
  ],
  [
    "Priya",
    "That's fair. Can we agree on a smaller first release, just transcription and the library?",
  ],
  [
    "Elena",
    "Yes. Search across everything is what people ask about most, so that goes in the first release.",
  ],
  [
    "Marcus",
    "Then I'll move the editor to next quarter and keep the export formats we already have.",
  ],
  ["Elena", "Good. I'll write up the plan and share it before Friday."],
];

const transcript = {
  item: library[0],
  word_count: 1840,
  speakers: [
    { name: "Priya", talk_ms: 17 * MINUTE },
    { name: "Marcus", talk_ms: 14 * MINUTE },
    { name: "Elena", talk_ms: 11 * MINUTE },
  ],
  transcript: {
    language: "en",
    duration_ms: 42 * MINUTE,
    model: "whisper-small",
    segments: lines.map(([speaker, text], index) => ({
      start_ms: index * 9_000 + 4_000,
      end_ms: index * 9_000 + 12_000,
      speaker,
      text,
    })),
  },
};

const jobs = [
  {
    id: 1,
    file: "/Recordings/Customer call.mp4",
    title: "Customer call",
    model: "whisper-small",
    state: "running",
    processed_ms: 19 * MINUTE,
    total_ms: 31 * MINUTE,
    error: null,
    item_id: null,
  },
  {
    id: 2,
    file: "/Recordings/Podcast episode 12.mp3",
    title: "Podcast episode 12",
    model: "whisper-small",
    state: "waiting",
    processed_ms: 0,
    total_ms: 64 * MINUTE,
    error: null,
    item_id: null,
  },
  {
    id: 3,
    file: "/Recordings/Quarterly planning meeting.m4a",
    title: "Quarterly planning meeting",
    model: "whisper-small",
    state: "done",
    processed_ms: 42 * MINUTE,
    total_ms: 42 * MINUTE,
    error: null,
    item_id: 1,
  },
];

export const SAMPLE_BACKEND = {
  app_info: {
    name: "Voice Studio",
    version: "0.1.0",
    coreVersion: "0.1.0",
    dataDir: "~/Library/Application Support/Voice Studio",
  },
  list_models: [
    model({
      id: "whisper-tiny",
      name: "Whisper tiny",
      kind: "stt",
      description: "Fastest and least accurate. For quick drafts of clear, close-up speech.",
      size_bytes: 77691713,
      installed: true,
    }),
    model({
      id: "whisper-base",
      name: "Whisper base",
      kind: "stt",
      description: "Fast and light. Good for clean recordings in a quiet room.",
      size_bytes: 147951465,
      installed: false,
    }),
    model({
      id: "whisper-small",
      name: "Whisper small",
      kind: "stt",
      description:
        "The best balance for most people: copes with accents and some background noise.",
      size_bytes: 487601967,
      installed: true,
    }),
    model({
      id: "whisper-medium",
      name: "Whisper medium",
      kind: "stt",
      description: "High accuracy for difficult audio. Noticeably slower.",
      size_bytes: 1533763059,
      installed: false,
    }),
    model({
      id: "speaker-identification",
      name: "Speaker identification (pyannote + WeSpeaker ResNet34)",
      kind: "diarization",
      description:
        "Tells voices apart so a transcript can label who is speaking. Small and fast on the CPU.",
      size_bytes: 32500000,
      installed: true,
      license: "Apache-2.0",
    }),
    model({
      id: "kokoro-82m",
      name: "Kokoro 82M (English voices)",
      kind: "tts",
      description: "Reads English text aloud in six natural American voices. Runs on the CPU.",
      size_bytes: 102000000,
      installed: true,
      languages: ["en"],
      license: "Apache-2.0",
    }),
  ],
  get_settings: { model: "whisper-small", language: null, device: "auto", identify_speakers: true },
  get_waveform: Array.from({ length: 240 }, (_, n) =>
    Math.min(1, 0.12 + 0.88 * Math.abs(Math.sin(n / 5) * Math.sin(n / 23 + 1)) ** 0.8),
  ),
  list_jobs: jobs,
  list_library: library,
  search_library: [],
  get_transcript: transcript,
  list_voices: [
    { id: "af_heart", name: "Heart", description: "American, female, warm", language: "en" },
    { id: "af_bella", name: "Bella", description: "American, female, bright", language: "en" },
    { id: "am_michael", name: "Michael", description: "American, male, steady", language: "en" },
    { id: "am_fenrir", name: "Fenrir", description: "American, male, deep", language: "en" },
  ],
};
